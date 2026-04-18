mod audio_player;
mod composition;
mod icon;
mod style;
mod track;
mod track_read;
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
        widget::column,
    },
};

const DEFAULT_TEXT_SIZE: f32 = 14.0;

fn main() -> Result {
    iced::application(Prism::new, Prism::update, Prism::view)
        .settings(iced::Settings {
            default_text_size: DEFAULT_TEXT_SIZE.into(),
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
                    Task::done(Message::TrackList(track_list::Message::TrackActivateNext))
                }
                playback::Event::TrackActivatePrevious => Task::done(Message::TrackList(
                    track_list::Message::TrackActivatePrevious,
                )),
            },
            Message::TrackList(message) => match self.track_list.update(message) {
                track_list::Event::None => Task::none(),
                track_list::Event::TaskPerform(task) => task.map(Message::TrackList),
                track_list::Event::TrackActivate(track) => {
                    Task::done(Message::Playback(playback::Message::TrackPlay(track)))
                }
            },
        }
    }

    fn view(&self) -> Element<'_, Message> {
        column![
            self.playback.view().map(Message::Playback),
            self.track_list.view().map(Message::TrackList),
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
