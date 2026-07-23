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
            formats::{
                FormatReader,
                TrackType,
                probe::Hint,
            },
            io::MediaSourceStream,
            meta::StandardTag,
            units::Timestamp,
        },
        default,
    },
};

fn from_directory(path: &Path) -> Vec<Track> {
    let mut paths: Vec<PathBuf> = fs::read_dir(path)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect();
    paths.sort();
    paths.iter().flat_map(|path| from_path(path)).collect()
}

fn from_file(path: &Path) -> Option<Track> {
    let mut format = probe_file(path)?;

    let mut album = None;
    let mut artist = None;
    let mut replay_gain = None;
    let mut title = None;
    if let Some(revision) = format.metadata().skip_to_latest() {
        for tag in &revision.media.tags {
            if let Some(standard_tag) = &tag.std {
                match standard_tag {
                    StandardTag::Album(value) => album = Some(value.to_string()),
                    StandardTag::Artist(value) => artist = Some(value.to_string()),
                    StandardTag::ReplayGainTrackGain(value) => {
                        replay_gain = value.trim_end_matches(" dB").parse().ok();
                    }
                    StandardTag::TrackTitle(value) => title = Some(value.to_string()),
                    _ => {}
                }
            }
        }
    }

    let duration = format.default_track(TrackType::Audio).and_then(|track| {
        track
            .time_base?
            .calc_time(Timestamp::try_from(track.duration?.get()).ok()?)
            .and_then(|time| u64::try_from(time.as_nanos()).ok())
            .map(Duration::from_nanos)
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

fn probe_file(path: &Path) -> Option<Box<dyn FormatReader>> {
    default::get_probe()
        .probe(
            &Hint::new(),
            MediaSourceStream::new(Box::new(File::open(path).ok()?), Default::default()),
            Default::default(),
            Default::default(),
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
        self.duration.as_ref().map_or(0.0, Duration::as_secs_f32)
    }

    pub fn replay_gain_f32(&self) -> f32 {
        self.replay_gain.unwrap_or(0.0)
    }

    pub fn title_str(&self) -> &str {
        self.title.as_deref().unwrap_or_default()
    }
}

pub fn cover_from_file(path: &Path) -> Option<Vec<u8>> {
    probe_file(path)?
        .metadata()
        .skip_to_latest()?
        .media
        .visuals
        .first()
        .map(|visual| visual.data.to_vec())
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
