use {
    crate::{
        icons,
        track,
        track::Track,
    },
    futures::channel::mpsc::{
        UnboundedReceiver,
        UnboundedSender,
        unbounded,
    },
    iced::{
        Alignment,
        ContentFit,
        Element,
        Event::Keyboard,
        Length,
        Subscription,
        event,
        event::Status,
        keyboard::{
            Event::KeyPressed,
            Key,
            key::Named,
        },
        time,
        widget::{
            Button,
            Space,
            button,
            center,
            column,
            container,
            image,
            row,
            slider,
            svg,
            text,
        },
    },
    rodio::{
        Decoder,
        DeviceSinkBuilder,
        MixerDeviceSink,
        Player,
        Source,
        source::EmptyCallback,
    },
    std::{
        fs::File,
        hash,
        path::Path,
        sync::{
            Arc,
            Mutex,
        },
        time::Duration,
    },
};

const BUTTON_SIZE: u32 = 32;
const ICON_SIZE: u32 = 16;
const SEEKBAR_MINIMUM: f32 = 0.0;
const SEEKBAR_STEP: f32 = 0.001;
const SEEKBAR_TICK_INTERVAL: Duration = Duration::from_millis(16);
const VOLUME_MAXIMUM: f32 = 1.0;
const VOLUME_MINIMUM: f32 = 0.0;
const VOLUME_STEP: f32 = 0.01;
const VOLUME_WIDTH: u32 = 100;

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
            cover: None,
            handle: DeviceSinkBuilder::open_default_sink().unwrap(),
            player: None,
            seek_position: None,
            track: None,
            track_end_receiver: TrackEndReceiver(Arc::new(Mutex::new(Some(receiver)))),
            track_end_sender: sender,
            volume: VOLUME_MAXIMUM,
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let keyboard_subscription = event::listen_with(|event, status, _window| match event {
            Keyboard(KeyPressed {
                key: Key::Named(Named::Space),
                ..
            }) if status == Status::Ignored => Some(Message::ButtonPauseOrPlayPress),
            _ => None,
        });
        let slider_seekbar_subscription =
            time::every(SEEKBAR_TICK_INTERVAL).map(|_| Message::SliderSeekbarTick);
        let track_end_subscription =
            Subscription::run_with(self.track_end_receiver.clone(), on_track_end);
        Subscription::batch([
            keyboard_subscription,
            slider_seekbar_subscription,
            track_end_subscription,
        ])
    }

    #[must_use]
    pub fn update(&mut self, message: Message) -> Event {
        match message {
            Message::ButtonNextPress => Event::TrackActivateNext,
            Message::ButtonPauseOrPlayPress => {
                if let Some(player) = &self.player {
                    if player.is_paused() {
                        player.play();
                    } else {
                        player.pause();
                    }
                }
                Event::None
            }
            Message::ButtonPreviousPress => Event::TrackActivatePrevious,
            Message::SliderSeekbarMouseDrag(position) => {
                self.seek_position = Some(position);
                Event::None
            }
            Message::SliderSeekbarMouseRelease => {
                if let (Some(position), Some(player)) = (self.seek_position.take(), &self.player) {
                    let _ = player.try_seek(Duration::from_secs_f32(position));
                }
                Event::None
            }
            Message::SliderSeekbarTick => Event::None,
            Message::SliderVolumeChange(volume) => {
                self.volume = volume;
                if let Some(player) = &self.player {
                    player.set_volume(volume);
                }
                Event::None
            }
            Message::TrackPlay(track) => {
                let Ok(file) = File::open(&track.file_path) else {
                    return Event::None;
                };
                let Ok(decoder) = Decoder::try_from(file) else {
                    return Event::None;
                };
                let player = Player::connect_new(self.handle.mixer());
                player.set_volume(self.volume);
                let sender = self.track_end_sender.clone();
                player.append(decoder.amplify_decibel(track.replay_gain));
                player.append(EmptyCallback::new(Box::new(move || {
                    let _ = sender.unbounded_send(Message::ButtonNextPress);
                })));
                self.player = Some(player);
                self.cover = track::cover_from_file(Path::new(&track.file_path))
                    .map(image::Handle::from_bytes);
                self.track = Some(track);
                Event::None
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let playing = self
            .player
            .as_ref()
            .is_some_and(|player| !player.is_paused());
        let pause_or_play_icon =
            svg::Handle::from_memory(if playing { icons::PAUSE } else { icons::PLAY });
        let controls = container(row![
            icon_button(svg::Handle::from_memory(icons::PREVIOUS))
                .on_press(Message::ButtonPreviousPress),
            icon_button(pause_or_play_icon).on_press(Message::ButtonPauseOrPlayPress),
            icon_button(svg::Handle::from_memory(icons::NEXT)).on_press(Message::ButtonNextPress),
            slider(
                VOLUME_MINIMUM..=VOLUME_MAXIMUM,
                self.volume,
                Message::SliderVolumeChange,
            )
            .step(VOLUME_STEP)
            .width(VOLUME_WIDTH),
        ])
        .align_x(Alignment::Center)
        .width(Length::Fill);

        let mut cover_and_information = row![].height(Length::Fill);
        if let Some(cover) = &self.cover {
            cover_and_information = cover_and_information.push(
                image(cover.clone())
                    .content_fit(ContentFit::Contain)
                    .height(Length::Fill),
            );
        }
        if let Some(track) = &self.track {
            cover_and_information = cover_and_information.push(
                column![]
                    .height(Length::Fill)
                    .push(Space::new().height(Length::Fill))
                    .push(text(&track.title))
                    .push(Space::new().height(Length::Fill))
                    .push(text(&track.artist))
                    .push(Space::new().height(Length::Fill))
                    .push(text(&track.album))
                    .push(Space::new().height(Length::Fill)),
            );
        }

        let duration = self
            .track
            .as_ref()
            .and_then(|track| track.duration)
            .unwrap_or(SEEKBAR_MINIMUM);
        let position = self.seek_position.unwrap_or_else(|| {
            self.player
                .as_ref()
                .map(|player| player.get_pos().as_secs_f32())
                .unwrap_or(SEEKBAR_MINIMUM)
        });
        let slider_seekbar = slider(
            SEEKBAR_MINIMUM..=duration,
            position,
            Message::SliderSeekbarMouseDrag,
        )
        .on_release(Message::SliderSeekbarMouseRelease)
        .step(SEEKBAR_STEP)
        .width(Length::Fill);

        column![cover_and_information, controls, slider_seekbar].into()
    }
}

pub enum Event {
    None,
    TrackActivateNext,
    TrackActivatePrevious,
}

#[derive(Clone, Debug)]
pub enum Message {
    ButtonNextPress,
    ButtonPauseOrPlayPress,
    ButtonPreviousPress,
    SliderSeekbarMouseDrag(f32),
    SliderSeekbarMouseRelease,
    SliderSeekbarTick,
    SliderVolumeChange(f32),
    TrackPlay(Track),
}

pub struct Playback {
    cover: Option<image::Handle>,
    handle: MixerDeviceSink,
    player: Option<Player>,
    seek_position: Option<f32>,
    track: Option<Track>,
    track_end_receiver: TrackEndReceiver,
    track_end_sender: UnboundedSender<Message>,
    volume: f32,
}

#[derive(Clone, Debug)]
struct TrackEndReceiver(Arc<Mutex<Option<UnboundedReceiver<Message>>>>);
