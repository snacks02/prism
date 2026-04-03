use crate::track;
use crate::track::Track;
use iced::widget::{
    Button,
    button,
    center,
    container,
    row,
    stack,
    svg,
    text_input,
};
use iced::{
    Alignment,
    ContentFit,
    Element,
    Length,
    Task,
};
use std::path::PathBuf;

const BUTTON_SIZE: u32 = 32;
const HEIGHT: u32 = 48;
const ICON_FILE_PATH: &str = "icons/file.svg";
const ICON_FOLDER_PATH: &str = "icons/folder.svg";
const ICON_SEARCH_PATH: &str = "icons/search.svg";
const ICON_SIZE: u32 = 16;
const PADDING: u16 = 8;
const SPACING: u32 = 8;

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

fn search_field<'a>(value: &str) -> Element<'a, Message> {
    stack![
        text_input("Search", value)
            .on_input(Message::SearchInput)
            .width(Length::Fill),
        container(
            svg(svg::Handle::from_path(ICON_SEARCH_PATH))
                .height(ICON_SIZE)
                .width(ICON_SIZE),
        )
        .align_y(Alignment::Center)
        .height(Length::Fill),
    ]
    .into()
}

impl Toolbar {
    pub fn new() -> Self {
        Self {
            search_query: String::new(),
        }
    }

    #[must_use]
    pub fn update(&mut self, message: Message) -> Event {
        match message {
            Message::FileOpen => Event::Performed(Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .pick_file()
                        .await
                        .map(|handle| handle.path().to_owned())
                },
                Message::PathPick,
            )),
            Message::FolderOpen => Event::Performed(Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .pick_folder()
                        .await
                        .map(|handle| handle.path().to_owned())
                },
                Message::PathPick,
            )),
            Message::PathPick(None) => Event::None,
            Message::PathPick(Some(path)) => Event::Performed(Task::perform(
                async move {
                    if path.is_dir() {
                        track::from_directory(&path)
                    } else {
                        track::from_file(&path).into_iter().collect()
                    }
                },
                Message::TrackListExtend,
            )),
            Message::SearchInput(search_query) => {
                self.search_query = search_query.clone();
                Event::SearchInputed(search_query)
            }
            Message::TrackListExtend(tracks) => Event::TrackListExtended(tracks),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        row![
            search_field(&self.search_query),
            icon_button(svg::Handle::from_path(ICON_FILE_PATH)).on_press(Message::FileOpen),
            icon_button(svg::Handle::from_path(ICON_FOLDER_PATH)).on_press(Message::FolderOpen),
        ]
        .height(HEIGHT)
        .padding(PADDING)
        .spacing(SPACING)
        .into()
    }
}

pub enum Event {
    None,
    Performed(Task<Message>),
    SearchInputed(String),
    TrackListExtended(Vec<Track>),
}

#[derive(Clone, Debug)]
pub enum Message {
    FileOpen,
    FolderOpen,
    PathPick(Option<PathBuf>),
    SearchInput(String),
    TrackListExtend(Vec<Track>),
}

pub struct Toolbar {
    search_query: String,
}
