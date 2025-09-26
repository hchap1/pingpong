use iced::{widget::{text, text_input, Column, Container, Scrollable}, Element, Length, Task};
use iroh::NodeId;

use crate::{frontend::{application::Page, message::{Chat, Global, Message}}, networking::{abstraction::NetworkTask, packet::{Packet, PacketType}}};

pub struct ChatPage {
    remote_id: NodeId,
    message_box: String,
    conversation: Vec<Packet>
}

impl ChatPage {
    pub fn new(remote_id: NodeId) -> Self {
        Self {
            remote_id,
            message_box: String::default(),
            conversation: Vec::default()
        }
    }
}

impl Page for ChatPage {
    fn view(&self) -> Element<Message> {

        Column::new()
            .push(text(self.remote_id.to_string()))
            .push(
                Container::new(Scrollable::new(Column::from_iter(
                    self.conversation.iter().map(
                        |p| text(format!("{:?}: {:?}",
                            if p.author == self.remote_id { "LOCAL" } else { "REMOTE" },
                            if p.packet_type == PacketType::String {
                                if let Ok(content) = p.content.clone() {
                                    match String::from_utf8(content) {
                                        Ok(message) => message,
                                        Err(_) => String::from("INVALID UTF-8")
                                    }
                                } else { String::from("EMPTY") }
                            } else { String::from("NOT A STRING") }
                        )).into()
                    )
                ))).width(Length::Fill).height(Length::Fill))
            .push(
                text_input(&format!("Message {:?}", self.remote_id), &self.message_box)
                    .on_input(|v| Message::Chat(Chat::MessageBox(v)))
                    .on_submit(Message::Chat(Chat::SendMessage))
            ).into()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        if let Message::Chat(chat) = message {
            match chat {
                Chat::AddPacketToCache(packet) => self.conversation.push(packet),
                Chat::SetConversation(mut packets) => self.conversation.append(&mut packets),
                Chat::MessageBox(new_value) => self.message_box = new_value,
                Chat::SendMessage => return Message::Global(Global::NetworkTask(NetworkTask::SendMessage(
                    self.remote_id, std::mem::take(&mut self.message_box).into_bytes(), PacketType::String
                ))).task()
            }
        }
        Message::None.task()
    }
}
