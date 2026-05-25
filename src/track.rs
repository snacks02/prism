use {
    std::{
        fs,
        fs::File,
        path::{
            Path,
            PathBuf,
        },
        time::Duration,
    },
    symphonia::{
        core::{
            io::MediaSourceStream,
            meta::{
                StandardTagKey,
                Tag,
            },
            probe::ProbeResult,
        },
        default,
    },
};

fn collect_tags(probe_result: &mut ProbeResult) -> Vec<Tag> {
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
    format_tags.into_iter().chain(probe_tags).collect()
}

fn from_directory(path: &Path) -> Vec<Track> {
    let mut paths: Vec<PathBuf> = fs::read_dir(path)
        .into_iter()
        .flatten()
        .filter_map(|directory_entry| Some(directory_entry.ok()?.path()))
        .collect();
    paths.sort();
    paths.iter().flat_map(|path| from_path(path)).collect()
}

fn from_file(path: &Path) -> Option<Track> {
    let mut probe_result = probe_file(path)?;

    let mut album = None;
    let mut artist = None;
    let mut replay_gain = None;
    let mut title = None;
    for tag in collect_tags(&mut probe_result) {
        let Some(std_key) = tag.std_key else { continue };
        let value = tag.value.to_string();
        match std_key {
            StandardTagKey::Album => album = Some(value),
            StandardTagKey::Artist => artist = Some(value),
            StandardTagKey::ReplayGainTrackGain => {
                replay_gain = value.trim_end_matches(" dB").parse().ok();
            }
            StandardTagKey::TrackTitle => title = Some(value),
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

fn probe_file(path: &Path) -> Option<ProbeResult> {
    default::get_probe()
        .format(
            &Default::default(),
            MediaSourceStream::new(Box::new(File::open(path).ok()?), Default::default()),
            &Default::default(),
            &Default::default(),
        )
        .ok()
}

impl Track {
    pub fn album_str(&self) -> &str {
        self.album.as_deref().unwrap_or_default()
    }

    pub fn artist_str(&self) -> &str {
        self.artist.as_deref().unwrap_or_default()
    }

    pub fn duration_seconds(&self) -> f32 {
        self.duration.map_or(0.0, |duration| duration.as_secs_f32())
    }

    pub fn replay_gain_f32(&self) -> f32 {
        self.replay_gain.unwrap_or(0.0)
    }

    pub fn title_str(&self) -> &str {
        self.title.as_deref().unwrap_or_default()
    }
}

pub fn cover_from_file(path: &Path) -> Option<Vec<u8>> {
    let mut probe_result = probe_file(path)?;

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

pub fn from_path(path: &Path) -> Vec<Track> {
    if path.is_dir() {
        from_directory(path)
    } else {
        from_file(path).into_iter().collect()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Track {
    pub album: Option<String>,
    pub artist: Option<String>,
    pub duration: Option<Duration>,
    pub path: PathBuf,
    pub replay_gain: Option<f32>,
    pub title: Option<String>,
}
