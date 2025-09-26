use frontend::{application::Application, message::{Global, Message}};

mod networking;
mod frontend;
mod backend;
mod error;

fn main() -> iced::Result {

    /*

        Current issue:

        server/client ID are not the same, so they must be stored in pairs.
        When a connection is requested to a server, the client requesting the connection must send the ID of its server so that it may be messaged back.

    */
    iced::application("Pingpong", Application::update, Application::view)
        .run_with(|| (Application::default(), Message::Global(Global::StartNetworkRelays).task()))
}
