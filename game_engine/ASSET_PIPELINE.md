# Asset Pipeline - 资源优化管线

## 概述

Asset Pipeline是游戏引擎的一站式资源优化解决方案，提供自动化的LOD生成、纹理压缩、着色器优化和资源打包功能。

## 功能特性

### 1. 自动LOD生成 (Level of Detail)
- 支持GLTF、OBJ、FBX格式的3D模型
- 自动生成多级细节（LOD0-LODn）
- 基于质量阈值的智能简化
- 保持UV和法线信息

### 2. 纹理压缩
- 支持多种压缩格式：
  - BC1/DXT1: 4:1压缩比，无透明度
  - BC3/DXT5: 4:1压缩比，有透明度
  - BC7: 高质量压缩，8:1压缩比
  - ASTC 4x4: 自适应压缩
  - ETC2: 移动平台优化
- 自动生成MIP链
- 自动调整分辨率

### 3. 着色器优化
- WGSL着色器代码优化
- 死代码消除
- 常量折叠
- 函数内联提示
- 数学运算优化

### 4. 资源打包
- Pak格式：单文件打包，带压缩
- Loose格式：松散文件结构
- Virtual格式：虚拟文件系统

### 5. 质量分析
- 纹理分辨率检查
- 多边形数量分析
- 内存使用估算
- 加载时间预测
- HTML格式报告生成

## 安装

确保启用了CLI功能：

```bash
cargo build --features cli
```

## 使用方法

### 命令行界面

#### 优化资源

```bash
# 基础用法
game-engine optimize ./assets -o ./assets_optimized

# 指定质量预设
game-engine optimize ./assets -o ./assets_high --quality High

# 指定目标平台
game-engine optimize ./assets -o ./assets_mobile --platform Mobile

# 完整参数
game-engine optimize ./assets \
  -o ./assets_optimized \
  --quality High \
  --platform PC \
  --jobs 8
```

#### 分析资源质量

```bash
# 分析资源
game-engine analyze ./assets

# 生成报告
game-engine analyze ./assets -o quality_report.html
```

#### 打包资源

```bash
# 创建Pak包
game-engine bundle ./assets_optimized -o game.pak

# 创建虚拟文件系统
game-engine bundle ./assets_optimized -o game.vfs --format virtual
```

### 质量预设

- **Low**: 低质量，适合性能受限的平台
  - LOD级别: [1.0, 0.3]
  - 纹理压缩: BC1
  - 最大分辨率: 1024

- **Medium**: 中等质量
  - LOD级别: [1.0, 0.5, 0.25]
  - 纹理压缩: BC3
  - 最大分辨率: 2048

- **High**: 高质量（默认）
  - LOD级别: [1.0, 0.75, 0.5, 0.25]
  - 纹理压缩: BC7
  - 最大分辨率: 2048

- **Ultra**: 超高质量
  - LOD级别: [1.0, 0.875, 0.75, 0.5, 0.25, 0.125]
  - 纹理压缩: BC7
  - 最大分辨率: 4096

### 目标平台

- **PC**: Windows, macOS, Linux
  - 推荐格式: BC7
  - 支持: BC1, BC3, BC7

- **Mobile**: iOS, Android
  - 推荐格式: ASTC 4x4
  - 备选: ETC2
  - 支持: ASTC4x4, ETC2

- **Web**: WebGL, WebGPU
  - 推荐格式: BC7
  - 支持: BC1, BC3

- **Console**: PS5, Xbox Series X
  - 推荐格式: BC7
  - 支持: BC1, BC3, BC7

## 编程API

### 基础用法

```rust
use game_engine::tools::asset_pipeline::{
    AssetPipeline, PipelineConfig, Platform, QualityPreset
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建配置
    let config = PipelineConfig {
        auto_lod: true,
        lod_levels: vec![1.0, 0.5, 0.25],
        auto_compress: true,
        auto_optimize_shaders: true,
        target_platform: Platform::PC,
        quality_preset: QualityPreset::High,
        ..Default::default()
    };

    // 创建管线
    let pipeline = AssetPipeline::new(config);

    // 运行优化
    let report = pipeline.optimize_assets(
        std::path::Path::new("./assets"),
        std::path::Path::new("./assets_optimized")
    ).await?;

    // 打印报告
    report.print_summary();

    Ok(())
}
```

### 使用质量预设

```rust
let pipeline = AssetPipeline::with_quality_preset(
    QualityPreset::High,
    Platform::PC
);
```

### 自定义纹理压缩

```rust
use game_engine::tools::asset_pipeline::{
    TextureOptimizerOptions, CompressionFormat
};

let options = TextureOptimizerOptions {
    compression_format: CompressionFormat::BC7,
    generate_mipmaps: true,
    max_mip_levels: Some(10),
    quality: 90,
    preserve_size: false,
    max_resolution: Some((2048, 2048)),
    srgb: true,
};
```

### 质量分析

```rust
use game_engine::tools::asset_pipeline::{QualityAnalyzer, QualityTargets};

let analyzer = QualityAnalyzer::with_targets(QualityTargets {
    max_texture_resolution: 2048,
    max_polygons_per_model: 100_000,
    max_draw_calls: 100,
    max_memory_mb: 500.0,
    max_load_time: 3.0,
});

let report = analyzer.analyze(&asset);
println!("Status: {}", report.overall_status().status_name());
```

## 输出结构

优化后的资源目录结构：

```
assets_optimized/
├── models/
│   ├── character.gltf/
│   │   ├── lod0.gltf      # 原始模型
│   │   ├── lod1.gltf      # 50% 多边形
│   │   └── lod2.gltf      # 25% 多边形
│   └── ...
├── textures/
│   ├── diffuse.png        # 压缩纹理
│   ├── normal.png
│   └── ...
├── shaders/
│   ├── vertex.wgsl        # 优化后的着色器
│   └── fragment.wgsl
└── quality_report.html    # 质量分析报告
```

## 性能优化建议

1. **并行处理**: 使用`--jobs`参数增加并发任务数
2. **增量优化**: 只优化修改过的资源
3. **批量处理**: 使用脚本批量优化多个目录
4. **质量平衡**: 根据目标平台选择合适的质量预设

## 故障排除

### 编译错误

如果遇到编译错误，确保启用了所需功能：

```bash
cargo build --features "cli,gltf"
```

### 运行时错误

1. **缺少输入目录**: 确保输入目录存在
2. **权限错误**: 确保有读写权限
3. **内存不足**: 减少并发任务数

## 限制和注意事项

1. **纹理压缩**: 当前实现为简化版本，实际项目应使用专用压缩库
2. **LOD生成**: 使用简化的网格简化算法，生产环境建议使用专业工具
3. **着色器优化**: 基础优化，某些复杂着色器可能需要手动优化

## 贡献

欢迎提交Issue和Pull Request！

## 许可证

MIT License
