use {
    crate::track::Track,
    std::{
        fs::File,
        path::Path,
        time::Duration,
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

    probe_result
        .format
        .metadata()
        .current()
        .and_then(|revision| {
            revision
                .visuals()
                .first()
                .map(|visual| visual.data.to_vec())
        })
        .or_else(|| {
            probe_result.metadata.get().and_then(|metadata| {
                metadata.current().and_then(|revision| {
                    revision
                        .visuals()
                        .first()
                        .map(|visual| visual.data.to_vec())
                })
            })
        })
}

pub fn from_directory(path: &Path) -> Vec<Track> {
    WalkDir::new(path)
        .sort_by_file_name()
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| from_file(entry.path()))
        .collect()
}

pub fn from_file(path: &Path) -> Option<Track> {
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

    let mut album = None;
    let mut artist = None;
    let mut replay_gain = None;
    let mut title = None;

    for tag in format_tags.iter().chain(probe_tags.iter()) {
        let value = tag.value.to_string();
        match tag.std_key {
            Some(StandardTagKey::Album) => album = Some(value),
            Some(StandardTagKey::Artist) => artist = Some(value),
            Some(StandardTagKey::ReplayGainTrackGain) => {
                if let Ok(parsed_value) = value.trim_end_matches(" dB").parse() {
                    replay_gain = Some(parsed_value);
                }
            }
            Some(StandardTagKey::TrackTitle) => title = Some(value),
            _ => {}
        }
    }

    let duration = probe_result.format.default_track().and_then(|track| {
        let n_frames = track.codec_params.n_frames?;
        let sample_rate = track.codec_params.sample_rate? as u64;
        let seconds = n_frames / sample_rate;
        let nanoseconds = (n_frames % sample_rate) * 1_000_000_000 / sample_rate;
        Some(Duration::new(seconds, nanoseconds as u32))
    });

    Some(Track {
        album,
        artist,
        duration,
        path: path.to_owned(),
        replay_gain,
        title,
    })
}
