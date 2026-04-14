use {
    crate::{
        composition::track_list::Track,
        icon,
        style,
        track_import,
        view_helper,
    },
    futures::channel::mpsc::{
        UnboundedReceiver,
        UnboundedSender,
        unbounded,
    },
    iced::{
        Alignment,
        Border,
        Color,
        Element,
        Event::Keyboard,
        Length,
        Padding,
        Subscription,
        Task,
        event,
        event::Status,
        keyboard::{
            Event::KeyPressed,
            Key,
            key::Named,
        },
        time,
        widget,
        widget::{
            center,
            column,
            container,
            container::Style,
            row,
            svg,
            text,
            text::{
                Ellipsis,
                Wrapping,
            },
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
        sync::{
            Arc,
            Mutex,
        },
        time::Duration,
    },
};

const BUTTON_SIZE: u32 = 40;
const COVER_BORDER_RADIUS: f32 = 10.0;
const COVER_BORDER_WIDTH: f32 = 4.0;
const COVER_ICON_SIZE: u32 = 48;
const COVER_SIZE: u32 = 200;
const PADDING_HORIZONTAL: f32 = 8.0;
const PADDING_VERTICAL: f32 = 16.0;
const SEEKBAR_SPACING: u32 = 8;
const SEEKBAR_STEP: f32 = 0.001;
const SEEKBAR_TICK_INTERVAL: Duration = Duration::from_millis(16);
const SPACING: u32 = 16;
const VOLUME_MAXIMUM: f32 = 1.0;
const VOLUME_STEP: f32 = 0.01;
const VOLUME_WIDTH: u32 = 88;

fn controls(playback: &Playback) -> Element<'_, Message> {
    let pause_or_play_icon = if playback
        .player
        .as_ref()
        .is_some_and(|player| !player.is_paused())
    {
        icon::PAUSE
    } else {
        icon::PLAY
    };
    center(
        row![
            view_helper::button(
                style::COLOR_GRAY_5,
                svg::Handle::from_memory(icon::SKIP_BACK),
                BUTTON_SIZE
            )
            .on_press(Message::ButtonPreviousPress),
            view_helper::button(
                Color::TRANSPARENT,
                svg::Handle::from_memory(pause_or_play_icon),
                BUTTON_SIZE
            )
            .style(|_, _| widget::button::Style {
                background: Some(style::COLOR_GRAY_5.into()),
                border: Border {
                    radius: f32::MAX.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .on_press(Message::ButtonPauseOrPlayPress),
            view_helper::button(
                style::COLOR_GRAY_5,
                svg::Handle::from_memory(icon::SKIP_FORWARD),
                BUTTON_SIZE
            )
            .on_press(Message::ButtonNextPress),
        ]
        .align_y(Alignment::Center),
    )
    .into()
}

fn cover(playback: &Playback) -> Element<'_, Message> {
    let container = if let Some(allocation) = &playback.cover_allocation {
        container(
            widget::image(allocation.handle().clone())
                .border_radius(COVER_BORDER_RADIUS)
                .height(COVER_SIZE)
                .width(COVER_SIZE),
        )
    } else {
        container(
            svg(svg::Handle::from_memory(icon::MUSIC))
                .height(COVER_ICON_SIZE)
                .style(|_theme, _status| svg::Style {
                    color: Some(style::COLOR_GRAY_3),
                })
                .width(COVER_ICON_SIZE),
        )
        .center(Length::Fill)
        .height(COVER_SIZE)
        .style(|_theme| Style {
            background: Some(style::COLOR_GRAY_2.into()),
            border: Border {
                color: style::COLOR_GRAY_3,
                radius: COVER_BORDER_RADIUS.into(),
                width: COVER_BORDER_WIDTH,
            },
            ..Default::default()
        })
        .width(COVER_SIZE)
    };

    center(container).into()
}

fn duration_format(seconds: f32) -> String {
    let total = seconds as u64;
    let seconds = total % 60;
    let minutes = (total / 60) % 60;
    let hours = total / 3600;
    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{:02}:{:02}", minutes, seconds)
    }
}

fn information(playback: &Playback) -> Element<'_, Message> {
    let value = if let Some(track) = &playback.track {
        format!(
            "{} :: {} :: {}",
            track.title_str(),
            track.artist_str(),
            track.album_str()
        )
    } else {
        "Select a track".to_string()
    };

    center(text(value).ellipsis(Ellipsis::End).wrapping(Wrapping::None)).into()
}

fn on_track_end(data: &TrackEndReceiver) -> UnboundedReceiver<Message> {
    data.0.lock().unwrap().take().unwrap()
}

fn seekbar(playback: &Playback) -> Element<'_, Message> {
    let duration = playback
        .track
        .as_ref()
        .and_then(|track| track.duration)
        .unwrap_or(0.0);
    let position = playback.seek_position.unwrap_or_else(|| {
        playback
            .player
            .as_ref()
            .map(|player| player.get_pos().as_secs_f32())
            .unwrap_or(0.0)
    });
    row![
        text(duration_format(position)),
        center(view_helper::slider(
            Message::SliderSeekbarMouseDrag,
            Some(Message::SliderSeekbarMouseRelease),
            0.0..=duration,
            SEEKBAR_STEP,
            position,
        )),
        text(duration_format(duration)),
    ]
    .align_y(Alignment::Center)
    .spacing(SEEKBAR_SPACING)
    .into()
}

fn volume(playback: &Playback) -> Element<'_, Message> {
    center(
        container(view_helper::slider(
            Message::SliderVolumeChange,
            None,
            0.0..=VOLUME_MAXIMUM,
            VOLUME_STEP,
            playback.volume,
        ))
        .width(VOLUME_WIDTH),
    )
    .into()
}

impl Playback {
    pub fn new() -> Self {
        let (sender, receiver) = unbounded::<Message>();
        Self {
            cover_allocation: None,
            mixer_device_sink: DeviceSinkBuilder::open_default_sink().unwrap(),
            player: None,
            seek_position: None,
            track: None,
            track_end_receiver: TrackEndReceiver(Arc::new(Mutex::new(Some(receiver)))),
            track_end_sender: sender,
            volume: VOLUME_MAXIMUM / 2f32,
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
            Message::CoverAllocationLoad(color_accent, allocation) => {
                self.cover_allocation = allocation;
                Event::AccentColorChange(color_accent)
            }
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
                let Ok(file) = File::open(&track.path) else {
                    return Event::None;
                };
                let Ok(decoder) = Decoder::try_from(file) else {
                    return Event::None;
                };
                let player = Player::connect_new(self.mixer_device_sink.mixer());
                player.set_volume(self.volume);
                let sender = self.track_end_sender.clone();
                player.append(decoder.amplify_decibel(track.replay_gain.unwrap_or(0.0)));
                player.append(EmptyCallback::new(Box::new(move || {
                    let _ = sender.unbounded_send(Message::ButtonNextPress);
                })));
                self.player = Some(player);
                self.track = Some(track.clone());
                let cover_task = match track_import::cover_from_file(&track.path) {
                    None => Task::done(Message::CoverAllocationLoad(style::COLOR_ACCENT, None)),
                    Some(bytes) => {
                        let cover = image::load_from_memory(&bytes).ok();
                        let color_accent = style::accent_color(cover.as_ref());
                        widget::image::allocate(widget::image::Handle::from_bytes(bytes)).map(
                            move |result| Message::CoverAllocationLoad(color_accent, result.ok()),
                        )
                    }
                };
                Event::TaskPerform(cover_task)
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        column![
            cover(self),
            volume(self),
            information(self),
            seekbar(self),
            controls(self),
        ]
        .padding(
            Padding::ZERO
                .horizontal(PADDING_HORIZONTAL)
                .vertical(PADDING_VERTICAL),
        )
        .spacing(SPACING)
        .width(Length::Fill)
        .into()
    }
}

impl hash::Hash for TrackEndReceiver {
    fn hash<Hasher: hash::Hasher>(&self, state: &mut Hasher) {
        Arc::as_ptr(&self.0).hash(state);
    }
}

pub enum Event {
    AccentColorChange(Color),
    None,
    TaskPerform(Task<Message>),
    TrackActivateNext,
    TrackActivatePrevious,
}

#[derive(Clone, Debug)]
pub enum Message {
    ButtonNextPress,
    ButtonPauseOrPlayPress,
    ButtonPreviousPress,
    CoverAllocationLoad(Color, Option<widget::image::Allocation>),
    SliderSeekbarMouseDrag(f32),
    SliderSeekbarMouseRelease,
    SliderSeekbarTick,
    SliderVolumeChange(f32),
    TrackPlay(Track),
}

pub struct Playback {
    cover_allocation: Option<widget::image::Allocation>,
    mixer_device_sink: MixerDeviceSink,
    player: Option<Player>,
    seek_position: Option<f32>,
    track: Option<Track>,
    track_end_receiver: TrackEndReceiver,
    track_end_sender: UnboundedSender<Message>,
    volume: f32,
}

#[derive(Clone, Debug)]
struct TrackEndReceiver(Arc<Mutex<Option<UnboundedReceiver<Message>>>>);
