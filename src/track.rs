use std::{
    path::PathBuf,
    time::Duration,
};

impl Track {
    pub fn album_str(&self) -> &str {
        self.album.as_deref().unwrap_or("")
    }

    pub fn artist_str(&self) -> &str {
        self.artist.as_deref().unwrap_or("")
    }

    pub fn duration_seconds(&self) -> f32 {
        self.duration
            .map(|duration| duration.as_secs_f32())
            .unwrap_or(0.0)
    }

    pub fn replay_gain_f32(&self) -> f32 {
        self.replay_gain.unwrap_or(0.0)
    }

    pub fn title_str(&self) -> &str {
        self.title.as_deref().unwrap_or("")
    }
}

#[derive(Clone, Debug)]
pub struct Track {
    pub album: Option<String>,
    pub artist: Option<String>,
    pub duration: Option<Duration>,
    pub path: PathBuf,
    pub replay_gain: Option<f32>,
    pub title: Option<String>,
}
