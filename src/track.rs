use std::fs::File;
use std::path::Path;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::{
    MetadataOptions,
    StandardTagKey,
};
use symphonia::core::probe::Hint;
use walkdir::WalkDir;

pub fn from_directory(directory: &Path) -> Vec<Track> {
    WalkDir::new(directory)
        .sort_by_file_name()
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| from_file(entry.path()))
        .collect()
}

pub fn cover_from_file(path: &Path) -> Option<Vec<u8>> {
    let file = File::open(path).ok()?;
    let mut probe_result = symphonia::default::get_probe()
        .format(
            &Hint::new(),
            MediaSourceStream::new(Box::new(file), Default::default()),
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .ok()?;

    let visual = probe_result
        .format
        .metadata()
        .current()
        .and_then(|revision| {
            revision
                .visuals()
                .first()
                .map(|visual| visual.data.to_vec())
        });

    if visual.is_some() {
        return visual;
    }

    probe_result.metadata.get().and_then(|metadata| {
        metadata.current().and_then(|revision| {
            revision
                .visuals()
                .first()
                .map(|visual| visual.data.to_vec())
        })
    })
}

pub fn from_file(path: &Path) -> Option<Track> {
    let fallback_title = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("Unknown")
        .to_string();
    let file = File::open(path).ok()?;

    let mut probe_result = symphonia::default::get_probe()
        .format(
            &Hint::new(),
            MediaSourceStream::new(Box::new(file), Default::default()),
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .ok()?;

    let format_tags = probe_result
        .format
        .metadata()
        .current()
        .map(|revision| revision.tags().to_vec())
        .unwrap_or_default();
    let probe_tags = probe_result
        .metadata
        .get()
        .and_then(|metadata| metadata.current().map(|revision| revision.tags().to_vec()))
        .unwrap_or_default();

    let mut album = "Unknown".to_string();
    let mut artist = "Unknown".to_string();
    let mut title = fallback_title;

    for tag in format_tags.iter().chain(probe_tags.iter()) {
        match tag.std_key {
            Some(StandardTagKey::TrackTitle) => title = tag.value.to_string(),
            Some(StandardTagKey::Album) => album = tag.value.to_string(),
            Some(StandardTagKey::Artist) => artist = tag.value.to_string(),
            _ => {}
        }
    }

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
