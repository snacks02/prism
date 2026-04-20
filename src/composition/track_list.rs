use {
    crate::{
        composition::Composition,
        icon,
        style,
        track::Track,
        track_read,
        view_helper,
    },
    iced::{
        Alignment,
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
        font::Weight,
        keyboard::{
            Event::KeyPressed,
            Key,
            key::Named,
        },
        widget::{
            center,
            column,
            container,
            container::Style,
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
    nucleo::{
        Utf32String,
        pattern::{
            CaseMatching,
            Normalization,
            Pattern,
        },
    },
    rfd::AsyncFileDialog,
    std::{
        cmp::Reverse,
        collections::HashSet,
        path::PathBuf,
        sync::Arc,
    },
};

const BUTTON_SIZE: u32 = 36;
const PADDING_HORIZONTAL: u32 = 10;
const ROW_HEIGHT: u32 = 36;
const SCROLLBAR_WIDTH: f32 = 10.0;

fn track_next(tracks: &[Arc<Track>], current: Option<&Arc<Track>>) -> Option<Arc<Track>> {
    match current {
        None => tracks.first().cloned(),
        Some(current) => tracks
            .iter()
            .skip_while(|track| !Arc::ptr_eq(*track, current))
            .nth(1)
            .or(Some(current))
            .cloned(),
    }
}

fn track_previous(tracks: &[Arc<Track>], current: Option<&Arc<Track>>) -> Option<Arc<Track>> {
    match current {
        None => tracks.first().cloned(),
        Some(current) => tracks
            .iter()
            .take_while(|track| !Arc::ptr_eq(*track, current))
            .last()
            .or(Some(current))
            .cloned(),
    }
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
    .height(ROW_HEIGHT)
    .padding(Padding::ZERO.horizontal(PADDING_HORIZONTAL))
    .into()
}

impl Composition for TrackList {
    fn new() -> Self {
        Self {
            current: None,
            search_query: String::new(),
            selected: None,
            tracks: vec![],
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        event::listen_with(|event, _status, _window| match event {
            Keyboard(KeyPressed { key, .. }) => match key {
                Key::Named(Named::ArrowDown) => Some(Message::KeyboardKeyArrowDownPress),
                Key::Named(Named::ArrowUp) => Some(Message::KeyboardKeyArrowUpPress),
                Key::Named(Named::Enter) => Some(Message::KeyboardKeyEnterPress),
                _ => None,
            },
            _ => None,
        })
    }

    fn update(&mut self, message: Message) -> Event {
        match message {
            Message::ButtonFileOpenPress => Event::TaskPerform(Task::perform(
                AsyncFileDialog::new().pick_file(),
                |handle| {
                    Message::TracksExtend(
                        handle.map_or(vec![], |handle| track_read::from_path(handle.path())),
                    )
                },
            )),
            Message::ButtonFolderOpenPress => Event::TaskPerform(Task::perform(
                AsyncFileDialog::new().pick_folder(),
                |handle| {
                    Message::TracksExtend(
                        handle.map_or(vec![], |handle| track_read::from_path(handle.path())),
                    )
                },
            )),
            Message::KeyboardKeyArrowDownPress => {
                self.selected = track_next(&self.visible_tracks(), self.selected.as_ref());
                Event::None
            }
            Message::KeyboardKeyArrowUpPress => {
                self.selected = track_previous(&self.visible_tracks(), self.selected.as_ref());
                Event::None
            }
            Message::KeyboardKeyEnterPress => self.selected.clone().map_or(Event::None, |track| {
                self.current = Some(track.clone());
                self.selected = Some(track.clone());
                Event::QueueSetCurrent(track)
            }),
            Message::SearchTextInput(search_query) => {
                self.search_query = search_query;
                Event::None
            }
            Message::TrackPlay(track) => {
                self.current = Some(track.clone());
                self.selected = Some(track);
                Event::None
            }
            Message::TrackPress(track) => {
                self.current = Some(track.clone());
                self.selected = Some(track.clone());
                Event::QueueSetCurrent(track)
            }
            Message::TracksExtend(tracks) => {
                let paths: HashSet<&PathBuf> =
                    self.tracks.iter().map(|track| &track.path).collect();
                let new_tracks: Vec<Arc<Track>> = tracks
                    .into_iter()
                    .filter(|track| !paths.contains(&track.path))
                    .map(Arc::new)
                    .collect();
                self.tracks.extend(new_tracks.clone());
                Event::QueueExtend(new_tracks)
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        column![self.toolbar(), self.tracks()].into()
    }

    type Event = Event;

    type Message = Message;
}

impl TrackList {
    fn toolbar(&self) -> Element<'_, Message> {
        let button_file_open = view_helper::button(
            Color::TRANSPARENT.into(),
            style::COLOR_GRAY_3,
            svg::Handle::from_memory(icon::FILE_PLUS),
            Message::ButtonFileOpenPress,
            BUTTON_SIZE,
        );

        let button_folder_open = view_helper::button(
            Color::TRANSPARENT.into(),
            style::COLOR_GRAY_3,
            svg::Handle::from_memory(icon::FOLDER_PLUS),
            Message::ButtonFolderOpenPress,
            BUTTON_SIZE,
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
            .width(BUTTON_SIZE),
            text_input("Search", &self.search_query)
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
            .height(BUTTON_SIZE)
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
        .padding(Padding::ZERO.right(SCROLLBAR_WIDTH))
        .style(|_theme| Style {
            background: Some(style::COLOR_GRAY_1.into()),
            ..Default::default()
        });

        let rows = scrollable(
            column(
                self.visible_tracks()
                    .into_iter()
                    .enumerate()
                    .map(|(position, track)| {
                        let current = self
                            .current
                            .as_ref()
                            .is_some_and(|current_track| Arc::ptr_eq(current_track, &track));
                        let selected = self
                            .selected
                            .as_ref()
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
                        .on_press(Message::TrackPress(track.clone()))
                        .into()
                    }),
            )
            .padding(Padding::ZERO.right(SCROLLBAR_WIDTH)),
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

        column![header, rows].height(Length::Fill).into()
    }

    fn visible_tracks(&self) -> Vec<Arc<Track>> {
        let pattern = Pattern::parse(
            &self.search_query,
            CaseMatching::Ignore,
            Normalization::Smart,
        );
        let mut matcher = Default::default();
        let mut scored: Vec<(Arc<Track>, u32)> = self
            .tracks
            .iter()
            .filter_map(|track| {
                pattern
                    .score(
                        Utf32String::from(format!(
                            "{} {} {}",
                            track.album_str(),
                            track.artist_str(),
                            track.title_str()
                        ))
                        .slice(..),
                        &mut matcher,
                    )
                    .map(|score| (track.clone(), score))
            })
            .collect();
        scored.sort_unstable_by_key(|&(_, score)| Reverse(score));
        scored.into_iter().map(|(track, _)| track).collect()
    }
}

pub enum Event {
    None,
    QueueExtend(Vec<Arc<Track>>),
    QueueSetCurrent(Arc<Track>),
    TaskPerform(Task<Message>),
}

#[derive(Clone, Debug)]
pub enum Message {
    ButtonFileOpenPress,
    ButtonFolderOpenPress,
    KeyboardKeyArrowDownPress,
    KeyboardKeyArrowUpPress,
    KeyboardKeyEnterPress,
    SearchTextInput(String),
    TrackPlay(Arc<Track>),
    TrackPress(Arc<Track>),
    TracksExtend(Vec<Track>),
}

pub struct TrackList {
    current: Option<Arc<Track>>,
    search_query: String,
    selected: Option<Arc<Track>>,
    tracks: Vec<Arc<Track>>,
}
