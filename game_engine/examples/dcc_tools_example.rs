//! # DCC工具使用示例
//!
//! 演示如何使用DCC工具进行网格编辑、材质调整、动画和UV编辑

use game_engine::tools::dcc::*;
use game_engine::render::mesh::Vertex3D;
use glam::*;

fn main() {
    println!("=== DCC工具集成示例 ===\n");

    // 示例1: 网格编辑
    println!("1. 网格编辑示例");
    mesh_editing_example();

    // 示例2: 材质编辑
    println!("\n2. 材质编辑示例");
    material_editing_example();

    // 示例3: 动画编辑
    println!("\n3. 动画编辑示例");
    animation_editing_example();

    // 示例4: UV编辑
    println!("\n4. UV编辑示例");
    uv_editing_example();

    // 示例5: 脚本生成
    println!("\n5. 脚本生成示例");
    script_generation_example();

    // 示例6: 完整工作流
    println!("\n6. 完整工作流示例");
    complete_workflow_example();

    println!("\n=== 所有示例完成 ===");
}

/// 网格编辑示例
fn mesh_editing_example() {
    let mut editor = MeshEditor::new();

    // 创建测试网格
    let vertices = vec![
        Vertex3D {
            pos: [0.0, 0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
            uv: [0.0, 0.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
        },
        Vertex3D {
            pos: [1.0, 0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
            uv: [1.0, 0.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
        },
        Vertex3D {
            pos: [0.5, 1.0, 0.0],
            normal: [0.0, 0.0, 1.0],
            uv: [0.5, 1.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
        },
    ];
    let indices = vec![0, 1, 2];

    editor.load_mesh(vertices, indices);
    println!("  - 已加载网格: {} 顶点, {} 三角形",
        editor.current_mesh.as_ref().unwrap().vertices.len(),
        editor.current_mesh.as_ref().unwrap().indices.len() / 3
    );

    // 设置编辑模式
    editor.edit_mode = EditMode::Vertex;
    println!("  - 编辑模式: {:?}", editor.edit_mode);

    // 选择顶点
    editor.selected_vertices.insert(0);
    editor.selected_vertices.insert(1);
    println!("  - 已选择 {} 个顶点", editor.selected_vertices.len());

    // 应用变换
    let transform = Mat4::from_translation(Vec3::new(1.0, 0.0, 0.0));
    editor.apply_transform(transform);
    println!("  - 已应用平移变换: (1.0, 0.0, 0.0)");
    println!("  - 操作历史: {} 条记录", editor.operation_history.len());
}

/// 材质编辑示例
fn material_editing_example() {
    let mut editor = DCCMaterialEditor::new();

    // 创建材质
    let metal_id = editor.add_material("Chrome".to_string());
    let plastic_id = editor.add_material("Plastic".to_string());
    println!("  - 创建了 {} 个材质", editor.materials.len());

    // 应用预设
    editor.apply_preset(metal_id, MaterialPreset::StandardMetal);
    editor.apply_preset(plastic_id, MaterialPreset::Plastic);
    println!("  - 已应用材质预设");

    // 修改材质参数
    if let Some(material) = editor.get_material_mut(metal_id) {
        material.metallic = 1.0;
        material.roughness = 0.1;
        material.emissive = Vec3::new(0.1, 0.1, 0.1);
        println!("  - 修改了金属材质参数");
    }

    // 显示材质信息
    let metal = editor.get_material(metal_id).unwrap();
    println!("  - Chrome材质: metallic={}, roughness={}",
        metal.metallic, metal.roughness
    );
}

/// 动画编辑示例
fn animation_editing_example() {
    let mut editor = DCCAnimationEditor::new();

    // 创建动画
    let anim_id = editor.add_animation("WalkCycle".to_string(), 2.0);
    println!("  - 创建动画: {}", anim_id);

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
        1.0,
        AnimatedValue::Float(1.0),
    );
    println!("  - 添加了关键帧");

    // 播放动画
    editor.timeline.playback_state = PlaybackState::Playing;
    println!("  - 开始播放动画");

    // 更新动画
    for _ in 0..10 {
        editor.update(0.016); // ~60fps
    }
    println!("  - 当前帧: {:.2}", editor.timeline.current_frame);
    println!("  - 当前时间: {:.2}秒", editor.get_current_time());
}

/// UV编辑示例
fn uv_editing_example() {
    let mut editor = UVEditor::new();

    // 加载UV数据
    let uvs = vec![
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(0.5, 1.0),
        Vec2::new(0.5, 0.5),
    ];
    let triangles = vec![[0, 1, 3], [1, 2, 3]];

    editor.load_uvs(uvs, triangles);
    println!("  - 加载UV数据: {} 个UV, {} 个三角形",
        editor.uv_islands[0].uvs.len(),
        editor.uv_islands[0].triangles.len()
    );

    // 选择UV
    editor.selected_uvs.insert(0);
    editor.selected_uvs.insert(1);
    println!("  - 选择了 {} 个UV", editor.selected_uvs.len());

    // 设置变换
    editor.transform.translation = Vec2::new(0.1, 0.1);
    editor.transform.scale = Vec2::new(1.2, 1.2);
    println!("  - 设置UV变换");

    // 应用变换
    editor.apply_transform();
    println!("  - 已应用UV变换");
}

/// 脚本生成示例
fn script_generation_example() {
    let mut generator = ScriptGenerator::new();

    // 添加操作
    generator.add_operation(EditorOperation::VertexTransform {
        vertices: vec![0, 1, 2],
        transform: Mat4::from_translation(Vec3::new(1.0, 0.0, 0.0)),
    });

    generator.add_operation(EditorOperation::MaterialChange {
        material: 0,
        params: PbrMaterialParams {
            base_color: Vec4::new(0.8, 0.8, 0.8, 1.0),
            metallic: 1.0,
            roughness: 0.2,
            ambient_occlusion: 1.0,
            emissive: Vec3::ZERO,
            normal_scale: 1.0,
        },
    });

    println!("  - 添加了 {} 个操作", generator.operations().len());

    // 生成Lua脚本
    let lua_script = generator.generate_script(ScriptLanguage::Lua);
    println!("  - Lua脚本长度: {} 字节", lua_script.code.len());
    println!("  - 前100字符:\n{}", &lua_script.code[..lua_script.code.len().min(100)]);

    // 生成Python脚本
    let python_script = generator.generate_script(ScriptLanguage::Python);
    println!("  - Python脚本长度: {} 字节", python_script.code.len());

    // 生成Rust代码
    let rust_code = generator.generate_script(ScriptLanguage::Rust);
    println!("  - Rust代码长度: {} 字节", rust_code.code.len());
}

/// 完整工作流示例
fn complete_workflow_example() {
    // 创建DCC工具套件
    let mut toolkit = DCCToolkit::new();
    println!("  - 创建DCC工具套件");

    // 1. 创建网格
    let vertices = vec![
        Vertex3D {
            pos: [0.0, 0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
            uv: [0.0, 0.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
        },
        Vertex3D {
            pos: [1.0, 0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
            uv: [1.0, 0.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
        },
        Vertex3D {
            pos: [0.5, 1.0, 0.0],
            normal: [0.0, 0.0, 1.0],
            uv: [0.5, 1.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
        },
    ];
    let indices = vec![0, 1, 2];
    toolkit.mesh_editor.load_mesh(vertices, indices);
    println!("  - 步骤1: 创建并加载网格");

    // 2. 编辑网格
    toolkit.mesh_editor.selected_vertices.insert(0);
    toolkit.mesh_editor.apply_transform(Mat4::IDENTITY);
    println!("  - 步骤2: 编辑网格（选择并变换顶点）");

    // 3. 创建材质
    let mat_id = toolkit.material_editor.add_material("MetalSurface".to_string());
    toolkit.material_editor.apply_preset(mat_id, MaterialPreset::StandardMetal);
    println!("  - 步骤3: 创建材质（应用金属预设）");

    // 4. 创建动画
    let anim_id = toolkit.animation_editor.add_animation("Animation".to_string(), 1.0);
    toolkit.animation_editor.add_keyframe(
        anim_id,
        "position".to_string(),
        0.0,
        AnimatedValue::Vec3(Vec3::ZERO),
    );
    println!("  - 步骤4: 创建动画（添加关键帧）");

    // 5. 编辑UV
    let uvs = vec![
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(0.5, 1.0),
    ];
    let triangles = vec![[0, 1, 2]];
    toolkit.uv_editor.load_uvs(uvs, triangles);
    toolkit.uv_editor.selected_uvs.insert(0);
    toolkit.uv_editor.apply_transform();
    println!("  - 步骤5: 编辑UV（选择并变换）");

    // 6. 导出脚本
    let script = toolkit.export_script(ScriptLanguage::Lua);
    println!("  - 步骤6: 导出脚本（{} 字节）", script.code.len());
    println!("  - 总操作数: {}", script.operation_count);

    println!("  - 完整工作流执行完成!");
}
