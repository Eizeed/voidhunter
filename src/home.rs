use iced::{
    alignment::Vertical,
    widget::{button, column, text},
    Element, Task,
};

pub enum Action {
    Run(Task<Message>),
    StartGame,
    None,
}

#[derive(Debug, Clone)]
pub enum Message {
    StartGame,
}

pub struct Home;

impl Home {
    pub fn new() -> Self {
        Home
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::StartGame => Action::StartGame,
        }
    }

    pub fn view(&self) -> Element<Message> {
        let text = text("Home page").size(20).align_y(Vertical::Center).into();
        let button = button("Start game").on_press(Message::StartGame).into();

        column(vec![text, button]).into()
    }
}
