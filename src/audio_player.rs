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
        !self.player.is_paused() && !self.player.empty()
    }

    pub fn new(on_track_end: Arc<dyn Fn() + Send + Sync>, volume: f32) -> Self {
        let mixer_device_sink = DeviceSinkBuilder::open_default_sink().unwrap();
        let player = Player::connect_new(mixer_device_sink.mixer());
        player.set_volume(volume);
        Self {
            _mixer_device_sink: mixer_device_sink,
            on_track_end,
            player,
            volume,
        }
    }

    pub fn pause_or_play(&self) {
        if self.player.is_paused() {
            self.player.play();
        } else {
            self.player.pause();
        }
    }

    pub fn play(&mut self, track: &Track) -> Result<(), Box<dyn Error>> {
        let file = File::open(&track.path)?;
        let decoder = Decoder::try_from(file)?;
        let on_track_end = self.on_track_end.clone();
        self.player.stop();
        self.player
            .append(decoder.amplify_decibel(track.replay_gain_f32()));
        self.player
            .append(EmptyCallback::new(Box::new(move || on_track_end())));
        Ok(())
    }

    pub fn position(&self) -> f32 {
        self.player.get_pos().as_secs_f32()
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.player.set_volume(volume);
        self.volume = volume;
    }

    pub fn try_seek(&self, seconds: f32) {
        self.player.try_seek(Duration::from_secs_f32(seconds)).ok();
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }
}

pub struct AudioPlayer {
    _mixer_device_sink: MixerDeviceSink,
    on_track_end: Arc<dyn Fn() + Send + Sync>,
    player: Player,
    volume: f32,
}
