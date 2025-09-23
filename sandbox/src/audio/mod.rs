use std::fs::File;
use std::path::Path;
use rodio::{Decoder, OutputStream, Sink, Source};

pub struct AudioEngine {
    stream: OutputStream,
    volume: f32,
}

impl AudioEngine {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            stream: rodio::OutputStreamBuilder::open_default_stream()
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
        let file = File::open(path).map_err(|err| err.to_string())?;
        let source = Decoder::try_from(file)
            .map_err(|err| err.to_string())?
            .amplify(self.volume);
        let sink = Sink::connect_new(self.stream.mixer());
        sink.append(source);
        sink.detach();
        Ok(())
    }
}
