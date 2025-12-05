//! 异步I/O操作测试
//!
//! 测试异步文件系统、网络和资源加载操作的正确性

use std::time::Duration;
use tokio::time::timeout;
use game_engine::platform::{Filesystem, NativeFilesystem};
use game_engine::network::{GameClient, GameServer, ClientConfig, ServerConfig};
use game_engine::resources::{AssetServer, Handle};
use tempfile::TempDir;

#[tokio::test]
async fn test_async_filesystem_read() {
    let temp_dir = TempDir::new().unwrap();
    let fs = NativeFilesystem::new();
    let test_file = temp_dir.path().join("test.txt");
    let test_content = "Hello, async world!";
    
    // 异步写入文件
    fs.write(&test_file, test_content.as_bytes()).await.unwrap();
    
    // 异步读取文件
    let read_content = fs.read(&test_file).await.unwrap();
    assert_eq!(read_content, test_content.as_bytes());
    
    // 测试exists_async
    assert!(fs.exists_async(&test_file).await.unwrap());
    
    // 异步删除文件
    fs.remove_file(&test_file).await.unwrap();
    assert!(!fs.exists_async(&test_file).await.unwrap());
}

#[tokio::test]
async fn test_async_filesystem_directory() {
    let temp_dir = TempDir::new().unwrap();
    let fs = NativeFilesystem::new();
    let test_dir = temp_dir.path().join("test_dir");
    
    // 异步创建目录
    fs.create_dir_all(&test_dir).await.unwrap();
    assert!(fs.exists_async(&test_dir).await.unwrap());
    
    // 异步读取目录
    let entries = fs.read_dir(&test_dir).await.unwrap();
    assert_eq!(entries.len(), 0); // 空目录
}

#[tokio::test]
async fn test_async_network_client() {
    // 创建测试配置
    let client_config = ClientConfig {
        server_address: "127.0.0.1".to_string(),
        server_port: 0, // 使用随机端口避免冲突
        reconnect_interval_ms: 100,
        max_reconnect_attempts: 1,
        enable_compression: false,
        enable_delay_compensation: false,
        client_name: "TestClient".to_string(),
    };
    
    let client = GameClient::new(client_config);
    
    // 测试初始状态
    assert!(!client.is_connected().await);
    assert_eq!(client.connection_state().await, game_engine::network::ConnectionState::Disconnected);
    
    // 注意：实际连接测试需要运行的服务器，这里只测试状态
    assert!(client.client_id().await.is_none());
}

#[tokio::test]
async fn test_async_network_server() {
    // 创建测试配置
    let server_config = ServerConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 0, // 使用随机端口避免冲突
        max_connections: 10,
        heartbeat_timeout_ms: 1000,
        enable_compression: false,
        enable_delay_compensation: false,
    };
    
    let server = GameServer::new(server_config);
    
    // 测试初始状态
    assert_eq!(server.client_count().await, 0);
    assert_eq!(server.current_tick().await, 0);
    
    // 测试tick更新
    server.update_tick().await;
    assert_eq!(server.current_tick().await, 1);
    
    // 测试获取客户端ID列表
    let client_ids = server.get_client_ids().await;
    assert_eq!(client_ids.len(), 0);
}

#[tokio::test]
async fn test_async_asset_loading() {
    let temp_dir = TempDir::new().unwrap();
    let asset_server = AssetServer::new();
    
    // 创建测试纹理文件
    let texture_path = temp_dir.path().join("test.png");
    let test_image = image::RgbaImage::new(64, 64);
    let image_data = test_image.to_raw();
    tokio::fs::write(&texture_path, &image_data).await.unwrap();
    
    // 测试异步加载纹理
    let texture_handle = asset_server.load_texture_async(&texture_path).await;
    assert!(texture_handle.is_ok());
    
    let handle = texture_handle.unwrap();
    
    // 等待加载完成（最多5秒）
    let loaded_handle = timeout(Duration::from_secs(5), async {
        loop {
            if handle.is_loaded() {
                break handle.get();
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }).await;
    
    assert!(loaded_handle.is_some());
}

#[tokio::test]
async fn test_async_asset_loading_timeout() {
    let temp_dir = TempDir::new().unwrap();
    let asset_server = AssetServer::new();
    
    // 创建一个不存在的文件路径
    let nonexistent_path = temp_dir.path().join("nonexistent.png");
    
    // 测试异步加载不存在的文件
    let texture_handle = asset_server.load_texture_async(&nonexistent_path).await;
    assert!(texture_handle.is_err());
    
    let error_msg = texture_handle.unwrap_err();
    assert!(error_msg.contains("Timeout") || error_msg.contains("Failed"));
}

#[tokio::test]
async fn test_async_compatibility() {
    // 测试同步版本的向后兼容性
    let temp_dir = TempDir::new().unwrap();
    let fs = NativeFilesystem::new();
    let test_file = temp_dir.path().join("compat_test.txt");
    let test_content = "Compatibility test";
    
    // 使用同步版本
    fs.write_sync(&test_file, test_content.as_bytes()).unwrap();
    let read_content = fs.read_sync(&test_file).unwrap();
    assert_eq!(read_content, test_content.as_bytes());
    
    // 测试异步版本与同步版本的一致性
    fs.write(&test_file, test_content.as_bytes()).await.unwrap();
    let async_read_content = fs.read(&test_file).await.unwrap();
    assert_eq!(async_read_content, test_content.as_bytes());
}

#[test]
fn test_sync_to_async_bridge() {
    // 测试同步和异步API之间的桥接
    let temp_dir = TempDir::new().unwrap();
    let fs = NativeFilesystem::new();
    let test_file = temp_dir.path().join("bridge_test.txt");
    let test_content = "Bridge test";
    
    // 在同步上下文中使用异步API
    let async_result = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            fs.write(&test_file, test_content.as_bytes()).await.unwrap();
            fs.read(&test_file).await.unwrap()
        })
    });
    
    assert_eq!(async_result, test_content.as_bytes());
}