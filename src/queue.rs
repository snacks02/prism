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

    pub fn next(&mut self, current: Option<&Arc<Track>>) -> Option<Arc<Track>> {
        match current {
            None => self.tracks.first(),
            Some(current) => self
                .tracks
                .iter()
                .skip_while(|&track| !Arc::ptr_eq(track, current))
                .nth(1)
                .or_else(|| self.tracks.first().filter(|_| self.repeat)),
        }
        .cloned()
    }

    pub fn previous(&mut self, current: Option<&Arc<Track>>) -> Option<Arc<Track>> {
        match current {
            None => self.tracks.first(),
            Some(current) => self
                .tracks
                .iter()
                .take_while(|&track| !Arc::ptr_eq(track, current))
                .last()
                .or_else(|| self.tracks.last().filter(|_| self.repeat)),
        }
        .cloned()
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

    pub fn shuffle(&self) -> bool {
        self.shuffle
    }

    pub fn shuffle_disable(&mut self, tracks: &[Arc<Track>]) {
        self.shuffle = false;
        self.tracks = tracks.to_vec();
    }

    pub fn shuffle_enable(&mut self) {
        self.shuffle = true;
        fastrand::shuffle(&mut self.tracks);
    }
}

#[cfg(test)]
#[path = "queue_test.rs"]
mod tests;

#[derive(Clone, Debug, Default)]
pub struct Queue {
    repeat: bool,
    shuffle: bool,
    tracks: Vec<Arc<Track>>,
}
