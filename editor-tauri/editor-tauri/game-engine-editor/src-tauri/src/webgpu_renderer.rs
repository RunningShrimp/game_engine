use crate::camera::{Camera, CameraController};
use crate::geometry::{Mesh, Vertex};
use glam::{Mat4, Vec3 as GlamVec3};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use wgpu::util::DeviceExt;
use wgpu::*;

/// Uniform buffer structure
#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct UniformBuffer {
    model: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    proj: [[f32; 4]; 4],
    light_direction: [f32; 3],
    _padding1: f32,
    light_color: [f32; 3],
    _padding2: f32,
    ambient_color: [f32; 3],
    ambient_strength: f32,
}

// Implement bytemuck traits manually to avoid orphan rule
unsafe impl bytemuck::Zeroable for UniformBuffer {
    fn zeroed() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

unsafe impl bytemuck::Pod for UniformBuffer {}

unsafe impl bytemuck::Zeroable for Vertex {
    fn zeroed() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

unsafe impl bytemuck::Pod for Vertex {}

impl Default for UniformBuffer {
    fn default() -> Self {
        Self {
            model: [[0.0; 4]; 4],
            view: [[0.0; 4]; 4],
            proj: [[0.0; 4]; 4],
            light_direction: [1.0, 1.0, 1.0],
            _padding1: 0.0,
            light_color: [1.0, 1.0, 1.0],
            _padding2: 0.0,
            ambient_color: [0.2, 0.2, 0.2],
            ambient_strength: 0.3,
        }
    }
}

/// WebGPU renderer state
pub struct WebGPURenderer<'a> {
    device: Option<Device>,
    queue: Option<Queue>,
    adapter: Option<Adapter>,
    surface: Option<&'a Surface<'a>>,
    surface_config: Option<SurfaceConfiguration>,
    render_pipeline: Option<RenderPipeline>,
    grid_pipeline: Option<RenderPipeline>,

    // Uniform buffer
    uniform_buffer: Option<Buffer>,
    uniform_bind_group: Option<BindGroup>,

    // Mesh buffers
    vertex_buffer: Option<Buffer>,
    index_buffer: Option<Buffer>,
    num_indices: u32,

    // Grid buffers
    grid_vertex_buffer: Option<Buffer>,
    grid_index_buffer: Option<Buffer>,
    grid_num_indices: u32,

    // Camera
    camera: Camera,
    camera_controller: CameraController,

    // Performance tracking
    last_frame_time: Instant,
    frame_count: u32,
    fps_update_time: Instant,
    current_fps: u32,
    frame_time_ms: f32,

    // State
    initialized: bool,
}

impl<'a> WebGPURenderer<'a> {
    pub fn new() -> Self {
        Self {
            device: None,
            queue: None,
            adapter: None,
            surface: None,
            surface_config: None,
            render_pipeline: None,
            grid_pipeline: None,
            uniform_buffer: None,
            uniform_bind_group: None,
            vertex_buffer: None,
            index_buffer: None,
            num_indices: 0,
            grid_vertex_buffer: None,
            grid_index_buffer: None,
            grid_num_indices: 0,
            camera: Camera::default(),
            camera_controller: CameraController::new(),
            last_frame_time: Instant::now(),
            frame_count: 0,
            fps_update_time: Instant::now(),
            current_fps: 60,
            frame_time_ms: 16.67,
            initialized: false,
        }
    }

    /// Initialize the WebGPU renderer
    pub async fn initialize(&mut self) -> Result<(), String> {
        // Create instance
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        });

        // This will be called with a valid surface in real usage
        // For now, we'll create a minimal setup
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or("Failed to find GPU adapter")?;

        // Get device and queue
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: Some("WebGPU Device"),
                    required_features: Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
                    required_limits: Limits::default(),
                    memory_hints: MemoryHints::default(),
                },
                None,
            )
            .await
            .map_err(|e| format!("Failed to create device: {}", e))?;

        self.device = Some(device);
        self.queue = Some(queue);
        self.adapter = Some(adapter);

        self.initialized = true;
        Ok(())
    }

    /// Setup rendering for a canvas surface
    pub fn setup_surface(&mut self, surface: &'a Surface<'a>, width: u32, height: u32) -> Result<(), String> {
        let device = self.device.as_ref().ok_or("Device not initialized")?;
        let adapter = self.adapter.as_ref().ok_or("Adapter not initialized")?;

        self.surface = Some(surface);

        // Get surface configuration
        let capabilities = surface.get_capabilities(adapter);

        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: *capabilities.formats.first().ok_or("No supported formats")?,
            width,
            height,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(device, &surface_config);
        self.surface_config = Some(surface_config.clone());

        // Update camera aspect ratio
        self.camera.set_aspect_ratio(width as f32 / height as f32);

        Ok(())
    }

    /// Create render pipeline and resources
    pub fn create_pipelines(&mut self) -> Result<(), String> {
        let device = self.device.as_ref().ok_or("Device not initialized")?;

        // Load shaders
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shader"),
            source: ShaderSource::Wgsl(include_str!("shaders.wgsl").into()),
        });

        // Create uniform buffer
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[UniformBuffer::default()]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        // Create uniform bind group layout
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("Uniform Bind Group Layout"),
            });

        // Create uniform bind group
        let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("Uniform Bind Group"),
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create main render pipeline
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            cache: None,
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as BufferAddress,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &vertex_attr_array![
                        0 => Float32x3, // Position
                        1 => Float32x3, // Normal
                        2 => Float32x2, // UV
                    ],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: self.surface_config.as_ref().unwrap().format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // Create grid pipeline (with blending for transparency)
        let grid_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Grid Pipeline"),
            layout: Some(&pipeline_layout),
            cache: None,
            vertex: VertexState {
                module: &shader,
                entry_point: "grid_vs",
                buffers: &[VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as BufferAddress,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &vertex_attr_array![
                        0 => Float32x3, // Position
                    ],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "grid_fs",
                targets: &[Some(ColorTargetState {
                    format: self.surface_config.as_ref().unwrap().format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // Create cube mesh
        let cube = Mesh::cube(1.0);
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&cube.vertices),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&cube.indices),
            usage: BufferUsages::INDEX,
        });

        self.num_indices = cube.indices.len() as u32;

        // Create grid mesh
        let grid = Mesh::grid(20.0, 20);
        let grid_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Grid Vertex Buffer"),
            contents: bytemuck::cast_slice(&grid.vertices),
            usage: BufferUsages::VERTEX,
        });

        let grid_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Grid Index Buffer"),
            contents: bytemuck::cast_slice(&grid.indices),
            usage: BufferUsages::INDEX,
        });

        self.grid_num_indices = grid.indices.len() as u32;

        // Store everything
        self.render_pipeline = Some(render_pipeline);
        self.grid_pipeline = Some(grid_pipeline);
        self.uniform_buffer = Some(uniform_buffer);
        self.uniform_bind_group = Some(uniform_bind_group);
        self.vertex_buffer = Some(vertex_buffer);
        self.index_buffer = Some(index_buffer);
        self.grid_vertex_buffer = Some(grid_vertex_buffer);
        self.grid_index_buffer = Some(grid_index_buffer);

        Ok(())
    }

    /// Render a frame
    pub fn render(&mut self) -> Result<FrameStats, String> {
        let device = self.device.as_ref().ok_or("Device not initialized")?;
        let queue = self.queue.as_ref().ok_or("Queue not initialized")?;
        let surface = self.surface.as_ref().ok_or("Surface not set up")?;

        // Calculate frame time
        let now = Instant::now();
        let delta = now.duration_since(self.last_frame_time);
        self.last_frame_time = now;
        self.frame_time_ms = delta.as_secs_f32() * 1000.0;

        // Update FPS counter
        self.frame_count += 1;
        let fps_delta = now.duration_since(self.fps_update_time);
        if fps_delta >= Duration::from_secs(1) {
            self.current_fps = self.frame_count;
            self.frame_count = 0;
            self.fps_update_time = now;
        }

        // Get the texture to render to
        let surface_texture = surface
            .get_current_texture()
            .map_err(|e| format!("Failed to get surface texture: {}", e))?;

        let texture_view = surface_texture.texture.create_view(&TextureViewDescriptor {
            label: Some("Surface Texture View"),
            format: None,
            dimension: None,
            aspect: TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

        // Create command encoder
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // Update uniform buffer
        let uniforms = self.create_uniforms();
        let queue_ref = self.queue.as_ref().unwrap();
        queue_ref.write_buffer(
            self.uniform_buffer.as_ref().unwrap(),
            0,
            bytemuck::cast_slice(&[uniforms]),
        );

        {
            // Begin render pass
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.06,
                            g: 0.06,
                            b: 0.08,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Draw grid
            if let (Some(grid_pipeline), Some(grid_vertex_buffer), Some(grid_index_buffer)) = (
                &self.grid_pipeline,
                &self.grid_vertex_buffer,
                &self.grid_index_buffer,
            ) {
                render_pass.set_pipeline(grid_pipeline);
                render_pass.set_bind_group(0, self.uniform_bind_group.as_ref().unwrap(), &[]);
                render_pass.set_vertex_buffer(0, grid_vertex_buffer.slice(..));
                render_pass.set_index_buffer(grid_index_buffer.slice(..), IndexFormat::Uint16);
                render_pass.draw_indexed(0..self.grid_num_indices, 0, 0..1);
            }

            // Draw cube
            if let (Some(pipeline), Some(vertex_buffer), Some(index_buffer)) = (
                &self.render_pipeline,
                &self.vertex_buffer,
                &self.index_buffer,
            ) {
                render_pass.set_pipeline(pipeline);
                render_pass.set_bind_group(0, self.uniform_bind_group.as_ref().unwrap(), &[]);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint16);
                render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
            }
        }

        // Submit commands
        queue.submit(std::iter::once(encoder.finish()));

        // Present
        surface_texture.present();

        Ok(FrameStats {
            fps: self.current_fps,
            frame_time_ms: self.frame_time_ms,
            draw_calls: 2, // Grid + Cube
            triangles: self.num_indices as u32 / 3 + self.grid_num_indices as u32 / 2,
        })
    }

    /// Create uniform buffer data
    fn create_uniforms(&self) -> UniformBuffer {
        UniformBuffer {
            model: Mat4::from_diagonal(glam::Vec4::new(1.0, 1.0, 1.0, 1.0)).to_cols_array_2d(),
            view: self.camera.view_matrix().to_cols_array_2d(),
            proj: self.camera.projection_matrix().to_cols_array_2d(),
            light_direction: [1.0, 1.0, 1.0],
            _padding1: 0.0,
            light_color: [1.0, 1.0, 0.9],
            _padding2: 0.0,
            ambient_color: [0.15, 0.15, 0.18],
            ambient_strength: 0.4,
        }
    }

    /// Resize the renderer
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), String> {
        let device = self.device.as_ref().ok_or("Device not initialized")?;
        let surface = self.surface.as_ref().ok_or("Surface not set up")?;

        if width == 0 || height == 0 {
            return Ok(());
        }

        // Update surface configuration
        if let Some(config) = &mut self.surface_config {
            config.width = width;
            config.height = height;
            surface.configure(device, config);

            // Update camera aspect ratio
            self.camera.set_aspect_ratio(width as f32 / height as f32);
        }

        Ok(())
    }

    /// Handle camera control
    pub fn handle_mouse_down(&mut self, x: f32, y: f32, button: u32) {
        self.camera_controller.handle_mouse_down(x, y, button);
    }

    pub fn handle_mouse_up(&mut self, button: u32) {
        self.camera_controller.handle_mouse_up(button);
    }

    pub fn handle_mouse_move(&mut self, x: f32, y: f32) {
        self.camera_controller
            .handle_mouse_move(x, y, &mut self.camera);
    }

    pub fn handle_scroll(&mut self, delta: f32) {
        self.camera_controller
            .handle_scroll(delta, &mut self.camera);
    }

    /// Get camera reference
    pub fn camera(&self) -> &Camera {
        &self.camera
    }
}

impl<'a> Default for WebGPURenderer<'a> {
    fn default() -> Self {
        Self::new()
    }
}

/// Frame statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameStats {
    pub fps: u32,
    pub frame_time_ms: f32,
    pub draw_calls: u32,
    pub triangles: u32,
}

impl Default for FrameStats {
    fn default() -> Self {
        Self {
            fps: 60,
            frame_time_ms: 16.67,
            draw_calls: 0,
            triangles: 0,
        }
    }
}

/// Legacy types for compatibility
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
            rotation: Quat { x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
            scale: Vec3 { x: 1.0, y: 1.0, z: 1.0 },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityData {
    pub id: String,
    pub name: String,
    pub transform: Transform,
    pub visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneData {
    pub entities: Vec<EntityData>,
    pub background_color: [f32; 4],
}
