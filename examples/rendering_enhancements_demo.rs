//! # Rendering Enhancements Demo
//!
//! 演示P2阶段渲染系统增强功能：
//! - 增强的GPU剔除系统
//! - 动态天空盒
//! - 后处理效果
//! - LOD系统
//! - 性能分析
//!
//! ## 运行
//!
//! ```bash
//! cargo run --example rendering_enhancements_demo
//! ```

use game_engine::render::{
    EnhancedGpuCuller, CullingEnhancedConfig,
    DynamicSkybox, SkyboxConfig, TimeOfDay,
    PostProcessPipeline, PostProcessConfig, AntialiasingMode, TonemapOperator,
    PerformanceAnalyzer, PerfConfig,
    gpu_driven::GpuInstance,
};
use game_engine::render::atmosphere::AtmosphereSystem;
use game_engine::render::lod::{LodSelector, LodConfig, LodQuality};
use wgpu::*;

#[derive(Debug)]
struct DemoState {
    // 增强的GPU剔除
    gpu_culler: EnhancedGpuCuller,
    instances: Vec<GpuInstance>,

    // 动态天空盒
    skybox: DynamicSkybox,
    time_of_day: f32,

    // 后处理
    post_pipeline: PostProcessPipeline,

    // LOD系统
    lod_selector: LodSelector,

    // 性能分析
    perf_analyzer: PerformanceAnalyzer,

    // 渲染统计
    frame_count: u64,
    last_fps_update: f64,
}

impl DemoState {
    async fn new(adapter: &Adapter, device: &Device, queue: &Queue) -> Self {
        println!("🚀 初始化渲染增强演示...");

        // 1. 创建增强的GPU剔除器
        println!("📦 创建GPU剔除器...");
        let culling_config = CullingEnhancedConfig {
            enable_tiled_culling: true,
            tile_size: 64,
            enable_cpu_fallback: true,
            enable_stats: true,
            ..Default::default()
        };
        let gpu_culler = EnhancedGpuCuller::new(device, culling_config);

        // 生成测试实例
        let instances = Self::generate_test_instances(1000);
        println!("✅ 生成了 {} 个测试实例", instances.len());

        // 2. 创建动态天空盒
        println!("🌤️ 创建动态天空盒...");
        let skybox_config = SkyboxConfig {
            enable_atmospheric_scattering: true,
            enable_stars: true,
            enable_celestial_bodies: true,
            resolution: 1024,
            scattering_intensity: 1.0,
            ..Default::default()
        };
        let skybox = DynamicSkybox::new(device, &skybox_config)
            .expect("Failed to create skybox");
        println!("✅ 天空盒系统初始化完成");

        // 3. 创建后处理管线
        println!("🎨 创建后处理管线...");
        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: TextureFormat::Bgra8UnormSrgb,
            width: 1920,
            height: 1080,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Opaque,
            view_formats: vec![],
        };

        let post_config = PostProcessConfig {
            antialiasing: AntialiasingMode::FXAA,
            bloom_enabled: true,
            bloom_intensity: 0.6,
            bloom_threshold: 0.8,
            ssao_enabled: true,
            ssao_radius: 0.5,
            tonemap_enabled: true,
            tonemap_operator: TonemapOperator::ACES,
            exposure: 1.0,
            gamma: 2.2,
            motion_blur_enabled: false,
            depth_of_field_enabled: false,
            color_correction_enabled: true,
            ..Default::default()
        };

        let post_pipeline = PostProcessPipeline::new(device, &surface_config);
        println!("✅ 后处理管线初始化完成");

        // 4. 创建LOD选择器
        println!("📊 创建LOD系统...");
        let lod_config = LodConfig::builder()
            .add_level(0.0, 20.0, LodQuality::High)
            .add_level(20.0, 50.0, LodQuality::Medium)
            .add_level(50.0, 100.0, LodQuality::Low)
            .add_level(100.0, f32::MAX, LodQuality::VeryLow)
            .with_screen_coverage(vec![0.1, 0.05, 0.01])
            .build();

        let lod_selector = LodSelector::new(lod_config);
        println!("✅ LOD系统初始化完成");

        // 5. 创建性能分析器
        println!("📈 创建性能分析器...");
        let perf_config = PerfConfig {
            enable_gpu_timing: true,
            enable_cpu_timing: true,
            enable_memory_tracking: true,
            history_length: 60,
            ..Default::default()
        };
        let perf_analyzer = PerformanceAnalyzer::new(device, perf_config);
        println!("✅ 性能分析器初始化完成");

        println!("\n🎉 初始化完成！\n");

        Self {
            gpu_culler,
            instances,
            skybox,
            time_of_day: 0.3, // 上午开始
            post_pipeline,
            lod_selector,
            perf_analyzer,
            frame_count: 0,
            last_fps_update: 0.0,
        }
    }

    fn generate_test_instances(count: usize) -> Vec<GpuInstance> {
        use glam::{Mat4, Vec3};

        let mut instances = Vec::with_capacity(count);

        for i in 0..count {
            // 随机位置（在100x100x100的区域内）
            let x = ((i % 10) as f32 - 5.0) * 10.0;
            let y = ((i / 10 % 10) as f32 - 5.0) * 10.0;
            let z = ((i / 100) as f32 - 5.0) * 10.0;

            let position = Vec3::new(x, y, z);
            let model = Mat4::from_translation(position);

            // AABB（单位立方体）
            let aabb_min = Vec3::new(-0.5, -0.5, -0.5);
            let aabb_max = Vec3::new(0.5, 0.5, 0.5);

            instances.push(GpuInstance {
                model: model.to_cols_array_2d(),
                aabb_min: aabb_min.to_array(),
                aabb_max: aabb_max.to_array(),
                instance_id: i as u32,
                flags: 0,
            });
        }

        instances
    }

    fn update(&mut self, queue: &Queue, delta_time: f32, camera: &Camera) {
        // 更新时间（加速时间流逝）
        self.time_of_day += delta_time * 0.01; // 1秒现实时间 = 0.01游戏天
        if self.time_of_day >= 1.0 {
            self.time_of_day -= 1.0;
        }

        // 更新天空盒时间
        self.skybox.set_time_of_day(TimeOfDay::Custom(self.time_of_day));

        // 更新天空盒
        let view_proj = camera.view_projection_matrix();
        self.skybox.update(queue, &view_proj);

        // 更新LOD系统（模拟性能数据）
        let frame_time_ms = delta_time * 1000.0;
        self.lod_selector.update_performance(frame_time_ms, Some(0.65));

        // 更新后处理配置（根据时间调整）
        if self.time_of_day > 0.25 && self.time_of_day < 0.75 {
            // 白天：增加曝光
            self.post_pipeline.set_exposure(1.2);
        } else {
            // 夜晚：减少曝光
            self.post_pipeline.set_exposure(0.6);
        }
    }

    fn render(&mut self, device: &Device, encoder: &mut CommandEncoder, view: &TextureView) {
        // 开始性能分析
        self.perf_analyzer.begin_frame(device);

        // 1. GPU剔除
        self.perf_analyzer.begin_pass("GPU Culling");
        // 实际剔除会在这里执行...
        self.perf_analyzer.end_pass("GPU Culling");

        // 2. 渲染天空盒
        self.perf_analyzer.begin_pass("Skybox");
        // 实际渲染会在这里执行...
        self.perf_analyzer.end_pass("Skybox");

        // 3. 渲染场景
        self.perf_analyzer.begin_pass("Scene Rendering");
        // 实际渲染会在这里执行...
        self.perf_analyzer.end_pass("Scene Rendering");

        // 4. 后处理
        self.perf_analyzer.begin_pass("Post Processing");
        // 实际后处理会在这里执行...
        self.perf_analyzer.end_pass("Post Processing");

        // 结束性能分析
        self.perf_analyzer.end_frame(device);

        // 每60帧打印一次性能报告
        self.frame_count += 1;
        if self.frame_count % 60 == 0 {
            self.print_stats();
        }
    }

    fn print_stats(&self) {
        println!("\n════════════════════════════════════════");
        println!("📊 性能统计报告 - 帧 #{}", self.frame_count);
        println!("════════════════════════════════════════");

        // 打印时间信息
        let time_str = match TimeOfDay::Custom(self.time_of_day) {
            TimeOfDay::Midnight => "午夜",
            TimeOfDay::Dawn => "黎明",
            TimeOfDay::Noon => "正午",
            TimeOfDay::Dusk => "黄昏",
            TimeOfDay::Custom(t) if t < 0.25 => "深夜",
            TimeOfDay::Custom(t) if t < 0.5 => "早晨",
            TimeOfDay::Custom(t) if t < 0.75 => "下午",
            TimeOfDay::Custom(_) => "傍晚",
        };
        println!("🌍 游戏时间: {:.2} ({})", self.time_of_day, time_str);

        // 打印GPU剔除统计
        let culling_stats = self.gpu_culler.get_stats();
        println!("🎯 GPU剔除:");
        println!("  - 总实例数: {}", culling_stats.total_instances);
        println!("  - 可见实例: {}", culling_stats.visible_instances);
        println!("  - 剔除率: {:.1}%", culling_stats.culling_rate * 100.0);
        println!("  - GPU时间: {:.2} ms", culling_stats.gpu_time_ms);

        // 打印LOD统计
        let adaptive_config = self.lod_selector.adaptive_config();
        println!("📏 LOD系统:");
        println!("  - 距离偏移: {:.2}", adaptive_config.frame_time_history.len());
        println!("  - 平均帧时间: {:.2} ms", adaptive_config.average_frame_time());

        // 打印性能报告
        println!("\n{}", self.perf_analyzer.generate_report());

        println!("════════════════════════════════════════\n");
    }
}

// 简单的相机结构
#[derive(Debug)]
struct Camera {
    position: glam::Vec3,
    target: glam::Vec3,
    up: glam::Vec3,
    fov: f32,
    aspect: f32,
    near: f32,
    far: f32,
}

impl Camera {
    fn new() -> Self {
        Self {
            position: glam::Vec3::new(0.0, 10.0, 50.0),
            target: glam::Vec3::new(0.0, 0.0, 0.0),
            up: glam::Vec3::Y,
            fov: 60.0,
            aspect: 1920.0 / 1080.0,
            near: 0.1,
            far: 1000.0,
        }
    }

    fn view_projection_matrix(&self) -> glam::Mat4 {
        let view = glam::Mat4::look_at_rh(
            self.position,
            self.target,
            self.up,
        );

        let projection = glam::Mat4::perspective_rh(
            self.fov.to_radians(),
            self.aspect,
            self.near,
            self.far,
        );

        projection * view
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔════════════════════════════════════════╗");
    println!("║   渲染系统增强演示                    ║");
    println!("║   P2 Rendering Enhancements Demo       ║");
    println!("╚════════════════════════════════════════╝\n");

    // 初始化WebGPU
    let instance = Instance::new(InstanceDescriptor {
        backends: Backends::all(),
        ..Default::default()
    });

    let adapter = instance
        .request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            ..Default::default()
        })
        .await
        .expect("Failed to find an appropriate adapter");

    let (device, queue) = adapter
        .request_device(
            &DeviceDescriptor {
                label: Some("Device"),
                required_features: Features::empty(),
                required_limits: Limits::default(),
            },
            None,
        )
        .await
        .expect("Failed to create device");

    // 创建演示状态
    let mut state = DemoState::new(&adapter, &device, &queue).await;
    let camera = Camera::new();

    println!("🎮 演示控制:");
    println!("  - 程序会自动运行并模拟渲染");
    println!("  - 每60帧打印一次性能报告");
    println!("  - 时间会自动流逝（1秒现实 = 0.01游戏天）");
    println!("  - 观察不同时间的天空和光照变化\n");

    // 模拟渲染循环
    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("Main Encoder"),
    });

    // 模拟100帧
    for frame in 0..100 {
        // 更新
        let delta_time = 0.016; // 假设60 FPS
        state.update(&queue, delta_time, &camera);

        // 渲染（这里只是模拟，实际不会渲染到屏幕）
        if frame % 10 == 0 {
            println!("🖼️ 渲染帧 #{}", frame);
        }

        state.render(&device, &mut encoder, &queue.as_fake_view().unwrap());
    }

    println!("\n🎉 演示完成！");

    // 最终性能报告
    println!("\n📋 最终性能报告:");
    state.print_stats();

    Ok(())
}

// 辅助函数（用于编译）
trait FakeQueue {
    fn as_fake_view(&self) -> Option<&TextureView>;
}

impl FakeQueue for Queue {
    fn as_fake_view(&self) -> Option<&TextureView> {
        None // 这是假的实现，仅用于编译演示
    }
}
