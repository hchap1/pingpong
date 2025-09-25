use crate::backend::relay::Relay;
use crate::networking::abstraction::{NetworkOutput, NetworkTask};
use crate::{error::Res, frontend::message::Message, networking::abstraction::run_network};
use crate::frontend::message::Global;

use async_channel::{unbounded, Receiver, Sender};
use iced::{Element, Task};
use tokio::{spawn, task::JoinHandle};

use super::message::Chat;

pub trait Page {
    fn view(&self) -> Element<Message>;
    fn update(&mut self, message: Message) -> Task<Message>;
}

pub struct Application {
    networking_task_sender: Sender<NetworkTask>,
    networking_output_receiver: Receiver<NetworkOutput>,
    networker: JoinHandle<Res<()>>,
    page: Box<dyn Page>
}

impl Application {
    pub fn view(&self) -> Element<Message> {
        
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Global(global) => match global {
                Global::StartNetworkRelays => Task::stream(
                    Relay::consume_receiver(
                        self.networking_output_receiver,
                        |o| match o {
                            NetworkOutput::AddPacket(packet) => Some(Message::Chat(Chat::AddPacketToCache(packet))),
                            NetworkOutput::ConversationRecord(packets) => Some(Message::Chat(Chat::SetConversation(packets))),
                            NetworkOutput::NonFatalError(e) => Some(Message::Global(Global::Warn(e)))
                        }
                    )
                ),

                Global::Warn(e) => {
                    eprintln!("ERROR: {e:?}");
                    Message::None.task()
                }
            },

            Message::None => Message::None.task(),
            page_specific => self.page.update(page_specific)
        }
    }
}

impl Default for Application {
    fn default() -> Self {

        let (task_sender, task_receiver) = unbounded();
        let (output_sender, output_receiver) = unbounded();

        Self {
            networking_task_sender: task_sender,
            networking_output_receiver: output_receiver,
            networker: spawn(run_network(task_receiver, output_sender)),
        }
    }
}
