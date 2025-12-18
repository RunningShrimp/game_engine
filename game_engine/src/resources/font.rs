use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Common {
    #[serde(rename = "scaleW")]
    pub scale_w: u32,
    #[serde(rename = "scaleH")]
    pub scale_h: u32,
}

#[derive(Debug, Deserialize)]
pub struct Info {
    #[serde(default, rename = "distanceRange")]
    pub distance_range: Option<f32>,
    #[serde(default)]
    pub ascent: Option<f32>,
    #[serde(default)]
    pub descent: Option<f32>,
    #[serde(default)]
    pub baseline: Option<f32>,
}

#[derive(Debug, Deserialize)]
pub struct CharEntry {
    pub id: u32,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub xoffset: f32,
    pub yoffset: f32,
    pub xadvance: f32,
    #[serde(default)]
    pub page: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct MsdfDoc {
    pub common: Common,
    pub info: Option<Info>,
    pub chars: Vec<CharEntry>,
}

pub struct MsdfGlyph {
    pub uv_offset: [f32; 2],
    pub uv_scale: [f32; 2],
    pub size: [f32; 2],
    pub bearing: [f32; 2],
    pub advance: f32,
    pub px_range: f32,
}

pub struct MsdfFontAtlas {
    pub size: [u32; 2],
    pub px_range: f32,
    pub ascent: f32,
    pub descent: f32,
    pub baseline: f32,
    pub glyphs: std::collections::HashMap<u32, MsdfGlyph>,
}

impl MsdfFontAtlas {
    pub fn from_json(data: &str) -> Option<Self> {
        if let Ok(doc) = serde_json::from_str::<MsdfDoc>(data) {
            let w = doc.common.scale_w as f32;
            let h = doc.common.scale_h as f32;
            let mut glyphs = std::collections::HashMap::new();
            let pr = doc
                .info
                .as_ref()
                .and_then(|i| i.distance_range)
                .unwrap_or(4.0);
            let asc = doc.info.as_ref().and_then(|i| i.ascent).unwrap_or(0.8 * h);
            let des = doc.info.as_ref().and_then(|i| i.descent).unwrap_or(0.2 * h);
            let base = doc
                .info
                .as_ref()
                .and_then(|i| i.baseline)
                .unwrap_or(0.85 * h);
            for ch in doc.chars {
                let uv_off = [ch.x as f32 / w, ch.y as f32 / h];
                let uv_scale = [ch.width as f32 / w, ch.height as f32 / h];
                glyphs.insert(
                    ch.id,
                    MsdfGlyph {
                        uv_offset: uv_off,
                        uv_scale,
                        size: [ch.width as f32, ch.height as f32],
                        bearing: [ch.xoffset, ch.yoffset],
                        advance: ch.xadvance,
                        px_range: pr,
                    },
                );
            }
            return Some(Self {
                size: [doc.common.scale_w, doc.common.scale_h],
                px_range: pr,
                ascent: asc,
                descent: des,
                baseline: base,
                glyphs,
            });
        }
        None
    }

    pub fn glyph(
        &self,
        codepoint: u32,
    ) -> Option<([f32; 2], [f32; 2], [f32; 2], [f32; 2], f32, f32)> {
        let g = self.glyphs.get(&codepoint)?;
        Some((
            g.uv_offset,
            g.uv_scale,
            g.size,
            g.bearing,
            g.advance,
            g.px_range,
        ))
    }
}
