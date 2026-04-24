use {
    crate::{
        audio_player::AudioPlayer,
        composition::Composition,
        icon,
        list::List,
        queue::Queue,
        style,
        track,
        track::Track,
        view_helper,
    },
    async_channel::Receiver,
    iced::{
        Alignment,
        Border,
        Color,
        Element,
        Event::Keyboard,
        Font,
        Length,
        Padding,
        Subscription,
        Task,
        Theme,
        event,
        event::Status,
        font::Weight,
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
            mouse_area,
            row,
            scrollable,
            scrollable::{
                Rail,
                Scroller,
            },
            svg,
            text,
            text::{
                Ellipsis,
                Wrapping,
            },
            text_input,
        },
    },
    rfd::AsyncFileDialog,
    std::{
        sync::Arc,
        time::Duration,
    },
};

const COVER_BORDER_RADIUS: f32 = 10.0;
const COVER_BORDER_WIDTH: f32 = 4.0;
const COVER_ICON_SIZE: u32 = 48;
const COVER_SIZE: u32 = 200;
const LIST_BUTTON_SIZE: u32 = 36;
const LIST_PADDING_HORIZONTAL: u32 = 10;
const LIST_ROW_HEIGHT: u32 = 36;
const LIST_SCROLLBAR_WIDTH: f32 = 10.0;
const PLAYBACK_BUTTON_SIZE: u32 = 40;
const PLAYBACK_PADDING_HORIZONTAL: f32 = 8.0;
const PLAYBACK_PADDING_VERTICAL: f32 = 16.0;
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

fn track_text_container(value: impl Into<String>, weight: Weight) -> Element<'static, Message> {
    container(
        text(value.into())
            .ellipsis(Ellipsis::End)
            .font(Font {
                weight,
                ..Default::default()
            })
            .width(Length::Fill)
            .wrapping(Wrapping::None),
    )
    .align_y(Alignment::Center)
    .height(LIST_ROW_HEIGHT)
    .padding(Padding::ZERO.horizontal(LIST_PADDING_HORIZONTAL))
    .into()
}

impl Composition for Main {
    fn new() -> Self {
        Self {
            audio_player: AudioPlayer::new(VOLUME_DEFAULT),
            cover_allocation: None,
            list: Default::default(),
            queue: Default::default(),
            seekbar_position: None,
            track: None,
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let keyboard_subscription = event::listen_with(|event, status, _window| match event {
            Keyboard(KeyPressed { key, .. }) => match key {
                Key::Named(Named::Space) if status == Status::Ignored => {
                    Some(Message::ButtonPauseOrPlayPress)
                }
                Key::Named(Named::ArrowDown) => Some(Message::KeyboardKeyArrowDownPress),
                Key::Named(Named::ArrowUp) => Some(Message::KeyboardKeyArrowUpPress),
                Key::Named(Named::Enter) => Some(Message::KeyboardKeyEnterPress),
                _ => None,
            },
            _ => None,
        });
        let seekbar_subscription =
            time::every(SEEKBAR_TICK_INTERVAL).map(|_| Message::SliderSeekbarTick);
        let track_end_subscription =
            Subscription::run_with(self.audio_player.track_end_receiver(), |receiver| {
                Receiver::clone(&receiver.0)
            })
            .map(|_| Message::ButtonNextPress);
        Subscription::batch([
            keyboard_subscription,
            seekbar_subscription,
            track_end_subscription,
        ])
    }

    fn update(&mut self, message: Message) -> Event {
        match message {
            Message::ButtonFileOpenPress => Event::Perform(Task::perform(
                AsyncFileDialog::new().pick_file(),
                |handle| {
                    Message::ListExtend(
                        handle.map_or(vec![], |handle| track::from_path(handle.path())),
                    )
                },
            )),
            Message::ButtonFolderOpenPress => Event::Perform(Task::perform(
                AsyncFileDialog::new().pick_folder(),
                |handle| {
                    Message::ListExtend(
                        handle.map_or(vec![], |handle| track::from_path(handle.path())),
                    )
                },
            )),
            Message::ButtonNextPress => self.queue.next().cloned().map_or(Event::None, |track| {
                self.list.set_current_and_selected(&track);
                self.play(track)
            }),
            Message::ButtonPauseOrPlayPress => {
                self.audio_player.pause_or_play();
                Event::None
            }
            Message::ButtonPreviousPress => {
                self.queue.previous().cloned().map_or(Event::None, |track| {
                    self.list.set_current_and_selected(&track);
                    self.play(track)
                })
            }
            Message::ButtonRepeatPress => {
                if self.queue.repeat() {
                    self.queue.repeat_disable();
                } else {
                    self.queue.repeat_enable();
                }
                Event::None
            }
            Message::ButtonShufflePress => {
                if self.queue.shuffle() {
                    self.queue.shuffle_disable(self.list.tracks());
                } else {
                    self.queue.shuffle_enable();
                }
                Event::None
            }
            Message::CoverAllocationLoad(allocation) => {
                self.cover_allocation = allocation;
                Event::None
            }
            Message::KeyboardKeyArrowDownPress => {
                self.list.select_next();
                Event::None
            }
            Message::KeyboardKeyArrowUpPress => {
                self.list.select_previous();
                Event::None
            }
            Message::KeyboardKeyEnterPress => {
                self.list.selected().cloned().map_or(Event::None, |track| {
                    self.list.set_current_and_selected(&track);
                    self.queue.set_current(&track);
                    self.play(track)
                })
            }
            Message::ListExtend(tracks) => {
                let new_tracks = self.list.extend(tracks.into_iter().map(Arc::new).collect());
                self.queue.extend(new_tracks);
                Event::None
            }
            Message::ListPress(track) => {
                self.list.set_current_and_selected(&track);
                self.queue.set_current(&track);
                self.play(track)
            }
            Message::None => Event::None,
            Message::PrimaryColorLoad(color) => Event::PrimaryColorChange(color),
            Message::SearchTextInput(query) => {
                self.list.search(query);
                Event::None
            }
            Message::SliderSeekbarChange(position) => {
                self.seekbar_position = Some(position);
                Event::None
            }
            Message::SliderSeekbarRelease => {
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
        column![self.playback(), self.tracks()]
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }

    type Event = Event;

    type Message = Message;
}

impl Main {
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
                    PLAYBACK_BUTTON_SIZE,
                ),
                view_helper::button(
                    Color::TRANSPARENT.into(),
                    style::COLOR_GRAY_4,
                    svg::Handle::from_memory(icon::SKIP_BACK),
                    Message::ButtonPreviousPress,
                    PLAYBACK_BUTTON_SIZE,
                ),
                container(view_helper::button(
                    style::COLOR_GRAY_4.into(),
                    Color::TRANSPARENT,
                    svg::Handle::from_memory(pause_or_play_icon),
                    Message::ButtonPauseOrPlayPress,
                    PLAYBACK_BUTTON_SIZE,
                ))
                .padding(Padding::ZERO.horizontal(SEEKBAR_SPACING)),
                view_helper::button(
                    Color::TRANSPARENT.into(),
                    style::COLOR_GRAY_4,
                    svg::Handle::from_memory(icon::SKIP_FORWARD),
                    Message::ButtonNextPress,
                    PLAYBACK_BUTTON_SIZE,
                ),
                view_helper::button(
                    Color::TRANSPARENT.into(),
                    shuffle_color,
                    svg::Handle::from_memory(icon::SHUFFLE),
                    Message::ButtonShufflePress,
                    PLAYBACK_BUTTON_SIZE,
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

    fn play(&mut self, track: Arc<Track>) -> Event {
        if self.audio_player.play(&track).is_err() {
            return Event::None;
        }
        self.track = Some(track.clone());
        match track::cover_from_file(&track.path) {
            None => Event::Perform(Task::batch([
                Task::done(Message::CoverAllocationLoad(None)),
                Task::done(Message::PrimaryColorLoad(style::COLOR_PRIMARY)),
            ])),
            Some(bytes) => Event::Perform(Task::batch([
                Task::done(Message::PrimaryColorLoad(style::color_primary(
                    image::load_from_memory(&bytes).ok().as_ref(),
                ))),
                widget::image::allocate(widget::image::Handle::from_bytes(bytes))
                    .map(|result| Message::CoverAllocationLoad(result.ok())),
            ])),
        }
    }

    fn playback(&self) -> Element<'_, Message> {
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
                .horizontal(PLAYBACK_PADDING_HORIZONTAL)
                .vertical(PLAYBACK_PADDING_VERTICAL),
        )
        .spacing(SPACING)
        .width(Length::Fill)
        .into()
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
                Message::SliderSeekbarChange,
                Message::SliderSeekbarRelease,
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

    fn toolbar(&self) -> Element<'_, Message> {
        let button_file_open = view_helper::button(
            Color::TRANSPARENT.into(),
            style::COLOR_GRAY_3,
            svg::Handle::from_memory(icon::FILE_PLUS),
            Message::ButtonFileOpenPress,
            LIST_BUTTON_SIZE,
        );

        let button_folder_open = view_helper::button(
            Color::TRANSPARENT.into(),
            style::COLOR_GRAY_3,
            svg::Handle::from_memory(icon::FOLDER_PLUS),
            Message::ButtonFolderOpenPress,
            LIST_BUTTON_SIZE,
        );

        let row_search = row![
            center(
                svg(svg::Handle::from_memory(icon::SEARCH))
                    .height(style::ICON_SIZE)
                    .style(|_theme, _status| svg::Style {
                        color: Some(style::COLOR_GRAY_3)
                    })
                    .width(style::ICON_SIZE),
            )
            .height(Length::Fill)
            .width(LIST_BUTTON_SIZE),
            text_input("Search", self.list.search_query())
                .on_input(Message::SearchTextInput)
                .style(|theme, status| text_input::Style {
                    background: Color::TRANSPARENT.into(),
                    border: Default::default(),
                    placeholder: style::COLOR_GRAY_3,
                    ..text_input::default(theme, status)
                }),
        ]
        .align_y(Alignment::Center);

        container(row![row_search, button_file_open, button_folder_open])
            .height(LIST_BUTTON_SIZE)
            .style(|_theme| Style {
                background: Some(style::COLOR_GRAY_1.into()),
                ..Default::default()
            })
            .into()
    }

    fn tracks(&self) -> Element<'_, Message> {
        let header = container(row![
            track_text_container("Title", Weight::Bold),
            track_text_container("Artist", Weight::Bold),
            track_text_container("Album", Weight::Bold),
        ])
        .padding(Padding::ZERO.right(LIST_SCROLLBAR_WIDTH))
        .style(|_theme| Style {
            background: Some(style::COLOR_GRAY_1.into()),
            ..Default::default()
        });

        let rows =
            scrollable(
                column(self.list.matching().iter().cloned().enumerate().map(
                    |(position, track)| {
                        let current = self
                            .list
                            .current()
                            .is_some_and(|current_track| Arc::ptr_eq(current_track, &track));
                        let selected = self
                            .list
                            .selected()
                            .is_some_and(|selected_track| Arc::ptr_eq(selected_track, &track));
                        mouse_area(
                            container(row![
                                track_text_container(track.title_str(), Weight::Normal),
                                track_text_container(track.artist_str(), Weight::Normal),
                                track_text_container(track.album_str(), Weight::Normal),
                            ])
                            .style(move |theme: &Theme| Style {
                                background: if current {
                                    Some(theme.palette().primary.base.color.into())
                                } else if selected {
                                    Some(style::COLOR_GRAY_2.into())
                                } else if position % 2 == 1 {
                                    Some(style::COLOR_GRAY_1.into())
                                } else {
                                    None
                                },
                                ..Default::default()
                            }),
                        )
                        .on_press(Message::ListPress(track.clone()))
                        .into()
                    },
                ))
                .padding(Padding::ZERO.right(LIST_SCROLLBAR_WIDTH)),
            )
            .style(|theme, status| scrollable::Style {
                vertical_rail: Rail {
                    background: None,
                    border: Default::default(),
                    scroller: Scroller {
                        background: style::COLOR_GRAY_1.into(),
                        border: Default::default(),
                    },
                },
                ..scrollable::default(theme, status)
            });

        column![self.toolbar(), column![header, rows].height(Length::Fill)].into()
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
    None,
    Perform(Task<Message>),
    PrimaryColorChange(Color),
}

#[derive(Clone, Debug)]
pub enum Message {
    ButtonFileOpenPress,
    ButtonFolderOpenPress,
    ButtonNextPress,
    ButtonPauseOrPlayPress,
    ButtonPreviousPress,
    ButtonRepeatPress,
    ButtonShufflePress,
    CoverAllocationLoad(Option<Allocation>),
    KeyboardKeyArrowDownPress,
    KeyboardKeyArrowUpPress,
    KeyboardKeyEnterPress,
    ListExtend(Vec<Track>),
    ListPress(Arc<Track>),
    None,
    PrimaryColorLoad(Color),
    SearchTextInput(String),
    SliderSeekbarChange(f32),
    SliderSeekbarRelease,
    SliderSeekbarTick,
    SliderVolumeChange(f32),
}

pub struct Main {
    audio_player: AudioPlayer,
    cover_allocation: Option<Allocation>,
    list: List,
    queue: Queue,
    seekbar_position: Option<f32>,
    track: Option<Arc<Track>>,
}
