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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atlas_from_json_texture_packer_format() {
        let json = r#"{
            "meta": {
                "size": {"w": 512, "h": 512}
            },
            "frames": {
                "sprite1.png": {
                    "frame": {"x": 0, "y": 0, "w": 64, "h": 64}
                },
                "sprite2.png": {
                    "frame": {"x": 64, "y": 0, "w": 32, "h": 32}
                }
            }
        }"#;

        let atlas = Atlas::from_json(json);
        assert!(atlas.is_some());
        let atlas = atlas.expect("Test: operation should succeed");

        assert_eq!(atlas.size, [512, 512]);
        assert_eq!(atlas.sprites.len(), 2);

        // Check sprite1
        let sprite1 = atlas.get("sprite1.png");
        assert!(sprite1.is_some());
        let (offset, scale) = sprite1.expect("Test: operation should succeed");
        assert_eq!(offset, [0.0, 0.0]);
        assert_eq!(scale, [64.0 / 512.0, 64.0 / 512.0]);

        // Check sprite2
        let sprite2 = atlas.get("sprite2.png");
        assert!(sprite2.is_some());
    }

    #[test]
    fn test_atlas_from_json_array_format() {
        let json = r#"{
            "meta": {
                "size": {"w": 256, "h": 256}
            },
            "frames": [
                {
                    "filename": "array_sprite1.png",
                    "frame": {"x": 0, "y": 0, "w": 32, "h": 32}
                },
                {
                    "filename": "array_sprite2.png",
                    "frame": {"x": 32, "y": 0, "w": 16, "h": 16}
                }
            ]
        }"#;

        let atlas = Atlas::from_json(json);
        assert!(atlas.is_some());
        let atlas = atlas.expect("Test: operation should succeed");

        assert_eq!(atlas.size, [256, 256]);
        assert_eq!(atlas.sprites.len(), 2);

        let sprite1 = atlas.get("array_sprite1.png");
        assert!(sprite1.is_some());
    }

    #[test]
    fn test_atlas_from_json_alternate_format() {
        let json = r#"{
            "meta": {
                "size": {"w": 128, "h": 128}
            },
            "sprites": {
                "alt_sprite1": {"x": 0, "y": 0, "w": 16, "h": 16},
                "alt_sprite2": {"x": 16, "y": 0, "w": 8, "h": 8}
            }
        }"#;

        let atlas = Atlas::from_json(json);
        assert!(atlas.is_some());
        let atlas = atlas.expect("Test: operation should succeed");

        assert_eq!(atlas.size, [128, 128]);
        assert_eq!(atlas.sprites.len(), 2);

        let sprite1 = atlas.get("alt_sprite1");
        assert!(sprite1.is_some());
    }

    #[test]
    fn test_atlas_get_nonexistent() {
        let json = r#"{
            "meta": {
                "size": {"w": 64, "h": 64}
            },
            "frames": {
                "existing.png": {
                    "frame": {"x": 0, "y": 0, "w": 16, "h": 16}
                }
            }
        }"#;

        let atlas = Atlas::from_json(json).expect("Test: operation should succeed");
        let result = atlas.get("nonexistent.png");
        assert!(result.is_none());
    }

    #[test]
    fn test_atlas_invalid_json() {
        let invalid_json = "{ invalid json }";
        let atlas = Atlas::from_json(invalid_json);
        assert!(atlas.is_none());
    }

    #[test]
    fn test_atlas_empty_json() {
        let json = r#"{
            "meta": {
                "size": {"w": 0, "h": 0}
            },
            "frames": {}
        }"#;

        let atlas = Atlas::from_json(json);
        assert!(atlas.is_some());
        let atlas = atlas.expect("Test: operation should succeed");
        assert_eq!(atlas.size, [0, 0]);
        assert_eq!(atlas.sprites.len(), 0);
    }

    #[test]
    fn test_atlas_uv_coordinates() {
        let json = r#"{
            "meta": {
                "size": {"w": 100, "h": 100}
            },
            "frames": {
                "full_atlas.png": {
                    "frame": {"x": 0, "y": 0, "w": 100, "h": 100}
                },
                "half_atlas.png": {
                    "frame": {"x": 0, "y": 0, "w": 50, "h": 50}
                }
            }
        }"#;

        let atlas = Atlas::from_json(json).expect("Test: operation should succeed");

        // Full atlas should have UV coordinates [0,0] to [1,1]
        let full = atlas.get("full_atlas.png").expect("Test: operation should succeed");
        assert_eq!(full, ([0.0, 0.0], [1.0, 1.0]));

        // Half atlas should have UV coordinates [0,0] to [0.5,0.5]
        let half = atlas.get("half_atlas.png").expect("Test: operation should succeed");
        assert_eq!(half, ([0.0, 0.0], [0.5, 0.5]));
    }

    #[test]
    fn test_atlas_meta_size() {
        let json = r#"{
            "meta": {
                "size": {"w": 256, "h": 512}
            },
            "frames": {
                "test.png": {
                    "frame": {"x": 0, "y": 0, "w": 32, "h": 32}
                }
            }
        }"#;

        let atlas = Atlas::from_json(json).expect("Test: operation should succeed");
        assert_eq!(atlas.size[0], 256);
        assert_eq!(atlas.size[1], 512);
    }
}
