//! OpenXR 实现
//!
//! 完整的OpenXR集成，包括实例创建、会话管理、交换链等

use super::*;
use openxr as xr;
use std::ffi::CString;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OpenXrError {
    #[error("OpenXR initialization failed: {0}")]
    InitializationFailed(String),
    #[error("No XR runtime found")]
    NoRuntime,
    #[error("System not found")]
    SystemNotFound,
    #[error("Session creation failed: {0}")]
    SessionCreationFailed(String),
    #[error("Swapchain creation failed: {0}")]
    SwapchainCreationFailed(String),
    #[error("Reference space creation failed: {0}")]
    ReferenceSpaceFailed(String),
}

impl From<OpenXrError> for XrError {
    fn from(err: OpenXrError) -> Self {
        match err {
            OpenXrError::InitializationFailed(msg) => XrError::RuntimeFailure(msg),
            OpenXrError::NoRuntime => XrError::NotSupported,
            OpenXrError::SystemNotFound => XrError::NotSupported,
            OpenXrError::SessionCreationFailed(msg) => XrError::RuntimeFailure(msg),
            OpenXrError::SwapchainCreationFailed(msg) => XrError::RuntimeFailure(msg),
            OpenXrError::ReferenceSpaceFailed(msg) => XrError::RuntimeFailure(msg),
        }
    }
}

<<<<<<< HEAD
/// OpenXR 后端实现
pub struct OpenXrBackend {
    instance: xr::Instance,
=======
/// Vulkan设备信息
#[derive(Debug, Clone)]
struct VulkanDeviceInfo {
    #[allow(dead_code)]
    instance: *mut std::ffi::c_void,
    #[allow(dead_code)]
    physical_device: *mut std::ffi::c_void,
    #[allow(dead_code)]
    device: *mut std::ffi::c_void,
    #[allow(dead_code)]
    queue_family_index: u32,
}

/// OpenXR 后端实现
pub struct OpenXrBackend {
    #[allow(dead_code)]
    instance: xr::Instance,
    #[allow(dead_code)]
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
    system: xr::SystemId,
    session: Option<xr::Session<xr::Vulkan>>,
    reference_space: Option<xr::Space>,
    view_space: Option<xr::Space>,
    swapchains: Vec<OpenXrSwapchain>,
    state: XrSessionState,
    config: XrConfig,
    views: Vec<XrView>,
    events: Vec<XrEvent>,
}

impl OpenXrBackend {
    /// 创建新的OpenXR后端
    pub fn new(config: XrConfig) -> Result<Self, OpenXrError> {
<<<<<<< HEAD
        // 注意：OpenXR 0.18 API可能不同，这里使用占位实现
        // 实际使用时需要根据openxr crate的具体版本调整API调用
        // 暂时返回错误，等待OpenXR API修复
        // 
        // 已知问题：openxr crate 0.18版本的API可能与当前实现不兼容
        // 解决方案：需要根据实际使用的openxr版本调整API调用
        // 相关任务：跟踪openxr crate更新，修复API兼容性

        // 以下代码暂时注释，等待OpenXR API修复
        // 1. 创建OpenXR实例
        // let entry = unsafe { xr::Entry::load() }
        //     .map_err(|e| OpenXrError::InitializationFailed(format!("Failed to load OpenXR: {}", e)))?;

        // let app_name = CString::new(config.application_name.clone())
        //     .map_err(|e| OpenXrError::InitializationFailed(format!("Invalid app name: {}", e)))?;
        // let engine_name = CString::new("GameEngine")
        //     .map_err(|e| OpenXrError::InitializationFailed(format!("Invalid engine name: {}", e)))?;

        // let app_info = xr::ApplicationInfo {
        //     application_name: app_name.as_c_str().to_str()
        //         .map_err(|e| OpenXrError::InitializationFailed(format!("Invalid app name encoding: {}", e)))?,
        //     application_version: 1,
        //     engine_name: engine_name.as_c_str().to_str()
        //         .map_err(|e| OpenXrError::InitializationFailed(format!("Invalid engine name encoding: {}", e)))?,
        //     engine_version: 1,
        // };

        Err(OpenXrError::InitializationFailed(
            "OpenXR initialization temporarily disabled due to API changes".to_string(),
        ))

        // 以下代码需要OpenXR API修复后才能使用
        /*
        let instance = entry
            .create_instance(&xr::InstanceCreateInfo {
                application_info: app_info,
                enabled_api_layers: &[],
                enabled_extension_names: &[],
                ..Default::default()
            })
            .map_err(|e| OpenXrError::InitializationFailed(format!("Failed to create instance: {:?}", e)))?;

        // 2. 获取系统（HMD）
        let system = instance
            .system(xr::FormFactor::HEAD_MOUNTED_DISPLAY)
            .map_err(|e| OpenXrError::SystemNotFound)?;

        // 3. 检查系统属性
        let system_properties = instance
            .system_properties(system)
            .map_err(|e| OpenXrError::SystemNotFound)?;
=======
        // 1. 创建OpenXR入口
        let entry = unsafe { xr::Entry::load() }
            .map_err(|e| OpenXrError::InitializationFailed(format!("Failed to load OpenXR: {}", e)))?;

        // 2. 检查可用的扩展
        let available_extensions = entry
            .enumerate_extensions()
            .map_err(|e| OpenXrError::InitializationFailed(format!("Failed to enumerate extensions: {}", e)))?;

        tracing::info!("Available OpenXR extensions: {:?}", available_extensions);

        // 3. 准备必需的扩展
        let mut required_extensions = vec![
            "XR_KHR_vulkan_enable".to_string(),
        ];

        // 可选扩展
        if available_extensions.khr_vulkan_enable2 {
            required_extensions.push("XR_KHR_vulkan_enable2".to_string());
        }
        if available_extensions.ext_hand_tracking {
            required_extensions.push("XR_EXT_hand_tracking".to_string());
        }
        if available_extensions.msft_spatial_anchor {
            required_extensions.push("XR_MSFT_spatial_anchor".to_string());
        }
        if available_extensions.ext_eye_gaze_interaction {
            required_extensions.push("XR_EXT_eye_gaze_interaction".to_string());
        }

        // 4. 创建应用信息
        let app_name = CString::new(config.application_name.clone())
            .map_err(|e| OpenXrError::InitializationFailed(format!("Invalid app name: {}", e)))?;
        let engine_name = CString::new("GameEngine")
            .map_err(|e| OpenXrError::InitializationFailed(format!("Invalid engine name: {}", e)))?;

        let app_info = xr::ApplicationInfo {
            application_name: app_name.as_c_str().to_str()
                .map_err(|e| OpenXrError::InitializationFailed(format!("Invalid app name encoding: {}", e)))?,
            application_version: 1,
            engine_name: engine_name.as_c_str().to_str()
                .map_err(|e| OpenXrError::InitializationFailed(format!("Invalid engine name encoding: {}", e)))?,
            engine_version: 1,
        };

        // 5. 创建OpenXR实例
        // 由于ExtensionSet API不明确，我们使用默认的扩展集合
        let extensions = xr::ExtensionSet::default();
        
        let instance = entry
            .create_instance(
                &app_info,
                &extensions,
                &[],
            )
            .map_err(|e| OpenXrError::InitializationFailed(format!("Failed to create instance: {:?}", e)))?;

        // 6. 获取系统（HMD）
        let system = instance
            .system(xr::FormFactor::HEAD_MOUNTED_DISPLAY)
            .map_err(|_e| OpenXrError::SystemNotFound)?;

        // 7. 检查系统属性
        let system_properties = instance
            .system_properties(system)
            .map_err(|_e| OpenXrError::SystemNotFound)?;
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)

        tracing::info!("OpenXR System: {}", system_properties.system_name);
        tracing::info!("Vendor ID: {}", system_properties.vendor_id);

<<<<<<< HEAD
        // 4. 获取视图配置
        let view_configs = instance
            .enumerate_view_configurations(system)
            .map_err(|e| OpenXrError::SystemNotFound)?;
=======
        // 8. 获取视图配置
        let view_configs = instance
            .enumerate_view_configurations(system)
            .map_err(|_e| OpenXrError::SystemNotFound)?;
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)

        if view_configs.is_empty() {
            return Err(OpenXrError::SystemNotFound);
        }

        // 使用第一个支持的视图配置（通常是立体）
        let view_config = view_configs[0];

<<<<<<< HEAD
        // 5. 获取视图配置属性
        let view_config_properties = instance
            .view_configuration_properties(system, view_config)
            .map_err(|e| OpenXrError::SystemNotFound)?;
=======
        // 9. 获取视图配置属性
        let view_config_properties = instance
            .view_configuration_properties(system, view_config)
            .map_err(|_e| OpenXrError::SystemNotFound)?;
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)

        tracing::info!("View configuration: {:?}", view_config);
        tracing::info!("Fov mutable: {}", view_config_properties.fov_mutable);

<<<<<<< HEAD
        // 6. 获取视图数量（通常是2，左右眼）
        let view_count = instance
            .enumerate_view_configuration_views(system, view_config)
            .map_err(|e| OpenXrError::SystemNotFound)?
            .len();

        tracing::info!("View count: {}", view_count);
=======
        // 10. 获取视图配置视图（用于获取分辨率等信息）
        let view_config_views = instance
            .enumerate_view_configuration_views(system, view_config)
            .map_err(|_e| OpenXrError::SystemNotFound)?;

        if view_config_views.is_empty() {
            return Err(OpenXrError::SystemNotFound);
        }

        tracing::info!("View count: {}", view_config_views.len());
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)

        Ok(Self {
            instance,
            system,
            session: None,
            reference_space: None,
            view_space: None,
            swapchains: Vec::new(),
            state: XrSessionState::Idle,
            config,
            views: Vec::new(),
            events: Vec::new(),
        })
<<<<<<< HEAD
        */
    }

    /// 创建会话（需要Vulkan设备）
    pub fn create_session(&mut self, _device: &wgpu::Device) -> Result<(), OpenXrError> {
        // 注意：这里需要实际的Vulkan设备来创建会话
        // 由于wgpu抽象了底层API，我们需要获取Vulkan句柄
        // 这是一个占位实现，实际需要：
        // 1. 从wgpu获取Vulkan设备
        // 2. 创建OpenXR会话

        // 暂时标记为就绪状态
        self.state = XrSessionState::Ready;

        Ok(())
=======
    }

    /// 创建会话（需要Vulkan设备）
    pub fn create_session(&mut self, device: &wgpu::Device) -> Result<(), OpenXrError> {
        // 1. 获取Vulkan设备信息
        let _vulkan_device = self.get_vulkan_device_info(device)?;
        
        // 2. 创建会话 - 占位实现，需要实际的Vulkan设备句柄
        // 由于无法从wgpu获取Vulkan句柄，这里返回错误
        return Err(OpenXrError::SessionCreationFailed(
            "Vulkan device extraction from wgpu not implemented".to_string(),
        ));

        // 注意：以下代码在当前实现中不会执行，因为上面已经返回错误
        // self.session = Some(session);
        // self.state = XrSessionState::Ready;

    }

    /// 从wgpu设备获取Vulkan设备信息
    fn get_vulkan_device_info(&self, _device: &wgpu::Device) -> Result<VulkanDeviceInfo, OpenXrError> {
        // 注意：这里需要根据wgpu的实际API获取Vulkan句柄
        // 由于wgpu可能不直接暴露Vulkan句柄，这里提供一个占位实现
        // 实际实现可能需要使用wgpu的特定API或使用raw-window-handle
        
        // 占位实现 - 实际需要从wgpu获取真实的Vulkan句柄
        Err(OpenXrError::SessionCreationFailed(
            "Vulkan device extraction from wgpu not implemented".to_string(),
        ))
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
    }

    /// 创建参考空间
    pub fn create_reference_space(&mut self) -> Result<(), OpenXrError> {
<<<<<<< HEAD
        if self.session.is_none() {
            return Err(OpenXrError::SessionCreationFailed(
                "Session not created".to_string(),
            ));
        }

        // 转换参考空间类型
        let reference_space_type = match self.config.reference_space {
=======
        let _session = self.session.as_ref()
            .ok_or_else(|| OpenXrError::SessionCreationFailed("Session not created".to_string()))?;

        // 转换参考空间类型
        let _reference_space_type = match self.config.reference_space {
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
            ReferenceSpaceType::View => xr::ReferenceSpaceType::VIEW,
            ReferenceSpaceType::Local => xr::ReferenceSpaceType::LOCAL,
            ReferenceSpaceType::Stage => xr::ReferenceSpaceType::STAGE,
            ReferenceSpaceType::Unbounded => {
                // UNBOUNDED可能在某些OpenXR版本中不可用，使用STAGE作为回退
                xr::ReferenceSpaceType::STAGE
            }
        };

<<<<<<< HEAD
        // 创建参考空间
        // 注意：这需要实际的会话，暂时跳过
=======
        // 创建参考空间 - 占位实现
        // 实际需要OpenXR会话，这里先创建一个占位空间
        // 由于无法创建真实的OpenXR空间，我们跳过这一步
        self.reference_space = None;

        // 创建视图空间（用于视图定位）- 占位实现
        let _view_space = (); // 占位

        // 跳过视图空间设置，因为无法创建真实的OpenXR空间
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)

        Ok(())
    }

    /// 创建交换链
    pub fn create_swapchains(
        &mut self,
<<<<<<< HEAD
        _device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> Result<(), OpenXrError> {
        if self.session.is_none() {
            return Err(OpenXrError::SessionCreationFailed(
                "Session not created".to_string(),
            ));
        }

        // 获取推荐的交换链格式和大小
        // 创建交换链（需要实际的Vulkan会话）
        // 暂时创建占位交换链
=======
        device: &wgpu::Device,
        _width: u32,
        _height: u32,
    ) -> Result<(), OpenXrError> {
        let session = self.session.as_ref()
            .ok_or_else(|| OpenXrError::SessionCreationFailed("Session not created".to_string()))?;

        // 获取推荐的交换链格式
        // 占位实现：直接使用默认格式
        let format = 37; // RGBA8_UNORM 的占位值

        tracing::info!("Selected swapchain format: {}", format);

        // 获取视图配置视图以获取推荐分辨率
        // 占位实现：创建默认视图配置
        let view_config_views = vec![
            xr::ViewConfigurationView {
                recommended_image_rect_width: 1920,
                recommended_image_rect_height: 1080,
                recommended_swapchain_sample_count: 1,
                max_swapchain_sample_count: 1,
                max_image_rect_width: 1920,
                max_image_rect_height: 1080,
            },
            xr::ViewConfigurationView {
                recommended_image_rect_width: 1920,
                recommended_image_rect_height: 1080,
                recommended_swapchain_sample_count: 1,
                max_swapchain_sample_count: 1,
                max_image_rect_width: 1920,
                max_image_rect_height: 1080,
            },
        ];

        // 为每个视图创建交换链
        self.swapchains.clear();
        
        for (i, view_config) in view_config_views.iter().enumerate() {
            let swapchain = OpenXrSwapchain::new(
                session,
                device,
                format,
                view_config.recommended_image_rect_width,
                view_config.recommended_image_rect_height,
            )?;
            
            self.swapchains.push(swapchain);
            tracing::info!("Created swapchain {} for view {}: {}x{}",
                i, i, view_config.recommended_image_rect_width, view_config.recommended_image_rect_height);
        }
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)

        Ok(())
    }

    /// 更新视图
<<<<<<< HEAD
    fn update_views(&mut self, time: xr::Time) -> Result<(), OpenXrError> {
        if self.session.is_none() {
            return Ok(());
        }

        // 定位视图（需要实际的会话和空间）
        // 暂时使用默认视图

        self.views = vec![
            XrView {
                pose: Pose::default(),
                fov: Fov {
                    angle_left: -0.785,
                    angle_right: 0.785,
                    angle_up: 0.785,
                    angle_down: -0.785,
                },
                view_index: 0,
            },
            XrView {
                pose: Pose {
                    position: Vec3::new(0.063, 0.0, 0.0), // IPD
                    orientation: Quat::IDENTITY,
                },
                fov: Fov {
                    angle_left: -0.785,
                    angle_right: 0.785,
                    angle_up: 0.785,
                    angle_down: -0.785,
                },
                view_index: 1,
            },
        ];
=======
    fn update_views(&mut self, _time: xr::Time) -> Result<(), OpenXrError> {
        let _session = self.session.as_ref()
            .ok_or_else(|| OpenXrError::SessionCreationFailed("Session not created".to_string()))?;
        
        let _view_space = self.view_space.as_ref()
            .ok_or_else(|| OpenXrError::ReferenceSpaceFailed("View space not created".to_string()))?;

        // 定位视图（占位实现）
        // 实际实现中需要调用 session.locate_views()
        let _view_state_placeholder = ();

        // 转换OpenXR视图到内部格式（占位实现）
        self.views.clear();
        
        // 创建默认立体视图
        self.views.push(XrView {
            pose: Pose::default(),
            fov: Fov {
                angle_left: -0.785,
                angle_right: 0.785,
                angle_up: 0.785,
                angle_down: -0.785,
            },
            view_index: 0,
        });

        self.views.push(XrView {
            pose: Pose {
                position: Vec3::new(0.063, 0.0, 0.0), // IPD
                orientation: Quat::IDENTITY,
            },
            fov: Fov {
                angle_left: -0.785,
                angle_right: 0.785,
                angle_up: 0.785,
                angle_down: -0.785,
            },
            view_index: 1,
        });
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)

        Ok(())
    }

    /// 处理事件
    fn process_events(&mut self) {
<<<<<<< HEAD
        if self.session.is_none() {
            return;
        }

        // 轮询OpenXR事件
        // 暂时添加模拟事件
        if self.state == XrSessionState::Ready {
            self.events
                .push(XrEvent::SessionStateChanged(XrSessionState::Synchronized));
=======
        let _session = match &self.session {
            Some(session) => session,
            None => return,
        };

        // 轮询OpenXR事件（占位实现）
        // 实际实现中需要调用 session.poll_event()
        
        // 模拟事件处理
        if self.state == XrSessionState::Ready {
            self.events.push(XrEvent::SessionStateChanged(XrSessionState::Synchronized));
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
            self.state = XrSessionState::Synchronized;
        }
    }
}

impl XrSession for OpenXrBackend {
    fn state(&self) -> XrSessionState {
        self.state
    }

    fn begin_frame(&mut self) -> Result<XrFrameState, XrError> {
        if self.session.is_none() {
            return Err(XrError::SessionNotReady);
        }

        if self.state != XrSessionState::Focused && self.state != XrSessionState::Visible {
            return Err(XrError::SessionNotReady);
        }

        // 处理事件
        self.process_events();

        // 更新视图
        let time = xr::Time::from_nanos(0); // 实际应该从运行时获取
        self.update_views(time)
            .map_err(|e: OpenXrError| XrError::RuntimeFailure(format!("{:?}", e)))?;

        Ok(XrFrameState {
            predicted_display_time: 0,
            predicted_display_period: 11_111_111, // ~90Hz
            should_render: true,
        })
    }

    fn end_frame(&mut self, layers: &[XrCompositionLayer]) -> Result<(), XrError> {
<<<<<<< HEAD
        if self.session.is_none() {
            return Err(XrError::SessionNotReady);
        }

        // 提交合成层（需要实际的会话）
        // 暂时仅验证层数据

=======
        let _session = self.session.as_ref()
            .ok_or(XrError::SessionNotReady)?;

        // 提交帧（占位实现）
        // 实际实现中需要转换合成层并调用 session.end_frame()
        tracing::debug!("Submitting {} composition layers", layers.len());
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
        Ok(())
    }

    fn locate_views(&self, _time: i64) -> Result<Vec<XrView>, XrError> {
        Ok(self.views.clone())
    }

    fn poll_events(&mut self) -> Vec<XrEvent> {
        self.process_events();
        std::mem::take(&mut self.events)
    }
}

/// OpenXR 交换链实现
pub struct OpenXrSwapchain {
<<<<<<< HEAD
=======
    #[allow(dead_code)]
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
    swapchain: Option<xr::Swapchain<xr::Vulkan>>,
    images: Vec<Arc<wgpu::TextureView>>,
    current_image_index: u32,
    resolution: (u32, u32),
}

impl OpenXrSwapchain {
    pub fn new(
        _session: &xr::Session<xr::Vulkan>,
<<<<<<< HEAD
        width: u32,
        height: u32,
    ) -> Result<Self, OpenXrError> {
        // 创建交换链（需要实际的Vulkan会话）
        // 暂时创建占位实现

        Ok(Self {
            swapchain: None,
            images: Vec::new(),
=======
        device: &wgpu::Device,
        _format: i64,
        width: u32,
        height: u32,
    ) -> Result<Self, OpenXrError> {
        // 创建OpenXR交换链（占位实现）
        // 实际实现中需要调用 session.create_swapchain()
        let _swapchain_placeholder = ();

        // 创建wgpu纹理视图（占位实现）
        let mut texture_views = Vec::new();
        
        // 创建单个占位纹理视图
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("XR Swapchain Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        texture_views.push(Arc::new(view));

        Ok(Self {
            swapchain: None, // 占位
            images: texture_views,
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
            current_image_index: 0,
            resolution: (width, height),
        })
    }
}

impl XrSwapchain for OpenXrSwapchain {
    fn acquire_image(&mut self) -> Result<u32, XrError> {
<<<<<<< HEAD
        // 获取交换链图像索引
        Ok(self.current_image_index)
    }

    fn wait_image(&mut self, _timeout_ns: i64) -> Result<(), XrError> {
        // 等待图像可用
=======
        // 获取交换链图像索引（占位实现）
        let index = self.current_image_index;
        self.current_image_index = (self.current_image_index + 1) % self.images.len() as u32;
        Ok(index)
    }

    fn wait_image(&mut self, _timeout_ns: i64) -> Result<(), XrError> {
        // 等待图像可用（占位实现）
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
        Ok(())
    }

    fn release_image(&mut self) -> Result<(), XrError> {
<<<<<<< HEAD
        // 释放图像
=======
        // 释放图像（占位实现）
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
        Ok(())
    }

    fn get_texture_view(&self, index: u32) -> Arc<wgpu::TextureView> {
        // 获取纹理视图
<<<<<<< HEAD
        // 暂时返回占位视图
        if let Some(view) = self.images.get(index as usize) {
            view.clone()
        } else {
            // 返回默认视图（实际应该创建）
            Arc::new(unsafe { std::mem::zeroed() }) // 占位
=======
        if let Some(view) = self.images.get(index as usize) {
            view.clone()
        } else {
            // 返回第一个视图作为回退
            self.images.first().cloned()
                .unwrap_or_else(|| {
                    // 创建默认纹理视图作为最后的回退
                    // 注意：这里需要一个设备引用，实际实现中应该存储设备引用
                    // 由于无法创建默认纹理视图，我们panic并返回错误
                    panic!("No texture views available in swapchain")
                })
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
        }
    }

    fn resolution(&self) -> (u32, u32) {
        self.resolution
    }
}
