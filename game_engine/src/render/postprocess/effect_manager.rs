//  后处理效果管理器
//
//  提供后处理效果链的动态管理、优化和性能监控功能。
//
//  ## 功能
//  - 动态效果链管理：运行时添加/移除效果
//  - 效果执行顺序优化：自动排序和合并兼容效果
//  - 性能监控：跟踪每个效果的GPU时间
//  - 自适应质量调整：根据性能自动降低质量
//  - 效果预设管理：保存和加载效果配置
//
//  # 示例
//
//  ```ignore
//  let mut manager = PostProcessEffectManager::new(&device, &config);
//  manager.add_effect(PostProcessEffect::Bloom { intensity: 0.8 });
//  manager.add_effect(PostProcessEffect::SSAO { radius: 0.5 });
//  manager.set_quality_mode(QualityMode::High);
//  manager.render(&mut encoder, &scene_view, &output_view);
//  ```

use crate::impl_default;
use std::collections::HashMap;
use wgpu::{CommandEncoder, Device, Queue, TextureView};

use super::{AntialiasingMode, PostProcessConfig, PostProcessPipeline, TonemapOperator};
use tracing::warn;

/// 后处理效果类型
#[derive(Debug, Clone, PartialEq)]
pub enum PostProcessEffect {
    /// 抗锯齿
    Antialiasing { mode: AntialiasingMode },
    /// Bloom 辉光
    Bloom {
        intensity: f32,
        threshold: f32,
        radius: f32,
    },
    /// SSAO 环境光遮蔽
    SSAO {
        radius: f32,
        intensity: f32,
        bias: f32,
    },
    /// SSR 屏幕空间反射
    SSR {
        max_distance: f32,
        step_count: u32,
        intensity: f32,
        edge_fade: f32,
    },
    /// 体积光
    VolumetricLighting {
        scattering_intensity: f32,
        sample_count: u32,
        god_ray_intensity: f32,
        fog_density: f32,
    },
    /// 程序化噪声
    ProceduralNoise {
        film_grain_intensity: f32,
        chromatic_aberration_intensity: f32,
        scanline_intensity: f32,
        noise_intensity: f32,
    },
    /// 运动模糊
    MotionBlur { intensity: f32, max_samples: u32 },
    /// 景深
    DepthOfField {
        focus_distance: f32,
        aperture: f32,
        near_blur: f32,
        far_blur: f32,
        max_blur_radius: f32,
    },
    /// 色彩校正
    ColorCorrection {
        brightness: f32,
        contrast: f32,
        saturation: f32,
        hue_shift: f32,
        chromatic_aberration: f32,
        vignette_intensity: f32,
        vignette_roundness: f32,
    },
    /// 色调映射
    Tonemap {
        operator: TonemapOperator,
        exposure: f32,
        gamma: f32,
    },
}

/// 效果执行顺序优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum EffectPriority {
    /// 最早执行（深度相关效果）
    Early = 0,
    /// 中期执行（光照和模糊效果）
    Mid = 1,
    /// 后期执行（色彩和输出效果）
    Late = 2,
    /// 最后执行（输出格式转换）
    Final = 3,
}

impl PostProcessEffect {
    /// 获取效果的执行优先级
    fn priority(&self) -> EffectPriority {
        match self {
            PostProcessEffect::SSAO { .. } => EffectPriority::Early,
            PostProcessEffect::SSR { .. } => EffectPriority::Early,
            PostProcessEffect::VolumetricLighting { .. } => EffectPriority::Mid,
            PostProcessEffect::Bloom { .. } => EffectPriority::Mid,
            PostProcessEffect::MotionBlur { .. } => EffectPriority::Mid,
            PostProcessEffect::DepthOfField { .. } => EffectPriority::Mid,
            PostProcessEffect::ProceduralNoise { .. } => EffectPriority::Late,
            PostProcessEffect::ColorCorrection { .. } => EffectPriority::Late,
            PostProcessEffect::Tonemap { .. } => EffectPriority::Final,
            PostProcessEffect::Antialiasing { .. } => EffectPriority::Final,
        }
    }

    /// 获取效果的GPU时间估算（毫秒）
    fn estimated_gpu_time(&self) -> f32 {
        match self {
            PostProcessEffect::SSAO { .. } => 2.0,
            PostProcessEffect::SSR { step_count, .. } => 2.5 + *step_count as f32 * 0.1,
            PostProcessEffect::VolumetricLighting { sample_count, .. } => {
                1.0 + *sample_count as f32 * 0.15
            }
            PostProcessEffect::ProceduralNoise { .. } => 0.5,
            PostProcessEffect::Bloom { radius, .. } => 1.0 + radius * 0.2,
            PostProcessEffect::MotionBlur { max_samples, .. } => 0.5 + *max_samples as f32 * 0.1,
            PostProcessEffect::DepthOfField {
                max_blur_radius, ..
            } => 1.5 + max_blur_radius * 0.15,
            PostProcessEffect::ColorCorrection { .. } => 0.3,
            PostProcessEffect::Tonemap { .. } => 0.2,
            PostProcessEffect::Antialiasing { .. } => 0.5,
        }
    }

    /// 检查效果是否可以合并
    fn can_merge_with(&self, other: &PostProcessEffect) -> bool {
        matches!(
            (self, other),
            (
                PostProcessEffect::ColorCorrection { .. },
                PostProcessEffect::ColorCorrection { .. }
            ) | (
                PostProcessEffect::Tonemap { .. },
                PostProcessEffect::Tonemap { .. }
            )
        )
    }
}

/// 质量模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualityMode {
    /// 低质量（性能优先）
    Low,
    /// 中等质量（平衡）
    Medium,
    /// 高质量（视觉效果优先）
    High,
    /// 极致质量（用于截图）
    Ultra,
}

impl QualityMode {
    /// 根据质量模式调整效果参数
    fn adjust_effect(&self, effect: &mut PostProcessEffect) {
        match (self, effect) {
            (QualityMode::Low, PostProcessEffect::Bloom { radius, .. }) => {
                *radius = (*radius * 0.5).max(2.0);
            }
            (QualityMode::Low, PostProcessEffect::SSAO { .. }) => {
                // 低质量下禁用SSAO
            }
            (QualityMode::Low, PostProcessEffect::MotionBlur { max_samples, .. }) => {
                *max_samples = (*max_samples / 2).max(4);
            }
            (
                QualityMode::Low,
                PostProcessEffect::DepthOfField {
                    max_blur_radius, ..
                },
            ) => {
                *max_blur_radius = (*max_blur_radius * 0.5).max(5.0);
            }
            (QualityMode::High, PostProcessEffect::Bloom { radius, .. }) => {
                *radius = (*radius * 1.5).min(15.0);
            }
            (QualityMode::High, PostProcessEffect::MotionBlur { max_samples, .. }) => {
                *max_samples = (*max_samples * 2).min(32);
            }
            (
                QualityMode::High,
                PostProcessEffect::DepthOfField {
                    max_blur_radius, ..
                },
            ) => {
                *max_blur_radius = (*max_blur_radius * 1.5).min(20.0);
            }
            (QualityMode::Ultra, PostProcessEffect::Bloom { radius, .. }) => {
                *radius = (*radius * 2.0).min(20.0);
            }
            (QualityMode::Ultra, PostProcessEffect::MotionBlur { max_samples, .. }) => {
                *max_samples = 32;
            }
            (
                QualityMode::Ultra,
                PostProcessEffect::DepthOfField {
                    max_blur_radius, ..
                },
            ) => {
                *max_blur_radius = 20.0;
            }
            _ => {}
        }
    }
}

/// 效果性能统计
#[derive(Debug, Clone, Default)]
pub struct EffectPerformanceStats {
    /// 平均GPU时间（毫秒）
    pub avg_gpu_time: f32,
    /// 最大GPU时间（毫秒）
    pub max_gpu_time: f32,
    /// 调用次数
    pub call_count: u32,
    /// 是否启用
    pub enabled: bool,
}

/// 效果预设
#[derive(Debug, Clone)]
pub struct EffectPreset {
    /// 预设名称
    pub name: String,
    /// 效果列表
    pub effects: Vec<PostProcessEffect>,
    /// 质量模式
    pub quality: QualityMode,
}

impl_default!(EffectPreset {
    name: "Default".to_string(),
    effects: vec![],
    quality: QualityMode::Medium,
});

/// 后处理效果管理器
pub struct PostProcessEffectManager {
    /// 基础后处理管线
    pipeline: PostProcessPipeline,

    /// 效果链（按优先级排序）
    effect_chain: Vec<PostProcessEffect>,

    /// 效果性能统计
    performance_stats: HashMap<String, EffectPerformanceStats>,

    /// 当前质量模式
    quality_mode: QualityMode,

    /// 自适应质量调整
    adaptive_quality: bool,

    /// 目标帧时间（毫秒）
    target_frame_time: f32,

    /// 效果预设
    presets: HashMap<String, EffectPreset>,

    /// 屏幕尺寸
    width: u32,
    height: u32,
}

impl PostProcessEffectManager {
    /// 创建后处理效果管理器
    pub fn new(device: &Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let pipeline = PostProcessPipeline::new(device, config);

        Self {
            pipeline,
            effect_chain: Vec::new(),
            performance_stats: HashMap::new(),
            quality_mode: QualityMode::Medium,
            adaptive_quality: false,
            target_frame_time: 16.67, // 60 FPS
            presets: HashMap::new(),
            width: config.width,
            height: config.height,
        }
    }

    /// 添加效果到链中
    pub fn add_effect(&mut self, effect: PostProcessEffect) {
        // 检查是否已存在相同类型的效果
        let effect_type = match &effect {
            PostProcessEffect::Antialiasing { .. } => "antialiasing",
            PostProcessEffect::Bloom { .. } => "bloom",
            PostProcessEffect::SSAO { .. } => "ssao",
            PostProcessEffect::SSR { .. } => "ssr",
            PostProcessEffect::VolumetricLighting { .. } => "volumetric_lighting",
            PostProcessEffect::ProceduralNoise { .. } => "procedural_noise",
            PostProcessEffect::MotionBlur { .. } => "motion_blur",
            PostProcessEffect::DepthOfField { .. } => "depth_of_field",
            PostProcessEffect::ColorCorrection { .. } => "color_correction",
            PostProcessEffect::Tonemap { .. } => "tonemap",
        };

        // 移除同类型的效果
        self.effect_chain.retain(|e| {
            let e_type = match e {
                PostProcessEffect::Antialiasing { .. } => "antialiasing",
                PostProcessEffect::Bloom { .. } => "bloom",
                PostProcessEffect::SSAO { .. } => "ssao",
                PostProcessEffect::SSR { .. } => "ssr",
                PostProcessEffect::VolumetricLighting { .. } => "volumetric_lighting",
                PostProcessEffect::ProceduralNoise { .. } => "procedural_noise",
                PostProcessEffect::MotionBlur { .. } => "motion_blur",
                PostProcessEffect::DepthOfField { .. } => "depth_of_field",
                PostProcessEffect::ColorCorrection { .. } => "color_correction",
                PostProcessEffect::Tonemap { .. } => "tonemap",
            };
            e_type != effect_type
        });

        // 应用质量模式调整
        let mut adjusted_effect = effect.clone();
        self.quality_mode.adjust_effect(&mut adjusted_effect);

        // 插入到正确的位置（按优先级排序）
        let priority = adjusted_effect.priority();
        let insert_pos = self
            .effect_chain
            .iter()
            .position(|e| e.priority() > priority)
            .unwrap_or(self.effect_chain.len());
        self.effect_chain.insert(insert_pos, adjusted_effect);

        // 初始化性能统计
        self.performance_stats.insert(
            effect_type.to_string(),
            EffectPerformanceStats {
                enabled: true,
                ..Default::default()
            },
        );
    }

    /// 移除效果
    pub fn remove_effect(&mut self, effect_type: &str) {
        self.effect_chain.retain(|e| {
            let e_type = match e {
                PostProcessEffect::Antialiasing { .. } => "antialiasing",
                PostProcessEffect::Bloom { .. } => "bloom",
                PostProcessEffect::SSAO { .. } => "ssao",
                PostProcessEffect::SSR { .. } => "ssr",
                PostProcessEffect::VolumetricLighting { .. } => "volumetric_lighting",
                PostProcessEffect::ProceduralNoise { .. } => "procedural_noise",
                PostProcessEffect::MotionBlur { .. } => "motion_blur",
                PostProcessEffect::DepthOfField { .. } => "depth_of_field",
                PostProcessEffect::ColorCorrection { .. } => "color_correction",
                PostProcessEffect::Tonemap { .. } => "tonemap",
            };
            e_type != effect_type
        });

        if let Some(stats) = self.performance_stats.get_mut(effect_type) {
            stats.enabled = false;
        }
    }

    /// 清空效果链
    pub fn clear_effects(&mut self) {
        self.effect_chain.clear();
        for stats in self.performance_stats.values_mut() {
            stats.enabled = false;
        }
    }

    /// 设置质量模式
    pub fn set_quality_mode(&mut self, mode: QualityMode) {
        self.quality_mode = mode;

        // 重新调整所有效果
        for effect in &mut self.effect_chain {
            self.quality_mode.adjust_effect(effect);
        }
    }

    /// 启用/禁用自适应质量
    pub fn set_adaptive_quality(&mut self, enabled: bool) {
        self.adaptive_quality = enabled;
    }

    /// 设置目标帧时间
    pub fn set_target_frame_time(&mut self, frame_time_ms: f32) {
        self.target_frame_time = frame_time_ms;
    }

    /// 优化效果链（合并兼容效果、移除冗余效果）
    pub fn optimize_chain(&mut self) {
        // 合并兼容的效果
        let mut merged: Vec<PostProcessEffect> = Vec::new();
        for effect in &self.effect_chain {
            if let Some(last) = merged.last_mut()
                && last.can_merge_with(effect)
            {
                // 合并效果（这里简化处理，实际应该合并参数）
                *last = effect.clone();
                continue;
            }
            merged.push(effect.clone());
        }
        self.effect_chain = merged;
    }

    /// 更新性能统计
    pub fn update_performance_stats(&mut self, effect_type: &str, gpu_time_ms: f32) {
        if let Some(stats) = self.performance_stats.get_mut(effect_type) {
            stats.call_count += 1;
            stats.avg_gpu_time = (stats.avg_gpu_time * (stats.call_count - 1) as f32 + gpu_time_ms)
                / stats.call_count as f32;
            stats.max_gpu_time = stats.max_gpu_time.max(gpu_time_ms);
        }
    }

    /// 自适应质量调整
    fn adaptive_quality_adjustment(&mut self) {
        if !self.adaptive_quality {
            return;
        }

        // 计算总GPU时间
        let total_time: f32 = self
            .performance_stats
            .values()
            .filter(|s| s.enabled)
            .map(|s| s.avg_gpu_time)
            .sum();

        // 如果超过目标时间，降低质量
        if total_time > self.target_frame_time {
            match self.quality_mode {
                QualityMode::Ultra => {
                    self.set_quality_mode(QualityMode::High);
                }
                QualityMode::High => {
                    self.set_quality_mode(QualityMode::Medium);
                }
                QualityMode::Medium => {
                    self.set_quality_mode(QualityMode::Low);
                }
                QualityMode::Low => {
                    // 禁用最耗时的效果
                    let mut effects_by_time: Vec<_> = self
                        .performance_stats
                        .iter()
                        .filter(|(_, s)| s.enabled)
                        .map(|(name, stats)| (name.clone(), stats.avg_gpu_time))
                        .collect();
                    effects_by_time.sort_by(|a, b| {
                        b.1.partial_cmp(&a.1).unwrap_or_else(|| {
                            warn!(
                                "Failed to compare GPU times for effect performance stats: {} and {}",
                                a.0, b.0
                            );
                            std::cmp::Ordering::Equal
                        })
                    });

                    if let Some((worst_effect, _)) = effects_by_time.first() {
                        self.remove_effect(worst_effect);
                    }
                }
            }
        }
    }

    /// 应用效果到配置
    fn apply_effects_to_config(&mut self) {
        // 重置配置
        self.pipeline.config = PostProcessConfig::default();

        for effect in &self.effect_chain {
            match effect {
                PostProcessEffect::Antialiasing { mode } => {
                    self.pipeline.config.antialiasing = *mode;
                }
                PostProcessEffect::Bloom {
                    intensity,
                    threshold,
                    radius,
                } => {
                    self.pipeline.config.bloom_enabled = true;
                    self.pipeline.config.bloom_intensity = *intensity;
                    self.pipeline.config.bloom_threshold = *threshold;
                    self.pipeline.config.bloom_radius = *radius;
                }
                PostProcessEffect::SSAO {
                    radius,
                    intensity,
                    bias,
                } => {
                    self.pipeline.config.ssao_enabled = true;
                    self.pipeline.config.ssao_radius = *radius;
                    self.pipeline.config.ssao_intensity = *intensity;
                    self.pipeline.config.ssao_bias = *bias;
                }
                PostProcessEffect::SSR {
                    max_distance,
                    step_count,
                    intensity,
                    edge_fade,
                } => {
                    // SSR效果暂未在PostProcessConfig中实现
                    // 这里只是占位，未来可以添加相应的配置字段
                    let _ = (*max_distance, *step_count, *intensity, *edge_fade);
                }
                PostProcessEffect::VolumetricLighting {
                    scattering_intensity,
                    sample_count,
                    god_ray_intensity,
                    fog_density,
                } => {
                    // 体积光效果暂未在PostProcessConfig中实现
                    // 这里只是占位，未来可以添加相应的配置字段
                    let _ = (
                        *scattering_intensity,
                        *sample_count,
                        *god_ray_intensity,
                        *fog_density,
                    );
                }
                PostProcessEffect::ProceduralNoise {
                    film_grain_intensity,
                    chromatic_aberration_intensity,
                    scanline_intensity,
                    noise_intensity,
                } => {
                    // 程序化噪声效果暂未在PostProcessConfig中实现
                    // 这里只是占位，未来可以添加相应的配置字段
                    let _ = (
                        *film_grain_intensity,
                        *chromatic_aberration_intensity,
                        *scanline_intensity,
                        *noise_intensity,
                    );
                }
                PostProcessEffect::MotionBlur {
                    intensity,
                    max_samples,
                } => {
                    self.pipeline.config.motion_blur_enabled = true;
                    self.pipeline.config.motion_blur_intensity = *intensity;
                    self.pipeline.config.motion_blur_max_samples = *max_samples;
                }
                PostProcessEffect::DepthOfField {
                    focus_distance,
                    aperture,
                    near_blur,
                    far_blur,
                    max_blur_radius,
                } => {
                    self.pipeline.config.depth_of_field_enabled = true;
                    self.pipeline.config.focus_distance = *focus_distance;
                    self.pipeline.config.aperture = *aperture;
                    self.pipeline.config.near_blur = *near_blur;
                    self.pipeline.config.far_blur = *far_blur;
                    self.pipeline.config.max_blur_radius = *max_blur_radius;
                }
                PostProcessEffect::ColorCorrection {
                    brightness,
                    contrast,
                    saturation,
                    hue_shift,
                    chromatic_aberration,
                    vignette_intensity,
                    vignette_roundness,
                } => {
                    self.pipeline.config.color_correction_enabled = true;
                    self.pipeline.config.brightness = *brightness;
                    self.pipeline.config.contrast = *contrast;
                    self.pipeline.config.saturation = *saturation;
                    self.pipeline.config.hue_shift = *hue_shift;
                    self.pipeline.config.chromatic_aberration = *chromatic_aberration;
                    self.pipeline.config.vignette_intensity = *vignette_intensity;
                    self.pipeline.config.vignette_roundness = *vignette_roundness;
                }
                PostProcessEffect::Tonemap {
                    operator,
                    exposure,
                    gamma,
                } => {
                    self.pipeline.config.tonemap_enabled = true;
                    self.pipeline.config.tonemap_operator = *operator;
                    self.pipeline.config.exposure = *exposure;
                    self.pipeline.config.gamma = *gamma;
                }
            }
        }
    }

    /// 渲染后处理效果
    pub fn render(
        &mut self,
        encoder: &mut CommandEncoder,
        device: &Device,
        queue: &Queue,
        scene_view: &TextureView,
        depth_view: Option<&TextureView>,
        motion_vector_view: Option<&TextureView>,
        output_view: &TextureView,
    ) {
        // 应用效果到配置
        self.apply_effects_to_config();

        // 自适应质量调整
        self.adaptive_quality_adjustment();

        // 执行渲染
        self.pipeline.render(
            encoder,
            device,
            queue,
            scene_view,
            depth_view,
            motion_vector_view,
            output_view,
        );
    }

    /// 调整大小
    pub fn resize(&mut self, device: &Device, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.pipeline.resize(device, width, height);
    }

    /// 保存预设
    pub fn save_preset(&mut self, name: String) {
        let preset = EffectPreset {
            name: name.clone(),
            effects: self.effect_chain.clone(),
            quality: self.quality_mode,
        };
        self.presets.insert(name, preset);
    }

    /// 加载预设
    pub fn load_preset(&mut self, name: &str) -> bool {
        if let Some(preset) = self.presets.get(name) {
            self.effect_chain = preset.effects.clone();
            self.set_quality_mode(preset.quality);
            true
        } else {
            false
        }
    }

    /// 获取效果链
    pub fn effect_chain(&self) -> &[PostProcessEffect] {
        &self.effect_chain
    }

    /// 获取性能统计
    pub fn performance_stats(&self) -> &HashMap<String, EffectPerformanceStats> {
        &self.performance_stats
    }

    /// 获取当前质量模式
    pub fn quality_mode(&self) -> QualityMode {
        self.quality_mode
    }
}
