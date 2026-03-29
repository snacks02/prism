use crate::track::Track;
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
};
use rodio::{
    Decoder,
    DeviceSinkBuilder,
    MixerDeviceSink,
    Player,
};
use std::fs::File;

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

impl Playback {
    pub fn new() -> Self {
        Self {
            handle: DeviceSinkBuilder::open_default_sink().unwrap(),
            player: None,
        }
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
                player.append(decoder);
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
}
