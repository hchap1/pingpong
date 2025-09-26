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

#[derive(Debug, Clone)]
pub enum NetworkTask {
    RequestConversation(NodeId),
    SendMessage(NodeId, Vec<u8>, PacketType)
}

#[derive(Debug, Clone)]
pub enum NetworkOutput {
    AddPacket(Packet),
    NonFatalError(Error),
    ConversationRecord(Vec<Packet>),
    AddChat(NodeId)
}

pub async fn run_network(tasks: Receiver<NetworkTask>, output: Sender<NetworkOutput>) -> Res<()> {
    let server: Server = Server::spawn().await?;
    let mut network: Network = Network {
        conversations: HashMap::new(),
        incoming: server
    };

    println!("NODE_ID: {}", network.incoming.get_address().node_id);

    let message_receiver: Receiver<Packet> = network.yield_receiver();
    let mut cycle_output: Vec<NetworkOutput> = Vec::new();

    loop {
        // First, check if there are any new messages. If there was a new client that failed to respond appropriately, emit an error.
        while let Ok(incoming) = message_receiver.try_recv() {

            println!("INCOMING MESSAGE: {incoming:?}");

            // Parse the incoming message and tell the application to track the new chat if it exists.
            match network.add_message(incoming.clone()).await {
                Ok(Some(new_contact)) => cycle_output.push(NetworkOutput::AddChat(new_contact)),
                Ok(None) => {},
                Err(e) => cycle_output.push(NetworkOutput::NonFatalError(e))
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

                    println!("SEND MESSAGE TASK, TO {target}");

                    match network.send_message(target, packet.clone(), packet_type).await {
                        Ok(potential_new_node) => {

                            // Firstly, check if we need to add a new contact. This should not happen.
                            if let Some(new_contact) = potential_new_node {
                                cycle_output.push(NetworkOutput::AddChat(new_contact));
                            }

                            // Second, add our own message onto the conversation stack mirrored in application.
                            cycle_output.push(NetworkOutput::AddPacket(Packet {
                                author: network.incoming.get_address().node_id,
                                content: Ok(packet),
                                packet_type
                            }))

                        }
                        Err(e) => cycle_output.push(NetworkOutput::NonFatalError(e))
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
    /// If a new foreign node was successfuly spawned, Option<NodeId> contains the foreign address.
    pub async fn add_message(&mut self, packet: Packet) -> Res<Option<NodeId>> {
        
        let new_node = if let Some(mut_ref) = self.conversations.get_mut(&packet.author) {
            mut_ref.conversation.push(packet);
            None
        } else {
            let author = packet.author;
            self.conversations.insert(packet.author, ForeignNode {
                send_client: ForeignNodeContact::client(packet.author).await?,
                conversation: vec![packet]
            });
            Some(author)
        };

        Ok(new_node)

    }

    /// Send a message to a target address, forming a connection if it does not already exist to their server.
    /// If a new foreign node was successfuly spawned, Option<NodeId> contains the foreign address.
    pub async fn send_message(&mut self, recipient: NodeId, packet: Vec<u8>, packet_type: PacketType) -> Res<Option<NodeId>> {
        
        let new_node = if let Some(mut_ref) = self.conversations.get_mut(&recipient) {
            println!("SEND MESSAGE RECIPIENT EXISTS!");
            mut_ref.send_client.send(packet.clone(), packet_type).await?;
            mut_ref.conversation.push(Packet {
                author: self.incoming.get_address().node_id,
                content: Ok(packet),
                packet_type
            });
            None
        } else {
            println!("SEND RECIPIENT MESSAGE DOES NOT YET EXIST");
            self.conversations.insert(recipient, ForeignNode {
                send_client: ForeignNodeContact::client(recipient).await?,
                conversation: vec![Packet {
                    author: self.incoming.get_address().node_id,
                    content: Ok(packet.clone()),
                    packet_type
                }]
            });

            if let Some(mut_ref) = self.conversations.get_mut(&recipient) {
                let _ = mut_ref.send_client.send(packet, packet_type).await;
            }

            Some(recipient)
        };

        Ok(new_node)
    }
}
