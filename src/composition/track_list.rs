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

fn search_text_input<'a>(value: &str) -> Element<'a, Message> {
    row![
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
        text_input("Search", value)
            .on_input(Message::SearchTextInput)
            .style(|theme, status| text_input::Style {
                background: Color::TRANSPARENT.into(),
                border: Default::default(),
                placeholder: style::COLOR_GRAY_3,
                ..text_input::default(theme, status)
            }),
    ]
    .align_y(Alignment::Center)
    .into()
}

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
            active: None,
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
                |handle| Message::PathPick(handle.map(|handle| handle.path().to_owned())),
            )),
            Message::ButtonFolderOpenPress => Event::TaskPerform(Task::perform(
                AsyncFileDialog::new().pick_folder(),
                |handle| Message::PathPick(handle.map(|handle| handle.path().to_owned())),
            )),
            Message::KeyboardKeyArrowDownPress => {
                self.selected = track_next(&self.visible_tracks(), self.selected.as_ref());
                Event::None
            }
            Message::KeyboardKeyArrowUpPress => {
                self.selected = track_previous(&self.visible_tracks(), self.selected.as_ref());
                Event::None
            }
            Message::KeyboardKeyEnterPress => match self.selected.clone() {
                None => Event::None,
                Some(track) => self.track_activate(track),
            },
            Message::PathPick(path) => path.map_or(Event::None, |path| {
                Event::TaskPerform(Task::done(Message::TrackListExtend(if path.is_dir() {
                    track_read::from_directory(&path)
                } else {
                    track_read::from_file(&path).into_iter().collect()
                })))
            }),
            Message::SearchTextInput(search_query) => {
                self.search_query = search_query;
                Event::None
            }
            Message::TrackListExtend(tracks) => {
                let opened_paths: HashSet<&PathBuf> =
                    self.tracks.iter().map(|track| &track.path).collect();
                let new_tracks: Vec<Arc<Track>> = tracks
                    .into_iter()
                    .filter(|track| !opened_paths.contains(&track.path))
                    .map(Arc::new)
                    .collect();
                self.tracks.extend(new_tracks);
                Event::None
            }
            Message::TrackPlay(track) => {
                self.active = Some(Arc::clone(&track));
                self.selected = Some(track);
                Event::None
            }
            Message::TrackPress(track) => self.track_activate(track),
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
        container(row![
            search_text_input(&self.search_query),
            view_helper::button(
                Color::TRANSPARENT.into(),
                style::COLOR_GRAY_3,
                svg::Handle::from_memory(icon::FILE_PLUS),
                Message::ButtonFileOpenPress,
                BUTTON_SIZE,
            ),
            view_helper::button(
                Color::TRANSPARENT.into(),
                style::COLOR_GRAY_3,
                svg::Handle::from_memory(icon::FOLDER_PLUS),
                Message::ButtonFolderOpenPress,
                BUTTON_SIZE,
            ),
        ])
        .height(BUTTON_SIZE)
        .style(|_theme| Style {
            background: Some(style::COLOR_GRAY_1.into()),
            ..Default::default()
        })
        .into()
    }

    fn track_activate(&mut self, track: Arc<Track>) -> Event {
        self.active = Some(Arc::clone(&track));
        self.selected = Some(Arc::clone(&track));
        Event::QueueSet(track, self.tracks.clone())
    }

    fn tracks(&self) -> Element<'_, Message> {
        let header = container(row![
            track_text_container("Title", Weight::Bold),
            track_text_container("Artist", Weight::Bold),
            track_text_container("Album", Weight::Bold),
        ])
        .style(|_theme| Style {
            background: Some(style::COLOR_GRAY_1.into()),
            ..Default::default()
        });

        let track_rows = self
            .visible_tracks()
            .into_iter()
            .enumerate()
            .map(|(position, track)| {
                let is_active = self
                    .active
                    .as_ref()
                    .is_some_and(|active| Arc::ptr_eq(active, &track));
                let is_selected = self
                    .selected
                    .as_ref()
                    .is_some_and(|selected| Arc::ptr_eq(selected, &track));
                mouse_area(
                    container(row![
                        track_text_container(track.title_str(), Weight::Normal),
                        track_text_container(track.artist_str(), Weight::Normal),
                        track_text_container(track.album_str(), Weight::Normal),
                    ])
                    .style(move |theme: &Theme| Style {
                        background: if is_active {
                            Some(theme.palette().primary.base.color.into())
                        } else if is_selected {
                            Some(style::COLOR_GRAY_2.into())
                        } else if position % 2 == 1 {
                            Some(style::COLOR_GRAY_1.into())
                        } else {
                            None
                        },
                        ..Default::default()
                    }),
                )
                .on_press(Message::TrackPress(Arc::clone(&track)))
                .into()
            });

        column![
            header.padding(Padding::ZERO.right(SCROLLBAR_WIDTH)),
            scrollable(column(track_rows).padding(Padding::ZERO.right(SCROLLBAR_WIDTH))).style(
                |theme, status| scrollable::Style {
                    vertical_rail: Rail {
                        background: None,
                        border: Default::default(),
                        scroller: Scroller {
                            background: style::COLOR_GRAY_1.into(),
                            border: Default::default()
                        },
                    },
                    ..scrollable::default(theme, status)
                }
            ),
        ]
        .height(Length::Fill)
        .into()
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
                    .map(|score| (Arc::clone(track), score))
            })
            .collect();
        scored.sort_unstable_by_key(|&(_, score)| Reverse(score));
        scored.into_iter().map(|(track, _)| track).collect()
    }
}

pub enum Event {
    None,
    QueueSet(Arc<Track>, Vec<Arc<Track>>),
    TaskPerform(Task<Message>),
}

#[derive(Clone, Debug)]
pub enum Message {
    ButtonFileOpenPress,
    ButtonFolderOpenPress,
    KeyboardKeyArrowDownPress,
    KeyboardKeyArrowUpPress,
    KeyboardKeyEnterPress,
    PathPick(Option<PathBuf>),
    SearchTextInput(String),
    TrackListExtend(Vec<Track>),
    TrackPlay(Arc<Track>),
    TrackPress(Arc<Track>),
}

pub struct TrackList {
    active: Option<Arc<Track>>,
    search_query: String,
    selected: Option<Arc<Track>>,
    tracks: Vec<Arc<Track>>,
}
