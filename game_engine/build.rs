/// 构建脚本
///
/// 用于验证feature组合和配置编译时选项。
fn main() {
    // 输出配置信息
    println!("cargo:rerun-if-changed=build.rs");

    // 输出feature配置信息
    #[cfg(feature = "secure_key_exchange")]
    {
        println!("cargo:warning=secure_key_exchange已启用 - 使用生产级密钥交换");
    }

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
