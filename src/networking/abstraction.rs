use std::collections::HashMap;

use iroh::NodeId;

use crate::networking::network::ForeignNodeContact;

use super::packet::Packet;

pub struct ForeignNode {
    send_client: ForeignNodeContact,
    conversation: Vec<Packet>
}

pub struct Network {
    sender_clients: HashMap<NodeId, ForeignNodeContact>,
}
