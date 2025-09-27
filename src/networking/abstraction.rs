use std::collections::HashMap;
use std::str::FromStr;

use async_channel::{Receiver, Sender};
use iroh::NodeId;
use tokio::time::sleep;
use tokio::time::Duration;

use crate::backend::database::DataLink;
use crate::error::{Error, Res};
use crate::networking::network::ForeignNodeContact;
use crate::networking::packet::Packet;
use crate::networking::network::Server;
use crate::networking::packet::PacketType;

use super::contact::Contact;

pub struct ForeignNode {
    send_client: ForeignNodeContact,
    conversation: Vec<Packet>
}

pub struct Network {
    conversations: HashMap<NodeId, ForeignNode>,
    client_to_server: HashMap<NodeId, NodeId>,
    incoming: Server,
    username: Option<String>
}

#[derive(Debug, Clone)]
pub enum NetworkTask {
    RequestConversation(NodeId),
    SendMessage(NodeId, Vec<u8>, PacketType),
    SetUsername(String),
}

#[derive(Debug, Clone)]
pub enum NetworkOutput {
    AddPacket(Packet),
    NonFatalError(Error),
    ConversationRecord(Vec<Packet>),
    AddChat(Contact),
    ContactName(NodeId, String)
}

pub async fn run_network(tasks: Receiver<NetworkTask>, output: Sender<NetworkOutput>, db: DataLink, username: Option<String>) -> Res<()> {

    let server: Server = Server::spawn(db.clone()).await?;
    let mut network: Network = Network {
        conversations: HashMap::new(),
        client_to_server: HashMap::new(),
        incoming: server,
        username
    };

    println!("NODE_ID: {}", network.incoming.get_address().node_id);

    let message_receiver: Receiver<Packet> = network.yield_receiver();
    let mut cycle_output: Vec<NetworkOutput> = Vec::new();

    if let Some(username) = network.username.as_ref() {
        for mutable_value in network.conversations.values_mut() {
            let _ = mutable_value.send_client.send(username.as_bytes().to_vec(), PacketType::Username).await;
        }
    }

    loop {
        // First, check if there are any new messages. If there was a new client that failed to respond appropriately, emit an error.
        while let Ok(mut incoming) = message_receiver.try_recv() {

            // Parse the incoming message and tell the application to track the new chat if it exists.
            match network.add_message(incoming.clone(), &db).await {
                Ok(Some(message)) => cycle_output.push(message),
                Ok(None) => {},
                Err(e) => cycle_output.push(NetworkOutput::NonFatalError(e))
            }

            if incoming.packet_type == PacketType::String {
                if let Some(author) = network.client_to_server.get(&incoming.author) {
                    incoming.author = *author;
                    cycle_output.push(NetworkOutput::AddPacket(incoming));
                }
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

                    match network.send_message(target, packet.clone(), packet_type, &db).await {
                        Ok(potential_new_node) => {

                            // Firstly, check if we need to add a new contact. This should not happen.
                            if let Some(new_contact) = potential_new_node {
                                cycle_output.push(NetworkOutput::AddChat(Contact::from_node_id(new_contact)));
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

                NetworkTask::SetUsername(username) => {
                    for mutable_value in network.conversations.values_mut() {
                        let _ = mutable_value.send_client.send(username.as_bytes().to_vec(), PacketType::Username).await;
                    }
                    network.username = Some(username);
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
    pub async fn add_message(&mut self, mut packet: Packet, db: &DataLink) -> Res<Option<NetworkOutput>> {
        
        match self.client_to_server.get(&packet.author) {
            Some(author) => if let Some(mut_ref) = self.conversations.get_mut(author) {
                packet.author = *author;
                
                match packet.packet_type {
                    PacketType::String => mut_ref.conversation.push(packet),
                    PacketType::Username => {
                        if let Ok(content) = packet.content {
                            if let Ok(username) = String::from_utf8(content) {
                                return Ok(Some(NetworkOutput::ContactName(packet.author, username)))
                            }
                        }
                    },
                    _ => {}
                }
            }

            None => if packet.packet_type == PacketType::Address {
                if let Ok(content) = packet.content {
                    if let Ok(string) = String::from_utf8(content) {
                        if let Ok(node_id) = NodeId::from_str(&string) {

                            // Associate the foreign client with the foreign server
                            self.client_to_server.insert(packet.author, node_id);

                            // Create a new converstation with the foreign server, do not include address packet
                            self.conversations.insert(node_id, ForeignNode {
                                send_client: ForeignNodeContact::client(node_id, db.clone()).await?,
                                conversation: Vec::new()
                            });

                            if let Some(mut_ref) = self.conversations.get_mut(&node_id) {
                                let _ = mut_ref.send_client.send(self.incoming.get_address().node_id.to_string().into_bytes(), PacketType::Address).await;
                                if let Some(username) = self.username.as_ref() {
                                     let _ = mut_ref.send_client.send(username.as_bytes().to_vec(), PacketType::Username).await;
                                }
                            }

                            return Ok(Some(NetworkOutput::AddChat(Contact::from_node_id(node_id))));
                        }
                    }
                }
            }
        }

        Ok(None)

    }

    /// Send a message to a target address, forming a connection if it does not already exist to their server.
    /// If a new foreign node was successfuly spawned, Option<NodeId> contains the foreign address.
    pub async fn send_message(&mut self, recipient: NodeId, packet: Vec<u8>, packet_type: PacketType, db: &DataLink) -> Res<Option<NodeId>> {

        let new_node = if let Some(mut_ref) = self.conversations.get_mut(&recipient) {
            mut_ref.send_client.send(packet.clone(), packet_type).await?;
            mut_ref.conversation.push(Packet {
                author: self.incoming.get_address().node_id,
                content: Ok(packet),
                packet_type
            });
            None
        } else {
            self.conversations.insert(recipient, ForeignNode {
                send_client: ForeignNodeContact::client(recipient, db.clone()).await?,
                conversation: vec![Packet {
                    author: self.incoming.get_address().node_id,
                    content: Ok(packet.clone()),
                    packet_type
                }]
            });

            if let Some(mut_ref) = self.conversations.get_mut(&recipient) {
                let _ = mut_ref.send_client.send(self.incoming.get_address().node_id.to_string().into_bytes(), PacketType::Address).await;
                if let Some(username) = self.username.as_ref() {
                     let _ = mut_ref.send_client.send(username.as_bytes().to_vec(), PacketType::Username).await;
                }
                let _ = mut_ref.send_client.send(packet, packet_type).await;
            }

            Some(recipient)
        };

        Ok(new_node)
    }
}
