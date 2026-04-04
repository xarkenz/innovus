use std::fs::File;
use std::path::Path;

pub struct AudioEngine {
    handle: rodio::MixerDeviceSink,
    volume: f32,
}

impl AudioEngine {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            handle: rodio::DeviceSinkBuilder::open_default_sink()
                .map_err(|err| err.to_string())?,
            volume: 0.8,
        })
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }

    pub fn play_sound(&self, path: impl AsRef<Path>) -> Result<(), String> {
        use rodio::Source;
        let path = path.as_ref();
        let file = File::open(path.with_extension("ogg"))
            .map_err(|err| err.to_string())?;
        let source = rodio::Decoder::try_from(file)
            .map_err(|err| err.to_string())?
            .amplify(self.volume);
        let player = rodio::Player::connect_new(self.handle.mixer());
        player.append(source);
        player.detach();
        Ok(())
    }
}
