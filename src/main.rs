mod composition;
mod icon;
mod style;
mod track_import;
mod trigram;
mod view_helper;

use {
    composition::{
        playback,
        track_list,
    },
    iced::{
        Color,
        Element,
        Length,
        Result,
        Subscription,
        Task,
        Theme,
        theme,
        widget::{
            column,
            container,
        },
    },
};

fn main() -> Result {
    iced::application(Prism::new, Prism::update, Prism::view)
        .settings(iced::Settings {
            default_text_size: 14.0.into(),
            ..Default::default()
        })
        .subscription(Prism::subscription)
        .theme(Prism::theme)
        .title("Prism")
        .run()
}

impl Prism {
    fn new() -> Self {
        Self {
            color_accent: style::COLOR_ACCENT,
            playback: playback::Playback::new(),
            track_list: track_list::TrackList::new(),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            self.playback.subscription().map(Message::Playback),
            self.track_list.subscription().map(Message::TrackList),
        ])
    }

    fn theme(&self) -> Theme {
        Theme::custom(
            String::from("Prism"),
            theme::palette::Seed {
                background: style::COLOR_BACKGROUND,
                primary: self.color_accent,
                text: style::COLOR_GRAY_4,
                ..theme::palette::Seed::DARK
            },
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Playback(message) => match self.playback.update(message) {
                playback::Event::AccentColorChange(color) => {
                    self.color_accent = color;
                    Task::none()
                }
                playback::Event::None => Task::none(),
                playback::Event::TaskPerform(task) => task.map(Message::Playback),
                playback::Event::TrackActivateNext => {
                    self.update(Message::TrackList(track_list::Message::TrackActivateNext))
                }
                playback::Event::TrackActivatePrevious => self.update(Message::TrackList(
                    track_list::Message::TrackActivatePrevious,
                )),
            },
            Message::TrackList(message) => match self.track_list.update(message) {
                track_list::Event::None => Task::none(),
                track_list::Event::TaskPerform(task) => task.map(Message::TrackList),
                track_list::Event::TrackActivated(track) => {
                    match self.playback.update(playback::Message::TrackPlay(track)) {
                        playback::Event::TaskPerform(task) => task.map(Message::Playback),
                        _ => Task::none(),
                    }
                }
            },
        }
    }

    fn view(&self) -> Element<'_, Message> {
        column![
            container(self.playback.view().map(Message::Playback)).height(Length::Shrink),
            container(self.track_list.view().map(Message::TrackList)).height(Length::Fill),
        ]
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    Playback(playback::Message),
    TrackList(track_list::Message),
}

struct Prism {
    color_accent: Color,
    playback: playback::Playback,
    track_list: track_list::TrackList,
}
