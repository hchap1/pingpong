use std::collections::HashMap;

use async_channel::Receiver;
use iroh::NodeId;

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

impl Network {

    /// Yield a receiver that receives all messages. The implementation is responsible for adding this into the conversation synchronously.
    pub fn yield_receiver(&self) -> Receiver<Packet> {
        self.incoming.yield_receiver()
    }

    /// Asynchronously add a message into the conversation stack, spawning a new foreign node if required.
    pub async fn add_message(&mut self, packet: Packet) {
        
        if let Some(mut_ref) = self.conversations.get_mut(&packet.author) {
            mut_ref.conversation.push(packet);
        } else {

        }

    }
}
