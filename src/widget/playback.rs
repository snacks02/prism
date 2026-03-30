use crate::track::Track;
use futures::channel::mpsc::{
    UnboundedReceiver,
    UnboundedSender,
    unbounded,
};
use iced::keyboard;
use iced::widget::{
    Button,
    button,
    center,
    row,
    svg,
};
use iced::{
    ContentFit,
    Element,
    Subscription,
    event,
};
use rodio::source::EmptyCallback;
use rodio::{
    Decoder,
    DeviceSinkBuilder,
    MixerDeviceSink,
    Player,
};
use std::fs::File;
use std::hash;
use std::sync::{
    Arc,
    Mutex,
};

const BUTTON_SIZE: u32 = 32;
const ICON_NEXT_PATH: &str = "icons/next.svg";
const ICON_PAUSE_PATH: &str = "icons/pause.svg";
const ICON_PLAY_PATH: &str = "icons/play.svg";
const ICON_PREVIOUS_PATH: &str = "icons/previous.svg";
const ICON_SIZE: u32 = 16;

fn icon_button<'a>(icon: svg::Handle) -> Button<'a, Message> {
    button(center(
        svg(icon)
            .content_fit(ContentFit::Fill)
            .height(ICON_SIZE)
            .width(ICON_SIZE),
    ))
    .height(BUTTON_SIZE)
    .padding(0)
    .width(BUTTON_SIZE)
}

fn on_track_end(data: &TrackEndReceiver) -> UnboundedReceiver<Message> {
    data.0.lock().unwrap().take().unwrap()
}

impl hash::Hash for TrackEndReceiver {
    fn hash<Hasher: hash::Hasher>(&self, state: &mut Hasher) {
        Arc::as_ptr(&self.0).hash(state);
    }
}

impl Playback {
    pub fn new() -> Self {
        let (sender, receiver) = unbounded::<Message>();
        Self {
            handle: DeviceSinkBuilder::open_default_sink().unwrap(),
            player: None,
            track_end_receiver: TrackEndReceiver(Arc::new(Mutex::new(Some(receiver)))),
            track_end_sender: sender,
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let keyboard_subscription = event::listen_with(|event, _status, _window| match event {
            iced::Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Space),
                ..
            }) => Some(Message::Pause),
            _ => None,
        });
        let track_end_subscription =
            Subscription::run_with(self.track_end_receiver.clone(), on_track_end);
        Subscription::batch([keyboard_subscription, track_end_subscription])
    }

    #[must_use]
    pub fn update(&mut self, message: Message) -> Event {
        match message {
            Message::Next => Event::Next,
            Message::Pause => {
                if let Some(player) = &self.player {
                    if player.is_paused() {
                        player.play();
                    } else {
                        player.pause();
                    }
                }
                Event::None
            }
            Message::Play(track) => {
                let Ok(file) = File::open(&track.file_path) else {
                    return Event::None;
                };
                let Ok(decoder) = Decoder::try_from(file) else {
                    return Event::None;
                };
                let player = Player::connect_new(self.handle.mixer());
                let sender = self.track_end_sender.clone();
                player.append(decoder);
                player.append(EmptyCallback::new(Box::new(move || {
                    let _ = sender.unbounded_send(Message::Next);
                })));
                self.player = Some(player);
                Event::None
            }
            Message::Previous => Event::Previous,
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let playing = self
            .player
            .as_ref()
            .is_some_and(|player| !player.is_paused());
        let icon = if playing {
            svg::Handle::from_path(ICON_PAUSE_PATH)
        } else {
            svg::Handle::from_path(ICON_PLAY_PATH)
        };
        row![
            icon_button(svg::Handle::from_path(ICON_PREVIOUS_PATH)).on_press(Message::Previous),
            icon_button(icon).on_press(Message::Pause),
            icon_button(svg::Handle::from_path(ICON_NEXT_PATH)).on_press(Message::Next),
        ]
        .into()
    }
}

pub enum Event {
    Next,
    None,
    Previous,
}

#[derive(Clone, Debug)]
pub enum Message {
    Next,
    Pause,
    Play(Track),
    Previous,
}

pub struct Playback {
    handle: MixerDeviceSink,
    player: Option<Player>,
    track_end_receiver: TrackEndReceiver,
    track_end_sender: UnboundedSender<Message>,
}

#[derive(Clone, Debug)]
struct TrackEndReceiver(Arc<Mutex<Option<UnboundedReceiver<Message>>>>);
