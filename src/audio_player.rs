use {
    crate::track::Track,
    rodio::{
        Decoder,
        DeviceSinkBuilder,
        MixerDeviceSink,
        Player,
        Source,
        source::EmptyCallback,
    },
    std::{
        error::Error,
        fs::File,
        sync::Arc,
        time::Duration,
    },
};

impl AudioPlayer {
    pub fn active(&self) -> bool {
        self.player
            .as_ref()
            .is_some_and(|player| !player.is_paused())
    }

    pub fn new(on_track_end: Arc<dyn Fn() + Send + Sync>, volume: f32) -> Self {
        Self {
            mixer_device_sink: DeviceSinkBuilder::open_default_sink().unwrap(),
            on_track_end,
            player: None,
            volume,
        }
    }

    pub fn pause_or_play(&self) {
        if let Some(player) = &self.player {
            if player.is_paused() {
                player.play();
            } else {
                player.pause();
            }
        }
    }

    pub fn play(&mut self, track: &Track) -> Result<(), Box<dyn Error>> {
        let file = File::open(&track.path)?;
        let decoder = Decoder::try_from(file)?;
        let on_track_end = Arc::clone(&self.on_track_end);
        let player = Player::connect_new(self.mixer_device_sink.mixer());
        player.set_volume(self.volume);
        player.append(decoder.amplify_decibel(track.replay_gain.unwrap_or(0.0)));
        player.append(EmptyCallback::new(Box::new(move || on_track_end())));
        self.player = Some(player);
        Ok(())
    }

    pub fn position(&self) -> f32 {
        self.player
            .as_ref()
            .map(|player| player.get_pos().as_secs_f32())
            .unwrap_or(0.0)
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
        if let Some(player) = &self.player {
            player.set_volume(volume);
        }
    }

    pub fn try_seek(&self, seconds: f32) {
        if let Some(player) = &self.player {
            player.try_seek(Duration::from_secs_f32(seconds)).ok();
        }
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }
}

pub struct AudioPlayer {
    mixer_device_sink: MixerDeviceSink,
    on_track_end: Arc<dyn Fn() + Send + Sync>,
    player: Option<Player>,
    volume: f32,
}
