use lofty::config::ParseOptions;
use lofty::prelude::{
    Accessor,
    TaggedFileExt,
};
use lofty::probe::Probe;
use std::path::Path;
use walkdir::WalkDir;

pub fn from_directory(directory: &Path) -> Vec<Track> {
    WalkDir::new(directory)
        .sort_by_file_name()
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| from_file(entry.path()))
        .collect()
}

pub fn from_file(path: &Path) -> Option<Track> {
    let fallback_title = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("Unknown")
        .to_owned();
    let tagged_file = Probe::open(path)
        .ok()?
        .options(ParseOptions::new().read_cover_art(false))
        .read()
        .ok()?;
    let tag = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag());
    let album = tag
        .and_then(|tag| tag.album().map(|value| value.into_owned()))
        .unwrap_or_else(|| "Unknown".to_owned());
    let artist = tag
        .and_then(|tag| tag.artist().map(|value| value.into_owned()))
        .unwrap_or_else(|| "Unknown".to_owned());
    let title = tag
        .and_then(|tag| tag.title().map(|value| value.into_owned()))
        .unwrap_or(fallback_title);
    Some(Track {
        album,
        artist,
        file_path: path.to_string_lossy().into_owned(),
        title,
    })
}

#[derive(Clone, Debug)]
pub struct Track {
    pub album: String,
    pub artist: String,
    pub file_path: String,
    pub title: String,
}
