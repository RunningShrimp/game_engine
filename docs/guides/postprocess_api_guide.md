# 后处理效果API指南

本指南详细介绍后处理效果系统的API使用方法和最佳实践。

## 概述

游戏引擎提供了两种后处理效果管理方式：

1. **PostProcessPipeline** - 固定效果链，简单易用
2. **PostProcessEffectManager** - 动态效果链，支持运行时管理和自适应质量

## PostProcessPipeline

### 基本使用

```rust
use game_engine::render::postprocess::{
    PostProcessPipeline, AntialiasingMode, TonemapOperator
};

// 创建后处理管线
let mut postprocess = PostProcessPipeline::new(device, &surface_config);

// 配置效果
postprocess.set_bloom_enabled(true);
postprocess.set_bloom_intensity(0.8);
postprocess.set_bloom_threshold(1.0);
postprocess.set_bloom_radius(5.0);

postprocess.set_ssao_enabled(true);
postprocess.set_ssao_params(0.5, 1.0, 0.025);

postprocess.set_motion_blur_enabled(true);
postprocess.set_motion_blur_intensity(0.3);
postprocess.set_motion_blur_max_samples(16);

postprocess.set_depth_of_field_enabled(true);
postprocess.set_focus_distance(10.0);
postprocess.set_aperture(0.5);
postprocess.set_depth_of_field_blur(1.0, 2.0);
postprocess.set_max_blur_radius(10.0);

postprocess.set_color_correction_enabled(true);
postprocess.set_brightness(0.0);
postprocess.set_contrast(1.0);
postprocess.set_saturation(1.0);
postprocess.set_hue_shift(0.0);
postprocess.set_chromatic_aberration(0.0);
postprocess.set_vignette_intensity(0.0);
postprocess.set_vignette_roundness(0.5);

postprocess.set_tonemap_operator(TonemapOperator::ACES);
postprocess.set_exposure(1.0);
postprocess.set_gamma(2.2);
```

### 渲染

```rust
postprocess.render(
    &mut encoder,
    device,
    queue,
    &scene_view,              // 场景纹理
    Some(&depth_view),        // 深度纹理（用于SSAO和景深）
    Some(&motion_vector_view), // 运动向量（用于运动模糊）
    &output_view,             // 输出纹理
);
```

## PostProcessEffectManager

### 创建和管理器

```rust
use game_engine::render::postprocess::{
    PostProcessEffectManager, PostProcessEffect, QualityMode
};

let mut manager = PostProcessEffectManager::new(device, &surface_config);
```

### 添加效果

```rust
// Bloom效果
manager.add_effect(PostProcessEffect::Bloom {
    intensity: 0.8,
    threshold: 1.0,
    radius: 5.0,
});

// SSAO效果
manager.add_effect(PostProcessEffect::SSAO {
    radius: 0.5,
    intensity: 1.0,
    bias: 0.025,
});

// 运动模糊
manager.add_effect(PostProcessEffect::MotionBlur {
    intensity: 0.3,
    max_samples: 16,
});

// 景深
manager.add_effect(PostProcessEffect::DepthOfField {
    focus_distance: 10.0,
    aperture: 0.5,
    near_blur: 1.0,
    far_blur: 2.0,
    max_blur_radius: 10.0,
});

// 色彩校正
manager.add_effect(PostProcessEffect::ColorCorrection {
    brightness: 0.0,
    contrast: 1.0,
    saturation: 1.0,
    hue_shift: 0.0,
    chromatic_aberration: 0.0,
    vignette_intensity: 0.0,
    vignette_roundness: 0.5,
});

// 色调映射
manager.add_effect(PostProcessEffect::Tonemap {
    operator: TonemapOperator::ACES,
    exposure: 1.0,
    gamma: 2.2,
});

// 抗锯齿
manager.add_effect(PostProcessEffect::Antialiasing {
    mode: AntialiasingMode::FXAA,
});
```

### 质量模式

```rust
// 设置质量模式
manager.set_quality_mode(QualityMode::Low);    // 性能优先
manager.set_quality_mode(QualityMode::Medium); // 平衡
manager.set_quality_mode(QualityMode::High);  // 视觉效果优先
manager.set_quality_mode(QualityMode::Ultra); // 极致质量（用于截图）
```

质量模式会自动调整效果参数：
- **Low**: 降低模糊半径、采样数，可能禁用SSAO
- **Medium**: 默认平衡设置
- **High**: 增加模糊半径、采样数
- **Ultra**: 最大质量设置

### 自适应质量

```rust
// 启用自适应质量调整
manager.set_adaptive_quality(true);

// 设置目标帧时间（毫秒）
manager.set_target_frame_time(16.67); // 60 FPS
manager.set_target_frame_time(33.33); // 30 FPS

// 管理器会根据实际性能自动调整质量
// 如果帧时间超过目标，会自动降低质量或禁用效果
```

### 效果链优化

```rust
// 优化效果链（合并兼容效果、移除冗余）
manager.optimize_chain();
```

优化会：
- 合并兼容的效果（如多个ColorCorrection合并为一个）
- 移除重复的效果
- 按优先级重新排序

### 预设管理

```rust
// 保存当前效果配置为预设
manager.save_preset("cinematic".to_string());
manager.save_preset("performance".to_string());
manager.save_preset("ultra_quality".to_string());

// 加载预设
manager.load_preset("cinematic");

// 创建自定义预设
manager.clear_effects();
manager.add_effect(PostProcessEffect::Bloom {
    intensity: 1.2,
    threshold: 0.8,
    radius: 8.0,
});
manager.add_effect(PostProcessEffect::SSAO {
    radius: 0.7,
    intensity: 1.5,
    bias: 0.02,
});
manager.set_quality_mode(QualityMode::Ultra);
manager.save_preset("custom".to_string());
```

### 性能监控

```rust
// 渲染后查看性能统计
manager.render(
    &mut encoder,
    device,
    queue,
    &scene_view,
    Some(&depth_view),
    Some(&motion_vector_view),
    &output_view,
);

// 获取性能统计
let stats = manager.performance_stats();
for (effect_name, stat) in stats {
    if stat.enabled {
        println!("{}: avg={:.2}ms, max={:.2}ms, calls={}",
                 effect_name, stat.avg_gpu_time, stat.max_gpu_time, stat.call_count);
    }
}
```

### 动态管理效果

```rust
// 移除效果
manager.remove_effect("bloom");
manager.remove_effect("ssao");

// 清空所有效果
manager.clear_effects();

// 获取当前效果链
let effects = manager.effect_chain();
for effect in effects {
    println!("Effect: {:?}", effect);
}
```

### 调整大小

```rust
// 窗口大小改变时调整后处理管线
manager.resize(device, new_width, new_height);
```

## 效果执行顺序

效果按以下顺序执行（由管理器自动排序）：

1. **Early (早期)**: SSAO - 深度相关效果
2. **Mid (中期)**: Bloom, Motion Blur, Depth of Field - 光照和模糊效果
3. **Late (后期)**: Color Correction - 色彩调整
4. **Final (最终)**: Tonemap, Antialiasing - 输出格式转换

## 最佳实践

### 1. 性能优化

```rust
// 根据目标平台选择质量模式
#[cfg(target_arch = "wasm32")]
manager.set_quality_mode(QualityMode::Low);

#[cfg(not(target_arch = "wasm32"))]
manager.set_quality_mode(QualityMode::High);

// 启用自适应质量以保持稳定帧率
manager.set_adaptive_quality(true);
manager.set_target_frame_time(16.67);
```

### 2. 效果组合

```rust
// 推荐的效果组合

// 电影风格
manager.add_effect(PostProcessEffect::Bloom { intensity: 1.0, threshold: 0.8, radius: 8.0 });
manager.add_effect(PostProcessEffect::SSAO { radius: 0.6, intensity: 1.2, bias: 0.02 });
manager.add_effect(PostProcessEffect::DepthOfField { 
    focus_distance: 10.0, aperture: 0.4, near_blur: 1.5, far_blur: 3.0, max_blur_radius: 15.0 
});
manager.add_effect(PostProcessEffect::ColorCorrection {
    brightness: 0.1, contrast: 1.1, saturation: 1.2, hue_shift: 0.0,
    chromatic_aberration: 0.1, vignette_intensity: 0.3, vignette_roundness: 0.6,
});
manager.add_effect(PostProcessEffect::Tonemap {
    operator: TonemapOperator::ACES, exposure: 1.2, gamma: 2.2,
});

// 性能优先
manager.add_effect(PostProcessEffect::Bloom { intensity: 0.5, threshold: 1.0, radius: 3.0 });
manager.add_effect(PostProcessEffect::Tonemap {
    operator: TonemapOperator::Reinhard, exposure: 1.0, gamma: 2.2,
});
manager.set_quality_mode(QualityMode::Low);
```

### 3. 预设使用

```rust
// 在游戏设置中提供预设选择
enum PostProcessPreset {
    Off,
    Low,
    Medium,
    High,
    Ultra,
    Custom,
}

fn apply_preset(manager: &mut PostProcessEffectManager, preset: PostProcessPreset) {
    match preset {
        PostProcessPreset::Off => {
            manager.clear_effects();
        }
        PostProcessPreset::Low => {
            manager.load_preset("low");
        }
        PostProcessPreset::Medium => {
            manager.load_preset("medium");
        }
        PostProcessPreset::High => {
            manager.load_preset("high");
        }
        PostProcessPreset::Ultra => {
            manager.load_preset("ultra");
        }
        PostProcessPreset::Custom => {
            // 加载用户自定义预设
        }
    }
}
```

### 4. 性能监控

```rust
// 定期检查性能并调整
fn update_postprocess_performance(manager: &mut PostProcessEffectManager) {
    let stats = manager.performance_stats();
    let total_time: f32 = stats.values()
        .filter(|s| s.enabled)
        .map(|s| s.avg_gpu_time)
        .sum();
    
    if total_time > 20.0 {
        // 如果总时间超过20ms，降低质量
        manager.set_quality_mode(QualityMode::Low);
    } else if total_time < 10.0 {
        // 如果总时间小于10ms，可以提高质量
        manager.set_quality_mode(QualityMode::High);
    }
}
```

## 常见问题

### Q: 如何禁用所有后处理效果？

```rust
manager.clear_effects();
// 或者
postprocess.config.bloom_enabled = false;
postprocess.config.ssao_enabled = false;
// ... 禁用其他效果
```

### Q: 效果顺序可以自定义吗？

效果顺序由优先级自动决定，不能手动调整。如果需要特定顺序，可以：
1. 使用 `PostProcessPipeline` 手动控制顺序
2. 分多次调用 `render()`，每次只应用部分效果

### Q: 如何实现自定义后处理效果？

目前不支持自定义效果，但可以通过以下方式扩展：
1. 实现新的 `PostProcessEffect` 变体
2. 在 `PostProcessEffectManager` 中添加处理逻辑
3. 创建对应的渲染通道

## 相关文档

- [API参考](../api_reference.md)
- [渲染系统文档](../architecture.md#渲染系统)
- [性能调优指南](../performance_tuning_guide.md)

