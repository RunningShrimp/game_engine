//! # DCC (Digital Content Creation) 工具集成
//!
//! 此模块提供数字内容创作工具的集成功能，包括：
//! - 网格编辑器（顶点、边、面编辑）
//! - 材质编辑器（PBR参数实时调整）
//! - 动画关键帧编辑器
//! - UV编辑器
//! - 脚本生成器（Lua/Python/Rust）

pub mod animation_editor;
pub mod integrator;
pub mod material_editor;
pub mod mesh_editor;
pub mod uv_editor;

#[cfg(feature = "blender")]
pub mod blender_bridge;

// 重新导出主要类型
pub use mesh_editor::{
    EdgeID, EditMode, FaceID, MeshEditor, MeshOperation, SelectionMode, TransformTool, VertexID,
};

pub use material_editor::{
    DCCMaterialEditor, MaterialID, PBRMaterialParams, PreviewRenderer, TextureSlot, TextureType,
};

pub use animation_editor::{
    AnimationCurve, AnimationID, DCCAnimationEditor, KeyframeEditor, KeyframeID, PlaybackState,
    Timeline,
};

pub use uv_editor::{SnapSettings, UVEditor, UVID, UVIsland, UVTransform};

pub use integrator::{
    EditorOperation, ExportOptions, GeneratedScript, ScriptGenerator, ScriptLanguage,
};

#[cfg(feature = "blender")]
pub use blender_bridge::{
    BlenderBridge, BlenderBridgeConfig, BlenderBridgeManager, BlenderError, BlenderMaterial,
    BlenderMesh, BlenderObject, BlenderScene,
};

/// DCC工具套件
///
/// 集成所有DCC工具的主入口
#[derive(Debug, Clone)]
pub struct DCCToolkit {
    pub mesh_editor: MeshEditor,
    pub material_editor: DCCMaterialEditor,
    pub animation_editor: DCCAnimationEditor,
    pub uv_editor: UVEditor,
    pub script_generator: ScriptGenerator,
}

impl DCCToolkit {
    /// 创建新的DCC工具套件
    pub fn new() -> Self {
        Self {
            mesh_editor: MeshEditor::new(),
            material_editor: DCCMaterialEditor::new(),
            animation_editor: DCCAnimationEditor::new(),
            uv_editor: UVEditor::new(),
            script_generator: ScriptGenerator::new(),
        }
    }

    /// 显示DCC工具UI
    pub fn show_ui(&mut self, ctx: &egui::Context) {
        self.mesh_editor.show_ui(ctx);
        self.material_editor.show_ui(ctx);
        self.animation_editor.show_ui(ctx);
        self.uv_editor.show_ui(ctx);
    }

    /// 导出为脚本
    pub fn export_script(&self, language: ScriptLanguage) -> GeneratedScript {
        self.script_generator.generate_script(language)
    }
}

impl Default for DCCToolkit {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dcc_toolkit_creation() {
        let toolkit = DCCToolkit::new();
        assert!(toolkit.script_generator.operations().is_empty());
    }

    #[test]
    fn test_export_empty_script() {
        let toolkit = DCCToolkit::new();
        let script = toolkit.export_script(ScriptLanguage::Lua);
        assert!(!script.code.is_empty());
        assert!(script.code.contains("-- Auto-generated"));
    }
}
