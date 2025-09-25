use iced::Task;

#[derive(Clone, Debug)]
pub enum Message {
    None,
    Global(Global)
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

}

#[derive(Clone, Debug)]
pub enum PageType {
    SelectAddress,
    MessageAddress
}
