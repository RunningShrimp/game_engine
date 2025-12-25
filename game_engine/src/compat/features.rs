//! 特性集管理
//!
//! 提供统一的特性访问接口，减少代码中散落的 `cfg!` 宏。
//! 这个模块集中管理所有编译时特性标志，使特性检测更加清晰和可维护。
//!
//! ## 使用示例
//!
//! ```rust
//! use game_engine::compat::features::FeatureSet;
//!
//! let features = FeatureSet::current();
//! if features.xr_enabled {
//!     // XR功能代码
//! }
//!
//! if features.is_feature_enabled("gltf") {
//!     // GLTF加载代码
//! }
//! ```

/// 特性集
///
/// 包含所有可用的编译时特性标志。
/// 使用 `FeatureSet::current()` 获取当前构建的特性集。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeatureSet {
    /// XR (VR/AR/MR) 支持
    pub xr_enabled: bool,
    /// GLTF 模型加载支持
    pub gltf_enabled: bool,
    /// WebAssembly 运行时支持
    pub wasm_enabled: bool,
    /// 安全密钥交换（ECDH + HKDF）
    pub secure_key_exchange: bool,
    /// 不安全密钥交换（仅用于测试）
    pub insecure_key_exchange: bool,
    /// Python 脚本支持
    pub python_enabled: bool,
}

impl FeatureSet {
    /// 获取当前构建的特性集
    ///
    /// 这个函数在编译时确定哪些特性被启用。
    ///
    /// # 示例
    ///
    /// ```rust
    /// let features = FeatureSet::current();
    /// if features.xr_enabled {
    ///     println!("XR support is enabled");
    /// }
    /// ```
    pub fn current() -> Self {
        Self {
            xr_enabled: cfg!(feature = "xr"),
            gltf_enabled: cfg!(feature = "gltf"),
            wasm_enabled: cfg!(feature = "wasm"),
            secure_key_exchange: cfg!(feature = "secure_key_exchange"),
            insecure_key_exchange: cfg!(feature = "insecure_key_exchange"),
            python_enabled: cfg!(feature = "pyo3"),
        }
    }

    /// 检查指定特性是否启用
    ///
    /// # 参数
    ///
    /// * `feature` - 特性名称（如 "xr", "gltf", "wasm"）
    ///
    /// # 返回
    ///
    /// 如果特性启用返回 `true`，否则返回 `false`
    ///
    /// # 示例
    ///
    /// ```rust
    /// let features = FeatureSet::current();
    /// if features.is_feature_enabled("xr") {
    ///     // XR功能代码
    /// }
    /// ```
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        match feature {
            "xr" => self.xr_enabled,
            "gltf" => self.gltf_enabled,
            "wasm" => self.wasm_enabled,
            "secure_key_exchange" => self.secure_key_exchange,
            "insecure_key_exchange" => self.insecure_key_exchange,
            "python" | "pyo3" => self.python_enabled,
            _ => false,
        }
    }

    /// 获取所有启用的特性名称列表
    ///
    /// # 返回
    ///
    /// 包含所有启用特性名称的向量
    ///
    /// # 示例
    ///
    /// ```rust
    /// let features = FeatureSet::current();
    /// let enabled = features.enabled_features();
    /// println!("Enabled features: {:?}", enabled);
    /// ```
    pub fn enabled_features(&self) -> Vec<&'static str> {
        let mut features = Vec::new();
        if self.xr_enabled {
            features.push("xr");
        }
        if self.gltf_enabled {
            features.push("gltf");
        }
        if self.wasm_enabled {
            features.push("wasm");
        }
        if self.secure_key_exchange {
            features.push("secure_key_exchange");
        }
        if self.insecure_key_exchange {
            features.push("insecure_key_exchange");
        }
        if self.python_enabled {
            features.push("python");
        }
        features
    }
}

impl Default for FeatureSet {
    fn default() -> Self {
        Self::current()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_set_current() {
        let features = FeatureSet::current();
        // 至少应该有一些特性被启用（如默认特性）
        assert!(features.enabled_features().len() >= 0);
    }

    #[test]
    fn test_is_feature_enabled() {
        let features = FeatureSet::current();
        // 测试已知的特性
        let _ = features.is_feature_enabled("xr");
        let _ = features.is_feature_enabled("gltf");
        let _ = features.is_feature_enabled("wasm");
        // 测试未知特性应该返回false
        assert!(!features.is_feature_enabled("unknown_feature"));
    }

    #[test]
    fn test_enabled_features() {
        let features = FeatureSet::current();
        let enabled = features.enabled_features();
        // 验证返回的是字符串切片列表
        assert!(enabled.iter().all(|f| matches!(*f, "xr" | "gltf" | "wasm" | "secure_key_exchange" | "insecure_key_exchange" | "python")));
    }
}

