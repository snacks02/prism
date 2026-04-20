use {
    crate::track::Track,
    std::{
        hash,
        sync::Arc,
    },
};

impl Default for Queue {
    fn default() -> Self {
        let (track_end_sender, track_end_receiver) = async_channel::unbounded();
        Self {
            current: None,
            repeat: false,
            shuffle: false,
            track_end_receiver: TrackEndReceiver(Arc::new(track_end_receiver)),
            track_end_sender,
            tracks: Arc::new(vec![]),
        }
    }
}

impl Queue {
    pub fn extend(&mut self, tracks: Vec<Arc<Track>>) {
        if self.shuffle {
            let queued = Arc::make_mut(&mut self.tracks);
            queued.extend(tracks);
            fastrand::shuffle(queued);
        } else {
            Arc::make_mut(&mut self.tracks).extend(tracks);
        }
    }

    pub fn next(&mut self) -> Option<&Arc<Track>> {
        let next = match self.current.as_ref() {
            None => self.tracks.first(),
            Some(current) => self
                .tracks
                .iter()
                .skip_while(|track| !Arc::ptr_eq(*track, current))
                .nth(1)
                .or_else(|| {
                    if self.repeat {
                        self.tracks.first()
                    } else {
                        None
                    }
                }),
        };
        self.current = Some(next?.clone());
        self.current.as_ref()
    }

    pub fn previous(&mut self) -> Option<&Arc<Track>> {
        let previous = match self.current.as_ref() {
            None => self.tracks.first(),
            Some(current) => self
                .tracks
                .iter()
                .take_while(|track| !Arc::ptr_eq(*track, current))
                .last()
                .or_else(|| {
                    if self.repeat {
                        self.tracks.last()
                    } else {
                        None
                    }
                }),
        };
        self.current = Some(previous?.clone());
        self.current.as_ref()
    }

    pub fn repeat(&self) -> bool {
        self.repeat
    }

    pub fn set_current(&mut self, track: &Arc<Track>) {
        assert!(self.tracks.iter().any(|queued| Arc::ptr_eq(queued, track)));
        self.current = Some(track.clone());
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
            fastrand::shuffle(Arc::make_mut(&mut self.tracks).as_mut_slice());
        }
    }

    pub fn track_end_receiver(&self) -> TrackEndReceiver {
        self.track_end_receiver.clone()
    }

    pub fn track_end_sender(&self) -> TrackEndSender {
        self.track_end_sender.clone()
    }
}

impl hash::Hash for TrackEndReceiver {
    fn hash<Hasher: hash::Hasher>(&self, state: &mut Hasher) {
        Arc::as_ptr(&self.0).hash(state);
    }
}

#[derive(Clone, Debug)]
pub struct Queue {
    current: Option<Arc<Track>>,
    repeat: bool,
    shuffle: bool,
    track_end_receiver: TrackEndReceiver,
    track_end_sender: TrackEndSender,
    tracks: Arc<Vec<Arc<Track>>>,
}

#[derive(Clone, Debug)]
pub struct TrackEndReceiver(pub Arc<async_channel::Receiver<()>>);

pub type TrackEndSender = async_channel::Sender<()>;
