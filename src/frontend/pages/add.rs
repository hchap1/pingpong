use std::str::FromStr;

use iced::{widget::text_input, Element, Task};
use iroh::NodeId;

use crate::{error::Error, frontend::{application::Page, message::{Add, Global, Message}}, networking::{abstraction::NetworkTask, packet::PacketType}};

#[derive(Default)]
pub struct AddPage {
    id_input: String
}

impl Page for AddPage {
    fn view(&self) -> Element<Message> {
        text_input("Enter IROH Public Key", &self.id_input)
            .on_input(|v| Message::Add(Add::InputBox(v)))
            .on_submit(Message::Add(Add::Submit))
            .into()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        if let Message::Add(message) = message {
            match message {
                Add::InputBox(new_value) => {
                    self.id_input = new_value;
                    Message::None.task()
                }

                Add::Submit => {
                    let id = std::mem::take(&mut self.id_input);
                    match NodeId::from_str(&id) {
                        Ok(id) => Message::Global(Global::NetworkTask(
                            NetworkTask::SendMessage(id, b"Hello".to_vec(), PacketType::String)
                        )).task(),
                        Err(_) => Message::Global(Global::Warn(Error::NoSuchClient)).task()
                    }
                }
            }
        } else {
            Message::None.task()
        }
    }
}
