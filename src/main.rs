use frontend::{application::Application, message::{Global, Message}};
use iced::Task;

mod networking;
mod frontend;
mod backend;
mod error;

fn main() -> iced::Result {
    iced::application("Pingpong", Application::update, Application::view)
        .run_with(|| (
                Application::default(),
                Task::batch(vec![
                    Message::Global(Global::StartNetworkRelays).task(),
                    Message::Global(Global::LoadContacts).task()
                ])
        ))
}
