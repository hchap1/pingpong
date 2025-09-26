use iced::Task;
use iroh::NodeId;

use crate::{error::Error, networking::{abstraction::NetworkTask, packet::Packet}};

#[derive(Clone, Debug)]
pub enum Message {
    None,
    Global(Global),
    Chat(Chat),
    Add(Add)
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
    Warn(Error),
    Load(PageType),
    NetworkTask(NetworkTask),
    AddChat(NodeId)
}

#[derive(Clone, Debug)]
pub enum Chat {
    MessageBox(String),
    AddPacketToCache(Packet),
    SetConversation(Vec<Packet>)
}

#[derive(Clone, Debug)]
pub enum Add {
    InputBox(String),
    Submit
}

#[derive(Clone, Debug)]
pub enum PageType {
    AddChat,
    Chat(NodeId)
}
