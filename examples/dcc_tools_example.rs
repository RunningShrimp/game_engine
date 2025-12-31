//! DCC工具演示
//!
//! 演示游戏引擎的数字内容创作(DCC)工具，包括：
//! - 网格编辑
//! - 材质编辑
//! - 动画编辑
//! - UV编辑
//! - 脚本生成
//! - Blender集成

use game_engine::tools::dcc::{
    DCCToolkit,
    mesh_editor::{MeshEditor, EditMode, TransformTool, MeshOperation},
    material_editor::{DCCMaterialEditor, PBRMaterialParams, TextureType},
    animation_editor::{DCCAnimationEditor, Timeline, AnimationCurve, CurveType, PlaybackState},
    uv_editor::UVEditor,
    integrator::{ScriptGenerator, ScriptLanguage, EditorOperation},
    blender_bridge::{BlenderBridge, BlenderBridgeConfig},
};
use glam::{Mat4, Vec2, Vec3, Vec4};
use std::path::PathBuf;

fn main() {
    println!("=== 游戏引擎DCC工具演示 ===\n");

    // 示例1: 网格编辑
    example_1_mesh_editing();

    // 示例2: 材质编辑
    example_2_material_editing();

    // 示例3: 动画编辑
    example_3_animation_editing();

    // 示例4: UV编辑
    example_4_uv_editing();

    // 示例5: 脚本生成
    example_5_script_generation();

    // 示例6: Blender集成
    example_6_blender_integration();

    // 示例7: 统一工具包
    example_7_unified_toolkit();
}

/// 示例1: 网格编辑
fn example_1_mesh_editing() {
    println!("=== 示例1: 网格编辑 ===\n");

    println!("✓ 创建网格编辑器:");
    let mut editor = MeshEditor::new();

    // 设置编辑模式
    editor.edit_mode = EditMode::Vertex;
    editor.transform_tool = TransformTool::Translate;

    println!("  编辑模式: {:?}", editor.edit_mode);
    println!("  变换工具: {:?}", editor.transform_tool);
    println!();

    println!("✓ 选择顶点:");
    editor.select_vertex(0);
    editor.select_vertex(1);
    editor.select_vertex(2);
    println!("  已选择顶点: {:?}", editor.selected_vertices);
    println!();

    println!("✓ 应用挤出操作:");
    let operation = MeshOperation::Extrude {
        elements: vec![0, 1, 2],
        distance: 1.0,
    };
    editor.apply_operation(operation.clone());
    println!("  操作: {:?}", operation);
    println!("  操作历史长度: {}", editor.operation_history.len());
    println!();

    println!("✓ 应用倒角操作:");
    let operation = MeshOperation::Bevel {
        vertices: vec![0],
        amount: 0.2,
        segments: 3,
    };
    editor.apply_operation(operation);
    println!("  倒角顶点: 0");
    println!("  倒角量: 0.2");
    println!("  段数: 3");
    println!();

    println!("✓ 启用对称编辑:");
    editor.symmetry_enabled = true;
    editor.symmetry_axis = Vec3::new(1.0, 0.0, 0.0); // X轴对称
    println!("  对称启用: {}", editor.symmetry_enabled);
    println!("  对称轴: {:?}", editor.symmetry_axis);
    println!();

    println!("✓ 启用软选择:");
    editor.soft_selection_enabled = true;
    editor.soft_selection_radius = 2.0;
    editor.soft_selection_falloff = 1.0;
    println!("  软选择启用: {}", editor.soft_selection_enabled);
    println!("  半径: {}", editor.soft_selection_radius);
    println!("  衰减: {}", editor.soft_selection_falloff);
    println!();

    println!("💡 可用的网格操作:");
    println!("  • 顶点变换 (VertexTransform)");
    println!("  • 挤出 (Extrude)");
    println!("  • 倒角 (Bevel)");
    println!("  • 焊接 (Weld)");
    println!("  • 删除 (Delete)");
    println!("  • 桥接 (Bridge)");
    println!();
}

/// 示例2: 材质编辑
fn example_2_material_editing() {
    println!("=== 示例2: 材质编辑 ===\n");

    println!("✓ 创建材质编辑器:");
    let mut editor = DCCMaterialEditor::new();

    println!("✓ 创建PBR材质:");
    let mut params = PBRMaterialParams::default();

    // 设置金属材质参数
    params.albedo = Vec4::new(0.8, 0.7, 0.2, 1.0); // 金色
    params.metallic = 1.0; // 完全金属
    params.roughness = 0.2; // 光滑
    params.ao = 1.0; // 无环境遮蔽
    params.emissive = Vec3::new(0.0, 0.0, 0.0); // 不发光
    params.normal_strength = 1.0; // 标准法线强度
    params.clearcoat = 0.5; // 有清漆
    params.clearcoat_roughness = 0.1; // 清漆光滑

    println!("  材质参数:");
    println!("    基础颜色: ({:.2}, {:.2}, {:.2}, {:.2})",
        params.albedo.x, params.albedo.y, params.albedo.z, params.albedo.w);
    println!("    金属度: {:.2}", params.metallic);
    println!("    粗糙度: {:.2}", params.roughness);
    println!("    清漆: {:.2}", params.clearcoat);
    println!();

    println!("✓ 设置纹理:");
    let texture_types = vec![
        TextureType::Albedo,
        TextureType::Normal,
        TextureType::Roughness,
        TextureType::Metallic,
        TextureType::AmbientOcclusion,
        TextureType::Emissive,
        TextureType::Clearcoat,
    ];

    for texture_type in texture_types {
        if let Some(slot) = params.textures.get_mut(&texture_type) {
            slot.path = Some(PathBuf::from(format!("textures/{}.png",
                format!("{:?}", texture_type).to_lowercase())));
            slot.scale = 1.0;
            slot.offset = [0.0, 0.0];
            slot.rotation = 0.0;
            slot.enabled = true;
        }
    }

    println!("  已设置纹理:");
    for (texture_type, slot) in &params.textures {
        if slot.enabled {
            println!("    • {:?}: {:?}", texture_type,
                slot.path.as_ref().and_then(|p| p.to_str()));
        }
    }
    println!();

    println!("✓ 应用材质:");
    editor.set_material_params(0, params);
    println!("  材质ID: 0");
    println!();

    println!("✓ 使用材质预设:");
    let presets = vec![
        "metal_gold",
        "metal_silver",
        "metal_copper",
        "plastic_red",
        "glass_clear",
        "wood_oak",
        "fabric_cotton",
    ];

    println!("  可用预设:");
    for preset in &presets {
        println!("    • {}", preset);
    }
    editor.apply_preset("metal_gold");
    println!("  已应用预设: metal_gold");
    println!();

    println!("💡 PBR材质系统支持:");
    println!("  • 基础颜色 (Albedo)");
    println!("  • 金属度 (Metallic)");
    println!("  • 粗糙度 (Roughness)");
    println!("  • 环境光遮蔽 (Ambient Occlusion)");
    println!("  • 发光 (Emissive)");
    println!("  • 法线强度 (Normal Strength)");
    println!("  • 清漆 (Clearcoat)");
    println!("  • 清漆粗糙度 (Clearcoat Roughness)");
    println!();
}

/// 示例3: 动画编辑
fn example_3_animation_editing() {
    println!("=== 示例3: 动画编辑 ===\n");

    println!("✓ 创建动画编辑器:");
    let mut editor = DCCAnimationEditor::new();

    println!("✓ 设置时间轴:");
    editor.timeline.zoom = 2.0;
    editor.timeline.scroll = 0.0;
    editor.timeline.current_frame = 0.0;
    editor.timeline.frame_rate = 60.0;
    editor.timeline.start_time = 0.0;
    editor.timeline.end_time = 10.0;
    editor.timeline.loop_playback = true;

    println!("  时间轴参数:");
    println!("    缩放: {:.1}", editor.timeline.zoom);
    println!("    当前帧: {:.1}", editor.timeline.current_frame);
    println!("    帧率: {:.1} FPS", editor.timeline.frame_rate);
    println!("    时间范围: {:.1}s - {:.1}s",
        editor.timeline.start_time, editor.timeline.end_time);
    println!("    循环播放: {}", editor.timeline.loop_playback);
    println!();

    println!("✓ 创建动画曲线:");
    let mut curve = AnimationCurve {
        name: "position_x".to_string(),
        curve_type: CurveType::Bezier,
        keyframes: vec![],
    };

    println!("  曲线类型: {:?}", curve.curve_type);
    println!("  曲线名称: {}", curve.name);
    println!();

    println!("✓ 添加关键帧:");
    editor.add_keyframe(0, 0.0, 0.0); // frame, time, value
    editor.add_keyframe(0, 1.0, 5.0);
    editor.add_keyframe(0, 2.0, 10.0);

    println!("  关键帧:");
    println!("    帧 0: 位置 = 0.0");
    println!("    帧 1: 位置 = 5.0");
    println!("    帧 2: 位置 = 10.0");
    println!();

    println!("✓ 播放动画:");
    editor.timeline.playback_state = PlaybackState::Playing;
    println!("  播放状态: {:?}", editor.timeline.playback_state);
    println!();

    println!("💡 动画编辑功能:");
    println!("  • 时间轴管理 (缩放/滚动/跳转)");
    println!("  • 播放控制 (播放/暂停/停止/循环)");
    println!("  • 关键帧编辑 (创建/删除/移动)");
    println!("  • 动画曲线 (线性/阶梯/三次样条/贝塞尔)");
    println!("  • 多值类型 (Float/Vec2/Vec3/Quat/Color)");
    println!();

    println!("💡 曲线类型:");
    println!("  • Linear - 线性插值");
    println!("  • Step - 阶梯插值");
    println!("  • Cubic - 三次样条插值");
    println!("  • Bezier - 贝塞尔曲线");
    println!();
}

/// 示例4: UV编辑
fn example_4_uv_editing() {
    println!("=== 示例4: UV编辑 ===\n");

    println!("✓ 创建UV编辑器:");
    let mut editor = UVEditor::new();

    println!("✓ UV编辑功能:");
    println!();

    println!("  1. UV坐标编辑:");
    println!("     • UV点选择和移动");
    println!("     • UV岛选择和变换");
    println!("     • UV坐标精确调整");
    println!();

    println!("  2. UV变换工具:");
    println!("     • 平移 (Translation)");
    println!("     • 旋转 (Rotation)");
    println!("     • 缩放 (Scale)");
    println!("     • 镜像 (Mirror)");
    println!();

    println!("  3. UV操作工具:");
    println!("     • UV展开 (Unwrap)");
    println!("     • UV打包 (Pack)");
    println!("     • UV缝合 (Stitch)");
    println!("     • UV拆分 (Split)");
    println!();

    println!("  4. UV显示选项:");
    println!("     • 纹理预览");
    println!("     • UV网格显示");
    println!("     • 重叠检查");
    println!("     • 岛边界显示");
    println!();

    println!("✓ UV变换示例:");
    // 选择UV点
    editor.select_uv(0);
    editor.select_uv(1);
    editor.select_uv(2);

    // 应用UV变换
    editor.transform_uvs(
        &[0, 1, 2],
        (0.1, 0.1),  // translation
        0.0,         // rotation
        (1.0, 1.0),  // scale
    );

    println!("  UV点: 0, 1, 2");
    println!("  平移: (0.1, 0.1)");
    println!("  旋转: 0°");
    println!("  缩放: (1.0, 1.0)");
    println!();

    println!("💡 UV工作流:");
    println!("  1. 选择需要编辑的UV点或UV岛");
    println!("  2. 应用变换或工具");
    println!("  3. 使用展开/打包工具优化布局");
    println!("  4. 检查重叠并修复");
    println!();
}

/// 示例5: 脚本生成
fn example_5_script_generation() {
    println!("=== 示例5: 脚本生成 ===\n");

    println!("✓ 创建脚本生成器:");
    let mut generator = ScriptGenerator::new();

    println!("✓ 记录编辑操作:");

    // 记录顶点变换操作
    let transform = Mat4::from_translation(Vec3::new(1.0, 0.0, 0.0));
    generator.record_operation(EditorOperation::VertexTransform {
        vertices: vec![0, 1, 2],
        transform,
    });
    println!("  • 顶点变换: 顶点0,1,2沿X轴平移1.0单位");

    // 记录材质更改操作
    use game_engine::tools::dcc::integrator::PbrMaterialParams;
    let material_params = PbrMaterialParams {
        base_color: Vec4::new(1.0, 0.0, 0.0, 1.0),
        metallic: 0.0,
        roughness: 0.5,
        ambient_occlusion: 1.0,
        emissive: Vec3::ZERO,
        normal_scale: 1.0,
    };
    generator.record_operation(EditorOperation::MaterialChange {
        material: 0,
        params: material_params,
    });
    println!("  • 材质更改: 材质0设置为红色");
    println!();

    println!("✓ 生成Lua脚本:");
    let lua_script = generator.generate_script(ScriptLanguage::Lua);
    println!("{}\n", lua_script.code);

    println!("✓ 生成Python脚本:");
    let python_script = generator.generate_script(ScriptLanguage::Python);
    println!("{}\n", python_script.code);

    println!("✓ 生成Rust代码:");
    let rust_code = generator.generate_script(ScriptLanguage::Rust);
    println!("{}\n", rust_code.code);

    println!("💡 脚本生成功能:");
    println!("  • 自动记录所有编辑操作");
    println!("  • 支持Lua/Python/Rust三种语言");
    println!("  • 自动添加注释和格式化");
    println!("  • 包含错误处理代码");
    println!();

    println!("💡 生成的脚本特点:");
    println!("  • 可读性强，带详细注释");
    println!("  • 包含错误处理");
    println!("  • 符合语言最佳实践");
    println!("  • 可直接在引擎中运行");
    println!();
}

/// 示例6: Blender集成
fn example_6_blender_integration() {
    println!("=== 示例6: Blender集成 ===\n");

    println!("✓ 创建Blender桥接配置:");
    let config = BlenderBridgeConfig {
        blender_path: PathBuf::from("/usr/local/bin/blender"),
        script_path: PathBuf::from("blender_bridge.py"),
        port: 9876,
        background_mode: true,
    };

    println!("  Blender路径: {:?}", config.blender_path);
    println!("  脚本路径: {:?}", config.script_path);
    println!("  端口: {}", config.port);
    println!("  后台模式: {}", config.background_mode);
    println!();

    println!("✓ Blender桥接功能:");
    println!();

    println!("  1. 进程控制:");
    println!("     • 启动Blender后台进程");
    println!("     • 启动Blender前台进程");
    println!("     • 进程通信和状态监控");
    println!();

    println!("  2. Python API集成:");
    println!("     • 通过Python脚本调用Blender API");
    println!("     • 访问Blender全部功能");
    println!("     • 数据交换和同步");
    println!();

    println!("  3. 数据同步:");
    println!("     • 网格数据同步");
    println!("     • 材质数据同步");
    println!("     • 动画数据同步");
    println!("     • UV数据同步");
    println!();

    println!("  4. 实时通信:");
    println!("     • 双向数据交换");
    println!("     • 实时更新支持");
    println!("     • 低延迟通信");
    println!();

    println!("💡 Blender集成工作流:");
    println!("  1. 在Blender中创建/编辑模型");
    println!("  2. 通过桥接连接到引擎");
    println!("  3. 实时同步数据到引擎");
    println!("  4. 在引擎中预览和调整");
    println!("  5. 可选：将更改同步回Blender");
    println!();

    println!("💡 使用场景:");
    println!("  • 资产管线集成");
    println!("  • 快速原型开发");
    println!("  • 实时预览和调优");
    println!("  • 跨工具协作");
    println!();

    println!("⚠️  注意:");
    println!("  • 需要安装Blender");
    println!("  • 需要Python环境");
    println!("  • 需要blender_bridge.py脚本");
    println!();
}

/// 示例7: 统一工具包
fn example_7_unified_toolkit() {
    println!("=== 示例7: 统一DCC工具包 ===\n");

    println!("✓ 创建DCC工具包:");
    let mut toolkit = DCCToolkit::new();

    println!("✓ 使用网格编辑器:");
    toolkit.mesh_editor.edit_mode = EditMode::Vertex;
    toolkit.mesh_editor.select_vertex(0);
    println!("  网格编辑器: 模式={:?}, 已选择{}个顶点",
        toolkit.mesh_editor.edit_mode,
        toolkit.mesh_editor.selected_vertices.len());
    println!();

    println!("✓ 使用材质编辑器:");
    let mut params = PBRMaterialParams::default();
    params.albedo = Vec4::new(0.5, 0.5, 0.5, 1.0);
    toolkit.material_editor.set_material_params(0, params);
    println!("  材质编辑器: 已设置材质0");
    println!();

    println!("✓ 使用动画编辑器:");
    toolkit.animation_editor.timeline.current_frame = 10.0;
    println!("  动画编辑器: 当前帧={:.1}",
        toolkit.animation_editor.timeline.current_frame);
    println!();

    println!("✓ 使用UV编辑器:");
    toolkit.uv_editor.select_uv(0);
    println!("  UV编辑器: 已选择{}个UV点",
        toolkit.uv_editor.selected_uvs.len());
    println!();

    println!("✓ 导出脚本:");
    let lua_script = toolkit.export_script(ScriptLanguage::Lua);
    println!("  脚本语言: {:?}", lua_script.language);
    println!("  操作数量: {}", lua_script.operation_count);
    println!("  代码长度: {}字节", lua_script.code.len());
    println!();

    println!("💡 DCCToolkit优势:");
    println!("  • 统一接口 - 所有编辑器通过同一接口访问");
    println!("  • 一致操作 - 所有编辑器使用相似的操作模式");
    println!("  • 集成显示 - 在同一UI中显示所有编辑器");
    println!("  • 脚本生成 - 从所有编辑器统一导出脚本");
    println!();

    println!("💡 典型工作流:");
    println!("  1. 使用网格编辑器创建和编辑网格");
    println!("  2. 使用材质编辑器创建和调整材质");
    println!("  3. 使用UV编辑器展开和优化UV");
    println!("  4. 使用动画编辑器创建动画");
    println!("  5. 使用脚本生成器导出可执行脚本");
    println!("  6. 可选：使用Blender桥接与Blender协作");
    println!();

    println!("═════════════════════════════════════════════════════");
    println!("✅ DCC工具演示完成");
    println!("═════════════════════════════════════════════════════");
    println!();

    println!("📚 更多信息:");
    println!("  • DCC工具文档: game_engine/src/tools/dcc/README.md");
    println!("  • 网格编辑器: game_engine/src/tools/dcc/mesh_editor.rs");
    println!("  • 材质编辑器: game_engine/src/tools/dcc/material_editor.rs");
    println!("  • 动画编辑器: game_engine/src/tools/dcc/animation_editor.rs");
    println!("  • UV编辑器: game_engine/src/tools/dcc/uv_editor.rs");
    println!("  • 脚本生成器: game_engine/src/tools/dcc/integrator.rs");
    println!("  • Blender桥接: game_engine/src/tools/dcc/blender_bridge.rs");
    println!();

    println!("🚀 下一步:");
    println!("  1. 使用DCC工具创建游戏资产");
    println!("  2. 生成可执行脚本");
    println!("  3. 与Blender集成进行高级建模");
    println!("  4. 探索更多DCC工具功能");
}

// 辅助函数：生成示例脚本
fn generate_sample_lua_script() -> String {
    r#"
-- 示例Lua脚本：创建立方体
-- 由DCC工具自动生成

local mesh = Engine.create_mesh()

-- 设置顶点
mesh:add_vertex(Vec3(0, 0, 0))
mesh:add_vertex(Vec3(1, 0, 0))
mesh:add_vertex(Vec3(1, 1, 0))
mesh:add_vertex(Vec3(0, 1, 0))

-- 设置索引
mesh:add_triangle(0, 1, 2)
mesh:add_triangle(0, 2, 3)

-- 设置材质
local material = Engine.create_material()
material:set_albedo(Color(1.0, 0.5, 0.2, 1.0))
material:set_metallic(0.8)
material:set_roughness(0.3)

mesh:set_material(material)

-- 添加到场景
Engine.add_mesh(mesh)
"#.to_string()
}

fn generate_sample_python_script() -> String {
    r#"
# 示例Python脚本：创建动画
# 由DCC工具自动生成

import game_engine as ge

# 创建实体
entity = ge.spawn_entity()

# 添加变换组件
transform = ge.Transform()
transform.position = ge.Vec3(0.0, 0.0, 0.0)
entity.add_component(transform)

# 创建动画
anim = ge.Animation("move_animation")

# 添加关键帧
anim.add_keyframe(0.0, ge.Vec3(0.0, 0.0, 0.0))  # 起点
anim.add_keyframe(1.0, ge.Vec3(5.0, 0.0, 0.0))  # 终点
anim.add_keyframe(2.0, ge.Vec3(0.0, 0.0, 0.0))  # 回到起点

# 设置动画循环
anim.loop = True

# 应用动画
entity.add_animation(anim)
anim.play()
"#.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_editor_creation() {
        let editor = MeshEditor::new();
        assert_eq!(editor.edit_mode, EditMode::Vertex);
        assert_eq!(editor.transform_tool, TransformTool::Translate);
    }

    #[test]
    fn test_material_params_default() {
        let params = PBRMaterialParams::default();
        assert_eq!(params.metallic, 0.0);
        assert_eq!(params.roughness, 0.5);
    }

    #[test]
    fn test_animation_timeline_default() {
        let timeline = Timeline::default();
        assert_eq!(timeline.frame_rate, 60.0);
        assert_eq!(timeline.current_frame, 0.0);
    }

    #[test]
    fn test_script_generator() {
        let generator = ScriptGenerator::new();
        let script = generator.generate_script(ScriptLanguage::Lua);
        assert_eq!(script.language, ScriptLanguage::Lua);
        assert!(!script.code.is_empty());
    }
}
