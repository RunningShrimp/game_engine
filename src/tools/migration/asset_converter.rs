//! Asset converter
//!
//! Converts Unity assets to engine-compatible formats.

use std::path::{Path, PathBuf};
use crate::error::{Error, Result};

/// Asset converter configuration
#[derive(Debug, Clone)]
pub struct ConverterConfig {
    /// Output format for textures
    pub texture_format: TextureFormat,
    /// Output format for models
    pub model_format: ModelFormat,
    /// Output format for audio
    pub audio_format: AudioFormat,
    /// Whether to preserve metadata
    pub preserve_metadata: bool,
    /// Output directory
    pub output_dir: PathBuf,
}

/// Texture format
#[derive(Debug, Clone, Copy)]
pub enum TextureFormat {
    PNG,
    JPEG,
    TGA,
    DDS,
    KTX,
    ASTC,
    ETC2,
}

/// Model format
#[derive(Debug, Clone, Copy)]
pub enum ModelFormat {
    GLTF,
    GLB,
    OBJ,
}

/// Audio format
#[derive(Debug, Clone, Copy)]
pub enum AudioFormat {
    WAV,
    OGG,
    MP3,
    FLAC,
}

/// Conversion result
#[derive(Debug, Clone)]
pub struct ConversionResult {
    pub source_path: PathBuf,
    pub output_path: PathBuf,
    pub success: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

/// Asset converter
pub struct AssetConverter {
    config: ConverterConfig,
}

impl AssetConverter {
    /// Create a new asset converter
    pub fn new(config: ConverterConfig) -> Self {
        Self { config }
    }

    /// Convert a texture
    pub fn convert_texture(&self, source_path: &Path) -> Result<ConversionResult> {
        if !source_path.exists() {
            return Err(Error::IoError(format!(
                "Texture file not found: {}",
                source_path.display()
            )));
        }

        let output_path = self.get_output_path(source_path, self.get_texture_extension());

        println!(
            "Converting texture: {} -> {}",
            source_path.display(),
            output_path.display()
        );

        // In a real implementation, this would:
        // 1. Load the source texture
        // 2. Convert to target format
        // 3. Generate mipmaps if needed
        // 4. Apply compression if needed
        // 5. Write to output path

        Ok(ConversionResult {
            source_path: source_path.to_path_buf(),
            output_path,
            success: true,
            warnings: vec![],
            errors: vec![],
        })
    }

    /// Convert a 3D model
    pub fn convert_model(&self, source_path: &Path) -> Result<ConversionResult> {
        if !source_path.exists() {
            return Err(Error::IoError(format!(
                "Model file not found: {}",
                source_path.display()
            )));
        }

        let output_path = self.get_output_path(source_path, self.get_model_extension());

        println!(
            "Converting model: {} -> {}",
            source_path.display(),
            output_path.display()
        );

        // In a real implementation, this would:
        // 1. Load FBX/OBJ model
        // 2. Extract geometry, materials, animations
        // 3. Convert to glTF format
        // 4. Embed or reference textures
        // 5. Write to output path

        Ok(ConversionResult {
            source_path: source_path.to_path_buf(),
            output_path,
            success: true,
            warnings: vec![],
            errors: vec![],
        })
    }

    /// Convert an audio file
    pub fn convert_audio(&self, source_path: &Path) -> Result<ConversionResult> {
        if !source_path.exists() {
            return Err(Error::IoError(format!(
                "Audio file not found: {}",
                source_path.display()
            )));
        }

        let output_path = self.get_output_path(source_path, self.get_audio_extension());

        println!(
            "Converting audio: {} -> {}",
            source_path.display(),
            output_path.display()
        );

        // In a real implementation, this would:
        // 1. Load source audio
        // 2. Convert to target format
        // 3. Adjust quality settings
        // 4. Write to output path

        Ok(ConversionResult {
            source_path: source_path.to_path_buf(),
            output_path,
            success: true,
            warnings: vec![],
            errors: vec![],
        })
    }

    /// Convert material
    pub fn convert_material(&self, source_path: &Path) -> Result<ConversionResult> {
        println!("Converting material: {}", source_path.display());

        // Unity materials use a specific format
        // This would parse the .mat file and create corresponding engine materials

        let output_path = self.get_output_path(source_path, ".toml");

        Ok(ConversionResult {
            source_path: source_path.to_path_buf(),
            output_path,
            success: true,
            warnings: vec![],
            errors: vec![],
        })
    }

    /// Convert animation
    pub fn convert_animation(&self, source_path: &Path) -> Result<ConversionResult> {
        println!("Converting animation: {}", source_path.display());

        let output_path = self.get_output_path(source_path, ".gltf");

        Ok(ConversionResult {
            source_path: source_path.to_path_buf(),
            output_path,
            success: true,
            warnings: vec![],
            errors: vec![],
        })
    }

    /// Batch convert assets
    pub fn convert_assets(&self, source_paths: &[PathBuf]) -> Vec<ConversionResult> {
        let mut results = Vec::new();

        for path in source_paths {
            let result = if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                match ext.to_lowercase().as_str() {
                    "png" | "jpg" | "jpeg" | "tga" | "psd" => self.convert_texture(path),
                    "fbx" | "obj" => self.convert_model(path),
                    "wav" | "mp3" | "ogg" => self.convert_audio(path),
                    "mat" => self.convert_material(path),
                    "anim" => self.convert_animation(path),
                    _ => Err(Error::IoError(format!("Unsupported file type: {}", ext))),
                }
            } else {
                Err(Error::IoError("Unknown file type".to_string()))
            };

            match result {
                Ok(r) => results.push(r),
                Err(e) => {
                    results.push(ConversionResult {
                        source_path: path.clone(),
                        output_path: PathBuf::new(),
                        success: false,
                        warnings: vec![],
                        errors: vec![e.to_string()],
                    });
                }
            }
        }

        results
    }

    /// Get output path for a converted asset
    fn get_output_path(&self, source_path: &Path, new_extension: &str) -> PathBuf {
        let file_name = source_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");

        let relative_path = source_path
            .strip_prefix("Assets")
            .unwrap_or(source_path);

        let mut output_path = self.config.output_dir.join(relative_path);
        output_path.set_file_name(format!("{}{}", file_name, new_extension));

        // Create parent directory
        if let Some(parent) = output_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        output_path
    }

    /// Get texture file extension
    fn get_texture_extension(&self) -> &str {
        match self.config.texture_format {
            TextureFormat::PNG => ".png",
            TextureFormat::JPEG => ".jpg",
            TextureFormat::TGA => ".tga",
            TextureFormat::DDS => ".dds",
            TextureFormat::KTX => ".ktx",
            TextureFormat::ASTC => ".astc",
            TextureFormat::ETC2 => ".etc2",
        }
    }

    /// Get model file extension
    fn get_model_extension(&self) -> &str {
        match self.config.model_format {
            ModelFormat::GLTF => ".gltf",
            ModelFormat::GLB => ".glb",
            ModelFormat::OBJ => ".obj",
        }
    }

    /// Get audio file extension
    fn get_audio_extension(&self) -> &str {
        match self.config.audio_format {
            AudioFormat::WAV => ".wav",
            AudioFormat::OGG => ".ogg",
            AudioFormat::MP3 => ".mp3",
            AudioFormat::FLAC => ".flac",
        }
    }
}

impl Default for ConverterConfig {
    fn default() -> Self {
        Self {
            texture_format: TextureFormat::PNG,
            model_format: ModelFormat::GLTF,
            audio_format: AudioFormat::WAV,
            preserve_metadata: true,
            output_dir: PathBuf::from("converted_assets"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_converter_config() {
        let config = ConverterConfig::default();
        assert_eq!(matches!(config.texture_format, TextureFormat::PNG), true);
    }

    #[test]
    fn test_asset_converter() {
        let config = ConverterConfig::default();
        let converter = AssetConverter::new(config);

        // Test with non-existent file
        let result = converter.convert_texture(Path::new("/nonexistent.png"));
        assert!(result.is_err());
    }
}
