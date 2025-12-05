/// 硬件检测结果缓存
/// 
/// 将硬件检测结果持久化到本地文件，避免每次启动都重新检测

use crate::{GpuInfo, NpuInfo, SocInfo};
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

/// 硬件信息缓存
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareCache {
    pub gpu: GpuInfo,
    pub npu: Option<NpuInfo>,
    pub soc: Option<SocInfo>,
    pub timestamp: u64,
    pub driver_version: String,
}

impl HardwareCache {
    /// 获取缓存文件路径
    fn cache_path() -> PathBuf {
        // 优先使用系统缓存目录
        let base_dir = dirs::cache_dir()
            .unwrap_or_else(|| {
                // 回退到当前目录
                std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
            });
        
        base_dir
            .join("game_engine")
            .join("hardware_cache.json")
    }
    
    /// 从文件加载缓存
    pub fn load() -> Option<Self> {
        let path = Self::cache_path();
        
        if !path.exists() {
            return None;
        }
        
        match fs::read_to_string(&path) {
            Ok(data) => {
                match serde_json::from_str::<HardwareCache>(&data) {
                    Ok(cache) => {
                        // 检查缓存是否过期（24小时）
                        let now = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs();
                        
                        if now - cache.timestamp > 86400 {
                            return None;
                        }
                        
                        Some(cache)
                    }
                    Err(_) => None
                }
            }
            Err(_) => None
        }
    }
    
    /// 保存缓存到文件
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::cache_path();
        
        // 确保目录存在
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // 序列化并写入文件
        let data = serde_json::to_string_pretty(self)?;
        fs::write(&path, data)?;
        
        Ok(())
    }
    
    /// 创建新的缓存
    pub fn new(gpu: GpuInfo, npu: Option<NpuInfo>, soc: Option<SocInfo>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let driver_version = gpu.driver_version.clone();
        
        Self {
            gpu,
            npu,
            soc,
            timestamp,
            driver_version,
        }
    }
    
    /// 检查缓存是否有效
    pub fn is_valid(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // 24小时内有效
        now - self.timestamp <= 86400
    }
}

/// 带缓存的硬件检测
pub fn detect_hardware_cached() -> (GpuInfo, Option<NpuInfo>, Option<SocInfo>) {
    use crate::{detect_gpu, detect_npu, detect_soc};
    
    // 尝试从缓存加载
    if let Some(cache) = HardwareCache::load() {
        if cache.is_valid() {
            return (cache.gpu, cache.npu, cache.soc);
        }
    }
    
    // 缓存未命中或已过期，执行完整检测
    let gpu = detect_gpu();
    let npu = detect_npu();
    let soc = detect_soc();
    
    // 保存到缓存
    let cache = HardwareCache::new(gpu.clone(), npu.clone(), soc.clone());
    let _ = cache.save();
    
    (gpu, npu, soc)
}
