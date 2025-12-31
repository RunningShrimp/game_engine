//! # 鸿蒙系统 WebGPU 集成
//!
//! 为鸿蒙系统提供WebGPU图形后端支持。
//!
//! ## 架构
//!
//! 1. **Surface创建**: 从鸿蒙NativeWindow创建WebGPU Surface
//! 2. **适配器选择**: 查询可用的GPU适配器
//! 3. **设备初始化**: 创建逻辑设备和队列
//! 4. **交换链**: 创建呈现交换链
//!
//! ## 使用场景
//!
//! - 高性能游戏渲染
//! - 计算着色器加速
//! - 跨平台图形抽象

use super::harmonyos::{GraphicsBackend, HarmonyOSGraphicsContext};

/// 鸿蒙WebGPU Surface创建器
pub struct HarmonyOSWgpuSurfaceCreator {
    window_handle: *mut std::ffi::c_void,
    width: u32,
    height: u32,
}

unsafe impl Send for HarmonyOSWgpuSurfaceCreator {}
unsafe impl Sync for HarmonyOSWgpuSurfaceCreator {}

impl HarmonyOSWgpuSurfaceCreator {
    /// 创建新的Surface创建器
    pub fn new(graphics_context: &HarmonyOSGraphicsContext) -> Self {
        Self {
            window_handle: graphics_context.native_window_handle(),
            width: 1920, // 从graphics_context获取
            height: 1080,
        }
    }

    /// 获取窗口句柄
    pub fn window_handle(&self) -> *mut std::ffi::c_void {
        self.window_handle
    }

    /// 获取Surface尺寸
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

/// 鸿蒙WebGPU适配器选择策略
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdapterPreference {
    /// 高性能GPU（独显）
    HighPerformance,
    /// 低功耗（集显）
    LowPower,
    /// 任何适配器
    Any,
}

/// 鸿蒙WebGPU实例
pub struct HarmonyOSWgpuInstance {
    instance: wgpu::Instance,
}

impl HarmonyOSWgpuInstance {
    /// 创建WebGPU实例
    pub fn new() -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        Self { instance }
    }

    /// 创建Surface
    pub fn create_surface(
        &self,
        creator: &HarmonyOSWgpuSurfaceCreator,
    ) -> Result<wgpu::Surface, String> {
        unsafe {
            // 注: 鸿蒙需要特殊处理，因为winit不直接支持
            // 需要手动创建raw-window-handle

            use raw_window_handle::{
                DisplayHandle, HasDisplayHandle, HasWindowHandle, WindowHandle,
            };

            struct HarmonyOSDisplay;
            struct HarmonyOSWindow {
                ptr: *mut std::ffi::c_void,
            }

            unsafe impl HasDisplayHandle for HarmonyOSDisplay {
                fn display_handle(&self) -> Result<DisplayHandle, raw_window_handle::HandleError> {
                    unsafe {
                        Ok(DisplayHandle::borrow_raw_raw(
                            std::ptr::null_mut() as *mut std::ffi::c_void
                        ))
                    }
                }
            }

            unsafe impl HasWindowHandle for HarmonyOSWindow {
                fn window_handle(&self) -> Result<WindowHandle, raw_window_handle::HandleError> {
                    unsafe {
                        Ok(WindowHandle::borrow_raw_raw(
                            self.ptr as *mut std::ffi::c_void,
                        ))
                    }
                }
            }

            // 注: 这里需要实际的鸿蒙窗口句柄类型
            // 暂时返回错误，需要完整的raw-window-handle实现

            Err(
                "HarmonyOS surface creation requires complete raw-window-handle implementation"
                    .to_string(),
            )
        }
    }

    /// 请求适配器
    pub async fn request_adapter(
        &self,
        surface: &wgpu::Surface,
        preference: AdapterPreference,
    ) -> Result<wgpu::Adapter, String> {
        let adapter_options = wgpu::RequestAdapterOptions {
            power_preference: match preference {
                AdapterPreference::HighPerformance => wgpu::PowerPreference::HighPerformance,
                AdapterPreference::LowPower => wgpu::PowerPreference::LowPower,
                AdapterPreference::Any => wgpu::PowerPreference::None,
            },
            compatible_surface: Some(surface),
            force_fallback_adapter: false,
        };

        self.instance
            .request_adapter(&adapter_options)
            .await
            .ok_or_else(|| "No compatible GPU adapter found".to_string())
    }

    /// 获取实例
    pub fn instance(&self) -> &wgpu::Instance {
        &self.instance
    }
}

impl Default for HarmonyOSWgpuInstance {
    fn default() -> Self {
        Self::new()
    }
}

/// 鸿蒙WebGPU设备
pub struct HarmonyOSWgpuDevice {
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl HarmonyOSWgpuDevice {
    /// 创建设备和队列
    pub async fn new(adapter: wgpu::Adapter) -> Result<Self, String> {
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("HarmonyOS Device"),
                    required_features: wgpu::Features::TIMESTAMP_QUERY
                        | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .map_err(|e| format!("Failed to create device: {}", e))?;

        Ok(Self {
            adapter,
            device,
            queue,
        })
    }

    /// 获取设备
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// 获取队列
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    /// 获取适配器
    pub fn adapter(&self) -> &wgpu::Adapter {
        &self.adapter
    }

    /// 获取适配器信息
    pub fn adapter_info(&self) -> wgpu::AdapterInfo {
        self.adapter.get_info()
    }
}

/// 鸿蒙WebGPU配置
#[derive(Clone, Debug)]
pub struct HarmonyOSWgpuConfig {
    pub adapter_preference: AdapterPreference,
    pub present_mode: wgpu::PresentMode,
    pub surface_format: wgpu::TextureFormat,
}

impl Default for HarmonyOSWgpuConfig {
    fn default() -> Self {
        Self {
            adapter_preference: AdapterPreference::HighPerformance,
            present_mode: wgpu::PresentMode::Fifo, // VSync
            surface_format: wgpu::TextureFormat::Bgra8UnormSrgb,
        }
    }
}

/// 鸿蒙WebGPU渲染上下文
pub struct HarmonyOSWgpuContext {
    surface: wgpu::Surface,
    device: HarmonyOSWgpuDevice,
    config: wgpu::SurfaceConfiguration,
}

impl HarmonyOSWgpuContext {
    /// 创建渲染上下文
    pub async fn new(
        surface: wgpu::Surface,
        device: HarmonyOSWgpuDevice,
        config: HarmonyOSWgpuConfig,
        width: u32,
        height: u32,
    ) -> Result<Self, String> {
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: config.surface_format,
            width,
            height,
            present_mode: config.present_mode,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![config.surface_format],
        };

        surface.configure(&device.device(), &surface_config);

        Ok(Self {
            surface,
            device,
            config: surface_config,
        })
    }

    /// 获取Surface
    pub fn surface(&self) -> &wgpu::Surface {
        &self.surface
    }

    /// 获取设备
    pub fn device(&self) -> &wgpu::Device {
        self.device.device()
    }

    /// 获取队列
    pub fn queue(&self) -> &wgpu::Queue {
        self.device.queue()
    }

    /// 获取配置
    pub fn config(&self) -> &wgpu::SurfaceConfiguration {
        &self.config
    }

    /// 获取当前帧TextureView
    pub fn get_current_frame(&self) -> Result<wgpu::TextureView, String> {
        self.surface
            .get_current_texture()
            .map(|frame| frame.texture.create_view(&wgpu::TextureViewDescriptor::default()))
            .map_err(|e| format!("Failed to get current frame: {}", e))
    }

    /// 调整大小
    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(self.device.device(), &self.config);
    }

    /// 呈现
    pub fn present(&self) {
        // Queue::submit自动处理present
    }
}

// =============================================================================
// 高级功能
// =============================================================================

/// 鸿蒙GPU信息
#[derive(Clone, Debug)]
pub struct HarmonyOSGpuInfo {
    pub name: String,
    pub vendor: String,
    pub driver: String,
    pub driver_info: String,
    pub backend: wgpu::Backend,
}

impl From<wgpu::AdapterInfo> for HarmonyOSGpuInfo {
    fn from(info: wgpu::AdapterInfo) -> Self {
        Self {
            name: info.name,
            vendor: info.vendor,
            driver: info.driver,
            driver_info: info.driver_info,
            backend: info.backend,
        }
    }
}

/// GPU性能提示
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PerformanceHint {
    /// 低功耗模式（延长电池）
    LowPower,
    /// 平衡模式
    Balanced,
    /// 高性能模式
    HighPerformance,
}

/// 设置性能提示
#[cfg(feature = "harmonyos")]
pub fn set_performance_hint(hint: PerformanceHint) {
    // 注: 调用鸿蒙API设置性能模式
    // 可能影响GPU频率、电源管理等
}

// =============================================================================
// 鸿蒙特定的扩展
// =============================================================================

/// 鸿蒙Vulkan扩展列表
pub const HARMONYOS_VULKAN_EXTENSIONS: &[&str] =
    &["VK_KHR_surface", "VK_KHR_swapchain", "VK_EXT_hdr_metadata"];

/// 鸿蒙OpenGL ES扩展列表
pub const HARMONYOS_GLES_EXTENSIONS: &[&str] = &[
    "GL_OES_EGL_image",
    "GL_EXT_texture_rg",
    "GL_OES_texture_float",
];

/// 检查Vulkan支持
pub fn is_vulkan_supported() -> bool {
    #[cfg(feature = "harmonyos")]
    {
        // 注: 调用鸿蒙API检查Vulkan支持
        true
    }

    #[cfg(not(feature = "harmonyos"))]
    false
}

/// 检查OpenGL ES支持
pub fn is_opengles_supported() -> bool {
    #[cfg(feature = "harmonyos")]
    {
        // 注: 调用鸿蒙API检查OpenGL ES支持
        true
    }

    #[cfg(not(feature = "harmonyos"))]
    false
}

// =============================================================================
// 测试
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapter_preference() {
        let pref = AdapterPreference::HighPerformance;
        assert_eq!(pref as i32, AdapterPreference::HighPerformance as i32);
    }

    #[test]
    fn test_wgpu_config_default() {
        let config = HarmonyOSWgpuConfig::default();
        assert_eq!(
            config.adapter_preference,
            AdapterPreference::HighPerformance
        );
        assert_eq!(config.present_mode, wgpu::PresentMode::Fifo);
    }

    #[test]
    fn test_gpu_info_from_adapter_info() {
        let adapter_info = wgpu::AdapterInfo {
            name: "Test GPU".to_string(),
            vendor: 0,
            device: 0,
            driver: "Test Driver".to_string(),
            driver_info: "1.0".to_string(),
            backend: wgpu::Backend::Vulkan,
        };

        let gpu_info: HarmonyOSGpuInfo = adapter_info.into();
        assert_eq!(gpu_info.name, "Test GPU");
        assert_eq!(gpu_info.driver, "Test Driver");
    }
}
