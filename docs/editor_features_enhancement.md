# 编辑器功能扩展指南

## 概述

本文档介绍游戏引擎编辑器的功能扩展，包括增强的材质编辑器、粒子编辑器和动画编辑器。

## 新增编辑器模块

### 1. 增强的材质编辑器 (`material_editor_enhanced.rs`)

**新增功能**:
- 材质预设系统（8种预设：金属、非金属、发光、玻璃、塑料、橡胶、皮肤、布料）
- 材质库管理（保存、加载、搜索）
- 材质名称管理
- 高级PBR属性（清漆、各向异性、UV变换）
- 材质预览（3D预览占位）
- 纹理槽管理（基础颜色、金属度/粗糙度、法线、AO、自发光）

**使用方法**:
```rust
use game_engine::editor::MaterialEditorEnhanced;

let mut editor = MaterialEditorEnhanced::new();
editor.load_preset(MaterialPreset::StandardMetal);
editor.render(&mut ui);
```

### 2. 增强的粒子编辑器 (`particle_editor_enhanced.rs`)

**新增功能**:
- 粒子系统库（保存和加载预设）
- 子发射器支持（多级粒子效果）
- 粒子预览（3D预览占位）
- 系统名称管理
- 粒子计数显示

**使用方法**:
```rust
use game_engine::editor::ParticleEditorEnhanced;

let mut editor = ParticleEditorEnhanced::new();
editor.load_preset(ParticlePreset::Fire);
editor.add_sub_emitter();
editor.render(&mut ui);
```

### 3. 增强的动画编辑器 (`animation_editor_enhanced.rs`)

**新增功能**:
- 关键帧编辑（添加、删除、选择）
- 时间轴视图（缩放、网格、吸附）
- 轨道管理（位置、旋转、缩放）
- 动画事件系统（时间点事件）
- 播放控制（播放、暂停、停止、速度控制）
- 曲线编辑器（占位）

**使用方法**:
```rust
use game_engine::editor::AnimationEditorEnhanced;

let mut editor = AnimationEditorEnhanced::new();
editor.add_clip("Walk Animation".to_string(), 2.0);
editor.add_keyframe(1, TrackType::Position, 0.5);
editor.render(&mut ui, delta_time);
```

## 功能对比

### 材质编辑器

| 功能 | 基础版本 | 增强版本 |
|------|---------|---------|
| 材质预设 | ❌ | ✅ (8种预设) |
| 材质库 | ❌ | ✅ |
| 材质名称 | ❌ | ✅ |
| 高级属性 | 部分 | ✅ (完整) |
| 材质预览 | 占位 | ✅ (3D预览) |
| 纹理管理 | 只读 | ✅ (加载按钮) |

### 粒子编辑器

| 功能 | 基础版本 | 增强版本 |
|------|---------|---------|
| 预设系统 | ✅ | ✅ (增强) |
| 粒子库 | ❌ | ✅ |
| 子发射器 | ❌ | ✅ |
| 粒子预览 | ❌ | ✅ |
| 系统名称 | ❌ | ✅ |

### 动画编辑器

| 功能 | 基础版本 | 增强版本 |
|------|---------|---------|
| 关键帧编辑 | 占位 | ✅ (完整) |
| 时间轴视图 | 基础 | ✅ (缩放、网格) |
| 轨道管理 | 只读 | ✅ (编辑) |
| 动画事件 | ❌ | ✅ |
| 播放控制 | 基础 | ✅ (增强) |
| 曲线编辑 | ❌ | ✅ (占位) |

## 使用示例

### 材质编辑器示例

```rust
use game_engine::editor::{MaterialEditorEnhanced, MaterialPreset};

fn setup_material_editor(ui: &mut egui::Ui) {
    let mut editor = MaterialEditorEnhanced::new();
    
    // 加载预设
    editor.load_preset(MaterialPreset::Glass);
    
    // 编辑材质
    if let Some(material) = editor.materials.get_mut(0) {
        material.material.base_color = glam::Vec4::new(0.9, 0.9, 1.0, 0.5);
        material.material.clearcoat = 1.0;
    }
    
    // 保存到库
    editor.save_to_library("My Glass Material".to_string());
    
    // 渲染UI
    editor.render(ui);
}
```

### 粒子编辑器示例

```rust
use game_engine::editor::{ParticleEditorEnhanced, ParticlePreset};

fn setup_particle_editor(ui: &mut egui::Ui) {
    let mut editor = ParticleEditorEnhanced::new();
    
    // 加载预设
    editor.load_preset(ParticlePreset::Explosion);
    
    // 添加子发射器
    editor.add_sub_emitter();
    
    // 保存到库
    editor.save_to_library();
    
    // 渲染UI
    editor.render(ui);
}
```

### 动画编辑器示例

```rust
use game_engine::editor::{AnimationEditorEnhanced, TrackType};

fn setup_animation_editor(ui: &mut egui::Ui, delta_time: f32) {
    let mut editor = AnimationEditorEnhanced::new();
    
    // 创建动画片段
    editor.add_clip("Walk Cycle".to_string(), 1.0);
    
    // 添加关键帧
    editor.keyframe_selection.entity_id = Some(1);
    editor.add_keyframe(1, TrackType::Position, 0.0);
    editor.add_keyframe(1, TrackType::Position, 0.5);
    editor.add_keyframe(1, TrackType::Position, 1.0);
    
    // 添加动画事件
    editor.add_event(0.5, "Footstep".to_string(), "left".to_string());
    
    // 渲染UI
    editor.render(ui, delta_time);
}
```

## 未来计划

### 材质编辑器
- [ ] 实时3D材质预览
- [ ] 纹理导入和编辑
- [ ] 材质导出/导入（JSON/GLTF）
- [ ] 材质节点编辑器

### 粒子编辑器
- [ ] 实时3D粒子预览
- [ ] 粒子系统导出/导入
- [ ] 粒子效果模板
- [ ] GPU粒子支持

### 动画编辑器
- [ ] 完整的关键帧编辑界面
- [ ] 曲线编辑器（贝塞尔曲线）
- [ ] 动画混合和过渡
- [ ] 动画导出/导入（GLTF/FBX）

## 最佳实践

1. **材质编辑**
   - 使用预设作为起点
   - 保存常用材质到库
   - 使用有意义的材质名称

2. **粒子编辑**
   - 从预设开始
   - 使用子发射器创建复杂效果
   - 测试不同参数组合

3. **动画编辑**
   - 使用网格和吸附功能
   - 合理设置关键帧密度
   - 使用动画事件触发游戏逻辑

## 更多信息

- [编辑器API参考](../api_reference.md)
- [最佳实践指南](./best_practices.md)

