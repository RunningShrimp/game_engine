use crate::error::safe_lock;
use std::sync::Mutex;

pub enum AssetEvent {
    FontJsonReady { name: String, data: String },
    TextureReady { name: String },
    AtlasReady { name: String },
}

static QUEUE: Mutex<Vec<AssetEvent>> = Mutex::new(Vec::new());

pub fn push_font_json_ready(name: String, data: String) {
    if let Ok(mut q) = safe_lock(&QUEUE, "AssetEvent.QUEUE") {
        q.push(AssetEvent::FontJsonReady { name, data });
    }
}

pub fn push_texture_ready(name: String) {
    if let Ok(mut q) = safe_lock(&QUEUE, "AssetEvent.QUEUE") {
        q.push(AssetEvent::TextureReady { name });
    }
}

pub fn push_atlas_ready(name: String) {
    if let Ok(mut q) = safe_lock(&QUEUE, "AssetEvent.QUEUE") {
        q.push(AssetEvent::AtlasReady { name });
    }
}

pub fn drain_events() -> Vec<AssetEvent> {
    if let Ok(mut q) = safe_lock(&QUEUE, "AssetEvent.QUEUE") {
        let mut out = Vec::new();
        std::mem::swap(&mut *q, &mut out);
        out
    } else {
        Vec::new()
    }
}
