use iroh::NodeId;

use crate::error::{Error, Res};

pub struct Packet {
    pub author: NodeId,
    pub content: Res<Vec<u8>>
}

impl Packet {
    pub fn success(author: NodeId, content: Vec<u8>) -> Self {
        Self { author, content: Ok(content) }
    }

    pub fn failure(author: NodeId, error: Error) -> Self {
        Self { author, content: Err(error) }
    }
}
