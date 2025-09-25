use crate::frontend::message::Message;

use iced::{Element, Task};

pub trait Page {
    fn view(&self) -> Element<Message>;
    fn update(&mut self, message: Message) -> Task<Message>;
}

pub struct Application {
    page: Box<dyn Page>
}

impl Application {
    pub fn view(&self) -> Element<Message> {
        
    }

    pub fn update(&mut self, message: Message) {

    }
}
