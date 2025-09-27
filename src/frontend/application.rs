use crate::backend::database::{Database, ItemStream};
use crate::backend::database_interface::DatabaseInterface;
use crate::backend::directory::Directory;
use crate::backend::relay::Relay;
use crate::error::Error;
use crate::networking::abstraction::{NetworkOutput, NetworkTask};
use crate::networking::contact::Contact;
use crate::{error::Res, frontend::message::Message, networking::abstraction::run_network};
use crate::frontend::message::Global;

use async_channel::{unbounded, Receiver, Sender};
use iced::widget::{button, text, text_input, Column, Row, Scrollable};
use iced::{Element, Length, Task};
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

    active_chats: Vec<Contact>,
    possible_chats: Vec<Contact>,
    username: Option<String>,
    username_input: String
}

impl Application {
    pub fn view(&self) -> Element<Message> {
        match self.username.as_ref() {
            Some(_) => Row::new()
                .push(
                    Column::new()
                        .push(
                            Scrollable::new(
                                Column::from_iter(
                                    self.active_chats.iter()
                                        .map(|c|
                                            Row::new()
                                                .push(
                                                    button(text(c.username.as_ref().unwrap_or(&c.server_address.to_string()).to_string()))
                                                        .on_press(Message::Global(Global::Load(PageType::Chat(c.server_address))))
                                                )
                                                .push(
                                                    button(text("ADD CONTACT"))
                                                        .on_press_maybe(
                                                            if self.possible_chats.iter().any(|ch| ch.server_address == c.server_address) {
                                                                None
                                                            } else {
                                                                Some(Message::Global(Global::AddContactToDatabase(c.clone())))
                                                            }
                                                        )
                                                ).into()
                                        )
                                ).width(Length::FillPortion(1))
                            ).height(Length::FillPortion(1))
                        ).push(
                            Scrollable::new(
                                Column::from_iter(
                                    self.possible_chats.iter()
                                        .map(|c|
                                            button(text(c.username.as_ref().unwrap_or(&c.server_address.to_string()).to_string()))
                                                .on_press(Message::Global(Global::AddChat(c.clone())))
                                                .into()
                                        )
                                ).width(Length::FillPortion(1))
                            ).height(Length::FillPortion(1))
                        )
                ).push(
                    self.page.view()
                ).into(),
            None => text_input("Enter username!", &self.username_input)
                .on_input(|v| Message::Global(Global::UsernameInput(v)))
                .on_submit(Message::Global(Global::UpdateUsername))
                .into()
        }
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
                            NetworkOutput::AddChat(c) => Some(Message::Global(Global::AddChat(c))),
                            NetworkOutput::ContactName(node_id, username) => Some(Message::Global(Global::ContactName(node_id, username)))
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

                Global::AddChat(contact) => {
                    self.active_chats.push(contact);
                    Message::None.task()
                },

                Global::NetworkTask(task) => {
                    match self.networking_task_sender.send_blocking(task) {
                        Ok(_) => Message::None.task(),
                        Err(_) => Message::Global(Global::Warn(Error::MPMCRecvError)).task()
                    }
                }

                Global::LoadContacts => {
                    Task::stream(Relay::consume_receiver(
                        DatabaseInterface::select_all_contacts(self.database.derive()),
                        |emmision| match emmision {
                            ItemStream::Value(row) => if let (Some(address), Some(username)) = (row.first(), row.get(1)) {
                                if let Ok(contact) = Contact::new(address.string(), username.string()) {
                                    Some(Message::Global(Global::DatabaseContactEmmision(contact)))
                                } else { Some(Message::None) }
                            } else { Some(Message::None) },
                            ItemStream::Error => None,
                            ItemStream::End => None
                        }
                    ))
                }

                Global::AddContactToDatabase(contact) => {
                    self.possible_chats.push(contact.clone());
                    DatabaseInterface::insert_contact(self.database.derive(), contact);
                    Message::None.task()
                }

                Global::DatabaseContactEmmision(contact) => {
                    self.possible_chats.push(contact);
                    Message::None.task()
                }

                Global::ContactName(addr, username) => {
                    for chat in &mut self.active_chats {
                        if chat.server_address == addr {
                            chat.username = Some(username);
                            break;
                        }
                    }
                    Message::None.task()
                }

                Global::UsernameInput(new_value) => {
                    self.username_input = new_value;
                    Message::None.task()
                }

                Global::UpdateUsername => {
                    self.username = Some(std::mem::take(&mut self.username_input));
                    DatabaseInterface::insert_username(self.database.derive(), self.username.as_ref().unwrap().clone());
                    Message::Global(Global::NetworkTask(NetworkTask::SetUsername(self.username.as_ref().unwrap().clone()))).task()
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

        let username = DatabaseInterface::select_username(database.derive());
        DatabaseInterface::make_tables_nonblocking(database.derive());

        Self {
            root: root.clone(),
            database,
            networking_task_sender: task_sender,
            networking_output_receiver: output_receiver,
            _networker: spawn(run_network(task_receiver, output_sender, db, username.clone())),
            page: Box::new(AddPage::default()),
            active_chats: Vec::new(),
            possible_chats: Vec::new(),
            username,
            username_input: String::default()
        }
    }
}
