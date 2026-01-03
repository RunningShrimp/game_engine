# DCC工具手册

数字内容创作(DCC)工具集成完整指南。

## 目录

- [工具概述](#工具概述)
- [网格编辑器](#网格编辑器)
- [UV编辑器](#uv编辑器)
- [材质编辑器](#材质编辑器)
- [动画编辑器](#动画编辑器)
- [Blender桥接](#blender桥接)

## 工具概述

DCC工具模块提供了一套完整的数字内容创作工具集成:

- ✅ **网格编辑器** - 完整的网格操作和优化
- ✅ **UV编辑器** - UV映射和自动展开
- ✅ **材质编辑器** - PBR材质工作流
- ✅ **动画编辑器** - 关键帧和骨骼动画
- ✅ **Blender桥接** - 无缝集成Blender工作流

所有工具均集成在统一编辑器界面中,使用egui提供跨平台GUI。

## 网格编辑器

### 功能特性

- ✅ 网格导入/导出 (FBX, OBJ, glTF)
- ✅ 顶点/边/面编辑
- ✅ 网格优化算法
- ✅ UV展开工具
- ✅ 法线计算
- ✅ 网格简化

### 基本使用

```rust
use game_engine::tools::dcc::mesh_editor::{DCCMeshEditor, MeshEditMode};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建网格编辑器
    let mut editor = DCCMeshEditor::new();

    // 导入网格
    editor.import_mesh("models/character.fbx").await?;

    // 切换到顶点编辑模式
    editor.set_edit_mode(MeshEditMode::Vertex);

    // 选择顶点
    editor.select_vertices(vec![0, 1, 2]);

    // 变换操作
    editor.translate_vertices([1.0, 0.0, 0.0]);
    editor.rotate_vertices(45.0);
    editor.scale_vertices(1.5);

    // 网格优化
    editor.optimize_mesh();

    // 导出网格
    editor.export_mesh("models/character_optimized.fbx").await?;

    Ok(())
}
```

### 网格操作API

#### 顶点编辑

```rust
// 添加顶点
editor.add_vertex(position: Vec3, uv: Vec2, normal: Vec3);

// 删除顶点
editor.remove_vertices(vertex_indices: Vec<usize>);

// 选择顶点
editor.select_vertices(indices: Vec<usize>);

// 取消选择
editor.deselect_all();

// 移动顶点
editor.translate_vertices(offset: Vec3);

// 旋转顶点
editor.rotate_vertices(angle_degrees: f32);

// 缩放顶点
editor.scale_vertices(factor: f32);
```

#### 边编辑

```rust
// 添加边
editor.add_edge(v0: usize, v1: usize);

// 删除边
editor.remove_edges(edge_indices: Vec<usize>);

// 桥接边
editor.bridge_edges(edge0: usize, edge1: usize);

// 切割边
editor.split_edge(edge_index: usize, t: f32);
```

#### 面编辑

```rust
// 添加面
editor.add_face(vertex_indices: Vec<usize>);

// 删除面
editor.remove_faces(face_indices: Vec<usize>);

// 挤出面
editor.extrude_faces(face_indices: Vec<usize>, distance: f32);

// 倒角面
editor.bevel_faces(face_indices: Vec<usize>, amount: f32);
```

#### 网格优化

```rust
// 网格简化
editor.simplify_mesh(target_face_count: usize);

// 移除重复顶点
editor.weld_vertices(tolerance: f32);

// 三角化
editor.triangulate();

// 四边形化
editor.quadrangulate();

// 重新计算法线
editor.recalculate_normals(angle_threshold: f32);

// 重新计算切线
editor.recalculate_tangents();
```

### 网格导入/导出

```rust
// 支持的格式
enum MeshFormat {
    FBX,      // Autodesk FBX
    OBJ,      // Wavefront OBJ
    GLTF,     // glTF 2.0
    GLB,      // glTF Binary
}

// 导入
editor.import_mesh("path/to/model.fbx").await?;

// 导出
editor.export_mesh("output/model.glb").await?;

// 设置导出选项
let options = ExportOptions {
    format: MeshFormat::GLTF,
    include_normals: true,
    include_tangents: true,
    include_uvs: true,
    apply_transforms: true,
};
editor.set_export_options(options);
```

## UV编辑器

### 功能特性

- ✅ UV映射算法
- ✅ 自动UV展开
- ✅ UV打包优化
- ✅ 多通道UV支持
- ✅ UV编辑工具集

### 基本使用

```rust
use game_engine::tools::dcc::uv_editor::{UVEditor, UVChannel};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut editor = UVEditor::new();

    // 加载网格
    editor.load_mesh("models/character.fbx").await?;

    // 自动展开UV
    editor.auto_unwrap(UVChannel::UV0)?;

    // 优化UV布局
    editor.optimize_uv_layout();

    // 打包UV岛
    editor.pack_uv_islands();

    // 导出UV布局图
    editor.export_uv_layout("textures/character_uv_layout.png").await?;

    Ok(())
}
```

### UV映射算法

#### 自动UV展开

```rust
// 智能UV展开
editor.smart_unwrap(
    uv_channel: UVChannel,
    island_margin: 0.02,
    angle_threshold: 45.0,
)?;

// 投影UV映射
editor.project_uv(
    projection_type: ProjectionType::Spherical,
    uv_channel: UVChannel,
)?;

// 棋盘格投影
editor.checker_project_uv(
    direction: Vec3,
    uv_channel: UVChannel,
)?;
```

#### UV优化

```rust
// 最小化拉伸
editor.minimize_stretch(uv_channel: UVChannel)?;

// 最小化重叠
editor.minimize_overlap(uv_channel: UVChannel)?;

// 最大化利用率
editor.maximize_utilization(uv_channel: UVChannel)?;

// 平衡优化
editor.balance_optimization(
    uv_channel: UVChannel,
    stretch_weight: 0.5,
    overlap_weight: 0.3,
    utilization_weight: 0.2,
)?;
```

### UV编辑工具

```rust
// 选择UV岛
editor.select_uv_island(uv_coord: Vec2);

// 移动UV
editor.move_uv(uv_coords: Vec<Vec2>, offset: Vec2);

// 旋转UV
editor.rotate_uv(uv_coords: Vec<Vec2>, angle_degrees: f32);

// 缩放UV
editor.scale_uv(uv_coords: Vec<Vec2>, scale: f32);

// 镜像UV
editor.mirror_uv(uv_coords: Vec<Vec2>, axis: MirrorAxis);

// 焊接UV
editor.weld_uv(uv0: Vec2, uv1: Vec2, tolerance: f32);

// 分割UV
editor.split_uv(edge_indices: Vec<usize>);
```

### 多通道UV

```rust
// 获取UV通道数
let channel_count = editor.get_uv_channel_count();

// 添加UV通道
editor.add_uv_channel();

// 设置UV通道
editor.set_uv_channel(channel_index: usize);

// 复制UV通道
editor.copy_uv_channel(src: usize, dst: usize);

// 删除UV通道
editor.remove_uv_channel(channel_index: usize);
```

## 材质编辑器

### 功能特性

- ✅ PBR材质工作流
- ✅ 纹理烘焙
- ✅ 材质预览
- ✅ 着色器编辑
- ✅ 材质库管理

### 基本使用

```rust
use game_engine::tools::dcc::material_editor::{MaterialEditor, PBRMaterial};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut editor = MaterialEditor::new();

    // 创建PBR材质
    let mut material = PBRMaterial::new("PlayerMaterial");

    // 设置基础属性
    material.set_albedo(Color::rgb(1.0, 0.8, 0.6));
    material.set_metallic(0.0);
    material.set_roughness(0.5);
    material.set_normal_scale(1.0);

    // 添加纹理
    material.set_albedo_map("textures/player_albedo.png");
    material.set_metallic_map("textures/player_metallic.png");
    material.set_roughness_map("textures/player_roughness.png");
    material.set_normal_map("textures/player_normal.png");

    // 编辑材质
    editor.edit_material(material);

    // 烘焙材质
    editor.bake_material(
        output_path: "textures/player_baked.png",
        resolution: (2048, 2048),
    ).await?;

    // 预览材质
    editor.set_preview_mesh("models/sphere.fbx");
    editor.toggle_preview(true);

    Ok(())
}
```

### PBR材质属性

```rust
// 反照率
material.set_albedo(color: Color);
material.set_albedo_map(texture_path: &str);

// 金属度
material.set_metallic(value: f32);  // 0.0 - 1.0
material.set_metallic_map(texture_path: &str);

// 粗糙度
material.set_roughness(value: f32);  // 0.0 - 1.0
material.set_roughness_map(texture_path: &str);

// 法线
material.set_normal_map(texture_path: &str);
material.set_normal_scale(scale: f32);

// 自发光
material.set_emissive(color: Color);
material.set_emissive_map(texture_path: &str);
material.set_emissive_intensity(intensity: f32);

// 透明度
material.set_opacity(value: f32);
material.set_opacity_mode(mode: OpacityMode);

// 环境光遮蔽
material.set_ao_map(texture_path: &str);
material.set_ao_strength(strength: f32);
```

### 纹理烘焙

```rust
// 烘焙设置
let bake_settings = BakeSettings {
    resolution: (2048, 2048),
    samples: 1024,
    bounces: 3,
    ao_only: false,
    include_lighting: true,
    format: TextureFormat::PNG,
};

// 烘焙材质
editor.bake_material_with_settings(
    output_path: "baked/output.png",
    settings: &bake_settings,
).await?;

// 烘焙特定通道
editor.bake_albedo_channel("baked/albedo.png").await?;
editor.bake_normal_channel("baked/normal.png").await?;
editor.bake_roughness_channel("baked/roughness.png").await?;
```

### 着色器编辑

```rust
// 创建自定义着色器
let mut shader = Shader::new("CustomShader");

// 添加着色器阶段
shader.add_vertex_stage(code: r#"
    #version 450
    layout(location = 0) in vec3 position;
    layout(location = 1) in vec2 uv;

    out vec2 v_uv;

    void main() {
        v_uv = uv;
        gl_Position = vec4(position, 1.0);
    }
"#);

shader.add_fragment_stage(code: r#"
    #version 450
    in vec2 v_uv;
    out vec4 frag_color;

    uniform sampler2D albedo_map;

    void main() {
        vec4 albedo = texture(albedo_map, v_uv);
        frag_color = albedo;
    }
"#);

// 编译着色器
editor.compile_shader(shader)?;

// 预览着色器
editor.set_shader_preview_mesh("models/preview_sphere.fbx");
```

## 动画编辑器

### 功能特性

- ✅ 时间轴管理
- ✅ 关键帧编辑
- ✅ 动画曲线
- ✅ 播放控制
- ✅ 动画压缩

### 基本使用

```rust
use game_engine::tools::dcc::animation_editor::{DCCAnimationEditor, AnimatedValue};

fn main() {
    let mut editor = DCCAnimationEditor::new();

    // 添加新动画
    let anim_id = editor.add_animation("Idle".to_string(), 2.5);

    // 添加关键帧
    editor.add_keyframe(
        anim_id,
        "position.x".to_string(),
        0.0,
        AnimatedValue::Float(0.0),
    );

    editor.add_keyframe(
        anim_id,
        "position.x".to_string(),
        1.25,
        AnimatedValue::Float(1.5),
    );

    // 播放动画
    editor.timeline.playback_state = PlaybackState::Playing;

    // 更新动画
    let delta_time = 0.016; // ~60fps
    editor.update(delta_time);
}
```

### 关键帧编辑

```rust
// 添加关键帧
editor.add_keyframe(
    animation_id: AnimationID,
    curve_name: String,
    time: f32,
    value: AnimatedValue,
);

// 删除关键帧
editor.remove_keyframe(
    animation_id: AnimationID,
    keyframe_id: KeyframeID,
);

// 移动关键帧
editor.move_keyframe(
    animation_id: AnimationID,
    keyframe_id: KeyframeID,
    new_time: f32,
);

// 缩放关键帧
editor.scale_keyframe(
    animation_id: AnimationID,
    keyframe_id: KeyframeID,
    scale_factor: f32,
);
```

### 动画曲线

```rust
// 设置曲线类型
curve.curve_type = CurveType::Linear;
curve.curve_type = CurveType::Step;
curve.curve_type = CurveType::Cubic;
curve.curve_type = CurveType::Bezier;

// 设置切线类型
keyframe.tangent_type = TangentType::Auto;
keyframe.tangent_type = TangentType::Free;
keyframe.tangent_type = TangentType::Linear;
keyframe.tangent_type = TangentType::Constant;

// 设置切线值
keyframe.tangent_in = Vec2::new(-0.3, 0.0);
keyframe.tangent_out = Vec2::new(0.3, 0.0);
```

### 动画压缩

```rust
// 设置压缩目标
let compression_settings = AnimationCompressionSettings {
    target_size_kb: 100,
    tolerance: 0.01,
    remove_trivial_keys: true,
    quantize_rotations: true,
    compression_level: CompressionLevel::Medium,
};

// 压缩动画
editor.compress_animation(
    animation_id: AnimationID,
    settings: &compression_settings,
)?;

// 分析压缩结果
let compression_report = editor.analyze_compression(animation_id)?;
println!("压缩前: {} KB", compression_report.original_size_kb);
println!("压缩后: {} KB", compression_report.compressed_size_kb);
println!("压缩率: {:.1}%", compression_report.compression_ratio * 100.0);
```

## Blender桥接

### 功能特性

- ✅ Blender集成
- ✅ Python脚本执行
- ✅ 数据导入/导出
- ✅ 批处理操作

### 基本使用

```rust
use game_engine::tools::dcc::blender_bridge::BlenderBridge;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bridge = BlenderBridge::new();

    // 执行Blender Python脚本
    let result = bridge.execute_python(r#"
        import bpy

        # 创建立方体
        bpy.ops.mesh.primitive_cube_add(size=2)
        cube = bpy.context.active_object

        # 输出信息
        print(f"Created: {cube.name}")
    "#).await?;

    println!("Blender输出: {}", result.output);

    // 导出模型
    bridge.export_model(
        input_blend: "scenes/character.blend",
        output_gltf: "models/character.glb",
    ).await?;

    Ok(())
}
```

### 数据交换

```rust
// 导入Blender场景
let scene = bridge.import_blend("scenes/game_level.blend").await?;

// 访问对象
for object in scene.objects {
    match object.object_type {
        ObjectType::Mesh => {
            println!("Mesh: {}", object.name);
        }
        ObjectType::Camera => {
            println!("Camera: {}", object.name);
        }
        ObjectType::Light => {
            println!("Light: {}", object.name);
        }
        _ => {}
    }
}

// 导入网格数据
if let Some(mesh) = scene.get_mesh("Cube") {
    println!("Vertices: {}", mesh.vertices.len());
    println!("Faces: {}", mesh.faces.len());
    println!("UVs: {}", mesh.uvs.len());
}

// 导出为引擎格式
bridge.export_to_engine(
    scene: &scene,
    output_path: "assets/game_level",
).await?;
```

## 最佳实践

### 网格优化

1. **LOD生成**: 为大型模型创建多个LOD级别
2. **拓扑简化**: 移除不可见的背面
3. **顶点合并**: 焊接重复顶点

### UV展开

1. **最小化拉伸**: 使用智能UV展开
2. **最大化空间**: 优化UV岛打包
3. **多通道UV**: 分离光照和材质UV

### 材质创建

1. **PBR工作流**: 使用物理准确的材质参数
2. **纹理优化**: 使用合适的纹理分辨率
3. **材质库**: 复用材质以提高效率

### 动画制作

1. **关键帧优化**: 移除冗余关键帧
2. **曲线平滑**: 使用合适的插值类型
3. **压缩**: 对最终动画进行压缩

## 集成示例

### 完整工作流示例

```rust
use game_engine::tools::dcc::{DCCMeshEditor, UVEditor, MaterialEditor, AnimationEditor};

async fn create_character_asset() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 导入并优化网格
    let mut mesh_editor = DCCMeshEditor::new();
    mesh_editor.import_mesh("source/character_highpoly.fbx").await?;
    mesh_editor.simplify_mesh(15000); // 15k面
    mesh_editor.recalculate_normals(45.0);

    // 2. UV展开
    let mut uv_editor = UVEditor::new();
    uv_editor.load_mesh_from_editor(&mesh_editor).await?;
    uv_editor.smart_unwrap(UVChannel::UV0, 0.02, 45.0)?;
    uv_editor.pack_uv_islands();

    // 3. 创建材质
    let mut material_editor = MaterialEditor::new();
    let mut material = PBRMaterial::new("CharacterMat");
    material.set_albedo_map("textures/character_albedo.png");
    material.set_normal_map("textures/character_normal.png");
    material.set_roughness_map("textures/character_roughness.png");
    material_editor.edit_material(material);

    // 4. 创建动画
    let mut anim_editor = AnimationEditor::new();
    let idle_anim = anim_editor.add_animation("Idle".to_string(), 2.0);
    // ... 添加关键帧

    // 5. 导出完整资产
    mesh_editor.export_mesh("assets/character.glb").await?;
    uv_editor.export_uv_layout("assets/character_uv.png").await?;

    Ok(())
}
```

## 故障排除

### 常见问题

1. **网格导入失败**
   - 检查文件格式支持
   - 确认文件路径正确
   - 验证网格不是空的

2. **UV展开错误**
   - 检查网格是否有UV
   - 确认UV通道存在
   - 尝试重置UV

3. **材质渲染异常**
   - 验证纹理路径
   - 检查着色器编译
   - 确认PBR参数范围

4. **动画播放问题**
   - 确认帧率设置
   - 检查关键帧数据
   - 验证循环设置

## 下一步

- [ ] 查看DCC工具源码: `src/tools/dcc/`
- [ ] 阅读测试用例: `src/tools/dcc/tests.rs`
- [ ] 集成到编辑器: `src/editor/dcc_panel.rs`

---

**注意**: DCC工具需要编辑器环境运行,独立使用需要初始化渲染上下文。
