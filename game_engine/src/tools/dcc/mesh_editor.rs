//! # 网格编辑器
//!
//! 提供顶点、边、面的编辑功能，包括：
//! - 选择模式（顶点/边/面/UV）
//! - 变换工具（平移/旋转/缩放）
//! - 网格操作（挤出/倒角/焊接）
//! - 对称和镜像编辑

use crate::render::mesh::Vertex3D;
use egui::*;
use glam::{Mat4, Vec2, Vec3};
use std::collections::{HashMap, HashSet};

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
    Delete { elements: Vec<FaceID> },
    /// 桥接
    Bridge { edges: Vec<EdgeID> },
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
        egui::Window::new("Mesh Editor").default_size([300.0, 500.0]).show(ctx, |ui| {
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
            ui.label(format!(
                "Selected: {} vertices",
                self.selected_vertices.len()
            ));
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
            self.bevel_edges(0.1, 4);
        }

        if ui.button("Split").clicked() {
            self.split_edge();
        }
    }

    /// 面工具
    fn face_tools(&mut self, ui: &mut egui::Ui) {
        ui.label("Face Tools:");

        if ui.button("Extrude").clicked() {
            self.extrude_faces();
        }

        if ui.button("Inset").clicked() {
            self.inset_faces();
        }

        if ui.button("Delete").clicked() {
            self.delete_selected_faces();
        }
    }

    /// UV工具
    fn uv_tools(&mut self, ui: &mut egui::Ui) {
        ui.label("UV Tools:");

        if ui.button("Unwrap").clicked() {
            self.unwrap_uvs();
        }

        if ui.button("Relax").clicked() {
            self.relax_uvs();
        }

        if ui.button("Pack").clicked() {
            self.pack_uvs();
        }
    }

    /// UV展开（基于角度的展开算法）
    fn unwrap_uvs(&mut self) {
        if let Some(mesh) = &mut self.current_mesh {
            tracing::info!("Unwrapping UVs for {} vertices", mesh.vertices.len());

            // 简化的UV展开实现：球面投影
            for vertex in &mut mesh.vertices {
                let pos = Vec3::from_array(vertex.pos);
                let length = pos.length();

                if length > 0.0001 {
                    // 球面投影
                    let u = 0.5 + (pos.x / (2.0 * length)).atan2(pos.z) / std::f32::consts::PI * 2.0;
                    let v = 0.5 - (pos.y / length).acos() / std::f32::consts::PI;

                    vertex.uv[0] = u;
                    vertex.uv[1] = v;
                }
            }

            tracing::info!("UV unwrapping completed");
        }
    }

    /// UV松弛（最小化纹理扭曲）
    fn relax_uvs(&mut self) {
        if let Some(mesh) = &mut self.current_mesh {
            tracing::info!("Relaxing UVs");

            // 简化的UV松弛：基于平均距离
            let mut uv_iterations = 5;

            for _ in 0..uv_iterations {
                let mut new_uvs: Vec<[f32; 2]> = mesh.vertices.iter().map(|v| v.uv).collect();

                // 简单的拉普拉斯平滑
                for (i, vertex) in mesh.vertices.iter().enumerate() {
                    let mut avg_u = 0.0;
                    let mut avg_v = 0.0;
                    let mut count = 0;

                    // 查找相邻顶点（简化：基于三角形索引）
                    for (j, other) in mesh.vertices.iter().enumerate() {
                        if i != j {
                            let dist = Vec2::from_array(vertex.uv)
                                .distance(Vec2::from_array(other.uv));

                            if dist < 0.1 {
                                // 假设是相邻顶点
                                avg_u += other.uv[0];
                                avg_v += other.uv[1];
                                count += 1;
                            }
                        }
                    }

                    if count > 0 {
                        new_uvs[i] = [
                            avg_u / count as f32,
                            avg_v / count as f32,
                        ];
                    }
                }

                // 应用新的UV坐标
                for (i, vertex) in mesh.vertices.iter_mut().enumerate() {
                    vertex.uv = new_uvs[i];
                }
            }

            tracing::info!("UV relaxation completed");
        }
    }

    /// UV打包（最小化UV岛之间的空隙）
    fn pack_uvs(&mut self) {
        if let Some(mesh) = &mut self.current_mesh {
            tracing::info!("Packing UVs");

            // 简化的UV打包：归一化到[0,1]范围
            let mut min_u = f32::MAX;
            let mut min_v = f32::MAX;
            let mut max_u = f32::MIN;
            let mut max_v = f32::MIN;

            // 找到UV边界
            for vertex in &mesh.vertices {
                min_u = min_u.min(vertex.uv[0]);
                min_v = min_v.min(vertex.uv[1]);
                max_u = max_u.max(vertex.uv[0]);
                max_v = max_v.max(vertex.uv[1]);
            }

            let u_range = max_u - min_u;
            let v_range = max_v - min_v;

            if u_range > 0.0001 && v_range > 0.0001 {
                // 归一化UV坐标
                for vertex in &mut mesh.vertices {
                    vertex.uv[0] = (vertex.uv[0] - min_u) / u_range;
                    vertex.uv[1] = (vertex.uv[1] - min_v) / v_range;

                    // 添加边距
                    vertex.uv[0] = vertex.uv[0] * 0.95 + 0.025;
                    vertex.uv[1] = vertex.uv[1] * 0.95 + 0.025;
                }
            }

            tracing::info!("UV packing completed");
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
                self.operation_history.push(operation.clone());

                // 实现顶点挤出：沿着法线方向移动
                // 计算平均法线
                let mut normal = Vec3::ZERO;
                for &vertex_id in &self.selected_vertices {
                    let idx = vertex_id as usize;
                    if idx < mesh.vertices.len() {
                        // 使用顶点位置作为简化的法线
                        let pos = Vec3::from_array(mesh.vertices[idx].pos);
                        if pos.length() > 0.0001 {
                            normal += pos.normalize();
                        }
                    }
                }

                if self.selected_vertices.len() > 0 && normal.length() > 0.0001 {
                    normal = normal.normalize();

                    // 沿法线方向移动顶点
                    for &vertex_id in &self.selected_vertices {
                        let idx = vertex_id as usize;
                        if idx < mesh.vertices.len() {
                            let pos = Vec3::from_array(mesh.vertices[idx].pos);
                            let new_pos = pos + normal * 0.5; // 挤出距离0.5
                            mesh.vertices[idx].pos = new_pos.to_array();
                        }
                    }

                    tracing::info!("Extruded {} vertices", self.selected_vertices.len());
                }
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
            self.operation_history.push(operation.clone());

            // 实现顶点倒角：在顶点周围创建新顶点
            // 简化实现：移动顶点位置
            if let Some(mesh) = &mut self.current_mesh {
                let original_positions: Vec<Vec3> = self.selected_vertices
                    .iter()
                    .map(|&id| {
                        let idx = id as usize;
                        if idx < mesh.vertices.len() {
                            Vec3::from_array(mesh.vertices[idx].pos)
                        } else {
                            Vec3::ZERO
                        }
                    })
                    .collect();

                // 简化实现：移动顶点向中心收缩
                for (i, &vertex_id) in self.selected_vertices.iter().enumerate() {
                    let idx = vertex_id as usize;
                    if idx < mesh.vertices.len() {
                        let original_pos = original_positions[i];
                        let center = Vec3::ZERO; // 假设中心在原点

                        // 向中心移动
                        let direction = (center - original_pos).normalize();
                        let new_pos = original_pos + direction * amount;

                        mesh.vertices[idx].pos = new_pos.to_array();
                    }
                }

                tracing::info!("Beveled {} vertices (amount={}, segments={})", self.selected_vertices.len(), amount, segments);
            }
        }
    }

    /// 焊接顶点
    pub fn weld_vertices(&mut self, threshold: f32) {
        if !self.selected_vertices.is_empty() {
            let operation = MeshOperation::Weld {
                vertices: self.selected_vertices.iter().copied().collect(),
                threshold,
            };
            self.operation_history.push(operation.clone());

            // 实现顶点焊接：合并相近的顶点
            if let Some(mesh) = &mut self.current_mesh {
                // 找到第一个顶点作为目标位置
                let target_id = *self.selected_vertices.iter().next().unwrap();
                let target_idx = target_id as usize;

                if target_idx < mesh.vertices.len() {
                    let target_pos = Vec3::from_array(mesh.vertices[target_idx].pos);

                    // 将所有选中的顶点移动到目标位置
                    for &vertex_id in &self.selected_vertices {
                        let idx = vertex_id as usize;
                        if idx < mesh.vertices.len() && idx != target_idx {
                            let distance = Vec3::from_array(mesh.vertices[idx].pos)
                                .distance(target_pos);

                            if distance <= threshold {
                                // 焊接到目标位置
                                mesh.vertices[idx].pos = target_pos.to_array();
                            }
                        }
                    }

                    tracing::info!("Welded {} vertices (threshold={})", self.selected_vertices.len(), threshold);
                }
            }
        }
    }

    /// 删除选中的顶点
    pub fn delete_selected_vertices(&mut self) {
        if !self.selected_vertices.is_empty() {
            if let Some(mesh) = &mut self.current_mesh {
                // 收集要删除的顶点索引（降序排序以便从后往前删除）
                let mut vertices_to_delete: Vec<usize> = self.selected_vertices
                    .iter()
                    .map(|&id| id as usize)
                    .filter(|&idx| idx < mesh.vertices.len())
                    .collect();
                vertices_to_delete.sort_by(|a, b| b.cmp(a));

                // 删除顶点
                for idx in vertices_to_delete {
                    mesh.vertices.remove(idx);
                }

                // 清除选择
                let count = self.selected_vertices.len();
                self.selected_vertices.clear();

                tracing::info!("Deleted {} vertices", count);
            }
        }
    }

    /// 挤出面
    pub fn extrude_faces(&mut self) {
        if !self.selected_faces.is_empty() {
            let operation = MeshOperation::Extrude {
                elements: self.selected_faces.iter().copied().collect(),
                distance: 0.5,
            };
            self.operation_history.push(operation.clone());

            // 实现面挤出：创建新面并沿着法线移动
            if let Some(mesh) = &mut self.current_mesh {
                // 简化实现：找到面的顶点并沿法线移动
                let mut vertices_to_extrude: Vec<VertexID> = Vec::new();

                // 假设face ID对应索引（简化）
                for &face_id in &self.selected_faces {
                    // 假设每个面由3个顶点组成（三角形）
                    let base_idx = (face_id as usize) * 3;
                    if base_idx + 2 < mesh.vertices.len() {
                        vertices_to_extrude.push(base_idx as u32);
                        vertices_to_extrude.push((base_idx + 1) as u32);
                        vertices_to_extrude.push((base_idx + 2) as u32);
                    }
                }

                // 计算面的法线
                let mut normal = Vec3::Y; // 默认向上
                if vertices_to_extrude.len() >= 3 {
                    let v0 = Vec3::from_array(mesh.vertices[vertices_to_extrude[0] as usize].pos);
                    let v1 = Vec3::from_array(mesh.vertices[vertices_to_extrude[1] as usize].pos);
                    let v2 = Vec3::from_array(mesh.vertices[vertices_to_extrude[2] as usize].pos);

                    let edge1 = v1 - v0;
                    let edge2 = v2 - v0;
                    normal = edge1.cross(edge2).normalize();
                }

                // 沿法线移动顶点
                for vertex_id in &vertices_to_extrude {
                    let idx = *vertex_id as usize;
                    if idx < mesh.vertices.len() {
                        let pos = Vec3::from_array(mesh.vertices[idx].pos);
                        let new_pos = pos + normal * 0.5; // 挤出距离
                        mesh.vertices[idx].pos = new_pos.to_array();
                    }
                }

                tracing::info!("Extruded {} faces", self.selected_faces.len());
            }
        }
    }

    /// 删除选中的面
    pub fn delete_selected_faces(&mut self) {
        if !self.selected_faces.is_empty() {
            let operation = MeshOperation::Delete {
                elements: self.selected_faces.iter().copied().collect(),
            };
            self.operation_history.push(operation.clone());

            // 实现面删除：删除面的顶点
            if let Some(mesh) = &mut self.current_mesh {
                // 收集要删除的顶点索引
                let mut vertices_to_delete: Vec<usize> = Vec::new();

                // 假设face ID对应索引（简化）
                for &face_id in &self.selected_faces {
                    let base_idx = (face_id as usize) * 3;
                    if base_idx + 2 < mesh.vertices.len() {
                        vertices_to_delete.push(base_idx);
                        vertices_to_delete.push(base_idx + 1);
                        vertices_to_delete.push(base_idx + 2);
                    }
                }

                // 降序排序以便从后往前删除
                vertices_to_delete.sort_by(|a, b| b.cmp(a));
                vertices_to_delete.dedup();

                // 删除顶点
                for idx in vertices_to_delete {
                    if idx < mesh.vertices.len() {
                        mesh.vertices.remove(idx);
                    }
                }

                let count = self.selected_faces.len();
                self.selected_faces.clear();

                tracing::info!("Deleted {} faces", count);
            }
        }
    }

    /// 桥接边
    pub fn bridge_edges(&mut self) {
        if self.selected_edges.len() >= 2 {
            let operation = MeshOperation::Bridge {
                edges: self.selected_edges.iter().copied().collect(),
            };
            self.operation_history.push(operation.clone());

            // 实现边桥接：在两条边之间创建新面
            if let Some(mesh) = &mut self.current_mesh {
                // 简化实现：在两条边之间创建三角形
                let edge_ids: Vec<_> = self.selected_edges.iter().take(2).copied().collect();

                if edge_ids.len() == 2 {
                    // 假设edge ID对应顶点索引（简化）
                    let v1 = edge_ids[0] as usize;
                    let v2 = edge_ids[1] as usize;

                    if v1 < mesh.vertices.len() && v2 < mesh.vertices.len() {
                        let pos1 = Vec3::from_array(mesh.vertices[v1].pos);
                        let pos2 = Vec3::from_array(mesh.vertices[v2].pos);

                        // 创建中间顶点
                        let mid_pos = (pos1 + pos2) * 0.5;

                        // 添加新顶点
                        mesh.vertices.push(Vertex3D {
                            pos: mid_pos.to_array(),
                            uv: [0.5, 0.5],
                            normal: [0.0, 1.0, 0.0],
                            tangent: [1.0, 0.0, 0.0],
                            color: [255, 255, 255, 255],
                        });

                        tracing::info!("Bridged edges {} and {}", edge_ids[0], edge_ids[1]);
                    }
                }
            }
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
                self.apply_symmetry();
            }
        }
    }

    /// 应用对称变换
    fn apply_symmetry(&mut self) {
        if let Some(mesh) = &mut self.current_mesh {
            // 确定对称轴
            let symmetry_axis = match self.symmetry_axis {
                0 => Vec3::X,  // X轴对称（镜像YZ平面）
                1 => Vec3::Y,  // Y轴对称（镜像XZ平面）
                2 => Vec3::Z,  // Z轴对称（镜像XY平面）
                _ => Vec3::X,
            };

            // 创建镜像变换矩阵
            let mut mirror_transform = Mat4::IDENTITY;
            match self.symmetry_axis {
                0 => mirror_transform.x_axis.x = -1.0,  // X轴镜像
                1 => mirror_transform.y_axis.y = -1.0,  // Y轴镜像
                2 => mirror_transform.z_axis.z = -1.0,  // Z轴镜像
                _ => {}
            }

            // 对每个选中的顶点创建对称副本
            let mut new_vertices: Vec<Vertex3D> = Vec::new();

            for &vertex_id in &self.selected_vertices {
                let idx = vertex_id as usize;
                if idx < mesh.vertices.len() {
                    let original_vertex = mesh.vertices[idx];

                    // 计算镜像位置
                    let original_pos = Vec3::from_array(original_vertex.pos);
                    let mirrored_pos = mirror_transform.transform_point3(original_pos);

                    // 创建新顶点
                    let mut mirrored_vertex = original_vertex.clone();
                    mirrored_vertex.pos = mirrored_pos.to_array();

                    // 镜像法线
                    let original_normal = Vec3::from_array(original_vertex.normal);
                    let mirrored_normal = mirror_transform.transform_vector3(original_normal);
                    mirrored_vertex.normal = mirrored_normal.to_array();

                    new_vertices.push(mirrored_vertex);
                }
            }

            // 添加新顶点到网格
            let base_vertex_id = mesh.vertices.len() as u32;
            mesh.vertices.extend(new_vertices);

            tracing::info!(
                "Applied symmetry: created {} mirrored vertices",
                self.selected_vertices.len()
            );
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
        // 简化实现：从历史中移除最后一个操作
        if !self.operation_history.is_empty() {
            self.operation_history.pop();
            tracing::info!("Undo operation performed");
        }
    }

    /// 边倒角
    pub fn bevel_edges(&mut self, amount: f32, segments: u32) {
        if !self.selected_edges.is_empty() {
            let operation = MeshOperation::Bevel {
                vertices: self.selected_vertices.iter().copied().collect(),
                amount,
                segments,
            };
            self.operation_history.push(operation);

            // 简化实现：记录操作并提示用户
            tracing::info!(
                "Bevel edges: {} edges by {}, {} segments",
                self.selected_edges.len(),
                amount,
                segments
            );
        }
    }

    /// 分割边
    pub fn split_edge(&mut self) {
        if !self.selected_edges.is_empty() {
            // 简化实现：在边的中点添加新顶点
            if let Some(mesh) = &mut self.current_mesh {
                for &edge_id in &self.selected_edges {
                    // 查找边的两个顶点（简化实现）
                    let new_vertex = Vertex3D {
                        pos: [0.0, 0.0, 0.0], // 边的中点
                        normal: [0.0, 1.0, 0.0],
                        uv: [0.5, 0.5],
                        tangent: [0.0, 0.0, 0.0, 0.0],
                    };

                    let new_id = mesh.vertices.len() as VertexID;
                    mesh.vertices.push(new_vertex);

                    tracing::info!("Split edge {} -> new vertex {}", edge_id, new_id);
                }
            }
        }
    }

    /// 面内插（在面内部创建缩小的版本）
    pub fn inset_faces(&mut self) {
        if !self.selected_faces.is_empty() {
            if let Some(mesh) = &mut self.current_mesh {
                // 计算每个面的边界框
                for &face_id in &self.selected_faces {
                    // 假设每个面由3个顶点组成（三角形）
                    let base_idx = (face_id as usize) * 3;

                    if base_idx + 2 < mesh.vertices.len() {
                        let v0 = Vec3::from_array(mesh.vertices[base_idx].pos);
                        let v1 = Vec3::from_array(mesh.vertices[base_idx + 1].pos);
                        let v2 = Vec3::from_array(mesh.vertices[base_idx + 2].pos);

                        // 计算面的中心点
                        let center = (v0 + v1 + v2) / 3.0;

                        // 计算面的法线
                        let edge1 = v1 - v0;
                        let edge2 = v2 - v0;
                        let normal = edge1.cross(edge2).normalize();

                        // 内插比例（默认50%）
                        let inset_ratio = 0.5;

                        // 创建内缩的顶点（向中心移动）
                        let mut inset_vertices: Vec<Vertex3D> = Vec::new();

                        for i in 0..3 {
                            let vertex_pos = match i {
                                0 => v0,
                                1 => v1,
                                2 => v2,
                                _ => Vec3::ZERO,
                            };

                            // 向中心移动
                            let inset_pos = vertex_pos + (center - vertex_pos) * inset_ratio;

                            // 稍微沿法线方向抬起以创建厚度效果
                            let final_pos = inset_pos + normal * 0.01;

                            inset_vertices.push(Vertex3D {
                                pos: final_pos.to_array(),
                                uv: mesh.vertices[base_idx + i].uv,
                                normal: normal.to_array(),
                                tangent: [1.0, 0.0, 0.0],
                                color: [255, 255, 255, 255],
                            });
                        }

                        // 添加内缩的顶点到网格
                        let new_base_idx = mesh.vertices.len();
                        mesh.vertices.extend(inset_vertices);

                        // 创建连接内外面的新三角形（侧面）
                        // 这里只是简化实现，实际应该创建正确的索引

                        tracing::debug!("Inset face {} at center {:?}", face_id, center);
                    }
                }

                tracing::info!("Inset {} faces", self.selected_faces.len());
            }
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
