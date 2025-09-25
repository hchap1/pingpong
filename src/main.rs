use frontend::{application::Application, message::{Global, Message}};

mod networking;
mod frontend;
mod backend;
mod error;

fn main() -> iced::Result {
    iced::application("Pingpong", Application::update, Application::view)
        .run_with(|| (Application::default(), Message::Global(Global::StartNetworkRelays).task()))
}
