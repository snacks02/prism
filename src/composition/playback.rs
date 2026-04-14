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
            Space,
            center,
            column,
            container,
            container::Style,
            row,
            slider,
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
const COVER_BORDER_WIDTH: f32 = 4.0;
const COVER_ICON_PADDING: f32 = 24.0;
const COVER_SIZE: u32 = 104;
const NOW_PLAYING_PADDING: f32 = 8.0;
const NOW_PLAYING_ROW_HEIGHT: u32 = 120;
const NOW_PLAYING_SPACING: u32 = 8;
const SEEKBAR_HEIGHT: f32 = 36.0;
const SEEKBAR_STEP: f32 = 0.001;
const SEEKBAR_TICK_INTERVAL: Duration = Duration::from_millis(16);
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
                style::COLOR_GRAY_5,
                svg::Handle::from_memory(pause_or_play_icon),
                BUTTON_SIZE
            )
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

fn now_playing(color_accent: Color, playback: &Playback) -> Element<'_, Message> {
    let cover = container(if let Some(allocation) = &playback.cover_allocation {
        Element::from(
            widget::image(allocation.handle().clone())
                .height(Length::Fill)
                .width(Length::Fill),
        )
    } else {
        Element::from(
            center(
                svg(svg::Handle::from_memory(icon::MUSIC))
                    .height(Length::Fill)
                    .style(|_theme, _status| svg::Style {
                        color: Some(style::COLOR_GRAY_3),
                    })
                    .width(Length::Fill),
            )
            .padding(COVER_ICON_PADDING),
        )
    })
    .height(COVER_SIZE)
    .padding(COVER_BORDER_WIDTH)
    .style(|_theme| Style {
        background: Some(style::COLOR_GRAY_2.into()),
        border: Border {
            color: style::COLOR_GRAY_3,
            width: COVER_BORDER_WIDTH,
            ..Default::default()
        },
        ..Default::default()
    })
    .width(COVER_SIZE);

    let track_details = if let Some(track) = &playback.track {
        column![
            Space::new().height(Length::Fill),
            track_detail_text(track.title_str()).size(18),
            Space::new().height(Length::Fill),
            track_detail_text(track.artist_str()).color(style::COLOR_GRAY_4),
            Space::new().height(Length::Fill),
            track_detail_text(track.album_str()).color(style::COLOR_GRAY_4),
            Space::new().height(Length::Fill),
        ]
        .width(Length::Fill)
    } else {
        column![].width(Length::Fill)
    };

    let volume_slider = slider(
        0.0..=VOLUME_MAXIMUM,
        playback.volume,
        Message::SliderVolumeChange,
    )
    .step(VOLUME_STEP)
    .style(move |_, _| slider::Style {
        handle: slider::Handle {
            background: color_accent.into(),
            border_color: color_accent,
            border_width: 0.0,
            shape: slider::HandleShape::Circle { radius: 6.0 },
        },
        rail: slider::Rail {
            backgrounds: (color_accent.into(), style::COLOR_GRAY_4.into()),
            border: Border {
                radius: 2.0.into(),
                ..Default::default()
            },
            width: 3.0,
        },
    })
    .width(VOLUME_WIDTH);

    row![cover, track_details, volume_slider]
        .height(NOW_PLAYING_ROW_HEIGHT)
        .padding(NOW_PLAYING_PADDING)
        .spacing(NOW_PLAYING_SPACING)
        .width(Length::Fill)
        .into()
}

fn on_track_end(data: &TrackEndReceiver) -> UnboundedReceiver<Message> {
    data.0.lock().unwrap().take().unwrap()
}

fn seekbar(color_accent: Color, playback: &Playback) -> Element<'_, Message> {
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
    slider(0.0..=duration, position, Message::SliderSeekbarMouseDrag)
        .height(SEEKBAR_HEIGHT)
        .on_release(Message::SliderSeekbarMouseRelease)
        .step(SEEKBAR_STEP)
        .style(move |_, _| slider::Style {
            handle: slider::Handle {
                background: style::COLOR_GRAY_1.into(),
                border_color: style::COLOR_GRAY_1,
                border_width: 0.0,
                shape: slider::HandleShape::Rectangle {
                    border_radius: Default::default(),
                    width: 0,
                },
            },
            rail: slider::Rail {
                backgrounds: (color_accent.into(), style::COLOR_GRAY_1.into()),
                border: Default::default(),
                width: 36.0,
            },
        })
        .into()
}

fn track_detail_text(value: &str) -> text::Text<'_> {
    text(value).ellipsis(Ellipsis::End).wrapping(Wrapping::None)
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

    pub fn view(&self, color_accent: Color) -> Element<'_, Message> {
        column![
            now_playing(color_accent, self),
            controls(self),
            seekbar(color_accent, self)
        ]
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
