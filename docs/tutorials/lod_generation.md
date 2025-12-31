# LOD生成系统使用指南

**版本**: v0.1.0
**更新日期**: 2025-12-31
**状态**: ✅ P0阶段完成

---

## 概述

LOD（Level of Detail）生成系统为游戏引擎提供自动化的多细节级别网格生成能力，显著减少95%的手动LOD创建工作。

---

## 核心组件

### 1. MeshSimplifier - 网格简化器

基于**Quadric Error Metrics (QEM)**算法实现，提供高质量的网格简化。

**位置**: `game_engine/src/render/mesh_simplifier.rs`

**基本用法**:
```rust
use game_engine::render::mesh_simplifier::{MeshSimplifier, SimplifyOptions};

let mesh = Mesh::from_vertices_and_indices(vertices, indices)?;
let simplifier = MeshSimplifier::new(mesh)?;

let options = SimplifyOptions {
    target_ratio: 0.5,  // 简化到50%
    preserve_boundaries: true,
    ..Default::default()
};

let simplified = simplifier.simplify(&options)?;
```

**性能特性**:
- 时间复杂度: O(n + m log m)，n为顶点数，m为边数
- 空间复杂度: ~80n + 48m 字节
- 10,000三角形网格: < 50ms
- 100,000三角形网格: < 500ms

---

### 2. LODGenerator - LOD生成器

自动从单个高质量网格生成多个LOD级别。

**位置**: `game_engine/src/render/lod_generator.rs`

**基本用法**:
```rust
use game_engine::render::lod_generator::{LODGenerator, LODConfig};

let config = LODConfig::with_levels(vec![1.0, 0.5, 0.25, 0.125]);
let generator = LODGenerator::with_config(config)?;

let lods = generator.generate_from_mesh(&high_quality_mesh)?;

// 访问不同LOD级别
let lod0 = lods.get_level(0)?;  // 100% 质量
let lod1 = lods.get_level(1)?;  // 50% 质量

// 根据屏幕尺寸选择最佳LOD
let best_lod = lods.select_level(0.5); // screen_size = 0.5
```

**LOD级别**:
- LOD0: 100% 三角形（全质量）
- LOD1: 50% 三角形
- LOD2: 25% 三角形
- LOD3: 12.5% 三角形
- LOD4: 6.25% 三角形（可选）

---

### 3. QualityAssessor - 质量评估器

自动评估网格复杂度并推荐最优LOD配置。

**位置**: `game_engine/src/render/quality_assessor.rs`

**基本用法**:
```rust
use game_engine::render::quality_assessor::{QualityAssessor, QualityConfig};

let config = QualityConfig {
    target_quality: TargetQuality::High,
    platform: PlatformConstraints::Desktop,
    performance: PerformanceRequirements::Medium,
};

let assessor = QualityAssessor::with_config(config);
let assessment = assessor.assess_lods(&lods);

println!("推荐LOD级别: {}", assessment.recommended_lod);
println!("质量评分: {:.2}", assessment.overall_score);
```

**复杂度分类**:
- VeryLow: < 100 三角形
- Low: 100 - 1,000 三角形
- Medium: 1,000 - 10,000 三角形
- High: 10,000 - 100,000 三角形
- VeryHigh: > 100,000 三角形

---

### 4. LODResourceManager - 资源管线集成

将LOD生成集成到资源加载管线，自动缓存和管理LOD。

**位置**: `game_engine/src/resources/lod_resource.rs`

**基本用法**:
```rust
use game_engine::resources::lod_resource::{LODResourceManager, load_mesh_with_lods};

// 创建管理器
let manager = LODResourceManager::with_defaults()?;

// 加载网格并自动生成LOD
let lod_resource = load_mesh_with_lods(
    &manager,
    "player_mesh".to_string(),
    mesh
).await?;

// 使用LOD
let lod_index = lod_resource.select_lod(0.3); // screen_size = 0.3
```

---

## 快速开始示例

### 示例1: 基础LOD生成

```rust
use game_engine::render::{
    mesh_simplifier::Mesh,
    lod_generator::generate_lods,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 加载或创建网格
    let mesh = load_mesh_from_file("model.obj")?;

    // 2. 自动生成LOD（使用默认4级）
    let lods = generate_lods(&mesh)?;

    println!("生成了 {} 个LOD级别", lods.level_count());
    for level in &lods.levels {
        println!(
            "LOD{}: {} 三角形, 内存: {} KB",
            level.index,
            level.triangle_count,
            level.memory_usage() / 1024
        );
    }

    Ok(())
}
```

---

### 示例2: 自定义LOD配置

```rust
use game_engine::render::lod_generator::{LODConfig, LODGenerator};

fn custom_lods() -> Result<(), Box<dyn std::error::Error>> {
    let mesh = load_mesh_from_file("building.fbx")?;

    // 自定义LOD级别
    let config = LODConfig {
        levels: vec![1.0, 0.75, 0.5, 0.25, 0.1], // 5个级别
        min_triangles: 50,
        preserve_boundaries: true,
        preserve_uv_seams: true,
        max_error: 0.5,
        ..Default::default()
    };

    let generator = LODGenerator::with_config(config)?;
    let lods = generator.generate_from_mesh(&mesh)?;

    Ok(())
}
```

---

### 示例3: 平台自适应LOD

```rust
use game_engine::render::quality_assessor::{
    QualityAssessor, QualityConfig, TargetQuality, PlatformConstraints
};

fn platform_adaptive_lods() -> Result<(), Box<dyn std::error::Error>> {
    let mesh = load_mesh_from_file("character.gltf")?;

    // 移动平台配置
    let config = QualityConfig {
        target_quality: TargetQuality::Medium,
        platform: PlatformConstraints::Mobile,
        ..Default::default()
    };

    let assessor = QualityAssessor::with_config(config);

    // 获取最优配置
    let (ratios, max_error, min_triangles) =
        assessor.generate_optimal_config(mesh.triangle_count());

    println!("推荐LOD级别: {:?}", ratios);
    println!("最大误差: {}", max_error);
    println!("最小三角形数: {}", min_triangles);

    Ok(())
}
```

---

## 集成到资源管线

### 编辑器集成

在编辑器中导入模型时自动生成LOD：

```rust
use game_engine::resources::lod_resource::LODResourceManager;

async fn import_model_with_lods(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let manager = LODResourceManager::with_defaults()?;
    manager.set_auto_generate(true);

    // 加载原始网格
    let mesh = load_gltf_mesh(path).await?;

    // 自动生成LOD
    let lod_resource = manager.generate_lods(
        path.file_stem().unwrap().to_string_lossy().to_string(),
        &mesh
    ).await?;

    println!("LOD生成完成，质量评分: {:.2}", lod_resource.quality_score);

    Ok(())
}
```

---

## 性能优化建议

### 1. 批量处理

对于大量网格，使用并行处理：

```rust
use rayon::prelude::*;

fn batch_generate_lods(meshes: Vec<Mesh>) -> Vec<Result<LODGroup, SimplificationError>> {
    meshes.par_iter()
        .map(|mesh| generate_lods(mesh))
        .collect()
}
```

### 2. 缓存策略

- 启用LOD缓存（默认1GB）
- 定期清理过期LOD（默认30天）
- 按需重新生成

### 3. 质量预设

针对不同平台使用预设：

```rust
// PC高端
TargetQuality::Ultra + PlatformConstraints::Desktop

// 移动端
TargetQuality::Medium + PlatformConstraints::Mobile

// Web端
TargetQuality::Low + PlatformConstraints::Web
```

---

## 故障排查

### 问题1: 生成时间过长

**原因**: 网格过于复杂（>100K三角形）
**解决**:
- 降低LOD级别数量
- 增加`min_triangles`限制
- 使用更激进的简化比例

### 问题2: 视觉质量差

**原因**: 简化过度
**解决**:
- 降低`max_error`阈值
- 启用`preserve_boundaries`
- 启用`preserve_uv_seams`

### 问题3: 内存占用高

**原因**: 缓存过多LOD
**解决**:
- 减少`max_cache_size`
- 清理LOD缓存
- 降低LOD级别数量

---

## 最佳实践

### ✅ 推荐做法

1. **使用自动质量评估**: 让QualityAssessor推荐最优配置
2. **启用边界保护**: `preserve_boundaries = true`保持网格边缘
3. **合理设置最小三角形数**: 避免过度简化
4. **使用平台预设**: 针对不同目标平台优化
5. **定期清理缓存**: 避免内存占用过高

### ❌ 避免做法

1. **过度简化**: 不要将复杂网格简化到<100三角形
2. **禁用边界保护**: 除非有特殊需求
3. **忽略质量评估**: 盲目使用默认配置
4. **缓存所有LOD**: 高频访问才需要缓存

---

## API参考

详细的API文档请参见：
- `game_engine::render::mesh_simplifier`
- `game_engine::render::lod_generator`
- `game_engine::render::quality_assessor`
- `game_engine::resources::lod_resource`

---

## 相关资源

- **研究论文**: Garland & Heckbert, "Surface Simplification Using Quadric Error Metrics", 1997
- **实现报告**: `docs/research/MESH_SIMPLIFICATION_RESEARCH.md`
- **实施计划**: `IMPLEMENTATION_PLAN.md`

---

**文档维护**: 随P0阶段更新
**下一步**: P0-2 资源压缩管线自动化
