/// 超分辨率技术集成框架
/// 
/// 支持DLSS、FSR、XeSS等专有超分辨率技术

use super::gpu_detect::{GpuInfo, GpuVendor};
use serde::{Serialize, Deserialize};

/// 超分辨率技术
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpscalingTech {
    /// 无超分辨率
    None,
    /// NVIDIA DLSS (Deep Learning Super Sampling)
    DLSS,
    /// AMD FSR (FidelityFX Super Resolution)
    FSR,
    /// Intel XeSS (Xe Super Sampling)
    XeSS,
    /// Apple MetalFX
    MetalFX,
    /// 通用TAA超采样
    TAAUpsampling,
}

/// 超分辨率质量模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpscalingQuality {
    /// 性能模式（最低内部分辨率）
    Performance,
    /// 平衡模式
    Balanced,
    /// 质量模式
    Quality,
    /// 超质量模式（最高内部分辨率）
    UltraQuality,
}

impl UpscalingQuality {
    /// 获取内部渲染分辨率缩放比例
    pub fn render_scale(&self) -> f32 {
        match self {
            Self::Performance => 0.5,    // 50%
            Self::Balanced => 0.67,      // 67%
            Self::Quality => 0.75,       // 75%
            Self::UltraQuality => 0.85,  // 85%
        }
    }
}

/// 超分辨率管理器
pub struct UpscalingManager {
    available_techs: Vec<UpscalingTech>,
    active_tech: UpscalingTech,
    quality_mode: UpscalingQuality,
    gpu_info: GpuInfo,
}

impl UpscalingManager {
    /// 创建超分辨率管理器
    pub fn new(gpu_info: GpuInfo) -> Self {
        let available_techs = Self::detect_available_techs(&gpu_info);
        let active_tech = Self::select_best_tech(&available_techs, &gpu_info);
        let quality_mode = Self::default_quality_mode(&gpu_info);
        
        Self {
            available_techs,
            active_tech,
            quality_mode,
            gpu_info,
        }
    }
    
    /// 检测可用的超分辨率技术
    fn detect_available_techs(gpu: &GpuInfo) -> Vec<UpscalingTech> {
        let mut techs = vec![UpscalingTech::None, UpscalingTech::TAAUpsampling];
        
        match gpu.vendor {
            GpuVendor::Nvidia => {
                // RTX系列支持DLSS
                if gpu.name.to_lowercase().contains("rtx") && gpu.supports_raytracing {
                    techs.push(UpscalingTech::DLSS);
                }
                // 所有NVIDIA GPU支持FSR
                techs.push(UpscalingTech::FSR);
            }
            GpuVendor::Amd => {
                // 所有AMD GPU支持FSR
                techs.push(UpscalingTech::FSR);
            }
            GpuVendor::Intel => {
                // Arc系列支持XeSS
                if gpu.name.to_lowercase().contains("arc") {
                    techs.push(UpscalingTech::XeSS);
                }
                // 也支持FSR
                techs.push(UpscalingTech::FSR);
            }
            GpuVendor::Apple => {
                // Apple GPU支持MetalFX
                techs.push(UpscalingTech::MetalFX);
            }
            _ => {
                // 其他GPU至少支持FSR（开源）
                techs.push(UpscalingTech::FSR);
            }
        }
        
        techs
    }
    
    /// 选择最佳超分辨率技术
    fn select_best_tech(available: &[UpscalingTech], _gpu: &GpuInfo) -> UpscalingTech {
        // 优先级：DLSS > XeSS > MetalFX > FSR > TAA > None
        
        if available.contains(&UpscalingTech::DLSS) {
            return UpscalingTech::DLSS;
        }
        
        if available.contains(&UpscalingTech::XeSS) {
            return UpscalingTech::XeSS;
        }
        
        if available.contains(&UpscalingTech::MetalFX) {
            return UpscalingTech::MetalFX;
        }
        
        if available.contains(&UpscalingTech::FSR) {
            return UpscalingTech::FSR;
        }
        
        if available.contains(&UpscalingTech::TAAUpsampling) {
            return UpscalingTech::TAAUpsampling;
        }
        
        UpscalingTech::None
    }
    
    /// 默认质量模式
    fn default_quality_mode(gpu: &GpuInfo) -> UpscalingQuality {
        use super::gpu_detect::GpuTier;
        
        match gpu.tier {
            GpuTier::Flagship | GpuTier::High => UpscalingQuality::Quality,
            GpuTier::MediumHigh => UpscalingQuality::Balanced,
            _ => UpscalingQuality::Performance,
        }
    }
    
    /// 获取可用技术列表
    pub fn available_techs(&self) -> &[UpscalingTech] {
        &self.available_techs
    }
    
    /// 获取当前激活的技术
    pub fn active_tech(&self) -> UpscalingTech {
        self.active_tech
    }
    
    /// 设置激活的技术
    pub fn set_active_tech(&mut self, tech: UpscalingTech) -> Result<(), String> {
        if !self.available_techs.contains(&tech) {
            return Err(format!("{:?} 在当前硬件上不可用", tech));
        }
        self.active_tech = tech;
        Ok(())
    }
    
    /// 获取质量模式
    pub fn quality_mode(&self) -> UpscalingQuality {
        self.quality_mode
    }
    
    /// 设置质量模式
    pub fn set_quality_mode(&mut self, mode: UpscalingQuality) {
        self.quality_mode = mode;
    }
    
    /// 计算内部渲染分辨率
    pub fn calculate_render_resolution(&self, target_width: u32, target_height: u32) -> (u32, u32) {
        if self.active_tech == UpscalingTech::None {
            return (target_width, target_height);
        }
        
        let scale = self.quality_mode.render_scale();
        let render_width = (target_width as f32 * scale) as u32;
        let render_height = (target_height as f32 * scale) as u32;
        
        (render_width, render_height)
    }
    
    /// 获取性能提升估算
    pub fn estimated_performance_gain(&self) -> f32 {
        if self.active_tech == UpscalingTech::None {
            return 1.0;
        }
        
        let scale = self.quality_mode.render_scale();
        // 性能提升约等于像素数减少的比例
        let pixel_reduction = scale * scale;
        
        // 考虑超分辨率本身的开销
        let overhead = match self.active_tech {
            UpscalingTech::DLSS => 0.95,      // DLSS开销很小
            UpscalingTech::XeSS => 0.93,      // XeSS稍高
            UpscalingTech::FSR => 0.98,       // FSR开销极小
            UpscalingTech::MetalFX => 0.96,   // MetalFX开销小
            UpscalingTech::TAAUpsampling => 0.90, // TAA开销较大
            _ => 1.0,
        };
        
        (1.0 / pixel_reduction) * overhead
    }
    
    /// 获取技术描述
    pub fn tech_description(&self, tech: UpscalingTech) -> &'static str {
        match tech {
            UpscalingTech::None => "不使用超分辨率技术",
            UpscalingTech::DLSS => "NVIDIA DLSS - 基于深度学习的超分辨率，质量最佳",
            UpscalingTech::FSR => "AMD FSR - 开源超分辨率技术，兼容性好",
            UpscalingTech::XeSS => "Intel XeSS - 基于AI的超分辨率，适用于Intel GPU",
            UpscalingTech::MetalFX => "Apple MetalFX - Apple平台专用超分辨率",
            UpscalingTech::TAAUpsampling => "TAA超采样 - 通用时域抗锯齿超采样",
        }
    }
    
    /// 获取推荐设置
    pub fn get_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        recommendations.push(format!(
            "推荐使用: {:?} ({})",
            self.active_tech,
            self.tech_description(self.active_tech)
        ));
        
        recommendations.push(format!(
            "推荐质量模式: {:?} (内部分辨率 {:.0}%)",
            self.quality_mode,
            self.quality_mode.render_scale() * 100.0
        ));
        
        let (render_w, render_h) = self.calculate_render_resolution(1920, 1080);
        recommendations.push(format!(
            "1080p输出时内部渲染: {}x{}",
            render_w, render_h
        ));
        
        recommendations.push(format!(
            "预期性能提升: {:.1}x",
            self.estimated_performance_gain()
        ));
        
        match self.active_tech {
            UpscalingTech::DLSS => {
                recommendations.push("提示: DLSS在4K分辨率下效果最佳".to_string());
                recommendations.push("提示: 确保NVIDIA驱动为最新版本".to_string());
            }
            UpscalingTech::FSR => {
                recommendations.push("提示: FSR 2.0+提供更好的时域稳定性".to_string());
                recommendations.push("提示: FSR在所有GPU上都可用".to_string());
            }
            UpscalingTech::XeSS => {
                recommendations.push("提示: XeSS在Intel Arc GPU上性能最佳".to_string());
            }
            UpscalingTech::MetalFX => {
                recommendations.push("提示: MetalFX在Apple Silicon上效率极高".to_string());
            }
            _ => {}
        }
        
        recommendations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::performance::hardware::detect_gpu;

    #[test]
    fn test_upscaling_manager() {
        let gpu = detect_gpu();
        let manager = UpscalingManager::new(gpu);
        
        println!("Available Upscaling Technologies:");
        for tech in manager.available_techs() {
            println!("  - {:?}: {}", tech, manager.tech_description(*tech));
        }
        
        println!("\nActive Tech: {:?}", manager.active_tech());
        println!("Quality Mode: {:?}", manager.quality_mode());
        
        let (w, h) = manager.calculate_render_resolution(3840, 2160);
        println!("\n4K Output -> Internal Resolution: {}x{}", w, h);
        
        println!("\nEstimated Performance Gain: {:.2}x", manager.estimated_performance_gain());
    }

    #[test]
    fn test_recommendations() {
        let gpu = detect_gpu();
        let manager = UpscalingManager::new(gpu);
        
        println!("Upscaling Recommendations:");
        for rec in manager.get_recommendations() {
            println!("  {}", rec);
        }
    }
}
