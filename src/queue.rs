use {
    crate::track::Track,
    iced::futures::channel::mpsc,
    std::{
        hash,
        sync::{
            Arc,
            Mutex,
        },
    },
};

impl Default for Queue {
    fn default() -> Self {
        let (track_end_sender, track_end_receiver) = mpsc::unbounded();
        Self {
            current: None,
            track_end_receiver: TrackEndReceiver(Arc::new(Mutex::new(Some(track_end_receiver)))),
            track_end_sender,
            tracks: vec![],
        }
    }
}

impl Queue {
    pub fn next(&mut self) -> Option<&Arc<Track>> {
        let current = self.current.clone();
        let next = match current.as_ref() {
            None => self.tracks.first().cloned(),
            Some(current) => self
                .tracks
                .iter()
                .skip_while(|track| !Arc::ptr_eq(*track, current))
                .nth(1)
                .cloned(),
        };
        if next.is_some() {
            self.current = next;
            self.current.as_ref()
        } else {
            None
        }
    }

    pub fn previous(&mut self) -> Option<&Arc<Track>> {
        let current = self.current.clone();
        self.current = match current.as_ref() {
            None => self.tracks.first().cloned(),
            Some(current) => Some(
                self.tracks
                    .iter()
                    .take_while(|track| !Arc::ptr_eq(*track, current))
                    .last()
                    .unwrap_or(current)
                    .clone(),
            ),
        };
        self.current.as_ref()
    }

    pub fn set(&mut self, track: &Arc<Track>, tracks: Vec<Arc<Track>>) {
        self.current = Some(Arc::clone(track));
        self.tracks = tracks;
    }

    pub fn track_end_receiver(&self) -> TrackEndReceiver {
        self.track_end_receiver.clone()
    }

    pub fn track_end_sender(&self) -> TrackEndSender {
        self.track_end_sender.clone()
    }
}

pub struct Queue {
    current: Option<Arc<Track>>,
    track_end_receiver: TrackEndReceiver,
    track_end_sender: TrackEndSender,
    tracks: Vec<Arc<Track>>,
}

#[derive(Clone, Debug)]
pub struct TrackEndReceiver(
    pub Arc<Mutex<Option<iced::futures::channel::mpsc::UnboundedReceiver<()>>>>,
);

pub type TrackEndSender = mpsc::UnboundedSender<()>;

impl hash::Hash for TrackEndReceiver {
    fn hash<Hasher: hash::Hasher>(&self, state: &mut Hasher) {
        Arc::as_ptr(&self.0).hash(state);
    }
}
