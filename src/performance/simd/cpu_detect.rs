/// CPU特性检测模块
/// 
/// 运行时检测CPU支持的SIMD指令集和特性

use std::sync::OnceLock;

/// CPU特性标志
#[derive(Debug, Clone)]
pub struct CpuFeatures {
    // x86/x64特性
    pub sse2: bool,
    pub sse3: bool,
    pub ssse3: bool,
    pub sse41: bool,
    pub sse42: bool,
    pub avx: bool,
    pub avx2: bool,
    pub avx512f: bool,
    pub avx512dq: bool,
    pub avx512vl: bool,
    pub fma: bool,
    
    // ARM特性
    pub neon: bool,
    pub sve: bool,
    pub sve2: bool,
    
    // 厂商信息
    pub vendor: CpuVendor,
    pub brand: String,
}

/// CPU厂商
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuVendor {
    Intel,
    Amd,
    AppleSilicon,
    Qualcomm,
    MediaTek,
    HiSilicon,
    Other,
}

impl CpuFeatures {
    /// 检测当前CPU特性
    fn detect() -> Self {
        #[cfg(target_arch = "x86_64")]
        {
            Self::detect_x86_64()
        }
        
        #[cfg(target_arch = "aarch64")]
        {
            Self::detect_aarch64()
        }
        
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            Self::default()
        }
    }
    
    #[cfg(target_arch = "x86_64")]
    fn detect_x86_64() -> Self {
        // 使用std::arch检测CPU特性
        let sse2 = is_x86_feature_detected!("sse2");
        let sse3 = is_x86_feature_detected!("sse3");
        let ssse3 = is_x86_feature_detected!("ssse3");
        let sse41 = is_x86_feature_detected!("sse4.1");
        let sse42 = is_x86_feature_detected!("sse4.2");
        let avx = is_x86_feature_detected!("avx");
        let avx2 = is_x86_feature_detected!("avx2");
        let fma = is_x86_feature_detected!("fma");
        
        // AVX-512检测
        let avx512f = is_x86_feature_detected!("avx512f");
        let avx512dq = avx512f && is_x86_feature_detected!("avx512dq");
        let avx512vl = avx512f && is_x86_feature_detected!("avx512vl");
        
        // 检测厂商
        let vendor = Self::detect_x86_vendor();
        let brand = Self::get_cpu_brand();
        
        Self {
            sse2,
            sse3,
            ssse3,
            sse41,
            sse42,
            avx,
            avx2,
            avx512f,
            avx512dq,
            avx512vl,
            fma,
            neon: false,
            sve: false,
            sve2: false,
            vendor,
            brand,
        }
    }
    
    #[cfg(target_arch = "aarch64")]
    fn detect_aarch64() -> Self {
        // ARM NEON在aarch64上是强制支持的
        let neon = true;
        
        // SVE检测（需要运行时检查）
        let sve = std::arch::is_aarch64_feature_detected!("sve");
        let sve2 = sve && std::arch::is_aarch64_feature_detected!("sve2");
        
        // 检测厂商
        let vendor = Self::detect_arm_vendor();
        let brand = Self::get_cpu_brand();
        
        Self {
            sse2: false,
            sse3: false,
            ssse3: false,
            sse41: false,
            sse42: false,
            avx: false,
            avx2: false,
            avx512f: false,
            avx512dq: false,
            avx512vl: false,
            fma: false,
            neon,
            sve,
            sve2,
            vendor,
            brand,
        }
    }
    
    #[cfg(target_arch = "x86_64")]
    fn detect_x86_vendor() -> CpuVendor {
        // 使用cpuid检测厂商
        #[cfg(target_feature = "sse2")]
        unsafe {
            use std::arch::x86_64::__cpuid;
            let result = __cpuid(0);
            let vendor_string = format!(
                "{}{}{}",
                std::str::from_utf8_unchecked(&result.ebx.to_le_bytes()),
                std::str::from_utf8_unchecked(&result.edx.to_le_bytes()),
                std::str::from_utf8_unchecked(&result.ecx.to_le_bytes())
            );
            
            match vendor_string.as_str() {
                "GenuineIntel" => CpuVendor::Intel,
                "AuthenticAMD" => CpuVendor::Amd,
                _ => CpuVendor::Other,
            }
        }
        
        #[cfg(not(target_feature = "sse2"))]
        CpuVendor::Other
    }
    
    #[cfg(target_arch = "aarch64")]
    fn detect_arm_vendor() -> CpuVendor {
        // 通过品牌字符串推断厂商
        let brand = Self::get_cpu_brand().to_lowercase();
        
        if brand.contains("apple") || brand.contains("m1") || brand.contains("m2") || brand.contains("m3") {
            CpuVendor::AppleSilicon
        } else if brand.contains("qualcomm") || brand.contains("snapdragon") {
            CpuVendor::Qualcomm
        } else if brand.contains("mediatek") || brand.contains("dimensity") {
            CpuVendor::MediaTek
        } else if brand.contains("hisilicon") || brand.contains("kirin") {
            CpuVendor::HiSilicon
        } else {
            CpuVendor::Other
        }
    }
    
    fn get_cpu_brand() -> String {
        // 尝试从/proc/cpuinfo读取（Linux）
        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
                for line in content.lines() {
                    if line.starts_with("model name") || line.starts_with("Hardware") {
                        if let Some(name) = line.split(':').nth(1) {
                            return name.trim().to_string();
                        }
                    }
                }
            }
        }
        
        // macOS可以使用sysctl
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            if let Ok(output) = Command::new("sysctl")
                .arg("-n")
                .arg("machdep.cpu.brand_string")
                .output()
            {
                if let Ok(brand) = String::from_utf8(output.stdout) {
                    return brand.trim().to_string();
                }
            }
        }
        
        "Unknown".to_string()
    }
}

impl Default for CpuFeatures {
    fn default() -> Self {
        Self {
            sse2: false,
            sse3: false,
            ssse3: false,
            sse41: false,
            sse42: false,
            avx: false,
            avx2: false,
            avx512f: false,
            avx512dq: false,
            avx512vl: false,
            fma: false,
            neon: false,
            sve: false,
            sve2: false,
            vendor: CpuVendor::Other,
            brand: "Unknown".to_string(),
        }
    }
}

/// 全局CPU特性缓存
static CPU_FEATURES: OnceLock<CpuFeatures> = OnceLock::new();

/// 检测CPU特性（缓存结果）
pub fn detect_cpu_features() -> &'static CpuFeatures {
    CPU_FEATURES.get_or_init(CpuFeatures::detect)
}

/// 打印CPU信息
pub fn print_cpu_info() {
    let features = detect_cpu_features();
    println!("=== CPU Information ===");
    println!("Vendor: {:?}", features.vendor);
    println!("Brand: {}", features.brand);
    println!();
    
    #[cfg(target_arch = "x86_64")]
    {
        println!("x86/x64 Features:");
        println!("  SSE2: {}", features.sse2);
        println!("  SSE3: {}", features.sse3);
        println!("  SSSE3: {}", features.ssse3);
        println!("  SSE4.1: {}", features.sse41);
        println!("  SSE4.2: {}", features.sse42);
        println!("  AVX: {}", features.avx);
        println!("  AVX2: {}", features.avx2);
        println!("  AVX-512F: {}", features.avx512f);
        println!("  AVX-512DQ: {}", features.avx512dq);
        println!("  AVX-512VL: {}", features.avx512vl);
        println!("  FMA: {}", features.fma);
    }
    
    #[cfg(target_arch = "aarch64")]
    {
        println!("ARM Features:");
        println!("  NEON: {}", features.neon);
        println!("  SVE: {}", features.sve);
        println!("  SVE2: {}", features.sve2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_detection() {
        let features = detect_cpu_features();
        println!("Detected CPU features: {:#?}", features);
        
        // 验证基本特性
        #[cfg(target_arch = "x86_64")]
        assert!(features.sse2, "SSE2 should be available on all x86_64 CPUs");
        
        #[cfg(target_arch = "aarch64")]
        assert!(features.neon, "NEON should be available on all aarch64 CPUs");
    }

    #[test]
    fn test_print_info() {
        print_cpu_info();
    }
}
