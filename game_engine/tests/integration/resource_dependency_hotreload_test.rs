//! 资源依赖和热重载集成测试
//!
//! 测试资源管理系统的完整功能，包括：
//! - 资源依赖管理
//! - 依赖加载顺序
//! - 循环依赖检测
//! - 资源热重载
//! - 依赖资源的热重载传播

use game_engine::resources::dependency_manager::{DependencyGraph, DependencyError, LoadState, ResourceDependency};
use game_engine::resources::hot_reload::{HotReloadManager, HotReloadEvent};
use game_engine::resources::unified_manager::UnifiedResourceManager;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::sleep;

// ============================================================================
// 资源依赖管理测试
// ============================================================================

#[tokio::test]
async fn test_dependency_graph_basic() {
    let mut graph = DependencyGraph::new();
    
    // 添加资源节点
    let resource1 = PathBuf::from("resource1.txt");
    let resource2 = PathBuf::from("resource2.txt");
    let resource3 = PathBuf::from("resource3.txt");
    
    graph.add_resource(resource1.clone());
    graph.add_resource(resource2.clone());
    graph.add_resource(resource3.clone());
    
    // 添加依赖关系：resource1 依赖 resource2
    let dep = ResourceDependency {
        path: resource2.clone(),
        dependency_type: "texture".to_string(),
        required: true,
    };
    
    assert!(graph.add_dependency(resource1.clone(), dep).is_ok());
    
    // 验证依赖关系
    let dependencies = graph.get_all_dependencies(&resource1);
    assert!(dependencies.contains(&resource2));
    
    // 验证反向依赖
    let dependents = graph.get_dependents(&resource2);
    assert!(dependents.contains(&resource1));
}

#[tokio::test]
async fn test_dependency_graph_load_order() {
    let mut graph = DependencyGraph::new();
    
    // 创建依赖链：A -> B -> C
    let a = PathBuf::from("a.txt");
    let b = PathBuf::from("b.txt");
    let c = PathBuf::from("c.txt");
    
    graph.add_resource(a.clone());
    graph.add_resource(b.clone());
    graph.add_resource(c.clone());
    
    // A 依赖 B
    graph.add_dependency(a.clone(), ResourceDependency {
        path: b.clone(),
        dependency_type: "texture".to_string(),
        required: true,
    }).unwrap();
    
    // B 依赖 C
    graph.add_dependency(b.clone(), ResourceDependency {
        path: c.clone(),
        dependency_type: "shader".to_string(),
        required: true,
    }).unwrap();
    
    // 获取加载顺序（拓扑排序）
    let load_order = graph.get_load_order().unwrap();
    
    // C 应该在 B 之前，B 应该在 A 之前
    let c_index = load_order.iter().position(|p| p == &c).unwrap();
    let b_index = load_order.iter().position(|p| p == &b).unwrap();
    let a_index = load_order.iter().position(|p| p == &a).unwrap();
    
    assert!(c_index < b_index);
    assert!(b_index < a_index);
}

#[tokio::test]
async fn test_dependency_graph_circular_dependency() {
    let mut graph = DependencyGraph::new();
    
    let a = PathBuf::from("a.txt");
    let b = PathBuf::from("b.txt");
    
    graph.add_resource(a.clone());
    graph.add_resource(b.clone());
    
    // A 依赖 B
    graph.add_dependency(a.clone(), ResourceDependency {
        path: b.clone(),
        dependency_type: "texture".to_string(),
        required: true,
    }).unwrap();
    
    // B 依赖 A（创建循环依赖）
    let result = graph.add_dependency(b.clone(), ResourceDependency {
        path: a.clone(),
        dependency_type: "shader".to_string(),
        required: true,
    });
    
    // 应该检测到循环依赖
    assert!(result.is_err());
    if let Err(DependencyError::CircularDependency(msg)) = result {
        assert!(msg.contains("circular") || msg.contains("cycle"));
    }
}

#[tokio::test]
async fn test_dependency_graph_load_state() {
    let mut graph = DependencyGraph::new();
    
    let resource = PathBuf::from("resource.txt");
    graph.add_resource(resource.clone());
    
    // 初始状态应该是 NotLoaded
    assert_eq!(graph.get_load_state(&resource), Some(LoadState::NotLoaded));
    
    // 设置加载状态
    graph.set_load_state(&resource, LoadState::Loading);
    assert_eq!(graph.get_load_state(&resource), Some(LoadState::Loading));
    
    graph.set_load_state(&resource, LoadState::Loaded);
    assert_eq!(graph.get_load_state(&resource), Some(LoadState::Loaded));
}

#[tokio::test]
async fn test_dependency_graph_can_load() {
    let mut graph = DependencyGraph::new();
    
    let resource = PathBuf::from("resource.txt");
    let dependency = PathBuf::from("dependency.txt");
    
    graph.add_resource(resource.clone());
    graph.add_resource(dependency.clone());
    
    // 添加必需依赖
    graph.add_dependency(resource.clone(), ResourceDependency {
        path: dependency.clone(),
        dependency_type: "texture".to_string(),
        required: true,
    }).unwrap();
    
    // 依赖未加载时，资源不能加载
    assert!(!graph.can_load(&resource));
    
    // 加载依赖
    graph.set_load_state(&dependency, LoadState::Loaded);
    
    // 依赖已加载时，资源可以加载
    assert!(graph.can_load(&resource));
}

// ============================================================================
// 统一资源管理器集成测试
// ============================================================================

#[tokio::test]
async fn test_unified_resource_manager_dependency_loading() {
    let manager = UnifiedResourceManager::new();
    
    // 创建依赖关系
    let resource = PathBuf::from("resource.txt");
    let dependency = PathBuf::from("dependency.txt");
    
    // 添加依赖关系
    let dep = ResourceDependency {
        path: dependency.clone(),
        dependency_type: "texture".to_string(),
        required: true,
    };
    
    // 注意：实际加载需要实现ResourceLoader，这里只测试依赖管理
    let result = manager.add_dependency(resource.clone(), dep);
    assert!(result.is_ok());
    
    // 验证依赖关系已添加
    let dependencies = manager.get_all_dependencies(&resource);
    assert!(dependencies.contains(&dependency));
}

#[tokio::test]
async fn test_unified_resource_manager_load_order() {
    let manager = UnifiedResourceManager::new();
    
    // 创建依赖链：A -> B -> C
    let a = PathBuf::from("a.txt");
    let b = PathBuf::from("b.txt");
    let c = PathBuf::from("c.txt");
    
    // 添加依赖关系
    manager.add_dependency(a.clone(), ResourceDependency {
        path: b.clone(),
        dependency_type: "texture".to_string(),
        required: true,
    }).unwrap();
    
    manager.add_dependency(b.clone(), ResourceDependency {
        path: c.clone(),
        dependency_type: "shader".to_string(),
        required: true,
    }).unwrap();
    
    // 获取加载顺序
    let load_order = manager.get_load_order().unwrap();
    
    // 验证顺序正确
    let c_index = load_order.iter().position(|p| p == &c).unwrap();
    let b_index = load_order.iter().position(|p| p == &b).unwrap();
    let a_index = load_order.iter().position(|p| p == &a).unwrap();
    
    assert!(c_index < b_index);
    assert!(b_index < a_index);
}

// ============================================================================
// 资源热重载测试
// ============================================================================

#[tokio::test]
async fn test_hot_reload_manager_creation() {
    let temp_dir = TempDir::new().unwrap();
    let watch_dir = temp_dir.path().to_path_buf();
    
    let dependency_graph = std::sync::Arc::new(std::sync::RwLock::new(DependencyGraph::new()));
    
    let manager = HotReloadManager::new(
        watch_dir.clone(),
        dependency_graph.clone(),
        Duration::from_millis(100),
    );
    
    assert!(manager.is_ok());
}

#[tokio::test]
async fn test_hot_reload_watch_resource() {
    let temp_dir = TempDir::new().unwrap();
    let watch_dir = temp_dir.path().to_path_buf();
    
    let dependency_graph = std::sync::Arc::new(std::sync::RwLock::new(DependencyGraph::new()));
    
    let manager = HotReloadManager::new(
        watch_dir.clone(),
        dependency_graph.clone(),
        Duration::from_millis(100),
    ).unwrap();
    
    // 创建测试文件
    let test_file = watch_dir.join("test.txt");
    tokio::fs::write(&test_file, b"initial content").await.unwrap();
    
    // 监视资源
    manager.watch_resource(test_file.clone()).await.unwrap();
    
    // 验证资源已被监视
    assert!(manager.is_watched(&test_file));
}

#[tokio::test]
async fn test_hot_reload_file_modification() {
    let temp_dir = TempDir::new().unwrap();
    let watch_dir = temp_dir.path().to_path_buf();
    
    let dependency_graph = std::sync::Arc::new(std::sync::RwLock::new(DependencyGraph::new()));
    
    let manager = HotReloadManager::new(
        watch_dir.clone(),
        dependency_graph.clone(),
        Duration::from_millis(100),
    ).unwrap();
    
    // 创建测试文件
    let test_file = watch_dir.join("test.txt");
    tokio::fs::write(&test_file, b"initial content").await.unwrap();
    
    // 监视资源
    manager.watch_resource(test_file.clone()).await.unwrap();
    
    // 修改文件
    tokio::fs::write(&test_file, b"modified content").await.unwrap();
    
    // 等待防抖延迟
    sleep(Duration::from_millis(200)).await;
    
    // 检查是否需要重新加载
    assert!(manager.needs_reload(&test_file));
}

#[tokio::test]
async fn test_hot_reload_dependency_propagation() {
    let temp_dir = TempDir::new().unwrap();
    let watch_dir = temp_dir.path().to_path_buf();
    
    let dependency_graph = std::sync::Arc::new(std::sync::RwLock::new(DependencyGraph::new()));
    
    // 创建依赖关系：resource 依赖 dependency
    let resource = watch_dir.join("resource.txt");
    let dependency = watch_dir.join("dependency.txt");
    
    {
        let mut graph = dependency_graph.write().unwrap();
        graph.add_resource(resource.clone());
        graph.add_resource(dependency.clone());
        graph.add_dependency(resource.clone(), ResourceDependency {
            path: dependency.clone(),
            dependency_type: "texture".to_string(),
            required: true,
        }).unwrap();
    }
    
    let manager = HotReloadManager::new(
        watch_dir.clone(),
        dependency_graph.clone(),
        Duration::from_millis(100),
    ).unwrap();
    
    // 创建文件
    tokio::fs::write(&resource, b"resource content").await.unwrap();
    tokio::fs::write(&dependency, b"dependency content").await.unwrap();
    
    // 监视两个资源
    manager.watch_resource(resource.clone()).await.unwrap();
    manager.watch_resource(dependency.clone()).await.unwrap();
    
    // 修改依赖资源
    tokio::fs::write(&dependency, b"modified dependency").await.unwrap();
    
    // 等待防抖延迟
    sleep(Duration::from_millis(200)).await;
    
    // 获取需要重新加载的资源列表（应该包括dependency和resource）
    let reload_targets = manager.get_reload_targets(&dependency);
    
    // 应该包含dependency本身和依赖它的resource
    assert!(reload_targets.contains(&dependency));
    assert!(reload_targets.contains(&resource));
}

#[tokio::test]
async fn test_hot_reload_event_handling() {
    let temp_dir = TempDir::new().unwrap();
    let watch_dir = temp_dir.path().to_path_buf();
    
    let dependency_graph = std::sync::Arc::new(std::sync::RwLock::new(DependencyGraph::new()));
    
    let manager = HotReloadManager::new(
        watch_dir.clone(),
        dependency_graph.clone(),
        Duration::from_millis(100),
    ).unwrap();
    
    // 创建测试文件
    let test_file = watch_dir.join("test.txt");
    tokio::fs::write(&test_file, b"initial content").await.unwrap();
    
    // 监视资源
    manager.watch_resource(test_file.clone()).await.unwrap();
    
    // 修改文件
    tokio::fs::write(&test_file, b"modified content").await.unwrap();
    
    // 等待防抖延迟
    sleep(Duration::from_millis(200)).await;
    
    // 尝试接收事件（注意：实际实现可能需要spawn任务来接收事件）
    // 这里只验证接口存在
    assert!(manager.needs_reload(&test_file));
}

// ============================================================================
// 完整工作流测试
// ============================================================================

#[tokio::test]
async fn test_complete_resource_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let watch_dir = temp_dir.path().to_path_buf();
    
    // 1. 创建资源管理器
    let manager = UnifiedResourceManager::new();
    
    // 2. 创建依赖图
    let resource1 = watch_dir.join("resource1.txt");
    let resource2 = watch_dir.join("resource2.txt");
    let resource3 = watch_dir.join("resource3.txt");
    
    // 创建文件
    tokio::fs::write(&resource1, b"resource1").await.unwrap();
    tokio::fs::write(&resource2, b"resource2").await.unwrap();
    tokio::fs::write(&resource3, b"resource3").await.unwrap();
    
    // 3. 设置依赖关系：resource1 -> resource2 -> resource3
    manager.add_dependency(resource1.clone(), ResourceDependency {
        path: resource2.clone(),
        dependency_type: "texture".to_string(),
        required: true,
    }).unwrap();
    
    manager.add_dependency(resource2.clone(), ResourceDependency {
        path: resource3.clone(),
        dependency_type: "shader".to_string(),
        required: true,
    }).unwrap();
    
    // 4. 获取加载顺序
    let load_order = manager.get_load_order().unwrap();
    assert_eq!(load_order.len(), 3);
    
    // 5. 创建热重载管理器
    let dependency_graph = manager.dependency_graph();
    let hot_reload = HotReloadManager::new(
        watch_dir.clone(),
        dependency_graph.clone(),
        Duration::from_millis(100),
    ).unwrap();
    
    // 6. 监视所有资源
    hot_reload.watch_resource(resource1.clone()).await.unwrap();
    hot_reload.watch_resource(resource2.clone()).await.unwrap();
    hot_reload.watch_resource(resource3.clone()).await.unwrap();
    
    // 7. 修改底层依赖
    tokio::fs::write(&resource3, b"modified resource3").await.unwrap();
    
    // 等待防抖延迟
    sleep(Duration::from_millis(200)).await;
    
    // 8. 验证热重载目标（应该包括所有依赖的资源）
    let reload_targets = hot_reload.get_reload_targets(&resource3);
    
    // 应该包含resource3本身和所有依赖它的资源
    assert!(reload_targets.contains(&resource3));
    assert!(reload_targets.contains(&resource2));
    assert!(reload_targets.contains(&resource1));
}

#[tokio::test]
async fn test_resource_dependency_complex_scenario() {
    let temp_dir = TempDir::new().unwrap();
    let watch_dir = temp_dir.path().to_path_buf();
    
    let manager = UnifiedResourceManager::new();
    
    // 创建复杂的依赖关系：
    // A -> B, C
    // B -> D
    // C -> D
    let a = watch_dir.join("a.txt");
    let b = watch_dir.join("b.txt");
    let c = watch_dir.join("c.txt");
    let d = watch_dir.join("d.txt");
    
    // 创建文件
    tokio::fs::write(&a, b"a").await.unwrap();
    tokio::fs::write(&b, b"b").await.unwrap();
    tokio::fs::write(&c, b"c").await.unwrap();
    tokio::fs::write(&d, b"d").await.unwrap();
    
    // 添加依赖关系
    manager.add_dependency(a.clone(), ResourceDependency {
        path: b.clone(),
        dependency_type: "texture".to_string(),
        required: true,
    }).unwrap();
    
    manager.add_dependency(a.clone(), ResourceDependency {
        path: c.clone(),
        dependency_type: "shader".to_string(),
        required: true,
    }).unwrap();
    
    manager.add_dependency(b.clone(), ResourceDependency {
        path: d.clone(),
        dependency_type: "model".to_string(),
        required: true,
    }).unwrap();
    
    manager.add_dependency(c.clone(), ResourceDependency {
        path: d.clone(),
        dependency_type: "model".to_string(),
        required: true,
    }).unwrap();
    
    // 获取加载顺序
    let load_order = manager.get_load_order().unwrap();
    
    // D 应该在 B 和 C 之前
    // B 和 C 应该在 A 之前
    let d_index = load_order.iter().position(|p| p == &d).unwrap();
    let b_index = load_order.iter().position(|p| p == &b).unwrap();
    let c_index = load_order.iter().position(|p| p == &c).unwrap();
    let a_index = load_order.iter().position(|p| p == &a).unwrap();
    
    assert!(d_index < b_index);
    assert!(d_index < c_index);
    assert!(b_index < a_index);
    assert!(c_index < a_index);
    
    // 测试热重载传播
    let dependency_graph = manager.dependency_graph();
    let hot_reload = HotReloadManager::new(
        watch_dir.clone(),
        dependency_graph.clone(),
        Duration::from_millis(100),
    ).unwrap();
    
    // 监视所有资源
    hot_reload.watch_resource(a.clone()).await.unwrap();
    hot_reload.watch_resource(b.clone()).await.unwrap();
    hot_reload.watch_resource(c.clone()).await.unwrap();
    hot_reload.watch_resource(d.clone()).await.unwrap();
    
    // 修改 D（底层依赖）
    tokio::fs::write(&d, b"modified d").await.unwrap();
    sleep(Duration::from_millis(200)).await;
    
    // 获取重新加载目标
    let reload_targets = hot_reload.get_reload_targets(&d);
    
    // 应该包含 D、B、C 和 A
    assert!(reload_targets.contains(&d));
    assert!(reload_targets.contains(&b));
    assert!(reload_targets.contains(&c));
    assert!(reload_targets.contains(&a));
}

