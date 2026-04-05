use {
    std::{
        fs::File,
        path::Path,
    },
    symphonia::{
        core::{
            formats::FormatOptions,
            io::MediaSourceStream,
            meta::{
                MetadataOptions,
                StandardTagKey,
            },
            probe::Hint,
        },
        default,
    },
    walkdir::WalkDir,
};

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
    let mut probe_result = default::get_probe()
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

    let mut probe_result = default::get_probe()
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
    let mut replay_gain = 0.0_f32;
    let mut title = fallback_title;

    for tag in format_tags.iter().chain(probe_tags.iter()) {
        match tag.std_key {
            Some(StandardTagKey::Album) => album = tag.value.to_string(),
            Some(StandardTagKey::Artist) => artist = tag.value.to_string(),
            Some(StandardTagKey::ReplayGainTrackGain) => {
                if let Ok(value) = tag.value.to_string().trim_end_matches(" dB").parse::<f32>() {
                    replay_gain = value;
                }
            }
            Some(StandardTagKey::TrackTitle) => title = tag.value.to_string(),
            _ => {}
        }
    }

    let duration = probe_result.format.default_track().and_then(|track| {
        let n_frames = track.codec_params.n_frames?;
        let sample_rate = track.codec_params.sample_rate?;
        Some(n_frames as f32 / sample_rate as f32)
    });

    Some(Track {
        album,
        artist,
        duration,
        file_path: path.to_string_lossy().into_owned(),
        replay_gain,
        title,
    })
}

#[derive(Clone, Debug)]
pub struct Track {
    pub album: String,
    pub artist: String,
    pub duration: Option<f32>,
    pub file_path: String,
    pub replay_gain: f32,
    pub title: String,
}
