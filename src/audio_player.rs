use {
    crate::track::Track,
    rodio::{
        Decoder,
        DeviceSinkBuilder,
        MixerDeviceSink,
        Player,
        source::{
            EmptyCallback,
            Source,
        },
    },
    smol::channel,
    smol::channel::Receiver,
    std::{
        error::Error,
        fs::File,
        time::Duration,
    },
};

impl AudioPlayer {
    pub fn active(&self) -> bool {
        !self.player.is_paused() && !self.player.empty()
    }

    pub fn new(volume: f32) -> Self {
        let mixer_device_sink = DeviceSinkBuilder::open_default_sink().unwrap();
        let player = Player::connect_new(mixer_device_sink.mixer());
        player.set_volume(volume);
        Self {
            _mixer_device_sink: mixer_device_sink,
            player,
        }
    }

    pub fn pause_or_play(&self) {
        if self.player.is_paused() {
            self.player.play();
        } else {
            self.player.pause();
        }
    }

    pub fn play(&self, track: &Track) -> Result<Receiver<()>, Box<dyn Error>> {
        let (audio_end_sender, audio_end_receiver) = channel::bounded(1);
        let decoder = Decoder::new(File::open(&track.path)?)?;
        self.player.stop();
        self.player
            .append(decoder.amplify_decibel(track.replay_gain_f32()));
        self.player.append(EmptyCallback::new(Box::new(move || {
            audio_end_sender.try_send(()).ok();
        })));
        self.player.play();
        Ok(audio_end_receiver)
    }

    pub fn position(&self) -> f32 {
        self.player.get_pos().as_secs_f32()
    }

    pub fn set_volume(&self, volume: f32) {
        self.player.set_volume(volume);
    }

    pub fn try_seek(&self, seconds: f32) {
        self.player.try_seek(Duration::from_secs_f32(seconds)).ok();
    }

    pub fn volume(&self) -> f32 {
        self.player.volume()
    }
}

pub struct AudioPlayer {
    _mixer_device_sink: MixerDeviceSink,
    player: Player,
}
