mod composition;
mod icons;
mod track;
mod trigram;

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
        widget::{
            column,
            container,
        },
    },
};

const FILL_PORTION_PLAYBACK: u16 = 2;
const FILL_PORTION_TRACK_LIST: u16 = 5;

fn main() -> Result {
    iced::application(Prism::new, Prism::update, Prism::view)
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
                playback::Event::Next => {
                    self.update(Message::TrackList(track_list::Message::TrackActivateNext))
                }
                playback::Event::None => Task::none(),
                playback::Event::Previous => self.update(Message::TrackList(
                    track_list::Message::TrackActivatePrevious,
                )),
            },
            Message::TrackList(message) => match self.track_list.update(message) {
                track_list::Event::None => Task::none(),
                track_list::Event::Performed(task) => task.map(Message::TrackList),
                track_list::Event::TrackActivated(track) => {
                    let _ = self.playback.update(playback::Message::TrackPlay(track));
                    Task::none()
                }
            },
        }
    }

    fn view(&self) -> Element<'_, Message> {
        column![
            container(self.playback.view().map(Message::Playback))
                .height(Length::FillPortion(FILL_PORTION_PLAYBACK)),
            container(self.track_list.view().map(Message::TrackList))
                .height(Length::FillPortion(FILL_PORTION_TRACK_LIST)),
        ]
        .height(Length::Fill)
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
