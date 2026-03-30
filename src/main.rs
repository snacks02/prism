mod track;
mod widget;

use iced::widget::column;
use iced::{
    Element,
    Subscription,
    Task,
};
use widget::{
    playback,
    toolbar,
    track_list,
};

fn main() -> iced::Result {
    iced::application(Prism::new, Prism::update, Prism::view)
        .subscription(Prism::subscription)
        .title("Prism")
        .run()
}

impl Prism {
    fn new() -> Self {
        Self {
            playback: playback::Playback::new(),
            toolbar: toolbar::Toolbar::new(),
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
                    self.update(Message::TrackList(track_list::Message::NextPress))
                }
                playback::Event::None => Task::none(),
                playback::Event::Previous => {
                    self.update(Message::TrackList(track_list::Message::PreviousPress))
                }
            },
            Message::Toolbar(message) => match self.toolbar.update(message) {
                toolbar::Event::None => Task::none(),
                toolbar::Event::Performed(task) => task.map(Message::Toolbar),
                toolbar::Event::TrackListExtended(tracks) => {
                    let _ = self
                        .track_list
                        .update(track_list::Message::TrackListExtend(tracks));
                    Task::none()
                }
            },
            Message::TrackList(message) => match self.track_list.update(message) {
                track_list::Event::None => Task::none(),
                track_list::Event::TrackActivated(track) => {
                    let _ = self.playback.update(playback::Message::Play(track));
                    Task::none()
                }
            },
        }
    }

    fn view(&self) -> Element<'_, Message> {
        column![
            self.playback.view().map(Message::Playback),
            self.toolbar.view().map(Message::Toolbar),
            self.track_list.view().map(Message::TrackList),
        ]
        .into()
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    Playback(playback::Message),
    Toolbar(toolbar::Message),
    TrackList(track_list::Message),
}

struct Prism {
    playback: playback::Playback,
    toolbar: toolbar::Toolbar,
    track_list: track_list::TrackList,
}
