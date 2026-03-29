use crate::track::Track;
use iced::keyboard::Modifiers;
use iced::widget::text::Wrapping;
use iced::widget::{
    column,
    container,
    mouse_area,
    row,
    scrollable,
    text,
};
use iced::{
    Element,
    Length,
};
use std::collections::HashSet;

impl TrackList {
    pub fn new() -> Self {
        Self {
            active: None,
            anchor: None,
            modifiers: Modifiers::default(),
            selected: HashSet::new(),
            tracks: vec![],
        }
    }

    #[must_use]
    pub fn update(&mut self, message: Message) -> Event {
        match message {
            Message::ModifiersChange(modifiers) => {
                self.modifiers = modifiers;
                Event::None
            }
            Message::TrackActivate(index) => {
                self.active = Some(index);
                Event::TrackActivated(index)
            }
            Message::TrackListExtend(tracks) => {
                self.tracks.extend(tracks);
                Event::None
            }
            Message::TrackSelect(index) => {
                if self.modifiers.shift() {
                    let anchor = self.anchor.unwrap_or(index);
                    if !self.modifiers.control() {
                        self.selected.clear();
                    }
                    self.selected.extend(anchor.min(index)..=anchor.max(index));
                } else if self.modifiers.control() {
                    if !self.selected.remove(&index) {
                        self.selected.insert(index);
                    }
                    self.anchor = Some(index);
                } else {
                    self.selected.clear();
                    self.selected.insert(index);
                    self.anchor = Some(index);
                }
                Event::None
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let header = row![
            text("Title").width(Length::Fill).wrapping(Wrapping::None),
            text("Artist").width(Length::Fill).wrapping(Wrapping::None),
            text("Album").width(Length::Fill).wrapping(Wrapping::None),
        ];

        let rows = self.tracks.iter().enumerate().map(|(index, track)| {
            let is_active = self.active == Some(index);
            let is_selected = self.selected.contains(&index);
            mouse_area(
                container(row![
                    text(&track.title)
                        .width(Length::Fill)
                        .wrapping(Wrapping::None),
                    text(&track.artist)
                        .width(Length::Fill)
                        .wrapping(Wrapping::None),
                    text(&track.album)
                        .width(Length::Fill)
                        .wrapping(Wrapping::None),
                ])
                .style(move |theme: &iced::Theme| container::Style {
                    background: match (is_active, is_selected) {
                        (_, true) => Some(theme.extended_palette().primary.weak.color.into()),
                        (true, _) => Some(theme.extended_palette().primary.strong.color.into()),
                        _ => None,
                    },
                    ..container::Style::default()
                })
                .width(Length::Fill),
            )
            .on_double_click(Message::TrackActivate(index))
            .on_press(Message::TrackSelect(index))
            .into()
        });

        scrollable(column![header].extend(rows)).into()
    }
}

pub enum Event {
    None,
    TrackActivated(usize),
}

#[derive(Clone, Debug)]
pub enum Message {
    ModifiersChange(Modifiers),
    TrackActivate(usize),
    TrackListExtend(Vec<Track>),
    TrackSelect(usize),
}

pub struct TrackList {
    active: Option<usize>,
    anchor: Option<usize>,
    modifiers: Modifiers,
    selected: HashSet<usize>,
    tracks: Vec<Track>,
}
