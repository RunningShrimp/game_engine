use rodio::{OutputStream, OutputStreamBuilder, Sink, Decoder, Source};
use std::fs::File;
use std::io::BufReader;
use std::collections::HashMap;

pub struct AudioService {
    _stream: OutputStream,
    sinks: HashMap<String, Sink>,
}

impl AudioService {
    pub fn new() -> Option<Self> {
        match OutputStreamBuilder::open_default_stream() {
            Ok(stream) => Some(Self {
                _stream: stream,
                sinks: HashMap::new(),
            }),
            Err(_) => None,
        }
    }

    pub fn play_sound(&mut self, name: &str, path: &str, volume: f32, looped: bool) {
        if self.sinks.contains_key(name) {
            return;
        }

        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);
            if let Ok(source) = Decoder::new(reader) {
                let sink = Sink::connect_new(self._stream.mixer());
                sink.set_volume(volume);
                if looped {
                    sink.append(source.repeat_infinite());
                } else {
                    sink.append(source);
                }
                self.sinks.insert(name.to_string(), sink);
            } else {
                eprintln!("Failed to decode audio: {}", path);
            }
        } else {
            eprintln!("Failed to open audio file: {}", path);
        }
    }

    pub fn stop_sound(&mut self, name: &str) {
        if let Some(sink) = self.sinks.remove(name) {
            sink.stop();
        }
    }

    pub fn set_volume(&mut self, name: &str, volume: f32) {
        if let Some(sink) = self.sinks.get(name) {
            sink.set_volume(volume);
        }
    }

    pub fn cleanup(&mut self) {
        self.sinks.retain(|_, sink| !sink.empty());
    }
}
