use {
    composition::{
        Composition,
        playback,
        track_list,
    },
    iced::{
        Color,
        Element,
        Length,
        Result,
        Settings,
        Subscription,
        Task,
        Theme,
        theme::palette::Seed,
        widget::column,
    },
};

mod audio_player;
mod composition;
mod icon;
mod queue;
mod style;
mod track;
mod track_read;
mod view_helper;

const DEFAULT_TEXT_SIZE: f32 = 14.0;

fn main() -> Result {
    iced::application(Prism::new, Prism::update, Prism::view)
        .settings(Settings {
            default_text_size: DEFAULT_TEXT_SIZE.into(),
            ..Default::default()
        })
        .subscription(Prism::subscription)
        .theme(Prism::theme)
        .title("Prism")
        .run()
}

impl Composition for Prism {
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

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Playback(message) => match self.playback.update(message) {
                playback::Event::AccentColorChange(color) => {
                    self.color_accent = color;
                    Task::none()
                }
                playback::Event::None => Task::none(),
                playback::Event::TrackPlay(task, track) => Task::batch([
                    task.map(Message::Playback),
                    Task::done(Message::TrackList(track_list::Message::TrackPlay(track))),
                ]),
            },
            Message::TrackList(message) => match self.track_list.update(message) {
                track_list::Event::None => Task::none(),
                track_list::Event::QueueExtend(tracks) => {
                    Task::done(Message::Playback(playback::Message::QueueExtend(tracks)))
                }
                track_list::Event::QueueSetCurrent(track) => {
                    Task::done(Message::Playback(playback::Message::QueueSetCurrent(track)))
                }
                track_list::Event::TaskPerform(task) => task.map(Message::TrackList),
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

    type Event = Task<Message>;

    type Message = Message;
}

impl Prism {
    fn theme(&self) -> Theme {
        Theme::custom(
            "Prism".to_string(),
            Seed {
                background: style::COLOR_BACKGROUND,
                primary: self.color_accent,
                text: style::COLOR_GRAY_4,
                ..Seed::DARK
            },
        )
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
