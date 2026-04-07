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
        Element,
        Length,
        Result,
        Subscription,
        Task,
        theme,
        widget::{
            column,
            container,
            container::Style,
        },
    },
};

fn main() -> Result {
    iced::application(Prism::new, Prism::update, Prism::view)
        .settings(iced::Settings {
            default_text_size: 14.0.into(),
            ..Default::default()
        })
        .style(|_state, theme| theme::Style {
            text_color: style::COLOR_GRAY_5,
            ..theme::default(theme)
        })
        .subscription(Prism::subscription)
        .title("Prism")
        .run()
}

impl Prism {
    fn new() -> Self {
        Self {
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
                playback::Event::None => Task::none(),
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
                    let _ = self.playback.update(playback::Message::TrackPlay(track));
                    Task::none()
                }
            },
        }
    }

    fn view(&self) -> Element<'_, Message> {
        container(
            column![
                container(self.playback.view().map(Message::Playback)).height(Length::Shrink),
                container(self.track_list.view().map(Message::TrackList)).height(Length::Fill),
            ]
            .height(Length::Fill),
        )
        .height(Length::Fill)
        .style(|_theme| Style {
            background: Some(style::COLOR_GRAY_1.into()),
            ..Default::default()
        })
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
    playback: playback::Playback,
    track_list: track_list::TrackList,
}
