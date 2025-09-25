use std::collections::HashMap;

use async_channel::{Receiver, Sender};
use iroh::NodeId;
use tokio::time::sleep;
use tokio::time::Duration;

use crate::error::{Error, Res};
use crate::networking::network::ForeignNodeContact;
use crate::networking::packet::Packet;
use crate::networking::network::Server;
use crate::networking::packet::PacketType;

pub struct ForeignNode {
    send_client: ForeignNodeContact,
    conversation: Vec<Packet>
}

pub struct Network {
    conversations: HashMap<NodeId, ForeignNode>,
    incoming: Server
}

pub enum NetworkTask {
    RequestConversation(NodeId),
    SendMessage(NodeId, Vec<u8>, PacketType)
}

pub enum NetworkOutput {
    AddPacket(Packet),
    NonFatalError(Error),
    ConversationRecord(Vec<Packet>)
}

pub async fn run_network(tasks: Receiver<NetworkTask>, output: Sender<NetworkOutput>) -> Res<()> {
    let server: Server = Server::spawn().await?;
    let mut network: Network = Network {
        conversations: HashMap::new(),
        incoming: server
    };

    let message_receiver: Receiver<Packet> = network.yield_receiver();

    let mut cycle_output: Vec<NetworkOutput> = Vec::new();

    loop {
        // First, check if there are any new messages. If there was a new client that failed to respond appropriately, emit an error.
        while let Ok(incoming) = message_receiver.try_recv() {
            if let Err(e) = network.add_message(incoming.clone()).await {
                cycle_output.push(NetworkOutput::NonFatalError(e));
            } else {
                cycle_output.push(NetworkOutput::AddPacket(incoming));
            }
        }

        // Second, parse any tasks that have been assigned to the network thread.
        while let Ok(task) = tasks.try_recv() {
            match task {
                NetworkTask::RequestConversation(node_id) => {
                    if let Some(client) = network.conversations.get(&node_id) {
                        cycle_output.push(
                            NetworkOutput::ConversationRecord(client.conversation.clone())
                        )
                    } else {
                        cycle_output.push(
                            NetworkOutput::NonFatalError(Error::NoSuchClient)
                        )
                    }
                }

                NetworkTask::SendMessage(target, packet, packet_type) => {
                    if let Err(e) = network.send_message(target, packet.clone(), packet_type).await {
                        cycle_output.push(NetworkOutput::NonFatalError(e));
                    } else {
                        cycle_output.push(NetworkOutput::AddPacket(Packet {
                            author: network.incoming.get_address().node_id,
                            content: Ok(packet),
                            packet_type
                        }))
                    }
                }
            }
        }

        // Finally output anything stored in the cycle list.
        for o in std::mem::take(&mut cycle_output) {
            if output.send(o).await.is_err() {
                return Err(Error::MPMCRecvError);
            }
        }

        // Poll at 50ms/cycle to avoid computational load
        sleep(Duration::from_millis(50)).await;
    }
}

impl Network {

    /// Yield a receiver that receives all messages. The implementation is responsible for adding this into the conversation synchronously.
    pub fn yield_receiver(&self) -> Receiver<Packet> {
        self.incoming.yield_receiver()
    }

    /// Asynchronously add a message into the conversation stack, spawning a new foreign node if required.
    pub async fn add_message(&mut self, packet: Packet) -> Res<()> {
        
        if let Some(mut_ref) = self.conversations.get_mut(&packet.author) {
            mut_ref.conversation.push(packet);
        } else {
            self.conversations.insert(packet.author, ForeignNode {
                send_client: ForeignNodeContact::client(packet.author).await?,
                conversation: vec![packet]
            });
        }

        Ok(())

    }

    /// Send a message to a target address, forming a connection if it does not already exist to their server.
    pub async fn send_message(&mut self, recipient: NodeId, packet: Vec<u8>, packet_type: PacketType) -> Res<()> {
        
        if let Some(mut_ref) = self.conversations.get_mut(&recipient) {
            mut_ref.send_client.send(packet.clone(), packet_type).await?;
            mut_ref.conversation.push(Packet {
                author: self.incoming.get_address().node_id,
                content: Ok(packet),
                packet_type
            });
        } else {
            self.conversations.insert(recipient, ForeignNode {
                send_client: ForeignNodeContact::client(recipient).await?,
                conversation: vec![Packet {
                    author: self.incoming.get_address().node_id,
                    content: Ok(packet.clone()),
                    packet_type
                }]
            });

            if let Some(mut_ref) = self.conversations.get_mut(&recipient) {
                mut_ref.send_client.send(packet, packet_type);
            }
        }

        Ok(())
    }
}
