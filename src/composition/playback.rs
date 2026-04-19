use {
    crate::{
        audio_player::AudioPlayer,
        composition::Composition,
        icon,
        style,
        track::Track,
        track_read,
        view_helper,
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
        futures::channel::mpsc,
        futures::channel::mpsc::UnboundedReceiver,
        keyboard::{
            Event::KeyPressed,
            Key,
            key::Named,
        },
        time,
        widget,
        widget::{
            Text,
            center,
            column,
            container,
            container::Style,
            image::Allocation,
            row,
            svg,
            text,
            text::{
                Ellipsis,
                Wrapping,
            },
        },
    },
    std::{
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
const SEEKBAR_DURATION_CLAMP: f32 = 5999.0;
const SEEKBAR_DURATION_OVERFLOW: f32 = 6000.0;
const SEEKBAR_DURATION_WIDTH: u32 = 40;
const SEEKBAR_SPACING: u32 = 8;
const SEEKBAR_STEP: f32 = 0.001;
const SEEKBAR_TICK_INTERVAL: Duration = Duration::from_millis(16);
const SPACING: u32 = 16;
const VOLUME_DEFAULT: f32 = VOLUME_MAXIMUM / 2.0;
const VOLUME_MAXIMUM: f32 = 1.0;
const VOLUME_STEP: f32 = 0.01;
const VOLUME_WIDTH: u32 = 88;

fn duration_text<'a>(seconds: f32) -> Text<'a> {
    let clamped = seconds.min(SEEKBAR_DURATION_CLAMP) as i32;
    let overflow = seconds >= SEEKBAR_DURATION_OVERFLOW;

    text(format!(
        "{:02}:{:02}{}",
        clamped / 60,
        clamped % 60,
        if overflow { "+" } else { "" }
    ))
    .align_x(Alignment::Center)
    .width(SEEKBAR_DURATION_WIDTH)
}

impl Composition for Playback {
    fn new() -> Self {
        let (track_end_sender, track_end_receiver) = mpsc::unbounded();
        let audio_player = AudioPlayer::new(
            Arc::new(move || {
                track_end_sender.unbounded_send(()).ok();
            }),
            VOLUME_DEFAULT,
        );
        Self {
            audio_player,
            cover_allocation: None,
            seekbar_position: None,
            track: None,
            track_end_receiver: TrackEndReceiver(Arc::new(Mutex::new(Some(track_end_receiver)))),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
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
            Subscription::run_with(self.track_end_receiver.clone(), |receiver| {
                receiver.0.lock().unwrap().take().unwrap()
            })
            .map(|_| Message::ButtonNextPress);
        Subscription::batch([
            keyboard_subscription,
            slider_seekbar_subscription,
            track_end_subscription,
        ])
    }

    fn update(&mut self, message: Message) -> Event {
        match message {
            Message::AccentColorLoad(color) => Event::AccentColorChange(color),
            Message::ButtonNextPress => Event::TrackActivateNext,
            Message::ButtonPauseOrPlayPress => {
                self.audio_player.pause_or_play();
                Event::None
            }
            Message::ButtonPreviousPress => Event::TrackActivatePrevious,
            Message::CoverAllocationLoad(allocation) => {
                self.cover_allocation = allocation;
                Event::None
            }
            Message::None => Event::None,
            Message::SliderSeekbarMouseChange(position) => {
                self.seekbar_position = Some(position);
                Event::None
            }
            Message::SliderSeekbarMouseRelease => {
                if let Some(position) = self.seekbar_position.take() {
                    self.audio_player.try_seek(position);
                }
                Event::None
            }
            Message::SliderSeekbarTick => Event::None,
            Message::SliderVolumeChange(volume) => {
                self.audio_player.set_volume(volume);
                Event::None
            }
            Message::TrackPlay(track) => {
                if self.audio_player.play(&track).is_err() {
                    return Event::None;
                }
                let cover_task = match track_read::cover_from_file(&track.path) {
                    None => {
                        self.cover_allocation = None;
                        Task::done(Message::AccentColorLoad(style::COLOR_ACCENT))
                    }
                    Some(bytes) => {
                        let cover = image::load_from_memory(&bytes).ok();
                        let color_accent = style::accent_color(cover.as_ref());
                        Task::batch([
                            Task::done(Message::AccentColorLoad(color_accent)),
                            widget::image::allocate(widget::image::Handle::from_bytes(bytes))
                                .map(|result| Message::CoverAllocationLoad(result.ok())),
                        ])
                    }
                };
                self.track = Some(track);
                Event::TaskPerform(cover_task)
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        column![
            self.cover(),
            self.volume(),
            self.information(),
            self.seekbar(),
            self.controls(),
        ]
        .height(Length::Shrink)
        .padding(
            Padding::ZERO
                .horizontal(PADDING_HORIZONTAL)
                .vertical(PADDING_VERTICAL),
        )
        .spacing(SPACING)
        .width(Length::Fill)
        .into()
    }

    type Event = Event;

    type Message = Message;
}

impl Playback {
    fn controls(&self) -> Element<'_, Message> {
        let pause_or_play_icon = if self.audio_player.active() {
            icon::PAUSE
        } else {
            icon::PLAY
        };
        center(
            row![
                view_helper::button(
                    Color::TRANSPARENT.into(),
                    style::COLOR_GRAY_4,
                    svg::Handle::from_memory(icon::SKIP_BACK),
                    Message::ButtonPreviousPress,
                    BUTTON_SIZE,
                ),
                view_helper::button(
                    style::COLOR_GRAY_4.into(),
                    Color::TRANSPARENT,
                    svg::Handle::from_memory(pause_or_play_icon),
                    Message::ButtonPauseOrPlayPress,
                    BUTTON_SIZE,
                ),
                view_helper::button(
                    Color::TRANSPARENT.into(),
                    style::COLOR_GRAY_4,
                    svg::Handle::from_memory(icon::SKIP_FORWARD),
                    Message::ButtonNextPress,
                    BUTTON_SIZE,
                ),
            ]
            .align_y(Alignment::Center),
        )
        .into()
    }

    fn cover(&self) -> Element<'_, Message> {
        let container = if let Some(allocation) = &self.cover_allocation {
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
                        color: Some(style::COLOR_GRAY_2),
                    })
                    .width(COVER_ICON_SIZE),
            )
            .center(COVER_SIZE)
            .style(|_theme| Style {
                background: Some(style::COLOR_GRAY_1.into()),
                border: Border {
                    color: style::COLOR_GRAY_2,
                    radius: COVER_BORDER_RADIUS.into(),
                    width: COVER_BORDER_WIDTH,
                },
                ..Default::default()
            })
        };

        center(container).into()
    }

    fn information(&self) -> Element<'_, Message> {
        let value = if let Some(track) = &self.track {
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

    fn seekbar(&self) -> Element<'_, Message> {
        let duration = self
            .track
            .as_ref()
            .map(|track| track.duration_seconds())
            .unwrap_or(0.0);
        let position = self
            .seekbar_position
            .unwrap_or_else(|| self.audio_player.position());
        row![
            duration_text(position),
            center(view_helper::slider(
                Message::SliderSeekbarMouseChange,
                Message::SliderSeekbarMouseRelease,
                0.0..=duration,
                SEEKBAR_STEP,
                position,
            )),
            duration_text(duration),
        ]
        .align_y(Alignment::Center)
        .spacing(SEEKBAR_SPACING)
        .into()
    }

    fn volume(&self) -> Element<'_, Message> {
        center(
            container(view_helper::slider(
                Message::SliderVolumeChange,
                Message::None,
                0.0..=VOLUME_MAXIMUM,
                VOLUME_STEP,
                self.audio_player.volume(),
            ))
            .width(VOLUME_WIDTH),
        )
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
    AccentColorLoad(Color),
    ButtonNextPress,
    ButtonPauseOrPlayPress,
    ButtonPreviousPress,
    CoverAllocationLoad(Option<Allocation>),
    None,
    SliderSeekbarMouseChange(f32),
    SliderSeekbarMouseRelease,
    SliderSeekbarTick,
    SliderVolumeChange(f32),
    TrackPlay(Track),
}

pub struct Playback {
    audio_player: AudioPlayer,
    cover_allocation: Option<Allocation>,
    seekbar_position: Option<f32>,
    track: Option<Track>,
    track_end_receiver: TrackEndReceiver,
}

#[derive(Clone, Debug)]
struct TrackEndReceiver(Arc<Mutex<Option<UnboundedReceiver<()>>>>);
