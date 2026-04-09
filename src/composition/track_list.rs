use {
    crate::{
        icon,
        style,
        track_import,
        trigram,
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
        Theme,
        event,
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
        collections::HashSet,
        path::PathBuf,
    },
};

const BUTTON_SIZE: u32 = 36;
const SCROLLBAR_WIDTH: f32 = 10.0;
const SEARCH_THRESHOLD: f32 = 0.1;
const TRACK_HEADER_TEXT_SIZE: f32 = 16.0;
const TRACK_PADDING_HORIZONTAL: u32 = 10;
const TRACK_ROW_HEIGHT: u32 = 36;
const TRACK_TEXT_SIZE: f32 = 14.0;

fn arrow_press(track_list: &mut TrackList, step: impl Fn(usize, usize) -> usize) {
    if track_list.tracks.is_empty() {
        return;
    }
    let index = match track_list.selected {
        Some(current) => step(current, track_list.tracks.len()),
        None => 0,
    };
    track_list.selected = Some(index);
}

fn search_text_input<'a>(value: &str) -> Element<'a, Message> {
    row![
        center(
            svg(svg::Handle::from_memory(icon::SEARCH))
                .height(style::ICON_SIZE)
                .style(|_theme, _status| svg::Style {
                    color: Some(style::COLOR_GRAY_4)
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
                placeholder: style::COLOR_GRAY_4,
                ..text_input::default(theme, status)
            }),
    ]
    .align_y(Alignment::Center)
    .into()
}

fn toolbar(track_list: &TrackList) -> Element<'_, Message> {
    container(row![
        search_text_input(&track_list.search_query),
        view_helper::button(
            style::COLOR_GRAY_4,
            svg::Handle::from_memory(icon::FILE),
            BUTTON_SIZE
        )
        .on_press(Message::ButtonFileOpenPress),
        view_helper::button(
            style::COLOR_GRAY_4,
            svg::Handle::from_memory(icon::FOLDER),
            BUTTON_SIZE
        )
        .on_press(Message::ButtonFolderOpenPress),
    ])
    .height(BUTTON_SIZE)
    .style(|_theme| Style {
        background: Some(style::COLOR_GRAY_2.into()),
        ..Default::default()
    })
    .into()
}

fn track_activate(index: usize, track_list: &mut TrackList) -> Event {
    track_list.active = Some(index);
    track_list.selected = Some(index);
    Event::TrackActivated(track_list.tracks[index].clone())
}

fn track_text_container<'a>(size: f32, value: &'a str) -> Element<'a, Message> {
    container(
        text(value)
            .ellipsis(Ellipsis::End)
            .size(size)
            .width(Length::Fill)
            .wrapping(Wrapping::None),
    )
    .align_y(Alignment::Center)
    .height(TRACK_ROW_HEIGHT)
    .padding(Padding::from(0.0).horizontal(TRACK_PADDING_HORIZONTAL))
    .into()
}

fn tracks(track_list: &TrackList) -> Element<'_, Message> {
    let header = container(row![
        track_text_container(TRACK_HEADER_TEXT_SIZE, "Title"),
        track_text_container(TRACK_HEADER_TEXT_SIZE, "Artist"),
        track_text_container(TRACK_HEADER_TEXT_SIZE, "Album"),
    ])
    .style(|_theme| Style {
        background: Some(style::COLOR_GRAY_2.into()),
        ..Default::default()
    });

    let track_rows = trigram::top_indexes(
        &track_list.search_query,
        &track_list
            .tracks
            .iter()
            .map(|track| format!("{} {} {}", track.album, track.artist, track.title))
            .collect::<Vec<String>>(),
        SEARCH_THRESHOLD,
    )
    .into_iter()
    .enumerate()
    .map(|(position, index)| {
        let track = &track_list.tracks[index];
        let is_active = track_list.active == Some(index);
        let is_selected = track_list.selected == Some(index);
        mouse_area(
            container(row![
                track_text_container(TRACK_TEXT_SIZE, &track.title),
                track_text_container(TRACK_TEXT_SIZE, &track.artist),
                track_text_container(TRACK_TEXT_SIZE, &track.album),
            ])
            .style(move |_theme: &Theme| Style {
                background: if is_active {
                    Some(style::COLOR_ACCENT.into())
                } else if is_selected {
                    Some(style::COLOR_GRAY_3.into())
                } else if position % 2 == 1 {
                    Some(style::COLOR_GRAY_2.into())
                } else {
                    None
                },
                ..Default::default()
            }),
        )
        .on_press(Message::TrackPress(index))
        .into()
    });

    scrollable(
        column![header]
            .extend(track_rows)
            .padding(Padding::from(0).right(SCROLLBAR_WIDTH)),
    )
    .style(|theme, status| scrollable::Style {
        container: container::Style {
            background: Some(style::COLOR_GRAY_1.into()),
            ..Default::default()
        },
        vertical_rail: scrollable::Rail {
            background: Some(style::COLOR_GRAY_1.into()),
            border: Default::default(),
            scroller: scrollable::Scroller {
                background: style::COLOR_GRAY_2.into(),
                border: Border {
                    radius: 2.0.into(),
                    ..Default::default()
                },
            },
        },
        ..scrollable::default(theme, status)
    })
    .into()
}

impl TrackList {
    pub fn new() -> Self {
        Self {
            active: None,
            search_query: String::new(),
            selected: None,
            tracks: vec![],
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
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

    #[must_use]
    pub fn update(&mut self, message: Message) -> Event {
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
                arrow_press(self, |index, length| (index + 1).min(length - 1));
                Event::None
            }
            Message::KeyboardKeyArrowUpPress => {
                arrow_press(self, |index, _| index.saturating_sub(1));
                Event::None
            }
            Message::KeyboardKeyEnterPress => match self.selected {
                Some(index) => track_activate(index, self),
                None => Event::None,
            },
            Message::PathPick(path) => path.map_or(Event::None, |path| {
                Event::TaskPerform(Task::done(Message::TrackListExtend(if path.is_dir() {
                    track_import::from_directory(&path)
                } else {
                    track_import::from_file(&path).into_iter().collect()
                })))
            }),
            Message::SearchTextInput(search_query) => {
                self.search_query = search_query;
                Event::None
            }
            Message::TrackActivateNext => {
                if self.tracks.is_empty() {
                    return Event::None;
                }
                let index = self
                    .active
                    .map_or(0, |index| (index + 1).min(self.tracks.len() - 1));
                track_activate(index, self)
            }
            Message::TrackActivatePrevious => {
                if self.tracks.is_empty() {
                    return Event::None;
                }
                let index = self.active.map_or(0, |index| index.saturating_sub(1));
                track_activate(index, self)
            }
            Message::TrackListExtend(tracks) => {
                let opened_paths: HashSet<PathBuf> =
                    self.tracks.iter().map(|track| track.path.clone()).collect();
                self.tracks.extend(
                    tracks
                        .into_iter()
                        .filter(|track| !opened_paths.contains(&track.path)),
                );
                Event::None
            }
            Message::TrackPress(index) => track_activate(index, self),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        column![toolbar(self), tracks(self)].into()
    }
}

pub enum Event {
    None,
    TaskPerform(Task<Message>),
    TrackActivated(Track),
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
    TrackActivateNext,
    TrackActivatePrevious,
    TrackListExtend(Vec<Track>),
    TrackPress(usize),
}

#[derive(Clone, Debug)]
pub struct Track {
    pub album: String,
    pub artist: String,
    pub duration: Option<f32>,
    pub path: PathBuf,
    pub replay_gain: f32,
    pub title: String,
}

pub struct TrackList {
    active: Option<usize>,
    search_query: String,
    selected: Option<usize>,
    tracks: Vec<Track>,
}
