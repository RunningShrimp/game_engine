/// 构建脚本
///
/// 用于验证feature组合和配置编译时选项。
fn main() {
    // 检查互斥的feature组合
    check_exclusive_features();

    // 输出配置信息
    println!("cargo:rerun-if-changed=build.rs");
}

/// 检查互斥的feature组合
///
/// 如果检测到冲突的feature组合，编译时会报错。
fn check_exclusive_features() {
    // 检查密钥交换feature冲突
    let has_secure = cfg!(feature = "secure_key_exchange");
    let has_insecure = cfg!(feature = "insecure_key_exchange");

    if has_secure && has_insecure {
        panic!(
            r#"
            ═══════════════════════════════════════════════════════════════
            错误: 不能同时启用 secure_key_exchange 和 insecure_key_exchange

            这两个特性提供了冲突的密钥交换实现，只能选择其中一个。

            ❌ 错误的用法:
                cargo build --features secure_key_exchange,insecure_key_exchange

            ✅ 正确的用法 (推荐):
                cargo build --features secure_key_exchange

            ✅ 正确的用法 (仅测试):
                cargo build --features insecure_key_exchange

            原因:
            - secure_key_exchange 使用X25519 ECDH + HKDF，提供生产级安全性
            - insecure_key_exchange 使用简化的SHA256实现，仅用于测试

            更多信息请参考: Cargo.toml中的[features]部分
            ═══════════════════════════════════════════════════════════════
            "#
        );
    }

    // 输出feature配置信息
    if has_secure {
        println!("cargo:warning=secure_key_exchange已启用 - 使用生产级密钥交换");
    }
    if has_insecure {
        println!("cargo:warning=⚠️  insecure_key_exchange已启用 - 仅用于测试，不可用于生产环境！");
    }

    // 检查Python绑定 (temporarily disabled)
    // #[cfg(feature = "pyo3")]
    // {
    //     println!("cargo:warning=pyo3已启用 - 需要Python开发环境");
    // }

    // 检查XR支持
    #[cfg(feature = "xr")]
    {
        println!("cargo:warning=XR已启用 - 需要OpenXR SDK");
    }

    // 检查Tracy Profiler
    #[cfg(feature = "tracy")]
    {
        println!("cargo:info=Tracy Profiler已启用 - 性能分析功能可用");
    }
}
