//! 导航网格测试
//!
//! 测试导航网格生成和路径规划功能。

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::ai::ColliderGeometry;
    use glam::Vec3;

    #[test]
    fn test_nav_mesh_config_default() {
        let config = NavMeshConfig::default();
        // Test that default config exists
        assert!(config.voxel_size >= 0.0);
    }

    #[test]
    fn test_nav_mesh_config_custom() {
        let config = NavMeshConfig {
            voxel_size: 1.0,
            max_slope: 45.0,
            agent_height: 1.8,
            agent_radius: 0.5,
            min_region_size: 0.1,
            max_edge_length: 5.0,
            enhanced: Default::default(),
        };
        assert_eq!(config.voxel_size, 1.0);
        assert_eq!(config.max_slope, 45.0);
        assert_eq!(config.agent_height, 1.8);
        assert_eq!(config.agent_radius, 0.5);
    }

    #[test]
    fn test_nav_polygon_creation() {
        let positions = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(10.0, 0.0, 0.0),
            Vec3::new(10.0, 0.0, 10.0),
            Vec3::new(0.0, 0.0, 10.0),
        ];
        let vertices = vec![0, 1, 2, 3];

        let polygon = NavPolygon::new(vertices, &positions);
        assert_eq!(polygon.vertices.len(), 4);
    }

    #[test]
    fn test_nav_polygon_center() {
        let positions = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(10.0, 0.0, 0.0),
            Vec3::new(5.0, 0.0, 10.0),
        ];
        let vertices = vec![0, 1, 2];

        let polygon = NavPolygon::new(vertices, &positions);
        let center = polygon.center;
        assert!(center.x > 0.0 && center.x < 10.0);
        assert_eq!(center.y, 0.0);
        assert!(center.z > 0.0 && center.z < 10.0);
    }

    #[test]
    fn test_nav_mesh_generator_creation() {
        let _generator = NavMeshGenerator::new();
        // Generator created successfully
    }

    #[test]
    fn test_nav_mesh_generation() {
        let generator = NavMeshGenerator::new();

        // Create simple geometry
        let vertices = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(10.0, 0.0, 0.0),
            Vec3::new(10.0, 0.0, 10.0),
            Vec3::new(0.0, 0.0, 10.0),
        ];

        let result = generator.generate_from_vertices(&vertices);
        assert!(result.is_ok());
    }

    #[test]
    fn test_nav_mesh_error_invalid_geometry() {
        let generator = NavMeshGenerator::new();

        // Invalid geometry (less than 3 vertices)
        let vertices = vec![Vec3::new(0.0, 0.0, 0.0), Vec3::new(10.0, 0.0, 0.0)];

        let result = generator.generate_from_vertices(&vertices);
        assert!(result.is_err());
    }

    #[test]
    fn test_collider_geometry_creation() {
        use crate::ai::navmesh::ColliderGeometry;
        let _geometry = ColliderGeometry {
            vertices: vec![
                Vec3::new(-1.0, -1.0, -1.0),
                Vec3::new(1.0, -1.0, -1.0),
                Vec3::new(1.0, 1.0, -1.0),
                Vec3::new(-1.0, 1.0, -1.0),
            ],
            indices: vec![0, 1, 2, 0, 2, 3],
            is_walkable: true,
        };
        // Geometry created successfully
    }

    #[test]
    fn test_nav_mesh_with_obstacles() {
        let generator = NavMeshGenerator::new();

        // Create walkable surface
        let vertices = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(20.0, 0.0, 0.0),
            Vec3::new(20.0, 0.0, 20.0),
            Vec3::new(0.0, 0.0, 20.0),
        ];

        let nav_mesh = generator.generate_from_vertices(&vertices);
        assert!(nav_mesh.is_ok());
    }

    #[test]
    fn test_nav_mesh_find_path() {
        let generator = NavMeshGenerator::new();

        let vertices = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(20.0, 0.0, 0.0),
            Vec3::new(20.0, 0.0, 20.0),
            Vec3::new(0.0, 0.0, 20.0),
        ];

        let nav_mesh = generator.generate_from_vertices(&vertices).unwrap();
        let path = nav_mesh.find_path(Vec3::ZERO, Vec3::new(15.0, 0.0, 15.0));
        assert!(path.is_ok());
    }

    #[test]
    fn test_nav_mesh_find_nearest_polygon() {
        let generator = NavMeshGenerator::new();

        let vertices = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(10.0, 0.0, 0.0),
            Vec3::new(10.0, 0.0, 10.0),
            Vec3::new(0.0, 0.0, 10.0),
        ];

        let nav_mesh = generator.generate_from_vertices(&vertices).unwrap();
        let nearest = nav_mesh.find_nearest_polygon(Vec3::new(5.0, 0.0, 5.0));
        assert!(nearest.is_some());
    }

    #[test]
    fn test_nav_mesh_raycast() {
        let generator = NavMeshGenerator::new();

        let vertices = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(10.0, 0.0, 0.0),
            Vec3::new(10.0, 0.0, 10.0),
            Vec3::new(0.0, 0.0, 10.0),
        ];

        let nav_mesh = generator.generate_from_vertices(&vertices).unwrap();
        let hit = nav_mesh.raycast(Vec3::new(5.0, 1.0, 5.0), Vec3::new(0.0, -1.0, 0.0));
        assert!(hit.is_some());
    }

    #[test]
    fn test_nav_mesh_bounds() {
        let generator = NavMeshGenerator::new();

        let vertices = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(10.0, 0.0, 0.0),
            Vec3::new(10.0, 0.0, 10.0),
            Vec3::new(0.0, 0.0, 10.0),
        ];

        let nav_mesh = generator.generate_from_vertices(&vertices).unwrap();
        let bounds = nav_mesh.bounds();
        assert!(bounds.0.x <= 0.0);
        assert!(bounds.1.x >= 10.0);
    }

    #[test]
    fn test_nav_polygon_contains_point() {
        let positions = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(10.0, 0.0, 0.0),
            Vec3::new(10.0, 0.0, 10.0),
            Vec3::new(0.0, 0.0, 10.0),
        ];
        let vertices = vec![0, 1, 2, 3];

        let polygon = NavPolygon::new(vertices, &positions);
        assert!(polygon.contains_point(Vec3::new(5.0, 0.0, 5.0), &positions));
        assert!(!polygon.contains_point(Vec3::new(15.0, 0.0, 5.0), &positions));
    }

    #[test]
    fn test_nav_mesh_clear() {
        let generator = NavMeshGenerator::new();

        let vertices = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(10.0, 0.0, 0.0),
            Vec3::new(10.0, 0.0, 10.0),
            Vec3::new(0.0, 0.0, 10.0),
        ];

        let mut nav_mesh = generator.generate_from_vertices(&vertices).unwrap();
        assert!(nav_mesh.polygons.len() > 0);

        nav_mesh.clear();
        assert_eq!(nav_mesh.polygons.len(), 0);
    }
}
