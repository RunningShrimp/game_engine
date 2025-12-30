//! 寻路系统测试
//!
//! 测试A*寻路算法和导航网格功能。

#[cfg(test)]
mod tests {
    use super::super::*;
    use glam::Vec3;

    #[test]
    fn test_path_node_creation() {
        let node = PathNode::new(0, Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(node.id, 0);
        assert_eq!(node.position, Vec3::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn test_path_connection_creation() {
        let conn = PathConnection {
            from: 0,
            to: 1,
            cost: 10.0,
        };
        assert_eq!(conn.from, 0);
        assert_eq!(conn.to, 1);
        assert_eq!(conn.cost, 10.0);
    }

    #[test]
    fn test_navigation_mesh_creation() {
        let nav_mesh = NavigationMesh::new();
        assert_eq!(nav_mesh.nodes.len(), 0);
        assert_eq!(nav_mesh.connections.len(), 0);
    }

    #[test]
    fn test_add_node_to_navmesh() {
        let mut nav_mesh = NavigationMesh::new();
        let node = PathNode::new(0, Vec3::new(0.0, 0.0, 0.0));
        nav_mesh.add_node(node);
        assert_eq!(nav_mesh.nodes.len(), 1);
    }

    #[test]
    fn test_add_connection_to_navmesh() {
        let mut nav_mesh = NavigationMesh::new();
        nav_mesh.add_node(PathNode::new(0, Vec3::ZERO));
        nav_mesh.add_node(PathNode::new(1, Vec3::new(10.0, 0.0, 0.0)));

        let conn = PathConnection {
            from: 0,
            to: 1,
            cost: 10.0,
        };
        nav_mesh.add_connection(conn);
        assert_eq!(nav_mesh.connections.len(), 1);
    }

    #[test]
    fn test_find_path_simple() {
        let mut nav_mesh = NavigationMesh::new();
        nav_mesh.add_node(PathNode::new(0, Vec3::ZERO));
        nav_mesh.add_node(PathNode::new(1, Vec3::new(10.0, 0.0, 0.0)));

        let conn = PathConnection {
            from: 0,
            to: 1,
            cost: 10.0,
        };
        nav_mesh.add_connection(conn);

        let path = nav_mesh.find_path(Vec3::ZERO, Vec3::new(10.0, 0.0, 0.0));
        assert!(path.is_some());
    }

    #[test]
    fn test_pathfinding_request() {
        let request = PathfindingRequest {
            request_id: 1,
            start: Vec3::ZERO,
            end: Vec3::new(10.0, 0.0, 0.0),
        };
        assert_eq!(request.start, Vec3::ZERO);
        assert_eq!(request.end, Vec3::new(10.0, 0.0, 0.0));
    }

    #[test]
    fn test_pathfinding_result() {
        let path = vec![
            Vec3::ZERO,
            Vec3::new(5.0, 0.0, 0.0),
            Vec3::new(10.0, 0.0, 0.0),
        ];
        let result = PathfindingResult {
            request_id: 1,
            path: Some(path),
        };
        assert!(result.path.is_some());
        assert_eq!(result.path.as_ref().unwrap().len(), 3);
    }

    #[test]
    fn test_pathfinding_service_creation() {
        let nav_mesh = NavigationMesh::new();
        let service = PathfindingService::new(nav_mesh);
        assert_eq!(service.nav_mesh.nodes.len(), 0);
    }

    #[test]
    fn test_empty_path_returns_none() {
        let nav_mesh = NavigationMesh::new();
        let path = nav_mesh.find_path(Vec3::ZERO, Vec3::new(10.0, 0.0, 0.0));
        assert!(path.is_none());
    }

    #[test]
    fn test_path_node_with_neighbors() {
        let node = PathNode::new(0, Vec3::ZERO);
        assert_eq!(node.id, 0);
        assert_eq!(node.position, Vec3::ZERO);
    }

    #[test]
    fn test_connection_cost() {
        let conn = PathConnection {
            from: 0,
            to: 1,
            cost: 15.5,
        };
        assert_eq!(conn.cost, 15.5);
    }

    #[test]
    fn test_multiple_nodes_navmesh() {
        let mut nav_mesh = NavigationMesh::new();
        for i in 0..5 {
            nav_mesh.add_node(PathNode::new(i, Vec3::new(i as f32 * 10.0, 0.0, 0.0)));
        }
        assert_eq!(nav_mesh.nodes.len(), 5);
    }

    #[test]
    fn test_navmesh_clear() {
        let mut nav_mesh = NavigationMesh::new();
        nav_mesh.add_node(PathNode::new(0, Vec3::ZERO));
        nav_mesh.add_node(PathNode::new(1, Vec3::new(10.0, 0.0, 0.0)));
        assert_eq!(nav_mesh.nodes.len(), 2);

        nav_mesh.clear();
        assert_eq!(nav_mesh.nodes.len(), 0);
        assert_eq!(nav_mesh.connections.len(), 0);
    }
}
