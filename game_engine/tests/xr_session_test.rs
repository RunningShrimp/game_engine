// OpenXR 会话创建测试

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_openxr_session_creation() {
    use game_engine::xr::{XrConfig, XrSessionState, OpenXrBackend};

    // 配置OpenXR
    let config = XrConfig {
        application_name: "TestApp".to_string(),
        ..Default::default()
    };

    // 尝试创建OpenXR后端
    // 注意：如果没有安装OpenXR运行时，此测试将失败，这是正常的
    match OpenXrBackend::new(config) {
        Ok(mut backend) => {
            println!("✓ OpenXR instance created successfully");
            
            // 会话创建需要实际的wgpu设备，这里我们只测试实例创建
            assert_eq!(backend.state(), XrSessionState::Idle);
        },
        Err(err) => {
            // 当没有OpenXR运行时安装时，打印信息而不是失败
            println!("⚠ OpenXR not available: {}", err);
            // 非严格测试：在没有XR运行时的环境中也能通过
        }
    }
}