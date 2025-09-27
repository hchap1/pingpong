use iroh::NodeId;

use crate::error::{Error, Res};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PacketType {
    Error,
    String,
    Address,
    Username,
}

impl PacketType {
    pub fn from_u8(n: u8) -> Self {
        match n {
            1 => Self::String,
            2 => Self::Address,
            3 => Self::Username,
            _ => Self::Error
        }
    }

    pub fn to_u8(self) -> u8 {
        match self {
            Self::String => 1,
            Self::Address => 2,
            Self::Username => 3,
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
    pub fn success(author: NodeId, content: Vec<u8>) -> Vec<Self> {
        let mut packets = Vec::new();
        let mut buf = content.as_slice();

        while buf.len() >= 5 {
            let packet_type = PacketType::from_u8(buf[0]);
            let len_bytes: [u8; 4] = buf[1..5].try_into().unwrap();
            let payload_len = u32::from_be_bytes(len_bytes) as usize;

            if buf.len() < 5 + payload_len {
                break;
            }

            let payload = buf[5..5 + payload_len].to_vec();

            packets.push(Self {
                author,
                content: Ok(payload),
                packet_type,
            });

            buf = &buf[5 + payload_len..];
        }

        packets
    }

    pub fn failure(author: NodeId, error: Error) -> Self {
        Self { author, content: Err(error), packet_type: PacketType::Error }
    }
}
