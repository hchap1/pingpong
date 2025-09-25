use iroh::NodeAddr;

use crate::error::Res;

pub struct Packet {
    pub author: NodeAddr,
    pub content: Res<Vec<u8>>
}

impl Packet {
    pub fn new(author: NodeAddr, content: Res<Vec<u8>>) -> Self {
        Self { author, content }
    }
}
