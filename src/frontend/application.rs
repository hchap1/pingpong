use crate::backend::database::Database;
use crate::backend::directory::Directory;
use crate::backend::relay::Relay;
use crate::error::Error;
use crate::networking::abstraction::{NetworkOutput, NetworkTask};
use crate::{error::Res, frontend::message::Message, networking::abstraction::run_network};
use crate::frontend::message::Global;

use async_channel::{unbounded, Receiver, Sender};
use iced::widget::{button, Column, Row, Scrollable, text};
use iced::{Element, Length, Task};
use iroh::NodeId;
use tokio::{spawn, task::JoinHandle};

use super::message::{Chat, PageType};
use super::pages::add::AddPage;
use super::pages::chat::ChatPage;

pub trait Page {
    fn view(&self) -> Element<Message>;
    fn update(&mut self, message: Message) -> Task<Message>;
}

pub struct Application {
    root: Directory,
    database: Database,
    networking_task_sender: Sender<NetworkTask>,
    networking_output_receiver: Receiver<NetworkOutput>,
    _networker: JoinHandle<Res<()>>,
    page: Box<dyn Page>,

    active_chats: Vec<NodeId>
}

impl Application {
    pub fn view(&self) -> Element<Message> {
        Row::new()
            .push(
                Scrollable::new(
                    Column::from_iter(
                        self.active_chats.iter()
                            .map(|c|
                                button(text(format!("{}", c)))
                                    .on_press(Message::Global(Global::Load(PageType::Chat(*c))))
                                    .into()
                            )
                    ).width(Length::FillPortion(1))
                )
            ).push(
                self.page.view()
            ).into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Global(global) => match global {
                Global::StartNetworkRelays => Task::stream(
                    Relay::consume_receiver(
                        self.networking_output_receiver.clone(),
                        |o| match o {
                            NetworkOutput::AddPacket(packet) => Some(Message::Chat(Chat::AddPacketToCache(packet))),
                            NetworkOutput::ConversationRecord(packets) => Some(Message::Chat(Chat::SetConversation(packets))),
                            NetworkOutput::NonFatalError(e) => Some(Message::Global(Global::Warn(e))),
                            NetworkOutput::AddChat(c) => Some(Message::Global(Global::AddChat(c)))
                        }
                    )
                ),

                Global::Warn(e) => {
                    eprintln!("ERROR: {e:?}");
                    Message::None.task()
                },

                Global::Load(page_type) => {
                    match page_type {
                        PageType::Chat(node_id) => {
                            self.page = Box::new(ChatPage::new(node_id));
                            Message::Global(Global::NetworkTask(NetworkTask::RequestConversation(node_id))).task()
                        },

                        PageType::AddChat => {
                            self.page = Box::new(AddPage::default());
                            Message::None.task()
                        }
                    }
                },

                Global::AddChat(node_id) => {
                    self.active_chats.push(node_id);
                    Message::None.task()
                },

                Global::NetworkTask(task) => {
                    match self.networking_task_sender.send_blocking(task) {
                        Ok(_) => Message::None.task(),
                        Err(_) => Message::Global(Global::Warn(Error::MPMCRecvError)).task()
                    }
                }
            },

            Message::None => Message::None.task(),
            page_specific => self.page.update(page_specific)
        }
    }
}

impl Default for Application {
    fn default() -> Self {

        let root = Directory::create_or_load().expect("[FATAL] Cannot proceed without creating directory, which failed.");
        let (task_sender, task_receiver) = unbounded();
        let (output_sender, output_receiver) = unbounded();
        let database = Database::new(root.get());
        let db = database.derive();

        Self {
            root: root.clone(),
            database,
            networking_task_sender: task_sender,
            networking_output_receiver: output_receiver,
            _networker: spawn(run_network(task_receiver, output_sender, db)),
            page: Box::new(AddPage::default()),
            active_chats: Vec::new()
        }
    }
}
