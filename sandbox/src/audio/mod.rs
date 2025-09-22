use std::fs::File;
use std::path::Path;
use rodio::{Decoder, OutputStream, Sink, Source};

pub struct AudioEngine {
    stream: OutputStream,
}

impl AudioEngine {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            stream: rodio::OutputStreamBuilder::open_default_stream()
                .map_err(|err| err.to_string())?,
        })
    }

    pub fn play_sound(&self, sound_path: impl AsRef<Path>) -> Result<(), String> {
        let file = File::open(sound_path).map_err(|err| err.to_string())?;
        let source = Decoder::try_from(file).map_err(|err| err.to_string())?.amplify(0.8);
        let sink = Sink::connect_new(self.stream.mixer());
        sink.append(source);
        sink.detach();
        Ok(())
    }
}
