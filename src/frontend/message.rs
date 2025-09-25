use iced::Task;
use iroh::NodeId;

use crate::{error::Error, networking::packet::Packet};

#[derive(Clone, Debug)]
pub enum Message {
    None,
    Global(Global),
    Chat(Chat)
}

impl Message {
    pub fn task(self) -> Task<Self> {
        match self {
            Self::None => Task::none(),
            other => Task::done(other)
        }
    }
}

#[derive(Clone, Debug)]
pub enum Global {
    StartNetworkRelays,
    Warn(Error)
}

#[derive(Clone, Debug)]
pub enum Chat {
    MessageBox(String),
    AddPacketToCache(Packet),
    SetConversation(Vec<Packet>)
}

#[derive(Clone, Debug)]
pub enum PageType {
    AddChat,
    Chat(NodeId)
}
