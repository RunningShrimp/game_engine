use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct AtlasMetaSize {
    pub w: u32,
    pub h: u32,
}

#[derive(Debug, Deserialize)]
pub struct AtlasMeta {
    pub size: AtlasMetaSize,
}

#[derive(Debug, Deserialize)]
pub struct FrameRect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

#[derive(Debug, Deserialize)]
pub struct FrameEntry {
    pub frame: FrameRect,
}

#[derive(Debug, Deserialize)]
pub struct ArrayFrameEntry {
    pub filename: String,
    pub frame: FrameRect,
}

#[derive(Clone, Debug)]
pub struct Atlas {
    pub size: [u32; 2],
    pub sprites: std::collections::HashMap<String, ([f32; 2], [f32; 2])>,
}

impl Atlas {
    pub fn from_json(data: &str) -> Option<Self> {
        // Try known TexturePacker-like format via dynamic matching
        // Fallback: dynamic parse supporting multiple variants
        if let Ok(v) = serde_json::from_str::<Value>(data) {
            let mut sprites = std::collections::HashMap::new();
            // meta size
            let (tw, th) = {
                if let Some(ms) = v.get("meta").and_then(|m| m.get("size")) {
                    let w = ms.get("w").and_then(|x| x.as_u64()).unwrap_or(0) as u32;
                    let h = ms.get("h").and_then(|x| x.as_u64()).unwrap_or(0) as u32;
                    (w, h)
                } else {
                    let w = v
                        .get("meta")
                        .and_then(|m| m.get("textureWidth"))
                        .and_then(|x| x.as_u64())
                        .unwrap_or(0) as u32;
                    let h = v
                        .get("meta")
                        .and_then(|m| m.get("textureHeight"))
                        .and_then(|x| x.as_u64())
                        .unwrap_or(0) as u32;
                    (w, h)
                }
            };
            let wf = if tw == 0 { 1.0 } else { tw as f32 };
            let hf = if th == 0 { 1.0 } else { th as f32 };
            // frames can be map or array
            if let Some(fr_map) = v.get("frames").and_then(|f| f.as_object()) {
                for (name, entry) in fr_map {
                    if let Some(fr) = entry.get("frame") {
                        let x = fr.get("x").and_then(|x| x.as_u64()).unwrap_or(0) as u32;
                        let y = fr.get("y").and_then(|x| x.as_u64()).unwrap_or(0) as u32;
                        let w = fr.get("w").and_then(|x| x.as_u64()).unwrap_or(0) as u32;
                        let h = fr.get("h").and_then(|x| x.as_u64()).unwrap_or(0) as u32;
                        let uv_off = [x as f32 / wf, y as f32 / hf];
                        let uv_scale = [w as f32 / wf, h as f32 / hf];
                        sprites.insert(name.clone(), (uv_off, uv_scale));
                    }
                }
            } else if let Some(fr_arr) = v.get("frames").and_then(|f| f.as_array()) {
                for e in fr_arr {
                    let name = e.get("filename").and_then(|x| x.as_str()).unwrap_or("");
                    if let Some(fr) = e.get("frame") {
                        let x = fr.get("x").and_then(|x| x.as_u64()).unwrap_or(0) as u32;
                        let y = fr.get("y").and_then(|x| x.as_u64()).unwrap_or(0) as u32;
                        let w = fr.get("w").and_then(|x| x.as_u64()).unwrap_or(0) as u32;
                        let h = fr.get("h").and_then(|x| x.as_u64()).unwrap_or(0) as u32;
                        let uv_off = [x as f32 / wf, y as f32 / hf];
                        let uv_scale = [w as f32 / wf, h as f32 / hf];
                        sprites.insert(name.to_string(), (uv_off, uv_scale));
                    }
                }
            } else if let Some(sprites_map) = v.get("sprites").and_then(|f| f.as_object()) {
                // Alternate format: { sprites: { name: {x,y,w,h} } }
                for (name, fr) in sprites_map {
                    let x = fr.get("x").and_then(|x| x.as_u64()).unwrap_or(0) as u32;
                    let y = fr.get("y").and_then(|x| x.as_u64()).unwrap_or(0) as u32;
                    let w = fr.get("w").and_then(|x| x.as_u64()).unwrap_or(0) as u32;
                    let h = fr.get("h").and_then(|x| x.as_u64()).unwrap_or(0) as u32;
                    let uv_off = [x as f32 / wf, y as f32 / hf];
                    let uv_scale = [w as f32 / wf, h as f32 / hf];
                    sprites.insert(name.clone(), (uv_off, uv_scale));
                }
            }
            return Some(Self {
                size: [tw, th],
                sprites,
            });
        }
        None
    }
    pub fn get(&self, name: &str) -> Option<([f32; 2], [f32; 2])> {
        self.sprites.get(name).copied()
    }
}
