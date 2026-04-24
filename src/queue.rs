use {
    crate::track::Track,
    std::sync::Arc,
};

impl Queue {
    pub fn extend(&mut self, tracks: Vec<Arc<Track>>) {
        self.tracks.extend(tracks);
        if self.shuffle {
            fastrand::shuffle(&mut self.tracks);
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

    pub fn set_repeat(&mut self, repeat: bool) {
        self.repeat = repeat;
    }

    pub fn set_shuffle(&mut self, shuffle: bool) {
        self.shuffle = shuffle;
        if self.shuffle {
            fastrand::shuffle(self.tracks.as_mut_slice());
        }
    }

    pub fn shuffle(&self) -> bool {
        self.shuffle
    }
}

#[cfg(test)]
#[path = "queue_test.rs"]
mod tests;

#[derive(Clone, Debug, Default)]
pub struct Queue {
    current: Option<Arc<Track>>,
    repeat: bool,
    shuffle: bool,
    tracks: Vec<Arc<Track>>,
}
