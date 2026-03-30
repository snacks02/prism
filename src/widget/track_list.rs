use crate::track::Track;
use iced::event;
use iced::keyboard;
use iced::widget::text::{
    Ellipsis,
    Wrapping,
};
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
    Subscription,
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
    let index = match track_list.shift_arrow_index.or(track_list.anchor) {
        Some(current) => step(current, track_list.tracks.len()),
        None => 0,
    };
    if track_list.keyboard_modifiers.shift() {
        track_list.shift_arrow_index = Some(index);
        let anchor = track_list.anchor.unwrap_or(index);
        track_list.selected.clear();
        track_list
            .selected
            .extend(anchor.min(index)..=anchor.max(index));
    } else {
        track_list.anchor = Some(index);
        track_list.selected.clear();
        track_list.selected.insert(index);
        track_list.shift_arrow_index = None;
    }
}

impl TrackList {
    pub fn new() -> Self {
        Self {
            active: None,
            anchor: None,
            keyboard_modifiers: keyboard::Modifiers::default(),
            selected: HashSet::new(),
            shift_arrow_index: None,
            tracks: vec![],
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        event::listen_with(|event, _status, _window| match event {
            iced::Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => match key {
                keyboard::Key::Named(keyboard::key::Named::ArrowDown) => {
                    Some(Message::ArrowDownPress)
                }
                keyboard::Key::Named(keyboard::key::Named::ArrowUp) => Some(Message::ArrowUpPress),
                _ => None,
            },
            iced::Event::Keyboard(keyboard::Event::ModifiersChanged(keyboard_modifiers)) => {
                Some(Message::KeyboardModifiersChange(keyboard_modifiers))
            }
            _ => None,
        })
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
            Message::KeyboardModifiersChange(keyboard_modifiers) => {
                self.keyboard_modifiers = keyboard_modifiers;
                Event::None
            }
            Message::NextPress => {
                if self.tracks.is_empty() {
                    return Event::None;
                }
                let index = self
                    .active
                    .map_or(0, |index| (index + 1).min(self.tracks.len() - 1));
                activate_track(index, self)
            }
            Message::PreviousPress => {
                if self.tracks.is_empty() {
                    return Event::None;
                }
                let index = self.active.map_or(0, |index| index.saturating_sub(1));
                activate_track(index, self)
            }
            Message::TrackDoubleClick(index) => activate_track(index, self),
            Message::TrackListExtend(tracks) => {
                self.tracks.extend(tracks);
                Event::None
            }
            Message::TrackPress(index) => {
                if self.keyboard_modifiers.shift() {
                    let anchor = self.anchor.unwrap_or(index);
                    if !self.keyboard_modifiers.control() {
                        self.selected.clear();
                    }
                    self.shift_arrow_index = Some(index);
                    self.selected.extend(anchor.min(index)..=anchor.max(index));
                } else if self.keyboard_modifiers.control() {
                    if !self.selected.remove(&index) {
                        self.selected.insert(index);
                    }
                    self.anchor = Some(index);
                    self.shift_arrow_index = None;
                } else {
                    self.anchor = Some(index);
                    self.selected.clear();
                    self.selected.insert(index);
                    self.shift_arrow_index = None;
                }
                Event::None
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
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

        let rows = self.tracks.iter().enumerate().map(|(index, track)| {
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
                .style(move |theme: &iced::Theme| container::Style {
                    background: if is_active {
                        Some(theme.palette().primary.strong.color.into())
                    } else if is_selected {
                        Some(theme.palette().primary.weak.color.into())
                    } else {
                        None
                    },
                    ..container::Style::default()
                })
                .width(Length::Fill),
            )
            .on_double_click(Message::TrackDoubleClick(index))
            .on_press(Message::TrackPress(index))
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
    KeyboardModifiersChange(keyboard::Modifiers),
    NextPress,
    PreviousPress,
    TrackDoubleClick(usize),
    TrackListExtend(Vec<Track>),
    TrackPress(usize),
}

pub struct TrackList {
    active: Option<usize>,
    anchor: Option<usize>,
    keyboard_modifiers: keyboard::Modifiers,
    selected: HashSet<usize>,
    shift_arrow_index: Option<usize>,
    tracks: Vec<Track>,
}
