//  XR (VR/AR) 演示程序
// 
//  展示OpenXR集成功能，包括：
//  - XR会话初始化
//  - 立体渲染
//  - 控制器输入
//  - 手部追踪
//  - 空间锚点

use game_engine::xr::*;
use game_engine::*;
use std::sync::Arc;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

#[derive(Default)]
struct XrDemoState {
    xr_backend: Option<OpenXrBackend>,
    xr_input: XrInputManager,
    hand_tracker: HandTracker,
    anchor_manager: SpatialAnchorManager,
    xr_renderer: XrRenderer,
    frame_count: u64,
}

impl XrDemoState {
    fn new() -> Self {
        Self::default()
    }

    fn initialize_xr(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 1. 创建XR配置
        let xr_config = XrConfig {
            application_name: "XR Demo".to_string(),
            blend_mode: BlendMode::Opaque,
            reference_space: ReferenceSpaceType::Stage,
        };

        // 2. 初始化OpenXR后端
        match OpenXrBackend::new(xr_config) {
            Ok(mut backend) => {
                tracing::info!("OpenXR backend initialized successfully");

                // 3. 初始化手部追踪
                if let Err(e) = self.hand_tracker.initialize() {
                    tracing::warn!("Failed to initialize hand tracking: {}", e);
                } else {
                    tracing::info!("Hand tracking initialized");
                }

                // 4. 初始化空间锚点管理器
                if let Err(e) = SpatialAnchorManager::new() {
                    tracing::warn!("Failed to initialize anchor manager: {}", e);
                } else {
                    tracing::info!("Anchor manager initialized");
                }

                self.xr_backend = Some(backend);
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to initialize OpenXR: {}", e);
                Err(Box::new(e))
            }
        }
    }

    fn create_xr_session(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref mut backend) = self.xr_backend {
            // 创建XR会话
            backend
                .create_session(device)
                .map_err(|e| format!("Failed to create XR session: {}", e))?;

            // 创建参考空间
            backend
                .create_reference_space()
                .map_err(|e| format!("Failed to create reference space: {}", e))?;

            // 创建交换链
            backend
                .create_swapchains(device, 1920, 1080)
                .map_err(|e| format!("Failed to create swapchains: {}", e))?;

            // 初始化XR渲染器
            self.xr_renderer = XrRenderer::new(Arc::new(device.clone()), Arc::new(queue.clone()));
            self.xr_renderer
                .initialize()
                .map_err(|e| format!("Failed to initialize XR renderer: {}", e))?;

            tracing::info!("XR session created successfully");
        }
        Ok(())
    }

    fn update_xr(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref mut backend) = self.xr_backend {
            // 更新手部追踪
            if let Err(e) = self.hand_tracker.update() {
                tracing::warn!("Failed to update hand tracking: {}", e);
            }

            // 从手部追踪器更新输入管理器
            self.xr_input.update_from_hand_tracker(&self.hand_tracker);

            // 处理XR事件
            let events = backend.poll_events();
            for event in events {
                self.handle_xr_event(event);
            }

            // 开始帧
            if let Ok(frame_state) = backend.begin_frame() {
                if frame_state.should_render {
                    // 获取视图
                    if let Ok(views) = backend.locate_views(frame_state.predicted_display_time) {
                        // 渲染帧
                        self.render_xr_frame(&views)?;
                    }
                }

                // 结束帧
                let layers = vec![XrCompositionLayer::Projection {
                    views: views
                        .iter()
                        .map(|view| XrProjectionView {
                            pose: view.pose,
                            fov: view.fov,
                            swapchain_index: view.view_index,
                            image_rect: [0, 0, 1920, 1080],
                        })
                        .collect(),
                }];

                if let Err(e) = backend.end_frame(&layers) {
                    tracing::warn!("Failed to end XR frame: {}", e);
                }
            }

            self.frame_count += 1;
        }
        Ok(())
    }

    fn handle_xr_event(&mut self, event: XrEvent) {
        match event {
            XrEvent::SessionStateChanged(state) => {
                tracing::info!("XR session state changed to: {:?}", state);

                match state {
                    XrSessionState::Ready => {
                        tracing::info!("XR session ready");
                    }
                    XrSessionState::Focused => {
                        tracing::info!("XR session focused");
                    }
                    XrSessionState::Visible => {
                        tracing::info!("XR session visible");
                    }
                    XrSessionState::Stopping => {
                        tracing::info!("XR session stopping");
                    }
                    XrSessionState::Exiting => {
                        tracing::info!("XR session exiting");
                    }
                    _ => {}
                }
            }
            XrEvent::ReferenceSpaceChanged => {
                tracing::info!("Reference space changed");
            }
            XrEvent::InteractionProfileChanged => {
                tracing::info!("Interaction profile changed");
            }
        }
    }

    fn render_xr_frame(&mut self, views: &[XrView]) -> Result<(), Box<dyn std::error::Error>> {
        // 这里应该渲染到XR交换链
        // 占位实现：只记录信息

        for (i, view) in views.iter().enumerate() {
            tracing::debug!(
                "Rendering view {}: position={:?}, fov={:?}",
                i,
                view.pose.position,
                view.fov
            );
        }

        // 模拟渲染一些内容
        if self.frame_count % 60 == 0 {
            // 每秒创建一个锚点
            if let Ok(ref mut anchor_manager) = SpatialAnchorManager::new() {
                let pose = Pose {
                    position: glam::Vec3::new(
                        (self.frame_count as f32 * 0.01).sin(),
                        1.5,
                        (self.frame_count as f32 * 0.01).cos(),
                    ),
                    orientation: glam::Quat::IDENTITY,
                };

                if let Ok(anchor_id) =
                    anchor_manager.create_anchor(pose, format!("AutoAnchor_{}", self.frame_count))
                {
                    tracing::info!("Created auto anchor: {:?}", anchor_id);
                }
            }
        }

        Ok(())
    }

    fn log_controller_state(&self) {
        // 记录控制器状态
        if let Some(left_controller) = self.xr_input.get_controller(Hand::Left) {
            if left_controller.trigger > 0.1 {
                tracing::info!("Left trigger: {:.2}", left_controller.trigger);
            }
            if left_controller.buttons.a {
                tracing::info!("Left A button pressed");
            }
        }

        if let Some(right_controller) = self.xr_input.get_controller(Hand::Right) {
            if right_controller.trigger > 0.1 {
                tracing::info!("Right trigger: {:.2}", right_controller.trigger);
            }
            if right_controller.buttons.a {
                tracing::info!("Right A button pressed");
            }
        }
    }

    fn log_hand_tracking(&self) {
        // 记录手部追踪状态
        if let Some(left_hand) = self.hand_tracker.get_hand_joints(Hand::Left) {
            if let Some(palm_pos) = left_hand.get_palm_position() {
                tracing::debug!("Left palm position: {:?}", palm_pos);
            }

            if let Some(index_tip) = left_hand.get_finger_tip(Finger::Index) {
                tracing::debug!("Left index tip: {:?}", index_tip);
            }
        }

        if let Some(right_hand) = self.hand_tracker.get_hand_joints(Hand::Right) {
            if let Some(palm_pos) = right_hand.get_palm_position() {
                tracing::debug!("Right palm position: {:?}", palm_pos);
            }

            if let Some(index_tip) = right_hand.get_finger_tip(Finger::Index) {
                tracing::debug!("Right index tip: {:?}", index_tip);
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("Starting XR Demo");

    let mut state = XrDemoState::new();

    // 初始化XR
    state.initialize_xr()?;

    // 创建事件循环
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title("XR Demo")
        .with_inner_size(winit::dpi::LogicalSize::new(1920, 1080))
        .build(&event_loop)?;

    // 初始化WGPU
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
    });

    let surface = unsafe { instance.create_surface(&window) }?;
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }))?;

    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            features: wgpu::Features::empty(),
            limits: wgpu::Limits::default(),
            label: None,
        },
        None,
    ))?;

    // 创建XR会话
    state.create_xr_session(&device, &queue)?;

    // 主循环
    event_loop.run(move |event, _, control_flow| {
        *control_flow = winit::event_loop::ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                tracing::info!("Close requested. Exiting...");
                *control_flow = winit::event_loop::ControlFlow::Exit;
            }
            Event::MainEventsCleared => {
                // 更新XR
                if let Err(e) = state.update_xr() {
                    tracing::error!("XR update error: {}", e);
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                }
            }
            Event::RedrawRequested(_) => {
                // 处理重绘请求
            }
            _ => {}
        }
    })?;

    tracing::info!("XR Demo finished");
    Ok(())
}
