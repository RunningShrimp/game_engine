//! # 资源导入工具（Asset Importer）
//!
//! 提供基于egui的图形化资源导入向导，支持拖拽导入和格式自动检测。
//!
//! ## 功能特性
//!
//! - **格式自动检测** - 自动识别GLTF、FBX、OBJ、纹理、音频等格式
//! - **资源验证** - 检测资源文件的完整性和常见问题
//! - **自动修复** - 自动修复常见的资源问题
//! - **批量导入** - 支持批量导入多个资源文件
//! - **导入向导** - 友好的图形化导入界面
//! - **预览功能** - 导入前预览资源内容
//!
//! ## 使用示例
//!
//! ```rust,no_run
//! use game_engine::tools::asset_importer::{AssetImportWizard, ImportSettings};
//!
//! // 创建导入向导
//! let wizard = AssetImportWizard::new();
//!
//! // 在渲染循环中显示
//! wizard.show(&egui_ctx);
//! ```
//!
//! ## 模块结构
//!
//! - [`wizard`]: 导入向导UI
//! - [`detector`]: 格式检测器
//! - [`importer`]: 资源导入器
//! - [`validator`]: 资源验证器
//! - [`fixer`]: 错误修复工具
//! - [`batch`]: 批量导入

pub mod batch;
pub mod detector;
pub mod fixer;
pub mod importer;
pub mod validator;
pub mod wizard;

#[cfg(test)]
mod tests;

// 重新导出主要类型
pub use batch::{BatchImportSettings, BatchImporter, BatchProgress, BatchReport};
pub use detector::{AssetDetector, AssetFormat, DetectorError, FileAnalysis};
pub use fixer::{AssetFixer, FixerError};
pub use importer::{AssetImporter, ImportOptions, ImportResult};
pub use validator::{AssetValidator, FixSuggestion, ValidationIssue, ValidationResult};
pub use wizard::{AssetImportWizard, ImportSettings, WizardResult, WizardStep};

/// 资源导入错误类型
#[derive(thiserror::Error, Debug)]
pub enum AssetImporterError {
    #[error("Detection error: {0}")]
    DetectionError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Import error: {0}")]
    ImportError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Unknown format")]
    UnknownFormat,
}

/// 资源导入结果类型
pub type Result<T> = std::result::Result<T, AssetImporterError>;

/// 压缩格式
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompressionFormat {
    None,
    BC1,
    BC2,
    BC3,
    BC4,
    BC5,
}

/// 预览数据
#[derive(Clone, Debug)]
pub enum PreviewData {
    Texture {
        width: u32,
        height: u32,
        format: String,
        size: usize,
    },
    Model {
        vertices: usize,
        triangles: usize,
        materials: usize,
        animations: usize,
    },
    Audio {
        duration: f32,
        channels: u16,
        sample_rate: u32,
        format: String,
    },
    Unknown {
        size: usize,
        format: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_format() {
        let format = CompressionFormat::BC1;
        assert_eq!(format, CompressionFormat::BC1);
    }

    #[test]
    fn test_preview_data() {
        let preview = PreviewData::Texture {
            width: 512,
            height: 512,
            format: "RGBA8".to_string(),
            size: 512 * 512 * 4,
        };

        match preview {
            PreviewData::Texture { width, height, .. } => {
                assert_eq!(width, 512);
                assert_eq!(height, 512);
            }
            _ => panic!("Expected texture preview"),
        }
    }
}
