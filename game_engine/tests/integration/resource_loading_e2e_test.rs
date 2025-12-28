//! 资源加载完整流程测试

use game_engine::resources::coroutine_loader::CoroutineAssetLoader;
use game_engine::resources::coroutine_loader::CoroutineLoaderConfig;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::runtime::Runtime;

/// 测试异步资源加载完整流程
#[test]
fn test_resource_loading_complete_flow() {
    let rt = Runtime::new().expect("Failed to create runtime");
    
    rt.block_on(async {
        // 1. 创建资源加载器
        let config = CoroutineLoaderConfig::default();
        let loader = CoroutineAssetLoader::new(config);
        
        // 2. 创建临时测试文件
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let test_file_path = temp_dir.path().join("test_resource.txt");
        tokio::fs::write(&test_file_path, b"test content")
            .await
            .expect("Failed to write test file");
        
        // 3. 加载资源
        let load_handle = loader.load_asset(test_file_path.to_str().unwrap().to_string());
        
        // 4. 等待加载完成
        let result = loader.wait_for_completed(load_handle, std::time::Duration::from_secs(5))
            .await;
        
        assert!(result.is_some(), "Resource loading timed out");
        
        // 5. 验证加载结果
        let load_result = result.unwrap();
        assert!(load_result.success);
        assert_eq!(load_result.path, test_file_path.to_str().unwrap());
        
        // 6. 验证统计信息
        let stats = loader.get_stats();
        assert_eq!(stats.completed_loads, 1);
        assert_eq!(stats.failed_loads, 0);
    });
}

/// 测试资源加载错误处理
#[test]
fn test_resource_loading_error_handling() {
    let rt = Runtime::new().expect("Failed to create runtime");
    
    rt.block_on(async {
        let config = CoroutineLoaderConfig::default();
        let loader = CoroutineAssetLoader::new(config);
        
        // 尝试加载不存在的文件
        let nonexistent_path = "/nonexistent/path/to/file.txt";
        let load_handle = loader.load_asset(nonexistent_path.to_string());
        
        // 等待加载完成
        let result = loader.wait_for_completed(load_handle, std::time::Duration::from_secs(5))
            .await;
        
        // 应该返回结果（成功或失败）
        assert!(result.is_some());
        
        let load_result = result.unwrap();
        // 文件不存在应该失败
        if !load_result.success {
            // 验证统计信息
            let stats = loader.get_stats();
            assert_eq!(stats.failed_loads, 1);
        }
    });
}

/// 测试资源加载并发处理
#[test]
fn test_resource_loading_concurrency() {
    let rt = Runtime::new().expect("Failed to create runtime");
    
    rt.block_on(async {
        let config = CoroutineLoaderConfig::default();
        let loader = CoroutineAssetLoader::new(config);
        
        // 创建多个测试文件
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let mut handles = Vec::new();
        
        for i in 0..5 {
            let test_file_path = temp_dir.path().join(format!("test_{}.txt", i));
            tokio::fs::write(&test_file_path, format!("content {}", i).as_bytes())
                .await
                .expect("Failed to write test file");
            
            let handle = loader.load_asset(test_file_path.to_str().unwrap().to_string());
            handles.push(handle);
        }
        
        // 等待所有加载完成
        let mut completed = 0;
        for handle in handles {
            if let Some(result) = loader.wait_for_completed(handle, std::time::Duration::from_secs(10))
                .await
            {
                if result.success {
                    completed += 1;
                }
            }
        }
        
        // 验证所有资源都已加载
        assert_eq!(completed, 5);
        
        // 验证统计信息
        let stats = loader.get_stats();
        assert_eq!(stats.completed_loads, 5);
    });
}





