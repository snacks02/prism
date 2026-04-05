use {
    crate::{
        icons,
        track_import,
        trigram,
    },
    iced::{
        Alignment,
        ContentFit,
        Element,
        Event::Keyboard,
        Length,
        Subscription,
        Task,
        Theme,
        event,
        keyboard::{
            Event::{
                KeyPressed,
                ModifiersChanged,
            },
            Key,
            Modifiers,
            key::Named,
        },
        widget::{
            Button,
            button,
            center,
            column,
            container,
            container::Style,
            mouse_area,
            row,
            scrollable,
            stack,
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

const BUTTON_SIZE: u32 = 32;
const HEIGHT: u32 = 48;
const ICON_SIZE: u32 = 16;
const PADDING: u16 = 8;
const SEARCH_THRESHOLD: f32 = 0.1;
const SPACING: u32 = 8;

fn arrow_press(track_list: &mut TrackList, step: impl Fn(usize, usize) -> usize) {
    if track_list.tracks.is_empty() {
        return;
    }
    let index = match track_list.shift_arrow_index.or(track_list.anchor) {
        Some(current) => step(current, track_list.tracks.len()),
        None => 0,
    };
    if track_list.keyboard_modifiers.shift() {
        track_select_shift(index, track_list);
    } else {
        track_select_single(index, track_list);
    }
}

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

fn search_text_input<'a>(value: &str) -> Element<'a, Message> {
    stack![
        text_input("Search", value)
            .on_input(Message::SearchTextInput)
            .width(Length::Fill),
        container(
            svg(svg::Handle::from_memory(icons::SEARCH))
                .height(ICON_SIZE)
                .width(ICON_SIZE),
        )
        .align_y(Alignment::Center)
        .height(Length::Fill),
    ]
    .into()
}

fn track_activate(index: usize, track_list: &mut TrackList) -> Event {
    track_list.active = Some(index);
    track_select_single(index, track_list);
    Event::TrackActivated(track_list.tracks[index].clone())
}

fn track_select(index: usize, track_list: &mut TrackList) {
    if track_list.keyboard_modifiers.shift() {
        track_select_shift(index, track_list);
    } else if track_list.keyboard_modifiers.control() {
        if !track_list.selected.remove(&index) {
            track_list.selected.insert(index);
        }
        track_list.anchor = Some(index);
        track_list.shift_arrow_index = None;
    } else {
        track_select_single(index, track_list);
    }
}

fn track_select_shift(index: usize, track_list: &mut TrackList) {
    if !track_list.keyboard_modifiers.control() {
        track_list.selected.clear();
    }
    let anchor = track_list.anchor.unwrap_or(index);
    track_list
        .selected
        .extend(anchor.min(index)..=anchor.max(index));
    track_list.shift_arrow_index = Some(index);
}

fn track_select_single(index: usize, track_list: &mut TrackList) {
    track_list.anchor = Some(index);
    track_list.selected.clear();
    track_list.selected.insert(index);
    track_list.shift_arrow_index = None;
}

impl TrackList {
    pub fn new() -> Self {
        Self {
            active: None,
            anchor: None,
            keyboard_modifiers: Modifiers::default(),
            search_query: String::new(),
            selected: HashSet::new(),
            shift_arrow_index: None,
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
            Keyboard(ModifiersChanged(keyboard_modifiers)) => {
                Some(Message::KeyboardModifiersChange(keyboard_modifiers))
            }
            _ => None,
        })
    }

    #[must_use]
    pub fn update(&mut self, message: Message) -> Event {
        match message {
            Message::ButtonFileOpenPress => Event::TaskPerform(Task::perform(
                async {
                    AsyncFileDialog::new()
                        .pick_file()
                        .await
                        .map(|handle| handle.path().to_owned())
                },
                Message::PathPick,
            )),
            Message::ButtonFolderOpenPress => Event::TaskPerform(Task::perform(
                async {
                    AsyncFileDialog::new()
                        .pick_folder()
                        .await
                        .map(|handle| handle.path().to_owned())
                },
                Message::PathPick,
            )),
            Message::KeyboardKeyArrowDownPress => {
                arrow_press(self, |index, length| (index + 1).min(length - 1));
                Event::None
            }
            Message::KeyboardKeyArrowUpPress => {
                arrow_press(self, |index, _| index.saturating_sub(1));
                Event::None
            }
            Message::KeyboardKeyEnterPress => match self.anchor {
                Some(index) => track_activate(index, self),
                None => Event::None,
            },
            Message::KeyboardModifiersChange(keyboard_modifiers) => {
                self.keyboard_modifiers = keyboard_modifiers;
                Event::None
            }
            Message::PathPick(None) => Event::None,
            Message::PathPick(Some(path)) => Event::TaskPerform(Task::perform(
                async move {
                    if path.is_dir() {
                        track_import::from_directory(&path)
                    } else {
                        track_import::from_file(&path).into_iter().collect()
                    }
                },
                Message::TrackListExtend,
            )),
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
            Message::TrackDoubleClick(index) => track_activate(index, self),
            Message::TrackListExtend(tracks) => {
                let opened_file_paths: HashSet<String> = self
                    .tracks
                    .iter()
                    .map(|track| track.file_path.clone())
                    .collect();
                self.tracks.extend(
                    tracks
                        .into_iter()
                        .filter(|track| !opened_file_paths.contains(&track.file_path)),
                );
                Event::None
            }
            Message::TrackPress(index) => {
                track_select(index, self);
                Event::None
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let toolbar = row![
            search_text_input(&self.search_query),
            icon_button(svg::Handle::from_memory(icons::FILE))
                .on_press(Message::ButtonFileOpenPress),
            icon_button(svg::Handle::from_memory(icons::FOLDER))
                .on_press(Message::ButtonFolderOpenPress),
        ]
        .height(HEIGHT)
        .padding(PADDING)
        .spacing(SPACING);

        let header = row![
            text("Title")
                .ellipsis(Ellipsis::End)
                .width(Length::Fill)
                .wrapping(Wrapping::None),
            text("Artist")
                .ellipsis(Ellipsis::End)
                .width(Length::Fill)
                .wrapping(Wrapping::None),
            text("Album")
                .ellipsis(Ellipsis::End)
                .width(Length::Fill)
                .wrapping(Wrapping::None),
        ];

        let track_rows = trigram::top_indexes(
            &self.search_query,
            &self
                .tracks
                .iter()
                .map(|track| format!("{} {} {}", track.album, track.artist, track.title))
                .collect::<Vec<String>>(),
            SEARCH_THRESHOLD,
        )
        .into_iter()
        .map(|index| {
            let track = &self.tracks[index];
            let is_active = self.active == Some(index);
            let is_selected = self.selected.contains(&index);
            mouse_area(
                container(row![
                    text(&track.title)
                        .ellipsis(Ellipsis::End)
                        .width(Length::Fill)
                        .wrapping(Wrapping::None),
                    text(&track.artist)
                        .ellipsis(Ellipsis::End)
                        .width(Length::Fill)
                        .wrapping(Wrapping::None),
                    text(&track.album)
                        .ellipsis(Ellipsis::End)
                        .width(Length::Fill)
                        .wrapping(Wrapping::None),
                ])
                .style(move |theme: &Theme| Style {
                    background: if is_active {
                        Some(theme.palette().primary.strong.color.into())
                    } else if is_selected {
                        Some(theme.palette().primary.weak.color.into())
                    } else {
                        None
                    },
                    ..Style::default()
                })
                .width(Length::Fill),
            )
            .on_double_click(Message::TrackDoubleClick(index))
            .on_press(Message::TrackPress(index))
            .into()
        });

        column![toolbar, scrollable(column![header].extend(track_rows))].into()
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
    KeyboardModifiersChange(Modifiers),
    PathPick(Option<PathBuf>),
    SearchTextInput(String),
    TrackActivateNext,
    TrackActivatePrevious,
    TrackDoubleClick(usize),
    TrackListExtend(Vec<Track>),
    TrackPress(usize),
}

#[derive(Clone, Debug)]
pub struct Track {
    pub album: String,
    pub artist: String,
    pub duration: Option<f32>,
    pub file_path: String,
    pub replay_gain: f32,
    pub title: String,
}

pub struct TrackList {
    active: Option<usize>,
    anchor: Option<usize>,
    keyboard_modifiers: Modifiers,
    search_query: String,
    selected: HashSet<usize>,
    shift_arrow_index: Option<usize>,
    tracks: Vec<Track>,
}
