# 后处理效果完整指南

**文档版本**: v1.0.0  
**最后更新**: 2026年1月2日

---

## 目录

- [1. 概述](#概述)
- [2. 支持的效果](#支持的效果)
- [3. 基本使用](#基本使用)
- [4. 效果详解](#效果详解)
- [5. 性能优化](#性能优化)
- [6. 完整示例](#完整示例)

---

## 概述

游戏引擎提供完整的高质量后处理管线，包含12种专业级效果：

**✨ 后处理效果特性**
- **高性能**: 所有效果在GPU上执行，使用计算着色器优化
- **灵活组合**: 可以自由组合多个效果
- **可配置**: 每个效果都有详细的配置选项
- **质量模式**: 提供Low/Medium/High/Ultra四种质量模式
- **预设模板**: 内置电影、游戏、仿真等多种预设

**📊 支持的效果**
```
✅ Bloom (辉光)
✅ SSAO (屏幕空间环境光遮蔽)
✅ SSR (屏幕空间反射)
✅ Tone Mapping (色调映射)
✅ Depth of Field (景深)
✅ Motion Blur (运动模糊)
✅ Color Correction (色彩校正)
✅ Volumetric Lighting (体积光)
✅ Procedural Noise (程序化噪声)
✅ Antialiasing (抗锯齿)
✅ Chromatic Aberration (色差)
✅ Vignette (暗角)
```

---

## 支持的效果

### 1. Bloom (辉光效果)

**功能**: 模拟相机镜头的光晕效果，增强亮部

**参数**:
- `intensity` (0.0-2.0): 辉光强度
- `threshold` (0.1-2.0): 亮度阈值，超过此值才产生辉光
- `radius` (1.0-20.0): 模糊半径

**适用场景**: 夜景、霓虹灯、魔法效果、发光物体

```rust
use game_engine::render::postprocess::{PostProcessPipeline, PostProcessConfig};

let config = PostProcessConfig {
    bloom_enabled: true,
    bloom_intensity: 0.8,      // 中等强度
    bloom_threshold: 0.7,       // 亮度超过70%时产生辉光
    bloom_radius: 8.0,           // 中等模糊半径
    ..Default::default()
};
```

### 2. SSAO (屏幕空间环境光遮蔽)

**功能**: 计算环境光遮蔽，增强场景深度感

**参数**:
- `radius` (0.1-2.0): 采样半径，影响遮蔽范围
- `intensity` (0.0-2.0): 遮蔽强度
- `bias` (0.001-0.1): 偏移，避免自遮蔽

**适用场景**: 所有场景，特别是室内、阴影区域

```rust
let config = PostProcessConfig {
    ssao_enabled: true,
    ssao_radius: 0.5,          // 中等采样半径
    ssao_intensity: 1.2,         // 中等强度
    ssao_bias: 0.025,            // 较小偏移
    ..Default::default()
};
```

### 3. SSR (屏幕空间反射)

**功能**: 在屏幕空间计算反射，无需光线追踪

**参数**:
- `max_distance` (5.0-50.0): 最大反射距离
- `step_count` (4-64): 追踪步数
- `intensity` (0.0-1.0): 反射强度
- `edge_fade` (0.0-1.0): 边缘淡出

**适用场景**: 反射表面（水面、镜子、金属）

```rust
let config = PostProcessConfig {
    // SSR通过effect_manager配置
    ..Default::default()
};

let ssr_effect = PostProcessEffect::SSR {
    max_distance: 20.0,        // 最大反射距离20米
    step_count: 32,             // 32步追踪
    intensity: 0.7,             // 70%反射强度
    edge_fade: 0.8,              // 边缘淡出
};
```

### 4. Tone Mapping (色调映射)

**功能**: HDR到SDR的色彩映射，防止过曝

**支持的算子**:
- **Reinhard**: 经典Reinhard色调映射
- **ACES**: 电影工业标准（推荐）
- **Filmic**: 胶片风格
- **None**: 禁用色调映射

**参数**:
- `exposure` (0.1-10.0): 曝光值
- `gamma` (1.0-3.0): Gamma校正值

**适用场景**: HDR渲染，高动态范围场景

```rust
use game_engine::render::postprocess::{TonemapOperator, PostProcessEffect};

let config = PostProcessConfig {
    tonemap_enabled: true,
    tonemap_operator: TonemapOperator::ACES,  // ACES电影标准
    exposure: 1.0,                       // 标准曝光
    gamma: 2.2,                           // sRGB gamma
    ..Default::default()
};
```

### 5. Depth of Field (景深)

**功能**: 模拟相机光圈，产生焦点和背景模糊

**参数**:
- `focus_distance` (0.0-100.0): 焦点距离
- `aperture` (0.0-1.0): 光圈大小（影响模糊强度）
- `near_blur` (0.0-10.0): 近景模糊强度
- `far_blur` (0.0-10.0): 远景模糊强度
- `max_blur_radius` (0.0-20.0): 最大模糊半径

**适用场景**: 电影质感、焦点切换、景深效果

```rust
let config = PostProcessConfig {
    depth_of_field_enabled: true,
    focus_distance: 10.0,     // 焦点距离10米
    aperture: 0.5,             // 中等光圈
    near_blur: 1.0,            // 近景模糊
    far_blur: 2.0,             // 远景模糊
    max_blur_radius: 8.0,      // 最大模糊半径
    ..Default::default()
};
```

### 6. Motion Blur (运动模糊)

**功能**: 模拟相机移动产生的运动模糊

**参数**:
- `intensity` (0.0-1.0): 模糊强度
- `max_samples` (4-32): 最大采样数

**适用场景**: 快速移动、赛车、动作游戏

```rust
let config = PostProcessConfig {
    motion_blur_enabled: true,
    motion_blur_intensity: 0.4,       // 中等强度
    motion_blur_max_samples: 16,       // 16次采样
    ..Default::default()
};
```

### 7. Color Correction (色彩校正)

**功能**: 调整图像的色彩属性

**参数**:
- `brightness` (-1.0-1.0): 亮度调整
- `contrast` (0.0-2.0): 对比度调整
- `saturation` (0.0-2.0): 饱和度调整
- `hue_shift` (-180.0-180.0): 色调偏移
- `chromatic_aberration` (0.0-1.0): 色差强度
- `vignette_intensity` (0.0-1.0): 暗角强度
- `vignette_roundness` (0.0-1.0): 暗角圆度

**适用场景**: 艺彩风格化、滤镜效果、艺术渲染

```rust
let config = PostProcessConfig {
    color_correction_enabled: true,
    brightness: 0.0,              // 标准亮度
    contrast: 1.1,                 // 稍高对比度
    saturation: 1.2,               // 高饱和度
    hue_shift: 0.0,               // 无色调偏移
    chromatic_aberration: 0.05,     // 轻微色差
    vignette_intensity: 0.3,         // 暗角
    vignette_roundness: 0.5,          // 圆形暗角
    ..Default::default()
};
```

### 8. Volumetric Lighting (体积光)

**功能**: 模拟光在介质中传播的体积效果

**参数**:
- `scattering_intensity` (0.0-2.0): 散射强度
- `sample_count` (8-128): 采样数
- `god_ray_intensity` (0.0-2.0): 上帝之光强度
- `fog_density` (0.0-1.0): 雾密度

**适用场景**: 雾、水下、上帝之光、大气散射

```rust
let config = PostProcessConfig {
    volumetric_lighting_enabled: true,
    scattering_intensity: 1.0,        // 标准散射
    sample_count: 32,                 // 中等采样
    god_ray_intensity: 0.5,           // 中等上帝之光
    fog_density: 0.1,                // 轻微雾
    ..Default::default()
};
```

### 9. Procedural Noise (程序化噪声)

**功能**: 添加胶片颗粒、色差、扫描线等噪声效果

**参数**:
- `film_grain_intensity` (0.0-1.0): 胶片颗粒强度
- `chromatic_aberration_intensity` (0.0-1.0): 色差强度
- `scanline_intensity` (0.0-1.0): 扫描线强度
- `noise_intensity` (0.0-1.0): 通用噪声强度

**适用场景**: 复古效果、电影质感、艺术风格

```rust
let config = PostProcessConfig {
    procedural_noise_enabled: true,
    film_grain_intensity: 0.15,        // 轻微胶片颗粒
    chromatic_aberration_intensity: 0.1,  // 色差
    scanline_intensity: 0.05,          // 轻微扫描线
    noise_intensity: 0.1,               // 通用噪声
    ..Default::default()
};
```

### 10. Antialiasing (抗锯齿)

**支持的模式**:
- **FXAA**: Fast Approximate Anti-Aliasing（快速近似抗锯齿）
- **TAA**: Temporal Anti-Aliasing（时序抗锯齿）

**适用场景**: 所有渲染，消除锯齿

```rust
use game_engine::render::postprocess::{AntialiasingMode, FxaaQuality};

let config = PostProcessConfig {
    antialiasing: AntialiasingMode::FXAA,
    fxaa_quality: FxaaQuality::High,  // 高质量FXAA
    ..Default::default()
};
```

---

## 基本使用

### 初始化后处理管线

```rust
use game_engine::render::postprocess::{PostProcessPipeline, PostProcessConfig, QualityMode};

// 创建后处理管线
let config = PostProcessConfig {
    // 启用多个效果
    bloom_enabled: true,
    tonemap_enabled: true,
    ssao_enabled: true,
    
    // 质量模式
    bloom_radius: 8.0,       // 中等质量
    tonemap_operator: TonemapOperator::ACES,
    
    ..Default::default()
};

let mut postprocess = PostProcessPipeline::new(&device, config, QualityMode::Medium)?;
```

### 渲染后处理

```rust
// 在主渲染循环中
postprocess.render(
    &mut encoder,           // 命令编码器
    &device,               // GPU设备
    &queue,               // GPU队列
    &scene_texture_view,    // 场景纹理
    Some(&depth_view),      // 深度纹理（可选，用于SSAO/DOF）
    Some(&motion_vector_view),  // 运动向量（可选，用于运动模糊）
    &output_view,          // 输出纹理
)?;
```

### 动态更新效果

```rust
// 在运行时更新效果
postprocess.set_bloom_intensity(0.9);  // 更新Bloom强度
postprocess.set_focus_distance(15.0);   // 更新景深焦点
postprocess.set_exposure(1.2);          // 更新曝光值
```

---

## 效果详解

### 效果执行顺序

后处理效果按以下顺序执行（从Early到Final）：

```
1. Early (早期效果)
   └─ SSAO (需要深度)
   
2. Mid (中期效果)
   ├─ Bloom
   ├─ Volumetric Lighting
   ├─ Motion Blur
   └─ Depth of Field (需要深度)
   
3. Late (后期效果)
   ├─ SSR
   ├─ Color Correction
   ├─ Procedural Noise
   └─ Antialiasing
   
4. Final (最终效果)
   └─ Tone Mapping (必须最后执行)
```

### 质量模式

| 模式 | Bloom半径 | SSR步数 | Volumetric采样 | 性能影响 |
|------|-----------|----------|----------------|----------|
| Low | 3.0 | 8 | 16 | 低 |
| Medium | 8.0 | 32 | 64 | 中 |
| High | 15.0 | 64 | 128 | 高 |
| Ultra | 25.0 | 128 | 256 | 极高 |

```rust
use game_engine::render::postprocess::QualityMode;

let postprocess = PostProcessPipeline::new(&device, config, QualityMode::High)?;
```

### 预设模板

引擎提供多个预设，可快速切换不同视觉风格：

```rust
use game_engine::render::postprocess::EffectPreset;

// 电影模式
let cinematic_config = EffectPreset::Cinematic;

// 游戏模式（平衡性能和质量）
let gaming_config = EffectPreset::Gaming;

// 仿真模式（高质量）
let simulation_config = EffectPreset::Simulation;

// 复古模式
let retro_config = EffectPreset::Retro;

let postprocess = PostProcessPipeline::from_preset(&device, cinematic_config)?;
```

---

## 性能优化

### 1. 按需启用效果

```rust
// 根据硬件性能动态调整
let quality = if hardware.is_high_end() {
    QualityMode::High
} else if hardware.is_mid_range() {
    QualityMode::Medium
} else {
    QualityMode::Low
};

let mut config = PostProcessConfig::default();

// 低端硬件禁用某些效果
if quality == QualityMode::Low {
    config.ssao_enabled = false;
    config.volumetric_lighting_enabled = false;
    config.bloom_radius = 4.0;
} else if quality == QualityMode::High {
    config.bloom_radius = 15.0;
    config.motion_blur_max_samples = 32;
}

let postprocess = PostProcessPipeline::new(&device, config, quality)?;
```

### 2. 使用性能监控

```rust
use game_engine::render::postprocess::EffectPerformanceStats;

let stats = postprocess.get_performance_stats();

println!("后处理性能统计:");
println!("  Bloom: {:.2} ms", stats.bloom_time_ms);
println!("  SSAO: {:.2} ms", stats.ssao_time_ms);
println!("  Tone Mapping: {:.2} ms", stats.tonemap_time_ms);
println!("  Total: {:.2} ms", stats.total_time_ms);
println!("  FPS: {:.1}", stats.current_fps);
```

### 3. 优化建议

| 效果 | 低端优化 | 中端优化 | 高端优化 |
|------|----------|----------|----------|
| Bloom | radius=3, samples=8 | radius=8, samples=16 | radius=15, samples=32 |
| SSAO | 禁用 | radius=0.3, samples=8 | radius=0.5, samples=16 |
| SSR | 禁用 | steps=16 | steps=64 |
| Tone Map | 禁用 | Reinhard | ACES |
| Volumetric | 禁用 | samples=32 | samples=128 |

---

## 完整示例

### 示例1: 基础后处理设置

```rust
use game_engine::render::postprocess::{PostProcessPipeline, PostProcessConfig, QualityMode};

fn setup_basic_postprocessing(device: &wgpu::Device) -> Result<PostProcessPipeline, RenderError> {
    let config = PostProcessConfig {
        // 启用基础效果
        bloom_enabled: true,
        tonemap_enabled: true,
        
        // Bloom配置
        bloom_intensity: 0.6,
        bloom_threshold: 0.8,
        bloom_radius: 6.0,
        
        // Tone Mapping配置
        tonemap_operator: TonemapOperator::ACES,
        exposure: 1.0,
        gamma: 2.2,
        
        ..Default::default()
    };
    
    // 使用中等质量平衡性能和视觉质量
    PostProcessPipeline::new(device, config, QualityMode::Medium)
}
```

### 示例2: 电影级后处理

```rust
use game_engine::render::postprocess::{PostProcessPipeline, PostProcessConfig, QualityMode};

fn setup_cinematic_postprocessing(device: &wgpu::Device) -> Result<PostProcessPipeline, RenderError> {
    let config = PostProcessConfig {
        // 启用所有高级效果
        bloom_enabled: true,
        ssao_enabled: true,
        tonemap_enabled: true,
        motion_blur_enabled: true,
        depth_of_field_enabled: true,
        color_correction_enabled: true,
        
        // Bloom - 增强亮度
        bloom_intensity: 0.9,
        bloom_threshold: 0.6,
        bloom_radius: 12.0,
        
        // SSAO - 增强深度
        ssao_radius: 0.6,
        ssao_intensity: 1.5,
        ssao_bias: 0.02,
        
        // Depth of Field - 电影景深
        focus_distance: 8.0,
        aperture: 0.7,
        near_blur: 2.5,
        far_blur: 4.0,
        max_blur_radius: 15.0,
        
        // Color Correction - 电影色调
        brightness: -0.05,
        contrast: 1.15,
        saturation: 0.9,
        chromatic_aberration: 0.1,
        vignette_intensity: 0.4,
        vignette_roundness: 0.5,
        
        // Tone Mapping - ACES
        tonemap_operator: TonemapOperator::ACES,
        exposure: 1.0,
        gamma: 2.2,
        
        ..Default::default()
    };
    
    // 高质量电影渲染
    PostProcessPipeline::new(device, config, QualityMode::High)
}
```

### 示例3: 性能优先级后处理

```rust
fn setup_performance_postprocessing(device: &wgpu::Device) -> Result<PostProcessPipeline, RenderError> {
    let config = PostProcessConfig {
        // 仅启用必要的后处理
        tonemap_enabled: true,
        antialiasing: AntialiasingMode::FXAA,
        
        // 禁用耗时效果
        ssao_enabled: false,
        bloom_enabled: false,
        motion_blur_enabled: false,
        volumetric_lighting_enabled: false,
        
        // 基础Tone Mapping
        tonemap_operator: TonemapOperator::Reinhard,
        exposure: 1.0,
        gamma: 2.2,
        
        // 低质量FXAA
        fxaa_quality: FxaaQuality::Low,
        
        ..Default::default()
    };
    
    // 低质量模式
    PostProcessPipeline::new(device, config, QualityMode::Low)
}
```

### 示例4: 动态效果切换

```rust
use game_engine::render::postprocess::{PostProcessPipeline, TonemapOperator};

struct GameState {
    postprocess: PostProcessPipeline,
    is_night: bool,
    is_underwater: bool,
}

impl GameState {
    fn new(device: &wgpu::Device) -> Result<Self, RenderError> {
        let postprocess = PostProcessPipeline::new(device, Default::default(), QualityMode::Medium)?;
        Ok(Self {
            postprocess,
            is_night: false,
            is_underwater: false,
        })
    }
    
    fn update_environment(&mut self, is_night: bool, is_underwater: bool) {
        if self.is_night != is_night {
            self.is_night = is_night;
            
            if is_night {
                // 夜间：增强Bloom和暗角
                self.postprocess.set_bloom_intensity(1.2);
                self.postprocess.set_vignette_intensity(0.6);
            } else {
                // 日间：正常效果
                self.postprocess.set_bloom_intensity(0.6);
                self.postprocess.set_vignette_intensity(0.2);
            }
        }
        
        if self.is_underwater != is_underwater {
            self.is_underwater = is_underwater;
            
            if is_underwater {
                // 水下：增加体积光和雾
                self.postprocess.set_volumetric_intensity(1.5);
                self.postprocess.set_fog_density(0.3);
                self.postprocess.set_saturation(0.7);  // 降低饱和度
            } else {
                // 地面：正常效果
                self.postprocess.set_volumetric_intensity(0.8);
                self.postprocess.set_fog_density(0.0);
                self.postprocess.set_saturation(1.0);
            }
        }
    }
    
    fn switch_tonemap_operator(&mut self, operator: TonemapOperator) {
        self.postprocess.set_tonemap_operator(operator);
    }
}
```

### 示例5: 自定义效果预设

```rust
use game_engine::render::postprocess::{PostProcessConfig, PostProcessPipeline, QualityMode};

fn create_custom_preset(device: &wgpu::Device) -> Result<PostProcessPipeline, RenderError> {
    // 自定义"霓虹赛博朋克"预设
    let config = PostProcessConfig {
        // 启用关键效果
        bloom_enabled: true,
        tonemap_enabled: true,
        color_correction_enabled: true,
        
        // Bloom - 强烈的霓虹光
        bloom_intensity: 1.5,
        bloom_threshold: 0.5,
        bloom_radius: 20.0,
        
        // Color Correction - 高饱和、高对比
        brightness: 0.1,
        contrast: 1.4,
        saturation: 1.6,
        hue_shift: 0.0,
        
        // 色差增强
        chromatic_aberration: 0.2,
        
        // Tone Mapping - 稍高的曝光
        tonemap_operator: TonemapOperator::ACES,
        exposure: 1.2,
        gamma: 2.2,
        
        ..Default::default()
    };
    
    PostProcessPipeline::new(device, config, QualityMode::High)
}
```

---

## API参考

### PostProcessPipeline 结构

```rust
pub struct PostProcessPipeline {
    pub config: PostProcessConfig,
    // 私有字段...
}

impl PostProcessPipeline {
    /// 创建新的后处理管线
    pub fn new(
        device: &wgpu::Device,
        config: PostProcessConfig,
        quality: QualityMode,
    ) -> Result<Self, RenderError>;
    
    /// 从预设创建
    pub fn from_preset(
        device: &wgpu::Device,
        preset: EffectPreset,
    ) -> Result<Self, RenderError>;
    
    /// 渲染后处理
    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene_view: &wgpu::TextureView,
        depth_view: Option<&wgpu::TextureView>,
        motion_vector_view: Option<&wgpu::TextureView>,
        output_view: &wgpu::TextureView,
    ) -> Result<(), RenderError>;
    
    /// 获取性能统计
    pub fn get_performance_stats(&self) -> EffectPerformanceStats;
    
    /// 动态更新效果
    pub fn set_bloom_intensity(&mut self, intensity: f32);
    pub fn set_focus_distance(&mut self, distance: f32);
    pub fn set_exposure(&mut self, exposure: f32);
    pub fn set_tonemap_operator(&mut self, operator: TonemapOperator);
    // ... 更多setter方法
}
```

---

## 故障排除

### 问题1: 后处理太慢

**症状**: 帧率显著下降

**解决方案**:
1. 降低质量模式（High → Medium → Low）
2. 禁用某些效果（SSAO, Volumetric Lighting）
3. 减少采样数量（Bloom半径, SSR步数）
4. 使用FXAA替代TAA

```rust
let config = PostProcessConfig {
    // 降低质量
    bloom_radius: 4.0,
    
    // 禁用最耗性能的效果
    ssao_enabled: false,
    volumetric_lighting_enabled: false,
    
    ..Default::default()
};
```

### 问题2: Bloom过曝

**症状**: 整个画面过亮

**解决方案**:
1. 提高Bloom阈值（threshold）
2. 降低Bloom强度（intensity）
3. 调整Tone Mapping曝光

```rust
let config = PostProcessConfig {
    bloom_enabled: true,
    bloom_threshold: 1.2,      // 提高阈值
    bloom_intensity: 0.4,        // 降低强度
    tonemap_enabled: true,
    exposure: 0.8,              // 降低曝光
    ..Default::default()
};
```

### 问题3: SSAO遮挡过强

**症状**: 阴影区域太暗

**解决方案**:
1. 降低SSAO强度
2. 减小采样半径
3. 增加偏移（bias）

```rust
let config = PostProcessConfig {
    ssao_enabled: true,
    ssao_intensity: 0.6,        // 降低强度
    ssao_radius: 0.3,           // 减小半径
    ssao_bias: 0.05,            // 增加偏移
    ..Default::default()
};
```

---

## 最佳实践

### 1. 效果组合建议

**电影风格**:
- Bloom + SSAO + Depth of Field + Tone Mapping + Color Correction

**游戏风格**:
- Bloom + SSR + Motion Blur + Tone Mapping

**仿真风格**:
- SSAO + SSR + Volumetric Lighting + Depth of Field + Tone Mapping

**艺术风格**:
- Bloom + Color Correction + Procedural Noise + Vignette

### 2. 性能预算建议

| GPU等级 | 推荐效果组合 | 目标FPS |
|---------|---------------|----------|
| 集成显卡 | 全部效果 | 30-60 |
| 中端显卡 | Bloom + SSR + Tone Mapping | 60-90 |
| 低端显卡 | Tone Mapping + FXAA | 60-120 |

### 3. 调试技巧

```rust
// 逐个测试效果，找出性能瓶颈
let mut config = PostProcessConfig::default();

// 1. 只启用Bloom
config.bloom_enabled = true;
let pipeline = PostProcessPipeline::new(&device, config, QualityMode::High)?;
measure_fps(&pipeline);

// 2. 启用SSAO
config.ssao_enabled = true;
let pipeline = PostProcessPipeline::new(&device, config, QualityMode::High)?;
measure_fps(&pipeline);

// 3. 启用所有效果
config.volumetric_lighting_enabled = true;
config.ssr_enabled = true;
let pipeline = PostProcessPipeline::new(&device, config, QualityMode::High)?;
measure_fps(&pipeline);
```

---

## 参考资源

- [高级功能使用指南](./ADVANCED_FEATURES_GUIDE.md)
- [VXGI全局光照](./GLOBAL_ILLUMINATION.md)
- [光线追踪文档](./RAY_TRACING_GUIDE.md)
- [性能优化文档](./PERFORMANCE_OPTIMIZATION_GUIDE.md)

---

**文档维护**: 如有问题或建议，请提交Issue或Pull Request

