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
