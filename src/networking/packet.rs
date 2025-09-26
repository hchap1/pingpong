use iroh::NodeId;

use crate::error::{Error, Res};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PacketType {
    Error,
    String,
    Address
}

impl PacketType {
    pub fn from_u8(n: u8) -> Self {
        match n {
            1 => Self::String,
            2 => Self::Address
            _ => Self::Error
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            Self::String => 1,
            Self::Address => 2,
            _ => 0
        }
    }
}

#[derive(Debug, Clone)]
pub struct Packet {
    pub author: NodeId,
    pub content: Res<Vec<u8>>,
    pub packet_type: PacketType
}

impl Packet {
    pub fn success(author: NodeId, mut content: Vec<u8>) -> Self {
        if content.len() < 2 { return Self { author, content: Err(Error::StreamClosed), packet_type: PacketType::Error }; }
        let contents = content.split_off(1);
        Self { author, content: Ok(contents), packet_type: PacketType::from_u8(content[0]) }
    }

    pub fn failure(author: NodeId, error: Error) -> Self {
        Self { author, content: Err(error), packet_type: PacketType::Error }
    }
}
