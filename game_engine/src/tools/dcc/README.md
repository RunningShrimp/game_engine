# DCC工具集成 (P2-4)

## 概述

本模块提供了完整的DCC（Digital Content Creation）工具集成，包括网格编辑、材质调整、动画关键帧和UV编辑等功能。

## 目录结构

```
src/tools/dcc/
├── mod.rs              # 模块定义和主要导出
├── mesh_editor.rs      # 网格编辑器
├── material_editor.rs  # 材质编辑器
├── animation_editor.rs # 动画编辑器
├── uv_editor.rs        # UV编辑器
├── integrator.rs       # 脚本生成器
└── tests.rs            # 集成测试
```

## 主要功能

### 1. 网格编辑器 (MeshEditor)

提供顶点、边、面的编辑功能：

```rust
use game_engine::tools::dcc::{MeshEditor, EditMode};

// 创建编辑器
let mut editor = MeshEditor::new();

// 加载网格
editor.load_mesh(vertices, indices);

// 设置编辑模式
editor.edit_mode = EditMode::Vertex;

// 选择顶点
editor.selected_vertices.insert(0);

// 应用变换
let transform = Mat4::IDENTITY;
editor.apply_transform(transform);
```

**功能特性：**
- 顶点选择模式（单选、框选、涂刷、环形）
- 变换工具（平移、旋转、缩放）
- 网格操作（挤出、倒角、焊接）
- 对称编辑
- 软选择

### 2. 材质编辑器 (DCCMaterialEditor)

提供PBR材质的实时编辑：

```rust
use game_engine::tools::dcc::{DCCMaterialEditor, MaterialPreset};

// 创建编辑器
let mut editor = DCCMaterialEditor::new();

// 添加材质
let mat_id = editor.add_material("Metal".to_string());

// 应用预设
editor.apply_preset(mat_id, MaterialPreset::StandardMetal);

// 调整参数
if let Some(material) = editor.get_material_mut(mat_id) {
    material.metallic = 1.0;
    material.roughness = 0.2;
}
```

**功能特性：**
- PBR参数调整（基础颜色、金属度、粗糙度、AO等）
- 纹理槽管理（Albedo、Normal、Roughness等）
- 材质预设（金属、非金属、玻璃、橡胶等）
- 实时预览
- 纹理变换（缩放、偏移、旋转）

### 3. 动画编辑器 (DCCAnimationEditor)

提供关键帧动画编辑：

```rust
use game_engine::tools::dcc::{DCCAnimationEditor, AnimatedValue};

// 创建编辑器
let mut editor = DCCAnimationEditor::new();

// 添加动画
let anim_id = editor.add_animation("Walk".to_string(), 2.0);

// 添加关键帧
editor.add_keyframe(
    anim_id,
    "position.x".to_string(),
    0.0,
    AnimatedValue::Float(0.0)
);

// 控制播放
editor.timeline.playback_state = PlaybackState::Playing;
editor.update(delta_time);
```

**功能特性：**
- 时间轴管理（缩放、滚动、播放头）
- 关键帧编辑（添加、删除、移动）
- 动画曲线（线性、阶梯、三次样条、贝塞尔）
- 播放控制（播放、暂停、停止、循环）
- 曲线可视化

### 4. UV编辑器 (UVEditor)

提供UV坐标编辑：

```rust
use game_engine::tools::dcc::UVEditor;

// 创建编辑器
let mut editor = UVEditor::new();

// 加载UV数据
editor.load_uvs(uvs, triangles);

// 选择UV
editor.selected_uvs.insert(0);

// 应用变换
editor.apply_transform();
```

**功能特性：**
- UV岛显示和选择
- UV变换（移动、旋转、缩放）
- 网格吸附
- 棋盘格背景
- UV边界显示

### 5. 脚本生成器 (ScriptGenerator)

自动生成多种语言的脚本：

```rust
use game_engine::tools::dcc::{ScriptGenerator, ScriptLanguage, EditorOperation};
use glam::Mat4;

// 创建生成器
let mut generator = ScriptGenerator::new();

// 添加操作
generator.add_operation(EditorOperation::VertexTransform {
    vertices: vec![0, 1, 2],
    transform: Mat4::IDENTITY,
});

// 生成Lua脚本
let lua_script = generator.generate_script(ScriptLanguage::Lua);

// 生成Python脚本
let python_script = generator.generate_script(ScriptLanguage::Python);

// 生成Rust代码
let rust_code = generator.generate_script(ScriptLanguage::Rust);
```

**支持的脚本语言：**
- Lua
- Python
- Rust

**生成的脚本特性：**
- 自动生成注释
- 错误处理
- 代码格式化
- 头信息

## 集成使用

### 创建DCC工具套件

```rust
use game_engine::tools::dcc::DCCToolkit;

// 创建完整工具套件
let mut toolkit = DCCToolkit::new();

// 显示UI（需要egui上下文）
toolkit.show_ui(&egui_context);

// 导出脚本
let script = toolkit.export_script(ScriptLanguage::Lua);
```

## 数据结构

### EditMode - 编辑模式

```rust
pub enum EditMode {
    Vertex,  // 顶点编辑
    Edge,    // 边编辑
    Face,    // 面编辑
    UV,      // UV编辑
}
```

### TransformTool - 变换工具

```rust
pub enum TransformTool {
    Translate,  // 平移
    Rotate,     // 旋转
    Scale,      // 缩放
}
```

### PlaybackState - 播放状态

```rust
pub enum PlaybackState {
    Stopped,  // 停止
    Playing,  // 播放中
    Paused,   // 暂停
}
```

## 测试

运行DCC工具测试：

```bash
cargo test --package game_engine --lib tools::dcc
```

## 示例

### 完整工作流程示例

```rust
use game_engine::tools::dcc::*;
use game_engine::render::mesh::Vertex3D;
use glam::*;

fn main() {
    // 1. 创建DCC工具套件
    let mut toolkit = DCCToolkit::new();

    // 2. 加载网格进行编辑
    let vertices = vec![
        Vertex3D {
            pos: [0.0, 0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
            uv: [0.0, 0.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
        },
        // ... 更多顶点
    ];
    let indices = vec![0, 1, 2];

    toolkit.mesh_editor.load_mesh(vertices, indices);

    // 3. 选择并变换顶点
    toolkit.mesh_editor.selected_vertices.insert(0);
    toolkit.mesh_editor.selected_vertices.insert(1);

    let transform = Mat4::from_translation(Vec3::new(1.0, 0.0, 0.0));
    toolkit.mesh_editor.apply_transform(transform);

    // 4. 创建材质
    let mat_id = toolkit.material_editor.add_material("Metal".to_string());
    toolkit.material_editor.apply_preset(mat_id, MaterialPreset::StandardMetal);

    // 5. 创建动画
    let anim_id = toolkit.animation_editor.add_animation("Walk".to_string(), 2.0);
    toolkit.animation_editor.add_keyframe(
        anim_id,
        "position.x".to_string(),
        0.0,
        AnimatedValue::Float(0.0),
    );

    // 6. 生成脚本
    let lua_script = toolkit.export_script(ScriptLanguage::Lua);
    println!("{}", lua_script.code);
}
```

## 限制和TODO

当前实现为基础框架，以下功能待完善：

1. **网格编辑器**
   - [ ] 完整的拓扑操作实现
   - [ ] 边环和面环检测
   - [ ] 布尔运算
   - [ ] 网格优化算法

2. **材质编辑器**
   - [ ] 实时预览渲染
   - [ ] 纹理导入和管理
   - [ ] 材质图层
   - [ ] 节点式材质编辑

3. **动画编辑器**
   - [ ] 曲线编辑器UI完善
   - [ ] 动画混合
   - [ ] 骨骼动画支持
   - [ ] 动画压缩

4. **UV编辑器**
   - [ ] UV展开算法（LSCM、ABF等）
   - [ ] UV松弛
   - [ ] UV岛打包
   - [ ] 纹理烘焙

5. **脚本生成器**
   - [ ] 更完整的API绑定
   - [ ] 脚本执行引擎
   - [ ] 宏录制
   - [ ] 批处理操作

## 依赖

- `egui` - UI框架
- `glam` - 数学库
- `serde` - 序列化（可选）

## 许可证

MIT OR Apache-2.0
