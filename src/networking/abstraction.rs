use std::collections::HashMap;

use async_channel::{Receiver, Sender};
use iroh::{NodeAddr, NodeId};

use crate::error::{Error, Res};
use crate::networking::network::ForeignNodeContact;
use crate::networking::packet::Packet;
use crate::networking::network::Server;

pub struct ForeignNode {
    send_client: ForeignNodeContact,
    conversation: Vec<Packet>
}

pub struct Network {
    conversations: HashMap<NodeId, ForeignNode>,
    incoming: Server
}

pub enum NetworkTask {
    RequestConversation(NodeId)
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
        cycle_output.clear();
        
        // First, check if there are any new messages. If there was a new client that failed to respond appropriately, emit an error.
        while let Ok(incoming) = message_receiver.try_recv() {
            if let Err(e) = network.add_message(incoming.clone()).await {
                cycle_output.push(NetworkOutput::NonFatalError(e));
            } else {
                cycle_output.push(NetworkOutput::AddPacket(incoming))
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
            }
        }
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
}
