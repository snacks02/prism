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

fn activate_track(index: usize, track_list: &mut TrackList) -> Event {
    track_list.active = Some(index);
    Event::TrackActivated(track_list.tracks[index].clone())
}

fn arrow_press(track_list: &mut TrackList, step: impl Fn(usize, usize) -> usize) {
    if track_list.tracks.is_empty() {
        return;
    }
    let index = match track_list.arrow_shift_index.or(track_list.anchor) {
        Some(current) => step(current, track_list.tracks.len()),
        None => 0,
    };
    if track_list.modifiers.shift() {
        track_list.arrow_shift_index = Some(index);
        let anchor = track_list.anchor.unwrap_or(index);
        track_list.selected.clear();
        track_list
            .selected
            .extend(anchor.min(index)..=anchor.max(index));
    } else {
        track_list.anchor = Some(index);
        track_list.arrow_shift_index = None;
        track_list.selected.clear();
        track_list.selected.insert(index);
    }
}

impl TrackList {
    pub fn new() -> Self {
        Self {
            active: None,
            anchor: None,
            arrow_shift_index: None,
            modifiers: Modifiers::default(),
            selected: HashSet::new(),
            tracks: vec![],
        }
    }

    #[must_use]
    pub fn update(&mut self, message: Message) -> Event {
        match message {
            Message::ArrowDownPress => {
                arrow_press(self, |index, length| (index + 1).min(length - 1));
                Event::None
            }
            Message::ArrowUpPress => {
                arrow_press(self, |index, _| index.saturating_sub(1));
                Event::None
            }
            Message::ModifiersChange(modifiers) => {
                self.modifiers = modifiers;
                Event::None
            }
            Message::Next => {
                if self.tracks.is_empty() {
                    return Event::None;
                }
                let index = self
                    .active
                    .map_or(0, |index| (index + 1).min(self.tracks.len() - 1));
                activate_track(index, self)
            }
            Message::Previous => {
                if self.tracks.is_empty() {
                    return Event::None;
                }
                let index = self.active.map_or(0, |index| index.saturating_sub(1));
                activate_track(index, self)
            }
            Message::TrackActivate(index) => activate_track(index, self),
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
                    self.arrow_shift_index = Some(index);
                    self.selected.extend(anchor.min(index)..=anchor.max(index));
                } else if self.modifiers.control() {
                    if !self.selected.remove(&index) {
                        self.selected.insert(index);
                    }
                    self.anchor = Some(index);
                    self.arrow_shift_index = None;
                } else {
                    self.selected.clear();
                    self.selected.insert(index);
                    self.anchor = Some(index);
                    self.arrow_shift_index = None;
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
                    background: if is_active {
                        Some(theme.extended_palette().primary.strong.color.into())
                    } else if is_selected {
                        Some(theme.extended_palette().primary.weak.color.into())
                    } else {
                        None
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
    TrackActivated(Track),
}

#[derive(Clone, Debug)]
pub enum Message {
    ArrowDownPress,
    ArrowUpPress,
    ModifiersChange(Modifiers),
    Next,
    Previous,
    TrackActivate(usize),
    TrackListExtend(Vec<Track>),
    TrackSelect(usize),
}

pub struct TrackList {
    active: Option<usize>,
    anchor: Option<usize>,
    arrow_shift_index: Option<usize>,
    modifiers: Modifiers,
    selected: HashSet<usize>,
    tracks: Vec<Track>,
}
