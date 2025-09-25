use iroh::NodeAddr;

use crate::error::{Error, Res};

pub struct Packet {
    pub author: NodeAddr,
    pub content: Res<Vec<u8>>
}

impl Packet {
    pub fn success(author: NodeAddr, content: Vec<u8>) -> Self {
        Self { author, content: Ok(content) }
    }

    pub fn failure(author: NodeAddr, error: Error) -> Self {
        Self { author, content: Ok(content) }
    }
}
