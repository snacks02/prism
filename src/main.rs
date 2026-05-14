use {
    crate::{
        audio_player::AudioPlayer,
        composition::Composition,
        list::List,
        queue::Queue,
        track::Track,
    },
    iced::{
        Alignment,
        Border,
        Color,
        Element,
        Event::Keyboard,
        Font,
        Length,
        Padding,
        Result,
        Settings,
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
        theme::palette::Seed,
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

mod audio_player;
mod composition;
mod icon;
mod list;
mod queue;
mod style;
mod track;
mod view_helper;

const COVER_BORDER_RADIUS: f32 = 10.0;
const COVER_BORDER_WIDTH: f32 = 4.0;
const COVER_ICON_SIZE: u32 = 48;
const COVER_SIZE: u32 = 200;
const DEFAULT_TEXT_SIZE: f32 = 14.0;
const PADDING_AND_SPACING_LARGE: u32 = 16;
const PADDING_AND_SPACING_SMALL: u32 = 8;
const PLAYBACK_BUTTON_SIZE: u32 = 40;
const SCROLLBAR_PADDING: f32 = 10.0;
const SEEKBAR_DURATION_CLAMP: f32 = SEEKBAR_DURATION_MAXIMUM - 1.0;
const SEEKBAR_DURATION_MAXIMUM: f32 = 6000.0;
const SEEKBAR_DURATION_WIDTH: u32 = 40;
const SEEKBAR_STEP: f32 = 0.001;
const SEEKBAR_TICK_INTERVAL: Duration = Duration::from_millis(16);
const TOOLBAR_SIZE: u32 = 36;
const TRACK_TEXT_CONTAINER_HEIGHT: u32 = 36;
const TRACK_TEXT_CONTAINER_PADDING_HORIZONTAL: u32 = 10;
const VOLUME_DEFAULT: f32 = VOLUME_MAXIMUM / 2.0;
const VOLUME_MAXIMUM: f32 = 1.0;
const VOLUME_STEP: f32 = 0.01;
const VOLUME_WIDTH: u32 = 88;

fn duration_text<'a>(seconds: f32) -> Text<'a> {
    let clamped_seconds = seconds.min(SEEKBAR_DURATION_CLAMP) as i32;

    text(format!(
        "{:02}:{:02}{}",
        clamped_seconds / 60,
        clamped_seconds % 60,
        if seconds >= SEEKBAR_DURATION_MAXIMUM {
            "+"
        } else {
            ""
        }
    ))
    .align_x(Alignment::Center)
    .width(SEEKBAR_DURATION_WIDTH)
}

fn main() -> Result {
    iced::application(Prism::new, Prism::update, Prism::view)
        .settings(Settings {
            default_text_size: DEFAULT_TEXT_SIZE.into(),
            ..Default::default()
        })
        .subscription(Prism::subscription)
        .theme(Prism::theme)
        .title("Prism")
        .run()
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
    .height(TRACK_TEXT_CONTAINER_HEIGHT)
    .padding(Padding::ZERO.horizontal(TRACK_TEXT_CONTAINER_PADDING_HORIZONTAL))
    .into()
}

impl Composition for Prism {
    fn new() -> Self {
        Self {
            audio_player: AudioPlayer::new(VOLUME_DEFAULT),
            color_primary: style::COLOR_PRIMARY,
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
                Key::Named(Named::ArrowDown) => Some(Message::KeyboardKeyArrowDownPress),
                Key::Named(Named::ArrowUp) => Some(Message::KeyboardKeyArrowUpPress),
                Key::Named(Named::Enter) => Some(Message::KeyboardKeyEnterPress),
                Key::Named(Named::Space) if status == Status::Ignored => {
                    Some(Message::ButtonPauseOrPlayPress)
                }
                _ => None,
            },
            _ => None,
        });
        let seekbar_subscription =
            time::every(SEEKBAR_TICK_INTERVAL).map(|_| Message::SliderSeekbarTick);
        let track_end_subscription =
            Subscription::run_with(self.audio_player.track_end_receiver(), |receiver| {
                receiver.0.as_ref().clone()
            })
            .map(|_| Message::ButtonNextPress);
        Subscription::batch([
            keyboard_subscription,
            seekbar_subscription,
            track_end_subscription,
        ])
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ButtonFileOpenPress => {
                Task::perform(AsyncFileDialog::new().pick_file(), |handle| {
                    Message::ListExtend(
                        handle.map_or(vec![], |handle| track::from_path(handle.path())),
                    )
                })
            }
            Message::ButtonFolderOpenPress => {
                Task::perform(AsyncFileDialog::new().pick_folder(), |handle| {
                    Message::ListExtend(
                        handle.map_or(vec![], |handle| track::from_path(handle.path())),
                    )
                })
            }
            Message::ButtonNextPress => self.queue.next().cloned().map_or(Task::none(), |track| {
                self.list.set_current_and_selected(&track);
                self.play(track)
            }),
            Message::ButtonPauseOrPlayPress => {
                self.audio_player.pause_or_play();
                Task::none()
            }
            Message::ButtonPreviousPress => {
                self.queue
                    .previous()
                    .cloned()
                    .map_or(Task::none(), |track| {
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
                Task::none()
            }
            Message::ButtonShufflePress => {
                if self.queue.shuffle() {
                    self.queue.shuffle_disable(self.list.tracks());
                } else {
                    self.queue.shuffle_enable();
                }
                Task::none()
            }
            Message::CoverAllocationLoad(allocation) => {
                self.cover_allocation = allocation;
                Task::none()
            }
            Message::KeyboardKeyArrowDownPress => {
                self.list.select_next();
                Task::none()
            }
            Message::KeyboardKeyArrowUpPress => {
                self.list.select_previous();
                Task::none()
            }
            Message::KeyboardKeyEnterPress => {
                self.list.selected().cloned().map_or(Task::none(), |track| {
                    self.list.set_current_and_selected(&track);
                    self.queue.set_current(&track);
                    self.play(track)
                })
            }
            Message::ListExtend(tracks) => {
                let new_tracks = self.list.extend(tracks.into_iter().map(Arc::new).collect());
                self.queue.extend(new_tracks);
                Task::none()
            }
            Message::ListPress(track) => {
                self.list.set_current_and_selected(&track);
                self.queue.set_current(&track);
                self.play(track)
            }
            Message::None => Task::none(),
            Message::SearchTextInput(query) => {
                self.list.search(query);
                Task::none()
            }
            Message::SliderSeekbarChange(position) => {
                self.seekbar_position = Some(position);
                Task::none()
            }
            Message::SliderSeekbarRelease => {
                if let Some(position) = self.seekbar_position.take() {
                    self.audio_player.try_seek(position);
                }
                Task::none()
            }
            Message::SliderSeekbarTick => Task::none(),
            Message::SliderVolumeChange(volume) => {
                self.audio_player.set_volume(volume);
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        column![self.playback(), self.tracks()]
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }

    type Event = Task<Message>;

    type Message = Message;
}

impl Prism {
    fn controls(&self) -> Element<'_, Message> {
        center(
            row![
                view_helper::button(
                    Color::TRANSPARENT.into(),
                    if self.queue.repeat() {
                        style::COLOR_GRAY_4
                    } else {
                        style::COLOR_GRAY_3
                    },
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
                    svg::Handle::from_memory(if self.audio_player.active() {
                        icon::PAUSE
                    } else {
                        icon::PLAY
                    }),
                    Message::ButtonPauseOrPlayPress,
                    PLAYBACK_BUTTON_SIZE,
                ))
                .padding(Padding::ZERO.horizontal(PADDING_AND_SPACING_LARGE)),
                view_helper::button(
                    Color::TRANSPARENT.into(),
                    style::COLOR_GRAY_4,
                    svg::Handle::from_memory(icon::SKIP_FORWARD),
                    Message::ButtonNextPress,
                    PLAYBACK_BUTTON_SIZE,
                ),
                view_helper::button(
                    Color::TRANSPARENT.into(),
                    if self.queue.shuffle() {
                        style::COLOR_GRAY_4
                    } else {
                        style::COLOR_GRAY_3
                    },
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
                    .style(|_, _| svg::Style {
                        color: Some(style::COLOR_GRAY_2),
                    })
                    .width(COVER_ICON_SIZE),
            )
            .center(COVER_SIZE)
            .style(|_| Style {
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
            [
                track.title.as_deref(),
                track.artist.as_deref(),
                track.album.as_deref(),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<&str>>()
            .join(" :: ")
        } else {
            "Select a track".to_string()
        };

        center(text(value).ellipsis(Ellipsis::End).wrapping(Wrapping::None)).into()
    }

    fn play(&mut self, track: Arc<Track>) -> Task<Message> {
        if self.audio_player.play(&track).is_err() {
            return Task::none();
        }
        let cover = track::cover_from_file(&track.path);
        self.track = Some(track);
        match cover {
            None => {
                self.color_primary = style::COLOR_PRIMARY;
                self.cover_allocation = None;
                Task::none()
            }
            Some(bytes) => {
                self.color_primary =
                    style::color_primary(image::load_from_memory(&bytes).ok().as_ref());
                widget::image::allocate(widget::image::Handle::from_bytes(bytes))
                    .map(|result| Message::CoverAllocationLoad(result.ok()))
            }
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
                .horizontal(PADDING_AND_SPACING_SMALL)
                .vertical(PADDING_AND_SPACING_LARGE),
        )
        .spacing(PADDING_AND_SPACING_LARGE)
        .width(Length::Fill)
        .into()
    }

    fn seekbar(&self) -> Element<'_, Message> {
        let duration = self
            .track
            .as_ref()
            .map_or(0.0, |track| track.duration_seconds());
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
        .spacing(PADDING_AND_SPACING_SMALL)
        .into()
    }

    fn theme(&self) -> Theme {
        Theme::custom(
            "Prism".to_string(),
            Seed {
                background: style::COLOR_BACKGROUND,
                primary: self.color_primary,
                text: style::COLOR_GRAY_4,
                ..Seed::DARK
            },
        )
    }

    fn toolbar(&self) -> Element<'_, Message> {
        let file_open_button = view_helper::button(
            Color::TRANSPARENT.into(),
            style::COLOR_GRAY_3,
            svg::Handle::from_memory(icon::FILE_PLUS),
            Message::ButtonFileOpenPress,
            TOOLBAR_SIZE,
        );

        let folder_open_button = view_helper::button(
            Color::TRANSPARENT.into(),
            style::COLOR_GRAY_3,
            svg::Handle::from_memory(icon::FOLDER_PLUS),
            Message::ButtonFolderOpenPress,
            TOOLBAR_SIZE,
        );

        let search_row = row![
            center(
                svg(svg::Handle::from_memory(icon::SEARCH))
                    .height(style::ICON_SIZE)
                    .style(|_, _| svg::Style {
                        color: Some(style::COLOR_GRAY_3)
                    })
                    .width(style::ICON_SIZE),
            )
            .height(Length::Fill)
            .width(TOOLBAR_SIZE),
            text_input("Search", self.list.search_query())
                .on_input(Message::SearchTextInput)
                .padding(0)
                .style(|theme, status| text_input::Style {
                    border: Default::default(),
                    placeholder: style::COLOR_GRAY_3,
                    ..text_input::default(theme, status)
                }),
        ]
        .align_y(Alignment::Center);

        container(row![search_row, file_open_button, folder_open_button])
            .height(Length::Shrink)
            .into()
    }

    fn tracks(&self) -> Element<'_, Message> {
        let header = container(row![
            track_text_container("Title", Weight::Bold),
            track_text_container("Artist", Weight::Bold),
            track_text_container("Album", Weight::Bold),
        ])
        .padding(Padding::ZERO.right(SCROLLBAR_PADDING));

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
                                } else if position % 2 == 0 {
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
                .padding(Padding::ZERO.right(SCROLLBAR_PADDING)),
            )
            .style(|theme, status| scrollable::Style {
                vertical_rail: Rail {
                    background: None,
                    border: Default::default(),
                    scroller: Scroller {
                        background: style::COLOR_GRAY_2.into(),
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
    SearchTextInput(String),
    SliderSeekbarChange(f32),
    SliderSeekbarRelease,
    SliderSeekbarTick,
    SliderVolumeChange(f32),
}

pub struct Prism {
    audio_player: AudioPlayer,
    color_primary: Color,
    cover_allocation: Option<Allocation>,
    list: List,
    queue: Queue,
    seekbar_position: Option<f32>,
    track: Option<Arc<Track>>,
}
