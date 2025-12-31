//! # DCC工具模块测试
//!
//! 集成测试和功能验证

#[cfg(test)]
mod integration_tests {
    use super::super::*;

    #[test]
    fn test_dcc_toolkit_full_workflow() {
        // 创建完整的DCC工具套件
        let toolkit = crate::tools::dcc::DCCToolkit::new();

        // 测试网格编辑器
        assert_eq!(toolkit.mesh_editor.edit_mode, crate::tools::dcc::EditMode::Vertex);

        // 测试材质编辑器
        let mat_id = toolkit.material_editor.add_material("TestMaterial".to_string());
        assert_eq!(mat_id, 0);

        // 测试动画编辑器
        assert_eq!(
            toolkit.animation_editor.timeline.playback_state,
            crate::tools::dcc::PlaybackState::Stopped
        );

        // 测试UV编辑器
        assert!(toolkit.uv_editor.selected_uvs.is_empty());

        // 测试脚本生成器
        assert!(toolkit.script_generator.operations().is_empty());
    }

    #[test]
    fn test_mesh_editor_operations() {
        use crate::tools::dcc::MeshEditor;
        use crate::render::mesh::Vertex3D;
        use glam::Mat4;

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
                pos: [0.0, 1.0, 0.0],
                normal: [0.0, 0.0, 1.0],
                uv: [0.0, 1.0],
                tangent: [1.0, 0.0, 0.0, 1.0],
            },
        ];
        let indices = vec![0, 1, 2];

        editor.load_mesh(vertices, indices);
        assert!(editor.current_mesh.is_some());

        // 测试选择
        editor.selected_vertices.insert(0);
        assert_eq!(editor.selected_vertices.len(), 1);

        // 测试变换
        let transform = Mat4::IDENTITY;
        editor.apply_transform(transform);
        assert_eq!(editor.operation_history.len(), 1);

        // 测试清除
        editor.clear_selection();
        assert!(editor.selected_vertices.is_empty());
    }

    #[test]
    fn test_material_editor_workflow() {
        use crate::tools::dcc::{DCCMaterialEditor, MaterialPreset};

        let mut editor = DCCMaterialEditor::new();

        // 添加材质
        let id1 = editor.add_material("Metal".to_string());
        let id2 = editor.add_material("Plastic".to_string());

        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(editor.materials.len(), 2);

        // 应用预设
        editor.apply_preset(id1, MaterialPreset::StandardMetal);
        editor.apply_preset(id2, MaterialPreset::Plastic);

        // 验证参数
        let metal = editor.get_material(id1).unwrap();
        assert_eq!(metal.metallic, 1.0);
        assert_eq!(metal.roughness, 0.2);

        let plastic = editor.get_material(id2).unwrap();
        assert_eq!(plastic.metallic, 0.0);
        assert_eq!(plastic.roughness, 0.3);

        // 移除材质
        editor.remove_material(id1);
        assert_eq!(editor.materials.len(), 1);
    }

    #[test]
    fn test_animation_editor_workflow() {
        use crate::tools::dcc::{DCCAnimationEditor, AnimatedValue};

        let mut editor = DCCAnimationEditor::new();

        // 添加动画
        let anim_id = editor.add_animation("Walk".to_string(), 2.0);
        assert_eq!(anim_id, 0);

        // 添加关键帧
        let key_id = editor.add_keyframe(
            anim_id,
            "position.x".to_string(),
            0.0,
            AnimatedValue::Float(0.0),
        );

        assert!(key_id.is_some());
        assert_eq!(key_id.unwrap(), 0);

        // 测试时间转换
        editor.set_current_time(1.0);
        assert!((editor.get_current_time() - 1.0).abs() < 0.01);

        // 测试播放
        editor.timeline.playback_state = crate::tools::dcc::PlaybackState::Playing;
        editor.update(0.1);
        assert!(editor.timeline.current_frame > 0.0);
    }

    #[test]
    fn test_uv_editor_workflow() {
        use crate::tools::dcc::UVEditor;
        use glam::Vec2;

        let mut editor = UVEditor::new();

        // 加载UV数据
        let uvs = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(0.5, 1.0),
        ];
        let triangles = vec![[0, 1, 2]];

        editor.load_uvs(uvs, triangles);
        assert_eq!(editor.uv_islands.len(), 1);

        // 测试选择
        editor.selected_uvs.insert(0);
        assert_eq!(editor.selected_uvs.len(), 1);

        // 测试清除
        editor.clear_selection();
        assert!(editor.selected_uvs.is_empty());
    }

    #[test]
    fn test_script_generation() {
        use crate::tools::dcc::{ScriptGenerator, ScriptLanguage, EditorOperation};
        use glam::Mat4;

        let mut generator = ScriptGenerator::new();

        // 添加操作
        generator.add_operation(EditorOperation::VertexTransform {
            vertices: vec![0, 1, 2],
            transform: Mat4::IDENTITY,
        });

        // 生成Lua脚本
        let lua_script = generator.generate_script(ScriptLanguage::Lua);
        assert!(lua_script.code.contains("-- Auto-generated"));
        assert_eq!(lua_script.operation_count, 1);

        // 生成Python脚本
        let python_script = generator.generate_script(ScriptLanguage::Python);
        assert!(python_script.code.contains("# Auto-generated"));
        assert_eq!(python_script.operation_count, 1);

        // 生成Rust代码
        let rust_script = generator.generate_script(ScriptLanguage::Rust);
        assert!(rust_script.code.contains("// Auto-generated"));
        assert_eq!(rust_script.operation_count, 1);

        // 清除操作
        generator.clear_operations();
        assert!(generator.operations().is_empty());
    }
}
