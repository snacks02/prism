use {
    crate::track::Track,
    std::{
        hash,
        sync::Arc,
    },
};

pub struct Queue {
    current: Option<Arc<Track>>,
    repeat: bool,
    shuffle: bool,
    track_end_receiver: TrackEndReceiver,
    track_end_sender: TrackEndSender,
    tracks: Vec<Arc<Track>>,
}

impl Default for Queue {
    fn default() -> Self {
        let (track_end_sender, track_end_receiver) = async_channel::unbounded();
        Self {
            current: None,
            repeat: false,
            shuffle: false,
            track_end_receiver: TrackEndReceiver(Arc::new(track_end_receiver)),
            track_end_sender,
            tracks: vec![],
        }
    }
}

impl Queue {
    pub fn next(&mut self) -> Option<&Arc<Track>> {
        let next = match self.current.as_ref() {
            None => self.tracks.first().cloned(),
            Some(current) => self
                .tracks
                .iter()
                .skip_while(|track| !Arc::ptr_eq(*track, current))
                .nth(1)
                .cloned()
                .or_else(|| {
                    if self.repeat {
                        self.tracks.first().cloned()
                    } else {
                        None
                    }
                }),
        };
        if next.is_some() {
            self.current = next;
            self.current.as_ref()
        } else {
            None
        }
    }

    pub fn previous(&mut self) -> Option<&Arc<Track>> {
        let previous = match self.current.as_ref() {
            None => self.tracks.first().cloned(),
            Some(current) => self
                .tracks
                .iter()
                .take_while(|track| !Arc::ptr_eq(*track, current))
                .last()
                .cloned()
                .or_else(|| {
                    if self.repeat {
                        self.tracks.last().cloned()
                    } else {
                        None
                    }
                }),
        };
        if previous.is_some() {
            self.current = previous;
            self.current.as_ref()
        } else {
            None
        }
    }

    pub fn repeat(&self) -> bool {
        self.repeat
    }

    pub fn set(&mut self, track: &Arc<Track>, mut tracks: Vec<Arc<Track>>) {
        if self.shuffle {
            fastrand::shuffle(&mut tracks);
        }
        self.current = Some(Arc::clone(track));
        self.tracks = tracks;
    }

    pub fn shuffle(&self) -> bool {
        self.shuffle
    }

    pub fn toggle_repeat(&mut self) {
        self.repeat = !self.repeat;
    }

    pub fn toggle_shuffle(&mut self) {
        self.shuffle = !self.shuffle;
        if self.shuffle {
            fastrand::shuffle(&mut self.tracks);
        }
    }

    pub fn track_end_receiver(&self) -> TrackEndReceiver {
        self.track_end_receiver.clone()
    }

    pub fn track_end_sender(&self) -> TrackEndSender {
        self.track_end_sender.clone()
    }
}

#[derive(Clone, Debug)]
pub struct TrackEndReceiver(pub Arc<async_channel::Receiver<()>>);

pub type TrackEndSender = async_channel::Sender<()>;

impl hash::Hash for TrackEndReceiver {
    fn hash<Hasher: hash::Hasher>(&self, state: &mut Hasher) {
        Arc::as_ptr(&self.0).hash(state);
    }
}
