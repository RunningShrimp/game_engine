//! # Mesh System Tests
//!
//! 测试网格系统的基础功能。

use game_engine::render::mesh::{Mesh, MeshVertex, PrimitiveTopology};
use glam::Vec3;

#[test]
fn test_mesh_creation() {
    let mesh = Mesh::new(PrimitiveTopology::TriangleList);

    assert_eq!(mesh.vertex_count(), 0);
    assert_eq!(mesh.index_count(), 0);
}

#[test]
fn test_mesh_add_vertices() {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    let vertices = vec![
        MeshVertex {
            position: Vec3::new(-0.5, -0.5, 0.0),
            normal: Vec3::new(0.0, 0.0, 1.0),
            uv: [0.0, 0.0],
        },
        MeshVertex {
            position: Vec3::new(0.5, -0.5, 0.0),
            normal: Vec3::new(0.0, 0.0, 1.0),
            uv: [1.0, 0.0],
        },
        MeshVertex {
            position: Vec3::new(0.0, 0.5, 0.0),
            normal: Vec3::new(0.0, 0.0, 1.0),
            uv: [0.5, 1.0],
        },
    ];

    mesh.set_vertices(vertices.clone());

    assert_eq!(mesh.vertex_count(), 3);
}

#[test]
fn test_mesh_add_indices() {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    let indices = vec![0u16, 1, 2];
    mesh.set_indices(indices);

    assert_eq!(mesh.index_count(), 3);
}

#[test]
fn test_mesh_triangle_topology() {
    let mesh = Mesh::new(PrimitiveTopology::TriangleList);

    // 应该是TriangleList拓扑
    assert_eq!(std::mem::discriminant(&PrimitiveTopology::TriangleList),
               std::mem::discriminant(&mesh.topology()));
}

#[test]
fn test_mesh_line_topology() {
    let mesh = Mesh::new(PrimitiveTopology::LineList);

    assert_eq!(std::mem::discriminant(&PrimitiveTopology::LineList),
               std::mem::discriminant(&mesh.topology()));
}

#[test]
fn test_mesh_clear() {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    let vertices = vec![
        MeshVertex {
            position: Vec3::new(0.0, 0.0, 0.0),
            normal: Vec3::new(0.0, 0.0, 1.0),
            uv: [0.0, 0.0],
        },
    ];

    mesh.set_vertices(vertices);
    mesh.clear();

    assert_eq!(mesh.vertex_count(), 0);
    assert_eq!(mesh.index_count(), 0);
}

#[test]
fn test_mesh_compute_bounds() {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    let vertices = vec![
        MeshVertex {
            position: Vec3::new(-1.0, -1.0, -1.0),
            normal: Vec3::ZERO,
            uv: [0.0, 0.0],
        },
        MeshVertex {
            position: Vec3::new(1.0, 1.0, 1.0),
            normal: Vec3::ZERO,
            uv: [0.0, 0.0],
        },
    ];

    mesh.set_vertices(vertices);

    // 计算边界框
    let bounds = mesh.compute_bounds();

    assert_eq!(bounds.min, Vec3::new(-1.0, -1.0, -1.0));
    assert_eq!(bounds.max, Vec3::new(1.0, 1.0, 1.0));
}
