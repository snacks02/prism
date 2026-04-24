use {
    crate::track::Track,
    async_channel::{
        Receiver,
        Sender,
    },
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
        hash,
        sync::Arc,
        time::Duration,
    },
};

impl AudioPlayer {
    pub fn active(&self) -> bool {
        !self.player.is_paused() && !self.player.empty()
    }

    pub fn new(volume: f32) -> Self {
        let (track_end_sender, track_end_receiver) = async_channel::unbounded::<()>();
        let mixer_device_sink = DeviceSinkBuilder::open_default_sink().unwrap();
        let player = Player::connect_new(mixer_device_sink.mixer());
        player.set_volume(volume);
        Self {
            _mixer_device_sink: mixer_device_sink,
            player,
            track_end_receiver: TrackEndReceiver(Arc::new(track_end_receiver)),
            track_end_sender,
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
        let sender = self.track_end_sender.clone();
        self.player.stop();
        self.player
            .append(decoder.amplify_decibel(track.replay_gain_f32()));
        self.player.append(EmptyCallback::new(Box::new(move || {
            sender.try_send(()).ok();
        })));
        Ok(())
    }

    pub fn position(&self) -> f32 {
        self.player.get_pos().as_secs_f32()
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.player.set_volume(volume);
        self.volume = volume;
    }

    pub fn track_end_receiver(&self) -> TrackEndReceiver {
        self.track_end_receiver.clone()
    }

    pub fn try_seek(&self, seconds: f32) {
        self.player.try_seek(Duration::from_secs_f32(seconds)).ok();
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }
}

impl hash::Hash for TrackEndReceiver {
    fn hash<Hasher: hash::Hasher>(&self, state: &mut Hasher) {
        Arc::as_ptr(&self.0).hash(state);
    }
}

pub struct AudioPlayer {
    _mixer_device_sink: MixerDeviceSink,
    player: Player,
    track_end_receiver: TrackEndReceiver,
    track_end_sender: Sender<()>,
    volume: f32,
}

#[derive(Clone, Debug)]
pub struct TrackEndReceiver(pub Arc<Receiver<()>>);
