//! 路径验证器

use super::error::{ValidationError, ValidationResult};
use std::path::Path;

/// 验证文件扩展名
///
/// # 参数
/// - `path`: 要验证的路径
/// - `allowed`: 允许的扩展名列表（不含点，如["png", "jpg", "gltf"]）
///
/// # 示例
///
/// ```
/// use game_engine::core::validation::validators::validate_extension;
/// use std::path::Path;
///
/// assert!(validate_extension(Path::new("model.gltf"), &["gltf", "glb"]).is_ok());
/// assert!(validate_extension(Path::new("texture.png"), &["gltf", "glb"]).is_err());
/// assert!(validate_extension(Path::new("no_ext"), &["txt"]).is_err());
/// ```
pub fn validate_extension<'p>(path: &'p Path, allowed: &[&str]) -> ValidationResult<&'p Path> {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .ok_or_else(|| ValidationError::MissingExtension(path.to_path_buf()))?;

    let ext_lower = ext.to_lowercase();

    if !allowed.iter().any(|&a| a.eq_ignore_ascii_case(ext)) {
        return Err(ValidationError::InvalidExtension {
            path: path.to_path_buf(),
            found: ext.to_string(),
            allowed: allowed.iter().map(|s| s.to_string()).collect(),
        });
    }

    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_extension() {
        assert!(validate_extension(Path::new("model.gltf"), &["gltf", "glb"]).is_ok());
        assert!(validate_extension(Path::new("model.glb"), &["gltf", "glb"]).is_ok());
        assert!(validate_extension(Path::new("MODEL.GLTf"), &["gltf"]).is_ok()); // 大小写不敏感

        assert!(validate_extension(Path::new("texture.png"), &["gltf", "glb"]).is_err());
        assert!(validate_extension(Path::new("no_ext"), &["txt"]).is_err());

        // 错误信息包含允许的扩展名
        match validate_extension(Path::new("test.png"), &["gltf", "glb"]) {
            Err(ValidationError::InvalidExtension { allowed, .. }) => {
                assert_eq!(allowed, vec!["gltf", "glb"]);
            }
            _ => panic!("Unexpected error type"),
        }
    }
}
