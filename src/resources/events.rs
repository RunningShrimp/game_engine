use std::sync::Mutex;

pub enum AssetEvent { FontJsonReady { name: String, data: String }, TextureReady { name: String }, AtlasReady { name: String } }

static QUEUE: Mutex<Vec<AssetEvent>> = Mutex::new(Vec::new());

pub fn push_font_json_ready(name: String, data: String) {
    let mut q = QUEUE.lock().unwrap();
    q.push(AssetEvent::FontJsonReady { name, data });
}

pub fn push_texture_ready(name: String) { let mut q = QUEUE.lock().unwrap(); q.push(AssetEvent::TextureReady { name }); }
pub fn push_atlas_ready(name: String) { let mut q = QUEUE.lock().unwrap(); q.push(AssetEvent::AtlasReady { name }); }

pub fn drain_events() -> Vec<AssetEvent> {
    let mut q = QUEUE.lock().unwrap();
    let mut out = Vec::new();
    std::mem::swap(&mut *q, &mut out);
    out
}
