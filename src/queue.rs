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

    pub fn repeat_disable(&mut self) {
        self.repeat = false;
    }

    pub fn repeat_enable(&mut self) {
        self.repeat = true;
    }

    pub fn set_current(&mut self, track: &Arc<Track>) {
        assert!(self.tracks.iter().any(|queued| Arc::ptr_eq(queued, track)));
        self.current = Some(track.clone());
    }

    pub fn shuffle(&self) -> bool {
        self.shuffle
    }

    pub fn shuffle_disable(&mut self, tracks: &[Arc<Track>]) {
        self.shuffle = false;
        self.tracks = tracks.to_vec();
    }

    pub fn shuffle_enable(&mut self) {
        self.shuffle = true;
        fastrand::shuffle(self.tracks.as_mut_slice());
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
