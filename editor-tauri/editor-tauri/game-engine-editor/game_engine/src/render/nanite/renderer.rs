//! # Nanite Renderer
//!
//! GPU-driven renderer for virtual geometry clusters.

use std::collections::HashMap;
use wgpu::*;
use crate::render::nanite::{Vec3, Camera, ClusterHierarchy, LODSelection};

/// Configuration for Nanite renderer
#[derive(Clone, Debug)]
pub struct RenderConfig {
    /// Enable compute shader acceleration
    pub enable_compute_acceleration: bool,
    /// Maximum instances per draw call
    pub max_instances_per_draw: u32,
    /// Enable indirect rendering
    pub enable_indirect_rendering: bool,
    /// Render pass color format
    pub color_format: TextureFormat,
    /// Render pass depth format
    pub depth_format: TextureFormat,
    /// Sample count for MSAA
    pub sample_count: u32,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            enable_compute_acceleration: true,
            max_instances_per_draw: 65536,
            enable_indirect_rendering: true,
            color_format: TextureFormat::Bgra8UnormSrgb,
            depth_format: TextureFormat::Depth32Float,
            sample_count: 4,
        }
    }
}

/// Rendering statistics
#[derive(Clone, Debug, Default)]
pub struct RenderStats {
    /// Number of visible clusters
    pub visible_clusters: usize,
    /// Number of visible triangles (estimated)
    pub visible_triangles: usize,
    /// Number of culled clusters
    pub culled_clusters: usize,
    /// Average LOD level
    pub average_lod: f32,
    /// Frame time in milliseconds
    pub frame_time_ms: f32,
    /// GPU memory usage in MB
    pub gpu_memory_mb: f32,
    /// Number of draw calls issued
    pub draw_calls: usize,
}

/// Context for rendering operations
pub struct RenderContext<'a> {
    /// Device
    pub device: &'a Device,
    /// Queue
    pub queue: &'a Queue,
    /// Command encoder
    pub encoder: &'a mut CommandEncoder,
    /// Camera
    pub camera: &'a Camera,
    /// Color target view
    pub color_view: &'a TextureView,
    /// Depth target view
    pub depth_view: &'a TextureView,
    /// Screen dimensions
    pub screen_size: (u32, u32),
}

/// Main Nanite renderer
pub struct NaniteRenderer {
    config: RenderConfig,
    /// Render pipeline
    render_pipeline: Option<RenderPipeline>,
    /// Compute pipeline for culling/LOD
    compute_pipeline: Option<ComputePipeline>,
    /// Bind group layouts
    bind_group_layouts: Vec<BindGroupLayout>,
    /// Pipeline layout
    pipeline_layout: Option<PipelineLayout>,
    /// Uniform buffers
    uniform_buffers: HashMap<String, Buffer>,
    /// Statistics
    stats: RenderStats,
}

impl NaniteRenderer {
    /// Create new Nanite renderer
    pub fn new(device: &Device, config: RenderConfig) -> Result<Self, RenderError> {
        let mut renderer = Self {
            config: config.clone(),
            render_pipeline: None,
            compute_pipeline: None,
            bind_group_layouts: Vec::new(),
            pipeline_layout: None,
            uniform_buffers: HashMap::new(),
            stats: RenderStats::default(),
        };

        renderer.initialize(device)?;

        Ok(renderer)
    }

    /// Initialize renderer resources
    fn initialize(&mut self, device: &Device) -> Result<(), RenderError> {
        // Create bind group layouts
        self.create_bind_group_layouts(device)?;

        // Create pipeline layout
        self.pipeline_layout = Some(device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Nanite Pipeline Layout"),
            bind_group_layouts: &self.bind_group_layouts,
            push_constant_ranges: &[],
        }));

        // Create render pipeline
        self.create_render_pipeline(device)?;

        // Create compute pipeline if enabled
        if self.config.enable_compute_acceleration {
            self.create_compute_pipeline(device)?;
        }

        // Create uniform buffers
        self.create_uniform_buffers(device)?;

        Ok(())
    }

    /// Create bind group layouts
    fn create_bind_group_layouts(&mut self, device: &Device) -> Result<(), RenderError> {
        // Camera uniform layout
        let camera_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Nanite Camera Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            layout: BindGroupLayoutEntry::default(),
        });

        // Instance buffer layout
        let instance_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Nanite Instance Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            layout: BindGroupLayoutEntry::default(),
        });

        self.bind_group_layouts.push(camera_layout);
        self.bind_group_layouts.push(instance_layout);

        Ok(())
    }

    /// Create render pipeline
    fn create_render_pipeline(&mut self, device: &Device) -> Result<(), RenderError> {
        // Shader code (simplified - real implementation would load from file)
        let shader_code = r#"
            struct Uniforms {
                mvp_matrix: mat4x4<f32>,
                camera_position: vec3<f32>,
                time: f32,
            }

            @group(0) @binding(0)
            uniforms: Uniforms;

            struct VertexInput {
                @location(0) position: vec3<f32>,
                @location(1) normal: vec3<f32>,
                @location(2) uv: vec2<f32>,
            }

            struct InstanceInput {
                @builtin(instance_index) instance_id: u32,
            }

            struct VertexOutput {
                @builtin(position) clip_position: vec4<f32>,
                @location(0) world_position: vec3<f32>,
                @location(1) normal: vec3<f32>,
                @location(2) uv: vec2<f32>,
            }

            @vertex
            fn vertex_main(
                vertex: VertexInput,
                instance: InstanceInput,
            ) -> VertexOutput {
                var output: VertexOutput;
                output.clip_position = uniforms.mvp_matrix * vec4<f32>(vertex.position, 1.0);
                output.world_position = vertex.position;
                output.normal = vertex.normal;
                output.uv = vertex.uv;
                return output;
            }

            @fragment
            fn fragment_main(
                @location(0) world_position: vec3<f32>,
                @location(1) normal: vec3<f32>,
            ) -> @location(0) vec4<f32> {
                let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
                let diffuse = max(dot(normal, light_dir), 0.0);
                let ambient = 0.2;
                let color = vec3<f32>(0.8, 0.8, 0.8) * (diffuse + ambient);
                return vec4<f32>(color, 1.0);
            }
        "#;

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Nanite Shader"),
            source: ShaderSource::Wgsl(shader_code.into()),
        });

        let pipeline_desc = RenderPipelineDescriptor {
            label: Some("Nanite Render Pipeline"),
            layout: self.pipeline_layout.as_ref().unwrap(),
            vertex: VertexState {
                module: &shader,
                entry_point: "vertex_main",
                buffers: &[
                    VertexBufferLayout {
                        array_stride: std::mem::size_of::<[f32; 8]>() as BufferAddress, // pos + normal + uv
                        step_mode: VertexStepMode::Vertex,
                        attributes: &[
                            VertexAttribute {
                                offset: 0,
                                shader_location: 0,
                                format: VertexFormat::Float32x3,
                            },
                            VertexAttribute {
                                offset: 12,
                                shader_location: 1,
                                format: VertexFormat::Float32x3,
                            },
                            VertexAttribute {
                                offset: 24,
                                shader_location: 2,
                                format: VertexFormat::Float32x2,
                            },
                        ],
                    },
                ],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fragment_main",
                targets: &[Some(ColorTargetState {
                    format: self.config.color_format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
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
            depth_stencil: Some(DepthStencilState {
                format: self.config.depth_format,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState {
                count: self.config.sample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        };

        self.render_pipeline = Some(device.create_render_pipeline(&pipeline_desc));

        Ok(())
    }

    /// Create compute pipeline for GPU-driven operations
    fn create_compute_pipeline(&mut self, device: &Device) -> Result<(), RenderError> {
        let shader_code = r#"
            struct CullingUniforms {
                view_matrix: mat4x4<f32>,
                projection_matrix: mat4x4<f32>,
                camera_position: vec3<f32>,
                padding: f32,
            }

            @group(0) @binding(0)
            uniforms: CullingUniforms;

            struct ClusterInput {
                bounding_sphere: vec4<f32>, // xyz = center, w = radius
                lod_level: f32,
                visible: u32,
            }

            @group(0) @binding(1)
           <storage> clusters: array<ClusterInput>;

            struct ClusterOutput {
                instance_id: u32,
                lod_level: u32,
            }

            @group(0) @binding(2)
           <storage, read_write> output_clusters: array<ClusterOutput>;

            @compute @workgroup_size(64)
            fn cull_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
                let index = global_id.x;
                if (index >= arrayLength(&clusters)) {
                    return;
                }

                let cluster = clusters[index];

                // Simple view frustum culling
                let view_pos = (uniforms.view_matrix * vec4<f32>(cluster.bounding_sphere.xyz, 1.0)).xyz;
                let dist = length(view_pos - uniforms.camera_position);

                // Check if cluster is potentially visible
                if (dist < 1000.0 && cluster.visible != 0u) {
                    output_clusters[index].instance_id = index;
                    output_clusters[index].lod_level = u32(cluster.lod_level);
                } else {
                    output_clusters[index].instance_id = 0xffffffffu;
                }
            }
        "#;

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Nanite Compute Shader"),
            source: ShaderSource::Wgsl(shader_code.into()),
        });

        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Nanite Compute Pipeline"),
            layout: self.pipeline_layout.as_ref().unwrap(),
            module: &shader,
            entry_point: "cull_main",
        });

        self.compute_pipeline = Some(pipeline);

        Ok(())
    }

    /// Create uniform buffers
    fn create_uniform_buffers(&mut self, device: &Device) -> Result<(), RenderError> {
        // Camera uniform buffer (MVP + position + time)
        let camera_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Nanite Camera Uniforms"),
            size: 256, // Enough for MVP matrix + extras
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        self.uniform_buffers.insert("camera".to_string(), camera_buffer);

        Ok(())
    }

    /// Render a frame
    pub fn render(
        &mut self,
        ctx: &mut RenderContext,
        hierarchies: &[ClusterHierarchy],
        lod_selections: &[LODSelection],
    ) -> Result<RenderStats, RenderError> {
        let start_time = std::time::Instant::now();

        // Update camera uniforms
        self.update_camera_uniforms(ctx)?;

        // Begin render pass
        let mut render_pass = ctx.encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Nanite Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: ctx.color_view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: ctx.depth_view,
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // Set pipeline and bind groups
        if let Some(ref pipeline) = self.render_pipeline {
            render_pass.set_pipeline(pipeline);

            // 使用标准绘制调用
            // This is simplified - real implementation would iterate through
            // LOD selections and issue draw calls

            self.stats.draw_calls = 1;
        }

        drop(render_pass);

        // Update statistics
        self.stats.visible_clusters = lod_selections.len();
        self.stats.visible_triangles = lod_selections.iter()
            .map(|s| s.triangle_count())
            .sum();
        self.stats.average_lod = if lod_selections.is_empty() {
            0.0
        } else {
            lod_selections.iter()
                .map(|s| s.lod_level as f32)
                .sum::<f32>() / lod_selections.len() as f32
        };
        self.stats.frame_time_ms = start_time.elapsed().as_secs_f64() as f32 * 1000.0;

        Ok(self.stats.clone())
    }

    /// Update camera uniform buffer
    fn update_camera_uniforms(&self, ctx: &RenderContext) -> Result<(), RenderError> {
        if let Some(buffer) = self.uniform_buffers.get("camera") {
            // Combine view and projection matrices
            let mvp = self.multiply_matrices(&ctx.camera.view_matrix, &ctx.camera.projection_matrix);

            // Flatten matrix into bytes
            let mut data = Vec::with_capacity(256);
            for row in &mvp {
                for val in row {
                    data.extend_from_slice(&val.to_ne_bytes());
                }
            }

            ctx.queue.write_buffer(buffer, 0, &data);
        }

        Ok(())
    }

    /// Multiply two 4x4 matrices
    fn multiply_matrices(&self, a: &[[f32; 4]; 4], b: &[[f32; 4]; 4]) -> [[f32; 4]; 4] {
        let mut result = [[0.0; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    result[i][j] += a[i][k] * b[k][j];
                }
            }
        }
        result
    }

    /// Get rendering statistics
    pub fn stats(&self) -> &RenderStats {
        &self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = RenderStats::default();
    }

    /// Clean up resources
    pub fn cleanup(&mut self) {
        self.render_pipeline = None;
        self.compute_pipeline = None;
        self.pipeline_layout = None;
        self.uniform_buffers.clear();
        self.bind_group_layouts.clear();
    }
}

/// Errors that can occur during rendering
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("Pipeline creation failed: {0}")]
    PipelineCreationFailed(String),

    #[error("Shader compilation failed: {0}")]
    ShaderCompilationFailed(String),

    #[error("Buffer creation failed: {0}")]
    BufferCreationFailed(String),

    #[error("Rendering failed: {0}")]
    RenderingFailed(String),

    #[error("Invalid render state: {0}")]
    InvalidRenderState(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_config_default() {
        let config = RenderConfig::default();
        assert_eq!(config.enable_compute_acceleration, true);
        assert_eq!(config.color_format, TextureFormat::Bgra8UnormSrgb);
    }

    #[test]
    fn test_render_stats_default() {
        let stats = RenderStats::default();
        assert_eq!(stats.visible_clusters, 0);
        assert_eq!(stats.draw_calls, 0);
    }

    #[test]
    fn test_matrix_multiplication() {
        let renderer = NaniteRenderer::new(&MockDevice::new(), RenderConfig::default());

        let identity = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];

        let result = renderer.multiply_matrices(&identity, &identity);

        for i in 0..4 {
            for j in 0..4 {
                assert!((result[i][j] - identity[i][j]).abs() < 0.001);
            }
        }
    }

    // Mock device for testing (simplified)
    struct MockDevice;
    impl MockDevice {
        fn new() -> Self { Self }
    }
}
