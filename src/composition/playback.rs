use {
    crate::{
        audio_player::AudioPlayer,
        composition::Composition,
        icon,
        queue::Queue,
        style,
        track::Track,
        track_read,
        view_helper,
    },
    async_channel::Receiver,
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
    std::sync::Arc,
    std::time::Duration,
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
        let queue = Queue::default();
        let sender = queue.track_end_sender();
        let audio_player = AudioPlayer::new(
            Arc::new(move || {
                sender.try_send(()).ok();
            }),
            VOLUME_DEFAULT,
        );
        Self {
            audio_player,
            cover_allocation: None,
            queue,
            seekbar_position: None,
            track: None,
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
            Subscription::run_with(self.queue.track_end_receiver(), |receiver| {
                Receiver::clone(&receiver.0)
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
            Message::ButtonNextPress => {
                let track = self.queue.next().cloned();
                self.track_play(track)
            }
            Message::ButtonPauseOrPlayPress => {
                self.audio_player.pause_or_play();
                Event::None
            }
            Message::ButtonPreviousPress => {
                let track = self.queue.previous().cloned();
                self.track_play(track)
            }
            Message::ButtonRepeatPress => {
                self.queue.toggle_repeat();
                Event::None
            }
            Message::ButtonShufflePress => {
                self.queue.toggle_shuffle();
                Event::None
            }
            Message::CoverAllocationLoad(allocation) => {
                self.cover_allocation = allocation;
                Event::None
            }
            Message::None => Event::None,
            Message::QueueSet(track, tracks) => {
                self.queue.set(&track, tracks);
                self.track_play(Some(track))
            }
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
        let repeat_color = if self.queue.repeat() {
            style::COLOR_GRAY_4
        } else {
            style::COLOR_GRAY_3
        };
        let shuffle_color = if self.queue.shuffle() {
            style::COLOR_GRAY_4
        } else {
            style::COLOR_GRAY_3
        };
        center(
            row![
                view_helper::button(
                    Color::TRANSPARENT.into(),
                    repeat_color,
                    svg::Handle::from_memory(icon::REPEAT),
                    Message::ButtonRepeatPress,
                    BUTTON_SIZE,
                ),
                view_helper::button(
                    Color::TRANSPARENT.into(),
                    style::COLOR_GRAY_4,
                    svg::Handle::from_memory(icon::SKIP_BACK),
                    Message::ButtonPreviousPress,
                    BUTTON_SIZE,
                ),
                container(view_helper::button(
                    style::COLOR_GRAY_4.into(),
                    Color::TRANSPARENT,
                    svg::Handle::from_memory(pause_or_play_icon),
                    Message::ButtonPauseOrPlayPress,
                    BUTTON_SIZE,
                ))
                .padding(Padding::ZERO.horizontal(SEEKBAR_SPACING)),
                view_helper::button(
                    Color::TRANSPARENT.into(),
                    style::COLOR_GRAY_4,
                    svg::Handle::from_memory(icon::SKIP_FORWARD),
                    Message::ButtonNextPress,
                    BUTTON_SIZE,
                ),
                view_helper::button(
                    Color::TRANSPARENT.into(),
                    shuffle_color,
                    svg::Handle::from_memory(icon::SHUFFLE),
                    Message::ButtonShufflePress,
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

    fn track_play(&mut self, track: Option<Arc<Track>>) -> Event {
        let Some(track) = track else {
            return Event::None;
        };
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
        self.track = Some(Arc::clone(&track));
        Event::TrackPlay(cover_task, track)
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

pub enum Event {
    AccentColorChange(Color),
    None,
    TrackPlay(Task<Message>, Arc<Track>),
}

#[derive(Clone, Debug)]
pub enum Message {
    AccentColorLoad(Color),
    ButtonNextPress,
    ButtonPauseOrPlayPress,
    ButtonPreviousPress,
    ButtonRepeatPress,
    ButtonShufflePress,
    CoverAllocationLoad(Option<Allocation>),
    None,
    QueueSet(Arc<Track>, Arc<Vec<Arc<Track>>>),
    SliderSeekbarMouseChange(f32),
    SliderSeekbarMouseRelease,
    SliderSeekbarTick,
    SliderVolumeChange(f32),
}

pub struct Playback {
    audio_player: AudioPlayer,
    cover_allocation: Option<Allocation>,
    queue: Queue,
    seekbar_position: Option<f32>,
    track: Option<Arc<Track>>,
}
