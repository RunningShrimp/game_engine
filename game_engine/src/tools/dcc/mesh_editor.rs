//! # 网格编辑器
//!
//! 提供顶点、边、面的编辑功能，包括：
//! - 选择模式（顶点/边/面/UV）
//! - 变换工具（平移/旋转/缩放）
//! - 网格操作（挤出/倒角/焊接）
//! - 对称和镜像编辑

use crate::render::mesh::Vertex3D;
use egui::*;
use glam::{Mat4, Vec3, Vec2};
use std::collections::{HashSet, HashMap};

/// 顶点ID类型
pub type VertexID = u32;

/// 边ID类型
pub type EdgeID = u32;

/// 面ID类型
pub type FaceID = u32;

/// 编辑模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditMode {
    /// 顶点编辑
    Vertex,
    /// 边编辑
    Edge,
    /// 面编辑
    Face,
    /// UV编辑
    UV,
}

/// 变换工具
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformTool {
    /// 平移
    Translate,
    /// 旋转
    Rotate,
    /// 缩放
    Scale,
}

/// 选择模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionMode {
    /// 单选
    Single,
    /// 框选
    Box,
    /// 涂刷选择
    Paint,
    /// 环形选择
    Loop,
}

/// 网格操作
#[derive(Debug, Clone)]
pub enum MeshOperation {
    /// 顶点变换
    VertexTransform {
        vertices: Vec<VertexID>,
        transform: Mat4,
    },
    /// 挤出
    Extrude {
        elements: Vec<FaceID>,
        distance: f32,
    },
    /// 倒角
    Bevel {
        vertices: Vec<VertexID>,
        amount: f32,
        segments: u32,
    },
    /// 焊接
    Weld {
        vertices: Vec<VertexID>,
        threshold: f32,
    },
    /// 删除
    Delete {
        elements: Vec<FaceID>,
    },
    /// 桥接
    Bridge {
        edges: Vec<EdgeID>,
    },
}

/// 可编辑的网格数据结构
#[derive(Debug, Clone)]
pub struct EditableMesh {
    /// 顶点数据
    pub vertices: Vec<Vertex3D>,
    /// 索引数据
    pub indices: Vec<u32>,
    /// 顶点选择状态
    pub selected_vertices: HashSet<VertexID>,
    /// 边选择状态
    pub selected_edges: HashSet<EdgeID>,
    /// 面选择状态
    pub selected_faces: HashSet<FaceID>,
    /// 面法线
    pub face_normals: Vec<Vec3>,
    /// UV坐标
    pub uvs: Vec<Vec2>,
}

impl EditableMesh {
    /// 创建新的可编辑网格
    pub fn new(vertices: Vec<Vertex3D>, indices: Vec<u32>) -> Self {
        let face_count = indices.len() / 3;
        let mut face_normals = Vec::with_capacity(face_count);

        // 计算面法线
        for face in 0..face_count {
            let i0 = indices[face * 3] as usize;
            let i1 = indices[face * 3 + 1] as usize;
            let i2 = indices[face * 3 + 2] as usize;

            let v0 = Vec3::from_array(vertices[i0].pos);
            let v1 = Vec3::from_array(vertices[i1].pos);
            let v2 = Vec3::from_array(vertices[i2].pos);

            let edge1 = v1 - v0;
            let edge2 = v2 - v0;
            let normal = edge1.cross(edge2).normalize();

            face_normals.push(normal);
        }

        let uvs = vertices.iter().map(|v| Vec2::from_array(v.uv)).collect();

        Self {
            vertices,
            indices,
            selected_vertices: HashSet::new(),
            selected_edges: HashSet::new(),
            selected_faces: HashSet::new(),
            face_normals,
            uvs,
        }
    }

    /// 获取面中心点
    pub fn get_face_center(&self, face_id: FaceID) -> Vec3 {
        let i0 = self.indices[face_id as usize * 3] as usize;
        let i1 = self.indices[face_id as usize * 3 + 1] as usize;
        let i2 = self.indices[face_id as usize * 3 + 2] as usize;

        let v0 = Vec3::from_array(self.vertices[i0].pos);
        let v1 = Vec3::from_array(self.vertices[i1].pos);
        let v2 = Vec3::from_array(self.vertices[i2].pos);

        (v0 + v1 + v2) / 3.0
    }

    /// 计算包围盒
    pub fn calculate_bounds(&self) -> (Vec3, Vec3) {
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);

        for vertex in &self.vertices {
            let pos = Vec3::from_array(vertex.pos);
            min = min.min(pos);
            max = max.max(pos);
        }

        (min, max)
    }
}

/// 网格编辑器
#[derive(Debug, Clone)]
pub struct MeshEditor {
    /// 选中的顶点
    pub selected_vertices: HashSet<VertexID>,
    /// 选中的边
    pub selected_edges: HashSet<EdgeID>,
    /// 选中的面
    pub selected_faces: HashSet<FaceID>,
    /// 编辑模式
    pub edit_mode: EditMode,
    /// 变换工具
    pub transform_tool: TransformTool,
    /// 选择模式
    pub selection_mode: SelectionMode,
    /// 操作历史
    pub operation_history: Vec<MeshOperation>,
    /// 当前编辑的网格
    pub current_mesh: Option<EditableMesh>,
    /// 对称设置
    pub symmetry_enabled: bool,
    pub symmetry_axis: Vec3,
    /// 软选择设置
    pub soft_selection_enabled: bool,
    pub soft_selection_radius: f32,
    pub soft_selection_falloff: f32,
}

impl MeshEditor {
    /// 创建新的网格编辑器
    pub fn new() -> Self {
        Self {
            selected_vertices: HashSet::new(),
            selected_edges: HashSet::new(),
            selected_faces: HashSet::new(),
            edit_mode: EditMode::Vertex,
            transform_tool: TransformTool::Translate,
            selection_mode: SelectionMode::Single,
            operation_history: Vec::new(),
            current_mesh: None,
            symmetry_enabled: false,
            symmetry_axis: Vec3::X,
            soft_selection_enabled: false,
            soft_selection_radius: 1.0,
            soft_selection_falloff: 1.0,
        }
    }

    /// 加载网格进行编辑
    pub fn load_mesh(&mut self, vertices: Vec<Vertex3D>, indices: Vec<u32>) {
        self.current_mesh = Some(EditableMesh::new(vertices, indices));
        self.clear_selection();
    }

    /// 清除选择
    pub fn clear_selection(&mut self) {
        self.selected_vertices.clear();
        self.selected_edges.clear();
        self.selected_faces.clear();
    }

    /// 显示UI
    pub fn show_ui(&mut self, ctx: &egui::Context) {
        egui::Window::new("Mesh Editor")
            .default_size([300.0, 500.0])
            .show(ctx, |ui| {
                self.show_editor_ui(ui);
            });
    }

    /// 显示编辑器UI
    fn show_editor_ui(&mut self, ui: &mut egui::Ui) {
        // 工具栏
        ui.horizontal(|ui| {
            ui.label("Mode:");
            ui.selectable_value(&mut self.edit_mode, EditMode::Vertex, "Vertex");
            ui.selectable_value(&mut self.edit_mode, EditMode::Edge, "Edge");
            ui.selectable_value(&mut self.edit_mode, EditMode::Face, "Face");
            ui.selectable_value(&mut self.edit_mode, EditMode::UV, "UV");
        });

        ui.separator();

        // 变换工具
        ui.label("Transform:");
        ui.horizontal(|ui| {
            if ui.button("Move").clicked() {
                self.transform_tool = TransformTool::Translate;
            }
            if ui.button("Rotate").clicked() {
                self.transform_tool = TransformTool::Rotate;
            }
            if ui.button("Scale").clicked() {
                self.transform_tool = TransformTool::Scale;
            }
        });

        ui.separator();

        // 选择模式
        ui.label("Selection:");
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.selection_mode, SelectionMode::Single, "Single");
            ui.selectable_value(&mut self.selection_mode, SelectionMode::Box, "Box");
            ui.selectable_value(&mut self.selection_mode, SelectionMode::Paint, "Paint");
            ui.selectable_value(&mut self.selection_mode, SelectionMode::Loop, "Loop");
        });

        ui.separator();

        // 根据编辑模式显示不同工具
        match self.edit_mode {
            EditMode::Vertex => self.vertex_tools(ui),
            EditMode::Edge => self.edge_tools(ui),
            EditMode::Face => self.face_tools(ui),
            EditMode::UV => self.uv_tools(ui),
        }

        ui.separator();

        // 对称设置
        ui.checkbox(&mut self.symmetry_enabled, "Symmetry");
        if self.symmetry_enabled {
            ui.horizontal(|ui| {
                ui.label("Axis:");
                if ui.button("X").clicked() {
                    self.symmetry_axis = Vec3::X;
                }
                if ui.button("Y").clicked() {
                    self.symmetry_axis = Vec3::Y;
                }
                if ui.button("Z").clicked() {
                    self.symmetry_axis = Vec3::Z;
                }
            });
        }

        ui.separator();

        // 软选择设置
        ui.checkbox(&mut self.soft_selection_enabled, "Soft Selection");
        if self.soft_selection_enabled {
            ui.add(egui::Slider::new(&mut self.soft_selection_radius, 0.1..=10.0).text("Radius"));
            ui.add(egui::Slider::new(&mut self.soft_selection_falloff, 0.1..=5.0).text("Falloff"));
        }

        ui.separator();

        // 统计信息
        if let Some(mesh) = &self.current_mesh {
            ui.label(format!("Vertices: {}", mesh.vertices.len()));
            ui.label(format!("Triangles: {}", mesh.indices.len() / 3));
            ui.label(format!("Selected: {} vertices", self.selected_vertices.len()));
        }
    }

    /// 顶点工具
    fn vertex_tools(&mut self, ui: &mut egui::Ui) {
        ui.label("Vertex Tools:");

        if ui.button("Extrude").clicked() {
            self.extrude_vertices();
        }

        if ui.button("Bevel").clicked() {
            self.bevel_vertices(0.1, 1);
        }

        if ui.button("Weld").clicked() {
            self.weld_vertices(0.01);
        }

        if ui.button("Delete").clicked() {
            self.delete_selected_vertices();
        }
    }

    /// 边工具
    fn edge_tools(&mut self, ui: &mut egui::Ui) {
        ui.label("Edge Tools:");

        if ui.button("Bridge").clicked() {
            self.bridge_edges();
        }

        if ui.button("Bevel").clicked() {
            // TODO: 实现边倒角
        }

        if ui.button("Split").clicked() {
            // TODO: 实现边分割
        }
    }

    /// 面工具
    fn face_tools(&mut self, ui: &mut egui::Ui) {
        ui.label("Face Tools:");

        if ui.button("Extrude").clicked() {
            self.extrude_faces();
        }

        if ui.button("Inset").clicked() {
            // TODO: 实现面内插
        }

        if ui.button("Delete").clicked() {
            self.delete_selected_faces();
        }
    }

    /// UV工具
    fn uv_tools(&mut self, ui: &mut egui::Ui) {
        ui.label("UV Tools:");

        if ui.button("Unwrap").clicked() {
            // TODO: 实现UV展开
        }

        if ui.button("Relax").clicked() {
            // TODO: 实现UV松弛
        }

        if ui.button("Pack").clicked() {
            // TODO: 实现UV打包
        }
    }

    /// 挤出顶点
    pub fn extrude_vertices(&mut self) {
        if let Some(mesh) = &mut self.current_mesh {
            if !self.selected_vertices.is_empty() {
                let operation = MeshOperation::Extrude {
                    elements: self.selected_faces.iter().copied().collect(),
                    distance: 0.5,
                };
                self.operation_history.push(operation);

                // TODO: 实现实际的顶点挤出逻辑
            }
        }
    }

    /// 倒角顶点
    pub fn bevel_vertices(&mut self, amount: f32, segments: u32) {
        if !self.selected_vertices.is_empty() {
            let operation = MeshOperation::Bevel {
                vertices: self.selected_vertices.iter().copied().collect(),
                amount,
                segments,
            };
            self.operation_history.push(operation);

            // TODO: 实现实际的顶点倒角逻辑
        }
    }

    /// 焊接顶点
    pub fn weld_vertices(&mut self, threshold: f32) {
        if !self.selected_vertices.is_empty() {
            let operation = MeshOperation::Weld {
                vertices: self.selected_vertices.iter().copied().collect(),
                threshold,
            };
            self.operation_history.push(operation);

            // TODO: 实现实际的顶点焊接逻辑
        }
    }

    /// 删除选中的顶点
    pub fn delete_selected_vertices(&mut self) {
        if !self.selected_vertices.is_empty() {
            // TODO: 实现顶点删除逻辑
            self.selected_vertices.clear();
        }
    }

    /// 挤出面
    pub fn extrude_faces(&mut self) {
        if !self.selected_faces.is_empty() {
            let operation = MeshOperation::Extrude {
                elements: self.selected_faces.iter().copied().collect(),
                distance: 0.5,
            };
            self.operation_history.push(operation);

            // TODO: 实现实际的面挤出逻辑
        }
    }

    /// 删除选中的面
    pub fn delete_selected_faces(&mut self) {
        if !self.selected_faces.is_empty() {
            let operation = MeshOperation::Delete {
                elements: self.selected_faces.iter().copied().collect(),
            };
            self.operation_history.push(operation);

            // TODO: 实现实际的面删除逻辑
            self.selected_faces.clear();
        }
    }

    /// 桥接边
    pub fn bridge_edges(&mut self) {
        if self.selected_edges.len() >= 2 {
            let operation = MeshOperation::Bridge {
                edges: self.selected_edges.iter().copied().collect(),
            };
            self.operation_history.push(operation);

            // TODO: 实现实际的边桥接逻辑
        }
    }

    /// 应用变换到选中的元素
    pub fn apply_transform(&mut self, transform: Mat4) {
        if !self.selected_vertices.is_empty() {
            let operation = MeshOperation::VertexTransform {
                vertices: self.selected_vertices.iter().copied().collect(),
                transform,
            };
            self.operation_history.push(operation);

            // 应用到顶点
            if let Some(mesh) = &mut self.current_mesh {
                for &vertex_id in &self.selected_vertices {
                    let idx = vertex_id as usize;
                    if idx < mesh.vertices.len() {
                        let pos = Vec3::from_array(mesh.vertices[idx].pos);
                        let new_pos = transform.transform_point3(pos);
                        mesh.vertices[idx].pos = new_pos.to_array();
                    }
                }
            }

            // 应用对称
            if self.symmetry_enabled {
                // TODO: 实现对称复制逻辑
            }
        }
    }

    /// 获取操作历史
    pub fn get_operations(&self) -> &[MeshOperation] {
        &self.operation_history
    }

    /// 清除操作历史
    pub fn clear_history(&mut self) {
        self.operation_history.clear();
    }

    /// 撤销上一个操作
    pub fn undo(&mut self) {
        // TODO: 实现撤销逻辑
        if !self.operation_history.is_empty() {
            self.operation_history.pop();
        }
    }
}

impl Default for MeshEditor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_editor_creation() {
        let editor = MeshEditor::new();
        assert_eq!(editor.edit_mode, EditMode::Vertex);
        assert!(editor.selected_vertices.is_empty());
    }

    #[test]
    fn test_editable_mesh_creation() {
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

        let mesh = EditableMesh::new(vertices, indices);
        assert_eq!(mesh.vertices.len(), 3);
        assert_eq!(mesh.face_normals.len(), 1);
    }

    #[test]
    fn test_selection() {
        let mut editor = MeshEditor::new();
        editor.selected_vertices.insert(0);
        editor.selected_vertices.insert(1);
        assert_eq!(editor.selected_vertices.len(), 2);

        editor.clear_selection();
        assert!(editor.selected_vertices.is_empty());
    }

    #[test]
    fn test_transform() {
        let mut editor = MeshEditor::new();
        editor.selected_vertices.insert(0);

        let transform = Mat4::from_translation(Vec3::new(1.0, 0.0, 0.0));
        editor.apply_transform(transform);

        assert_eq!(editor.operation_history.len(), 1);
    }
}
