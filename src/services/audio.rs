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

    /// 暂停指定音频
    pub fn pause_sound(&mut self, name: &str) {
        if let Some(sink) = self.sinks.get(name) {
            sink.pause();
        }
    }

    /// 恢复指定音频
    pub fn resume_sound(&mut self, name: &str) {
        if let Some(sink) = self.sinks.get(name) {
            sink.play();
        }
    }

    /// 检查音频是否正在播放
    pub fn is_playing(&self, name: &str) -> bool {
        self.sinks.get(name).map(|s| !s.is_paused() && !s.empty()).unwrap_or(false)
    }

    /// 检查音频是否暂停
    pub fn is_paused(&self, name: &str) -> bool {
        self.sinks.get(name).map(|s| s.is_paused()).unwrap_or(false)
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

pub trait AudioBackend {
    fn play(&mut self, name: &str, path: &str, volume: f32, looped: bool);
    fn stop(&mut self, name: &str);
    fn pause(&mut self, name: &str);
    fn resume(&mut self, name: &str);
    fn set_volume(&mut self, name: &str, volume: f32);
    fn is_playing(&self, name: &str) -> bool;
    fn is_paused(&self, name: &str) -> bool;
    fn cleanup(&mut self);
}

impl AudioBackend for AudioService {
    fn play(&mut self, name: &str, path: &str, volume: f32, looped: bool) { self.play_sound(name, path, volume, looped); }
    fn stop(&mut self, name: &str) { self.stop_sound(name); }
    fn pause(&mut self, name: &str) { self.pause_sound(name); }
    fn resume(&mut self, name: &str) { self.resume_sound(name); }
    fn set_volume(&mut self, name: &str, volume: f32) { AudioService::set_volume(self, name, volume); }
    fn is_playing(&self, name: &str) -> bool { AudioService::is_playing(self, name) }
    fn is_paused(&self, name: &str) -> bool { AudioService::is_paused(self, name) }
    fn cleanup(&mut self) { AudioService::cleanup(self); }
}

pub fn new_backend() -> Option<Box<dyn AudioBackend>> {
    AudioService::new().map(|s| Box::new(s) as Box<dyn AudioBackend>)
}

#[derive(Clone)]
pub enum AudioCommand {
    Play { name: String, path: String, volume: f32, looped: bool },
    Stop { name: String },
    Pause { name: String },
    Resume { name: String },
    SetVolume { name: String, volume: f32 },
    Cleanup,
}

#[derive(bevy_ecs::system::Resource, Clone)]
pub struct AudioQueueResource(pub crossbeam_channel::Sender<AudioCommand>);

pub fn start_audio_driver() -> Option<AudioQueueResource> {
    let (tx, rx) = crossbeam_channel::unbounded::<AudioCommand>();
    std::thread::spawn(move || {
        if let Some(mut backend) = new_backend() {
            loop {
                match rx.recv() {
                    Ok(AudioCommand::Play { name, path, volume, looped }) => backend.play(&name, &path, volume, looped),
                    Ok(AudioCommand::Stop { name }) => backend.stop(&name),
                    Ok(AudioCommand::Pause { name }) => backend.pause(&name),
                    Ok(AudioCommand::Resume { name }) => backend.resume(&name),
                    Ok(AudioCommand::SetVolume { name, volume }) => backend.set_volume(&name, volume),
                    Ok(AudioCommand::Cleanup) => backend.cleanup(),
                    Err(_) => break,
                }
            }
        }
    });
    Some(AudioQueueResource(tx))
}

pub fn audio_play(q: &AudioQueueResource, name: &str, path: &str, volume: f32, looped: bool) {
    let _ = q.0.send(AudioCommand::Play { name: name.to_string(), path: path.to_string(), volume, looped });
}

pub fn audio_stop(q: &AudioQueueResource, name: &str) {
    let _ = q.0.send(AudioCommand::Stop { name: name.to_string() });
}

pub fn audio_pause(q: &AudioQueueResource, name: &str) {
    let _ = q.0.send(AudioCommand::Pause { name: name.to_string() });
}

pub fn audio_resume(q: &AudioQueueResource, name: &str) {
    let _ = q.0.send(AudioCommand::Resume { name: name.to_string() });
}

pub fn audio_set_volume(q: &AudioQueueResource, name: &str, volume: f32) {
    let _ = q.0.send(AudioCommand::SetVolume { name: name.to_string(), volume });
}

pub fn audio_cleanup(q: &AudioQueueResource) {
    let _ = q.0.send(AudioCommand::Cleanup);
}
