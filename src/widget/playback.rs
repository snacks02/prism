use iced::Element;
use iced::widget::{
    container,
    text,
};

impl Playback {
    pub fn new() -> Self {
        Self {}
    }

    #[must_use]
    pub fn update(&mut self, message: Message) -> Event {
        match message {}
    }

    pub fn view(&self) -> Element<'_, Message> {
        container(text("Top")).into()
    }
}

pub enum Event {
    None,
}

#[derive(Clone, Debug)]
pub enum Message {}

pub struct Playback {}
