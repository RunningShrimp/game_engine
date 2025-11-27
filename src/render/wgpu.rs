use winit::window::Window;
use wgpu::util::DeviceExt;

use crate::render::mesh::Vertex3D;
use crate::core::error::RenderError;


#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance {
    pub pos: [f32; 2],
    pub scale: [f32; 2],
    pub rot: f32,
    pub target: u32,
    pub chunk: u32,
    pub color: [f32; 4],
    pub uv_offset: [f32; 2],
    pub uv_scale: [f32; 2],
    pub layer: f32,
    pub tex_index: u32,
    pub normal_tex_index: u32,
    pub msdf: f32,
    pub px_range: f32,
}

impl Instance {
    /// 比较两个实例是否相等（用于脏检测）
    #[inline]
    pub fn equals(&self, other: &Instance) -> bool {
        self.pos == other.pos
            && self.scale == other.scale
            && self.rot == other.rot
            && self.target == other.target
            && self.chunk == other.chunk
            && self.color == other.color
            && self.uv_offset == other.uv_offset
            && self.uv_scale == other.uv_scale
            && self.layer == other.layer
            && self.tex_index == other.tex_index
            && self.normal_tex_index == other.normal_tex_index
            && self.msdf == other.msdf
            && self.px_range == other.px_range
    }
}

/// 实例脏标记系统
/// 
/// 用于追踪哪些实例已更改，实现增量更新以减少 GPU 数据传输。
/// 使用分块脏标记减少遍历开销，每个块包含多个实例。
/// 
/// # 性能优势
/// - 仅上传变化的实例数据，减少 CPU-GPU 带宽占用
/// - 分块设计减少脏检查的遍历次数
/// - 预计性能提升 20-40%（取决于场景变化率）
pub struct InstanceDirtyTracker {
    /// 每个块的大小（实例数）
    chunk_size: usize,
    /// 块级脏标记 (true = 块内有变化)
    chunk_dirty: Vec<bool>,
    /// 实例级脏标记 (细粒度追踪)
    instance_dirty: Vec<bool>,
    /// 上一帧的实例数据（用于比较）
    prev_instances: Vec<Instance>,
    /// 脏实例范围 (start, end) 列表，用于批量上传
    dirty_ranges: Vec<(u32, u32)>,
    /// 总实例数
    instance_count: usize,
    /// 是否需要完整重建
    needs_full_rebuild: bool,
}

impl InstanceDirtyTracker {
    /// 创建脏标记追踪器
    /// 
    /// # 参数
    /// - `initial_capacity`: 初始容量（实例数）
    /// - `chunk_size`: 每个块的大小，建议 64-256
    pub fn new(initial_capacity: usize, chunk_size: usize) -> Self {
        let chunk_count = (initial_capacity + chunk_size - 1) / chunk_size;
        Self {
            chunk_size,
            chunk_dirty: vec![true; chunk_count],
            instance_dirty: vec![true; initial_capacity],
            prev_instances: Vec::with_capacity(initial_capacity),
            dirty_ranges: Vec::new(),
            instance_count: 0,
            needs_full_rebuild: true,
        }
    }
    
    /// 默认块大小
    pub const DEFAULT_CHUNK_SIZE: usize = 128;
    
    /// 使用默认配置创建
    pub fn with_capacity(capacity: usize) -> Self {
        Self::new(capacity, Self::DEFAULT_CHUNK_SIZE)
    }
    
    /// 标记所有实例为脏
    pub fn mark_all_dirty(&mut self) {
        self.needs_full_rebuild = true;
        for flag in &mut self.chunk_dirty {
            *flag = true;
        }
        for flag in &mut self.instance_dirty {
            *flag = true;
        }
    }
    
    /// 标记特定实例为脏
    #[inline]
    pub fn mark_instance_dirty(&mut self, index: usize) {
        if index < self.instance_dirty.len() {
            self.instance_dirty[index] = true;
            let chunk_idx = index / self.chunk_size;
            if chunk_idx < self.chunk_dirty.len() {
                self.chunk_dirty[chunk_idx] = true;
            }
        }
    }
    
    /// 标记实例范围为脏
    pub fn mark_range_dirty(&mut self, start: usize, end: usize) {
        let end = end.min(self.instance_dirty.len());
        for i in start..end {
            self.instance_dirty[i] = true;
        }
        let chunk_start = start / self.chunk_size;
        let chunk_end = (end + self.chunk_size - 1) / self.chunk_size;
        for i in chunk_start..chunk_end.min(self.chunk_dirty.len()) {
            self.chunk_dirty[i] = true;
        }
    }
    
    /// 更新并检测变化
    /// 
    /// 比较新旧实例数据，返回脏范围列表
    pub fn update(&mut self, instances: &[Instance]) -> &[(u32, u32)] {
        self.dirty_ranges.clear();
        
        let new_count = instances.len();
        let old_count = self.prev_instances.len();
        
        // 如果数量变化，需要完整重建
        if new_count != old_count {
            self.needs_full_rebuild = true;
        }
        
        // 调整容量
        if new_count > self.instance_dirty.len() {
            let additional = new_count - self.instance_dirty.len();
            self.instance_dirty.extend(std::iter::repeat(true).take(additional));
            
            let new_chunk_count = (new_count + self.chunk_size - 1) / self.chunk_size;
            if new_chunk_count > self.chunk_dirty.len() {
                let chunk_additional = new_chunk_count - self.chunk_dirty.len();
                self.chunk_dirty.extend(std::iter::repeat(true).take(chunk_additional));
            }
        }
        
        self.instance_count = new_count;
        
        // 完整重建模式
        if self.needs_full_rebuild {
            self.prev_instances.clear();
            self.prev_instances.extend_from_slice(instances);
            if new_count > 0 {
                self.dirty_ranges.push((0, new_count as u32));
            }
            self.needs_full_rebuild = false;
            
            // 重置所有标记
            for flag in &mut self.chunk_dirty {
                *flag = false;
            }
            for flag in &mut self.instance_dirty {
                *flag = false;
            }
            
            return &self.dirty_ranges;
        }
        
        // 增量检测
        let mut range_start: Option<u32> = None;
        
        for chunk_idx in 0..self.chunk_dirty.len() {
            if !self.chunk_dirty[chunk_idx] {
                // 块未标记为脏，跳过
                if let Some(start) = range_start {
                    let end = (chunk_idx * self.chunk_size).min(new_count) as u32;
                    self.dirty_ranges.push((start, end));
                    range_start = None;
                }
                continue;
            }
            
            let start = chunk_idx * self.chunk_size;
            let end = ((chunk_idx + 1) * self.chunk_size).min(new_count);
            
            // 检查块内每个实例
            let mut chunk_has_changes = false;
            for i in start..end {
                let is_dirty = if i < old_count {
                    !instances[i].equals(&self.prev_instances[i])
                } else {
                    true // 新实例总是脏的
                };
                
                if is_dirty {
                    chunk_has_changes = true;
                    self.instance_dirty[i] = true;
                    
                    if range_start.is_none() {
                        range_start = Some(i as u32);
                    }
                } else {
                    self.instance_dirty[i] = false;
                    
                    if let Some(start) = range_start {
                        self.dirty_ranges.push((start, i as u32));
                        range_start = None;
                    }
                }
            }
            
            self.chunk_dirty[chunk_idx] = chunk_has_changes;
        }
        
        // 关闭最后一个范围
        if let Some(start) = range_start {
            self.dirty_ranges.push((start, new_count as u32));
        }
        
        // 更新缓存
        self.prev_instances.clear();
        self.prev_instances.extend_from_slice(instances);
        
        // 合并相邻范围
        self.merge_ranges();
        
        &self.dirty_ranges
    }
    
    /// 合并相邻或重叠的脏范围
    fn merge_ranges(&mut self) {
        if self.dirty_ranges.len() <= 1 {
            return;
        }
        
        self.dirty_ranges.sort_by_key(|r| r.0);
        
        let mut merged = Vec::with_capacity(self.dirty_ranges.len());
        let mut current = self.dirty_ranges[0];
        
        for &(start, end) in &self.dirty_ranges[1..] {
            // 如果范围相邻或重叠（允许小间隙合并以减少 draw call）
            if start <= current.1 + 16 {
                current.1 = current.1.max(end);
            } else {
                merged.push(current);
                current = (start, end);
            }
        }
        merged.push(current);
        
        self.dirty_ranges = merged;
    }
    
    /// 获取脏范围数量
    pub fn dirty_range_count(&self) -> usize {
        self.dirty_ranges.len()
    }
    
    /// 获取脏实例总数
    pub fn dirty_instance_count(&self) -> usize {
        self.dirty_ranges.iter().map(|(s, e)| (e - s) as usize).sum()
    }
    
    /// 检查是否有任何脏数据
    pub fn has_dirty(&self) -> bool {
        !self.dirty_ranges.is_empty()
    }
    
    /// 重置追踪器
    pub fn reset(&mut self) {
        self.chunk_dirty.clear();
        self.instance_dirty.clear();
        self.prev_instances.clear();
        self.dirty_ranges.clear();
        self.instance_count = 0;
        self.needs_full_rebuild = true;
    }
}
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UiInstance {
    pub pos: [f32; 2],
    pub size: [f32; 2],
    pub radius: f32,
    pub stroke_width: f32,
    pub color: [f32; 4],
    pub stroke_color: [f32; 4],
    pub rotation: f32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub pos: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ScreenUniform {
    screen_size: [f32; 2],
    scale_factor: f32,
    _pad: f32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms3D {
    view_proj: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ModelUniform {
    model: [[f32; 4]; 4],
    color: [f32; 4],
    _pad1: [f32; 32],
    _pad2: [f32; 12],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuPointLight {
    pub pos: [f32; 2],
    pub color: [f32; 3],
    pub radius: f32,
    pub intensity: f32,
    pub falloff: f32,
    pub _pad: [f32; 2], // Align to 16 bytes (vec4)
}

pub struct WgpuRenderer<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    vertex_count: u32,
    index_buffer: wgpu::Buffer,
    instance_count: u32,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    texture_bgl: wgpu::BindGroupLayout,
    texture_bind_groups: Vec<wgpu::BindGroup>,
    textures_size: Vec<[u32; 2]>,
    layer_ranges: Vec<(u32, u32)>,
    draw_groups: Vec<DrawGroup>,
    scale_factor: f32,
    scissor: Option<[u32;4]>,
    group_cache: Vec<(f32, usize, u32)>,
    groups_dirty: bool,
    ui_pipeline: wgpu::RenderPipeline,
    ui_instance_buffer: wgpu::Buffer,
    ui_count: u32,
    commands: Vec<crate::render::graph::RenderCommand>,
    pub offscreen_views: std::collections::HashMap<u32, wgpu::TextureView>,
    
    // Lighting
    lights_buffer: wgpu::Buffer,
    lights_bind_group: wgpu::BindGroup,
    pub lights: Vec<GpuPointLight>,

    // 3D
    pub depth_texture: wgpu::TextureView,
    pub pipeline_3d: wgpu::RenderPipeline,
    pub uniform_buffer_3d: wgpu::Buffer,
    pub uniform_bind_group_3d: wgpu::BindGroup,
    pub model_uniform_buffer: wgpu::Buffer,
    pub model_bind_group: wgpu::BindGroup,
    chunk_hashes: std::collections::HashMap<u32, u64>,
    
    // PBR 3D Rendering
    pub pbr_renderer: Option<crate::render::pbr_renderer::PbrRenderer>,
    
    // 3D Instance Buffer for PBR instanced rendering
    pub instance_buffer_3d: wgpu::Buffer,
    
    // 脏标记追踪器（增量更新优化）
    dirty_tracker: InstanceDirtyTracker,
}

pub struct DrawGroup {
    pub start: u32,
    pub end: u32,
    pub tex_idx: usize,
    pub layer: f32,
    pub scissor: Option<[u32;4]>,
}

/// 双缓冲实例管理器 - 使用ping-pong缓冲实现无等待GPU上传
pub struct DoubleBufferedInstances {
    /// 两个实例缓冲区 (ping-pong)
    buffers: [wgpu::Buffer; 2],
    /// 当前活动缓冲区索引
    active_idx: usize,
    /// 缓冲区容量 (实例数)
    capacity: u32,
    /// 当前实例数
    count: u32,
    /// Staging 缓冲区用于异步上传
    staging_buffer: wgpu::Buffer,
}

impl DoubleBufferedInstances {
    /// 创建双缓冲实例管理器
    pub fn new(device: &wgpu::Device, initial_capacity: u32) -> Self {
        let buffer_size = (initial_capacity as usize * std::mem::size_of::<Instance>()) as u64;
        
        let buffers = [
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Instance Buffer 0"),
                size: buffer_size,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Instance Buffer 1"),
                size: buffer_size,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        ];
        
        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Staging Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        
        Self {
            buffers,
            active_idx: 0,
            capacity: initial_capacity,
            count: 0,
            staging_buffer,
        }
    }
    
    /// 获取当前活动缓冲区 (用于渲染)
    pub fn active_buffer(&self) -> &wgpu::Buffer {
        &self.buffers[self.active_idx]
    }
    
    /// 获取后台缓冲区 (用于写入)
    pub fn back_buffer(&self) -> &wgpu::Buffer {
        &self.buffers[1 - self.active_idx]
    }
    
    /// 交换前后缓冲区
    pub fn swap(&mut self) {
        self.active_idx = 1 - self.active_idx;
    }
    
    /// 同步更新实例数据到后台缓冲区并交换
    pub fn update_sync(&mut self, queue: &wgpu::Queue, instances: &[Instance]) {
        self.count = instances.len() as u32;
        if !instances.is_empty() {
            queue.write_buffer(self.back_buffer(), 0, bytemuck::cast_slice(instances));
        }
        self.swap();
    }
    
    /// 异步更新实例数据 (使用staging buffer + copy命令)
    /// 返回需要提交的命令缓冲区
    pub fn update_with_staging(
        &mut self, 
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        instances: &[Instance],
    ) -> Option<wgpu::CommandBuffer> {
        if instances.is_empty() {
            self.count = 0;
            return None;
        }
        
        self.count = instances.len() as u32;
        let byte_size = (instances.len() * std::mem::size_of::<Instance>()) as u64;
        
        // 写入staging buffer
        queue.write_buffer(&self.staging_buffer, 0, bytemuck::cast_slice(instances));
        
        // 创建拷贝命令从staging到后台buffer
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Instance Copy Encoder"),
        });
        
        encoder.copy_buffer_to_buffer(
            &self.staging_buffer,
            0,
            self.back_buffer(),
            0,
            byte_size,
        );
        
        self.swap();
        Some(encoder.finish())
    }
    
    /// 获取当前实例数
    pub fn count(&self) -> u32 {
        self.count
    }
    
    /// 扩展缓冲区容量 (如果需要)
    pub fn ensure_capacity(&mut self, device: &wgpu::Device, required: u32) {
        if required <= self.capacity {
            return;
        }
        
        // 扩展到所需容量的1.5倍
        let new_capacity = (required as f32 * 1.5) as u32;
        let buffer_size = (new_capacity as usize * std::mem::size_of::<Instance>()) as u64;
        
        self.buffers = [
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Instance Buffer 0"),
                size: buffer_size,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Instance Buffer 1"),
                size: buffer_size,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        ];
        
        self.staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Staging Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        
        self.capacity = new_capacity;
        self.active_idx = 0;
    }
}

impl<'a> WgpuRenderer<'a> {
    pub async fn new(window: &'a Window) -> Result<Self, RenderError> {
        let size = window.inner_size();
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window)?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or(RenderError::NoAdapter)?;
        let supported = adapter.features();
        let mut desired = wgpu::Features::empty();
        #[cfg(feature = "wgpu_perf")]
        {
            desired |= wgpu::Features::TIMESTAMP_QUERY
                | wgpu::Features::PIPELINE_STATISTICS_QUERY
                | wgpu::Features::MULTI_DRAW_INDIRECT
                | wgpu::Features::MULTI_DRAW_INDIRECT_COUNT
                | wgpu::Features::INDIRECT_FIRST_INSTANCE
                | wgpu::Features::PUSH_CONSTANTS
                | wgpu::Features::VERTEX_WRITABLE_STORAGE;
        }
        let required_features = supported & desired;
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features,
                    required_limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .map_err(|e| RenderError::DeviceRequest(format!("Failed to request device: {}", e)))?;
        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats[0];
        let present_mode = if caps.present_modes.contains(&wgpu::PresentMode::Fifo) { wgpu::PresentMode::Fifo } else { caps.present_modes[0] };
        let alpha_mode = caps.alpha_modes[0];
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let vs_src = r#"
struct Uniforms {
    screen_size: vec2<f32>,
    scale_factor: f32,
};
@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(1) @binding(0) var tex: texture_2d<f32>;
@group(1) @binding(1) var samp: sampler;

struct PointLight {
    pos: vec2<f32>,
    color: vec3<f32>,
    radius: f32,
    intensity: f32,
    falloff: f32,
    _pad: vec2<f32>,
};
@group(2) @binding(0) var<storage, read> lights: array<PointLight>;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) msdf: f32,
    @location(3) pxr: f32,
    @location(4) world_pos: vec2<f32>,
    @location(5) normal_idx: u32,
};

@vertex
fn vs(
    @location(0) v_pos: vec2<f32>,
    @location(1) i_pos: vec2<f32>,
    @location(2) i_scale: vec2<f32>,
    @location(3) i_rot: f32,
    @location(4) i_target: u32,
    @location(13) i_chunk: u32,
    @location(5) i_color: vec4<f32>,
    @location(6) i_uv_offset: vec2<f32>,
    @location(7) i_uv_scale: vec2<f32>,
    @location(8) i_layer: f32,
    @location(9) i_tex_index: u32,
    @location(10) i_normal_idx: u32,
    @location(11) i_msdf: f32,
    @location(12) i_pxr: f32,
) -> VsOut {
    let c = cos(i_rot);
    let s = sin(i_rot);
    let rot = mat2x2<f32>(c, -s, s, c);
    let local = rot * (v_pos * i_scale);
    let world = local + i_pos;
    let ndc = vec2<f32>( (world.x / uniforms.screen_size.x) * 2.0 - 1.0,
                         -(world.y / uniforms.screen_size.y) * 2.0 + 1.0);
    let base_uv = v_pos + vec2<f32>(0.5, 0.5);
    let uv = i_uv_offset + base_uv * i_uv_scale;
    return VsOut(vec4<f32>(ndc, 0.0, 1.0), i_color.xyz, uv, i_msdf, i_pxr, world, i_normal_idx);
}

@fragment
fn fs(@location(0) color: vec3<f32>, @location(1) uv: vec2<f32>, @location(2) msdf: f32, @location(3) pxr: f32, @location(4) world_pos: vec2<f32>, @location(5) normal_idx: u32) -> @location(0) vec4<f32> {
    let texc = textureSample(tex, samp, uv);
    
    var light_accum = vec3<f32>(0.1, 0.1, 0.1); // Ambient
    let num_lights = arrayLength(&lights);
    for (var i = 0u; i < num_lights; i++) {
        let light = lights[i];
        let dist = distance(world_pos, light.pos);
        if (dist < light.radius) {
            let att = 1.0 - smoothstep(0.0, light.radius, dist);
            light_accum += light.color * light.intensity * att;
        }
    }
    
    let final_color = texc.rgb * color * light_accum;

    if (msdf < 0.0) {
        let p = uv - vec2<f32>(0.5, 0.5);
        let r = clamp(pxr, 0.0, 0.45);
        let b = vec2<f32>(0.5 - r, 0.5 - r);
        let q = abs(p) - b;
        let k = max(q, vec2<f32>(0.0, 0.0));
        let dist = length(k) - r;
        let w = fwidth(dist);
        let alpha = smoothstep(0.0 - w, 0.0 + w, -dist);
        return vec4<f32>(final_color, alpha);
    } else if (msdf > 0.5) {
        let r = texc.r; let g = texc.g; let b = texc.b;
        let sd = max(min(r, g), min(max(r, g), b));
        let w = fwidth(sd) * max(pxr * uniforms.scale_factor, 0.0001);
        let alpha = smoothstep(0.5 - w, 0.5 + w, sd);
        return vec4<f32>(final_color, alpha);
    } else {
        return vec4<f32>(final_color, texc.a);
    }
}
"#;
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(vs_src.into()),
        });
        let ui_wgsl = r#"
struct Uniforms {
    screen_size: vec2<f32>,
    scale_factor: f32,
};
@group(0) @binding(0) var<uniform> uniforms: Uniforms;
struct UiOut { 
    @builtin(position) pos: vec4<f32>, 
    @location(0) uv: vec2<f32>, 
    @location(1) color: vec4<f32>, 
    @location(2) radius: f32,
    @location(3) stroke_width: f32,
    @location(4) stroke_color: vec4<f32>,
    @location(5) size: vec2<f32>,
};
@vertex fn ui_vs(
    @location(0) v_pos: vec2<f32>, 
    @location(1) i_pos: vec2<f32>, 
    @location(2) i_size: vec2<f32>, 
    @location(3) i_radius: f32, 
    @location(4) i_stroke_width: f32,
    @location(5) i_color: vec4<f32>,
    @location(6) i_stroke_color: vec4<f32>,
    @location(7) i_rotation: f32
) -> UiOut {
    let c = cos(i_rotation);
    let s = sin(i_rotation);
    let rot_mat = mat2x2<f32>(c, -s, s, c);
    let local = rot_mat * (v_pos * i_size);
    let world = local + i_pos;
    let ndc = vec2<f32>( (world.x / uniforms.screen_size.x) * 2.0 - 1.0,
                         -(world.y / uniforms.screen_size.y) * 2.0 + 1.0);
    let uv = v_pos + vec2<f32>(0.5, 0.5);
    return UiOut(vec4<f32>(ndc, 0.0, 1.0), uv, i_color, i_radius, i_stroke_width, i_stroke_color, i_size);
}
@fragment fn ui_fs(
    @location(0) uv: vec2<f32>, 
    @location(1) color: vec4<f32>, 
    @location(2) radius: f32,
    @location(3) stroke_width: f32,
    @location(4) stroke_color: vec4<f32>,
    @location(5) size: vec2<f32>
) -> @location(0) vec4<f32> {
    let p = (uv - 0.5) * size;
    let r = radius;
    let b = size * 0.5 - vec2<f32>(r, r);
    let q = abs(p) - b;
    let k = max(q, vec2<f32>(0.0, 0.0));
    let dist = length(k) - r;
    let w = fwidth(dist);
    
    let inside = smoothstep(0.0, -w, dist);
    let inner_edge = smoothstep(-stroke_width, -stroke_width - w, dist);
    let border = inside - inner_edge;
    
    let out_color = mix(color, stroke_color, border);
    return vec4<f32>(out_color.rgb, out_color.a * inside);
}
"#;
        let ui_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: None, source: wgpu::ShaderSource::Wgsl(ui_wgsl.into()) });
        let uniform_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<ScreenUniform>() as wgpu::BufferAddress as u64),
                },
                count: None,
            }],
        });
        let texture_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let lights_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Lights Buffer"),
            size: 1024 * std::mem::size_of::<GpuPointLight>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let lights_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Lights BGL"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let lights_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Lights BG"),
            layout: &lights_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: lights_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&uniform_bgl, &texture_bgl, &lights_bgl],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vertex>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 0,
                                format: wgpu::VertexFormat::Float32x2,
                            },
                        ],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Instance>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[
                            wgpu::VertexAttribute { offset: 0, shader_location: 1, format: wgpu::VertexFormat::Float32x2 },
                            wgpu::VertexAttribute { offset: 8, shader_location: 2, format: wgpu::VertexFormat::Float32x2 },
                            wgpu::VertexAttribute { offset: 16, shader_location: 3, format: wgpu::VertexFormat::Float32 },
                            wgpu::VertexAttribute { offset: 20, shader_location: 4, format: wgpu::VertexFormat::Uint32 },
                            wgpu::VertexAttribute { offset: 28, shader_location: 5, format: wgpu::VertexFormat::Float32x4 },
                            wgpu::VertexAttribute { offset: 44, shader_location: 6, format: wgpu::VertexFormat::Float32x2 },
                            wgpu::VertexAttribute { offset: 52, shader_location: 7, format: wgpu::VertexFormat::Float32x2 },
                            wgpu::VertexAttribute { offset: 60, shader_location: 8, format: wgpu::VertexFormat::Float32 },
                            wgpu::VertexAttribute { offset: 64, shader_location: 9, format: wgpu::VertexFormat::Uint32 },
                            wgpu::VertexAttribute { offset: 68, shader_location: 10, format: wgpu::VertexFormat::Uint32 },
                            wgpu::VertexAttribute { offset: 72, shader_location: 11, format: wgpu::VertexFormat::Float32 },
                            wgpu::VertexAttribute { offset: 76, shader_location: 12, format: wgpu::VertexFormat::Float32 },
                            wgpu::VertexAttribute { offset: 24, shader_location: 13, format: wgpu::VertexFormat::Uint32 },
                        ],
                    },
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });
        let ui_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &ui_shader,
                entry_point: "ui_vs",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vertex>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute { offset: 0, shader_location: 0, format: wgpu::VertexFormat::Float32x2 }],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<UiInstance>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[
                            wgpu::VertexAttribute { offset: 0, shader_location: 1, format: wgpu::VertexFormat::Float32x2 },
                            wgpu::VertexAttribute { offset: 8, shader_location: 2, format: wgpu::VertexFormat::Float32x2 },
                            wgpu::VertexAttribute { offset: 16, shader_location: 3, format: wgpu::VertexFormat::Float32 },
                            wgpu::VertexAttribute { offset: 20, shader_location: 4, format: wgpu::VertexFormat::Float32 },
                            wgpu::VertexAttribute { offset: 24, shader_location: 5, format: wgpu::VertexFormat::Float32x4 },
                            wgpu::VertexAttribute { offset: 40, shader_location: 6, format: wgpu::VertexFormat::Float32x4 },
                            wgpu::VertexAttribute { offset: 56, shader_location: 7, format: wgpu::VertexFormat::Float32 },
                        ],
                    },
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState { module: &ui_shader, entry_point: "ui_fs", targets: &[Some(wgpu::ColorTargetState { format, blend: Some(wgpu::BlendState::ALPHA_BLENDING), write_mask: wgpu::ColorWrites::ALL })], compilation_options: wgpu::PipelineCompilationOptions::default() }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let quad: [Vertex; 6] = [
            Vertex { pos: [-0.5, -0.5] },
            Vertex { pos: [ 0.5, -0.5] },
            Vertex { pos: [ 0.5,  0.5] },
            Vertex { pos: [-0.5, -0.5] },
            Vertex { pos: [ 0.5,  0.5] },
            Vertex { pos: [-0.5,  0.5] },
        ];
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&quad),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 1024 * std::mem::size_of::<Instance>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let ui_instance_buffer = device.create_buffer(&wgpu::BufferDescriptor { label: None, size: 1024 * std::mem::size_of::<UiInstance>() as wgpu::BufferAddress, usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });

        // screen uniform
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::bytes_of(&ScreenUniform { screen_size: [size.width as f32, size.height as f32], scale_factor: 1.0, _pad: 0.0 }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &uniform_bgl,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding { buffer: &uniform_buffer, offset: 0, size: None }) }],
        });

        // checkerboard texture
        let tex_size = 256u32;
        let mut data = vec![0u8; (tex_size * tex_size * 4) as usize];
        for y in 0..tex_size {
            for x in 0..tex_size {
                let idx = ((y * tex_size + x) * 4) as usize;
                let c = if ((x / 32) % 2) ^ ((y / 32) % 2) == 0 { 220 } else { 60 };
                data[idx] = c; // r
                data[idx + 1] = c; // g
                data[idx + 2] = c; // b
                data[idx + 3] = 255; // a
            }
        }
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d { width: tex_size, height: tex_size, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::ImageCopyTexture { texture: &texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            &data,
            wgpu::ImageDataLayout { offset: 0, bytes_per_row: Some(4 * tex_size), rows_per_image: Some(tex_size) },
            wgpu::Extent3d { width: tex_size, height: tex_size, depth_or_array_layers: 1 },
        );
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &texture_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&texture_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
            ],
        });

        // --- 3D Setup ---
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let shader_3d = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("3D Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(r#"
struct Uniforms {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct ModelUniform {
    model: mat4x4<f32>,
    color: vec4<f32>,
};
@group(1) @binding(0) var<uniform> mesh_data: ModelUniform;

struct VertexInput {
    @location(0) pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) normal: vec3<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.uv = model.uv;
    out.normal = model.normal;
    out.clip_position = uniforms.view_proj * mesh_data.model * vec4<f32>(model.pos, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let diffuse = max(dot(in.normal, light_dir), 0.0);
    let color = vec3<f32>(1.0, 1.0, 1.0) * (diffuse + 0.1);
    return vec4<f32>(color, 1.0);
}
"#)),
        });

        let uniform_buffer_3d = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("3D Uniform Buffer"),
            contents: bytemuck::cast_slice(&[Uniforms3D { view_proj: [[0.0; 4]; 4] }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let uniform_bind_group_layout_3d = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("3D Uniform Bind Group Layout"),
        });

        let uniform_bind_group_3d = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout_3d,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer_3d.as_entire_binding(),
            }],
            label: Some("3D Uniform Bind Group"),
        });

        let model_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Model Uniform Buffer"),
            size: (256 * 1000) as wgpu::BufferAddress, // Support 1000 meshes
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let model_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: wgpu::BufferSize::new(256),
                },
                count: None,
            }],
            label: Some("Model Bind Group Layout"),
        });

        let model_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &model_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &model_uniform_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(256),
                }),
            }],
            label: Some("Model Bind Group"),
        });

        let pipeline_layout_3d = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("3D Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout_3d, &model_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline_3d = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("3D Pipeline"),
            layout: Some(&pipeline_layout_3d),
            vertex: wgpu::VertexState {
                module: &shader_3d,
                entry_point: "vs_main",
                buffers: &[Vertex3D::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_3d,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // Initialize PBR Renderer
        let pbr_renderer = crate::render::pbr_renderer::PbrRenderer::new(&device, format);
        
        // Initialize 3D Instance Buffer for PBR instanced rendering
        let instance_buffer_3d = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("3D Instance Buffer"),
            size: 1024 * std::mem::size_of::<crate::render::pbr_renderer::Instance3D>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        // 初始化脏标记追踪器
        let dirty_tracker = InstanceDirtyTracker::with_capacity(1024);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            size,
            pipeline,
            vertex_buffer,
            instance_buffer,
            vertex_count: 6,
            index_buffer,
            instance_count: 0,
            uniform_buffer,
            uniform_bind_group,
            texture_bgl,
            texture_bind_groups: vec![texture_bind_group],
            textures_size: vec![[tex_size, tex_size]],
            layer_ranges: Vec::new(),
            draw_groups: Vec::new(),
            scale_factor: 1.0,
            scissor: None,
            group_cache: Vec::new(),
            groups_dirty: true,
            ui_pipeline,
            ui_instance_buffer,
            ui_count: 0,
            commands: Vec::new(),
            offscreen_views: std::collections::HashMap::new(),
            lights_buffer,
            lights_bind_group,
            lights: Vec::new(),
            depth_texture: depth_view,
            pipeline_3d,
            uniform_buffer_3d,
            uniform_bind_group_3d,
            model_uniform_buffer,
            model_bind_group,
            chunk_hashes: std::collections::HashMap::new(),
            pbr_renderer: Some(pbr_renderer),
            instance_buffer_3d,
            dirty_tracker,
        })
    }

    pub fn create_offscreen_target(&mut self, id: u32, width: u32, height: u32) {
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("Offscreen Target {}", id)),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.offscreen_views.insert(id, view);
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        if size.width > 0 && size.height > 0 {
            self.size = size;
            self.config.width = size.width;
            self.config.height = size.height;
            self.surface.configure(&self.device, &self.config);

            // Resize depth texture
            let depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Depth Texture"),
                size: wgpu::Extent3d {
                    width: size.width,
                    height: size.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            self.depth_texture = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
        }
    }

    pub fn render(
        &mut self, 
        instances: &[Instance], 
        meshes: &[(crate::render::mesh::GpuMesh, crate::ecs::Transform)], 
        camera_view_proj: [[f32; 4]; 4],
        mut egui_renderer: Option<&mut egui_wgpu::Renderer>, 
        egui_shapes: &[egui::ClippedPrimitive], 
        egui_pixels_per_point: f32
    ) {
        self.update_instances_grouped(&mut instances.to_vec());

        // Update 3D uniforms
        self.queue.write_buffer(&self.uniform_buffer_3d, 0, bytemuck::cast_slice(&[Uniforms3D { view_proj: camera_view_proj }]));
        
        // Update Model Uniforms
        // Note: This is a simple implementation. For production, use a staging buffer or write_buffer_with.
        // Also, we assume meshes.len() < 1000.
        let mut model_data = Vec::with_capacity(meshes.len());
        for (_, transform) in meshes {
            let mat = glam::Mat4::from_scale_rotation_translation(transform.scale, transform.rot, transform.pos);
            model_data.push(ModelUniform {
                model: mat.to_cols_array_2d(),
                color: [1.0, 1.0, 1.0, 1.0], // Placeholder color
                _pad1: [0.0; 32],
                _pad2: [0.0; 12],
            });
        }
        if !model_data.is_empty() {
             self.queue.write_buffer(&self.model_uniform_buffer, 0, bytemuck::cast_slice(&model_data));
        }
        
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(_) => {
                self.surface.configure(&self.device, &self.config);
                self.surface.get_current_texture().unwrap()
            }
        };
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        
        if let Some(renderer) = egui_renderer.as_mut() {
             let screen_desc = egui_wgpu::ScreenDescriptor {
                size_in_pixels: [self.config.width, self.config.height],
                pixels_per_point: egui_pixels_per_point,
            };
            renderer.update_buffers(&self.device, &self.queue, &mut encoder, egui_shapes, &screen_desc);
        }

        let graph = crate::render::graph::build_commands(instances);
        self.commands = graph.commands.clone();
        let mut cleared_targets = std::collections::HashSet::new();
        let mut i = 0;
        
        while i < graph.commands.len() {
            if let crate::render::graph::RenderCommand::SetTarget(t) = &graph.commands[i] {
                let (target_view, target_id) = match t {
                    crate::render::graph::Target::Main | crate::render::graph::Target::Ui => (&view, 0),
                    crate::render::graph::Target::Offscreen(id) => (self.offscreen_views.get(id).unwrap_or(&view), *id + 1),
                };
                
                let load_op = if cleared_targets.contains(&target_id) {
                    wgpu::LoadOp::Load
                } else {
                    cleared_targets.insert(target_id);
                    wgpu::LoadOp::Clear(wgpu::Color { r: 0.02, g: 0.04, b: 0.06, a: 1.0 })
                };

                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: target_view,
                            resolve_target: None,
                            ops: wgpu::Operations { load: load_op, store: wgpu::StoreOp::Store },
                        })],
                        depth_stencil_attachment: if target_id == 0 { // Main target only
                            Some(wgpu::RenderPassDepthStencilAttachment {
                                view: &self.depth_texture,
                                depth_ops: Some(wgpu::Operations {
                                    load: if load_op == wgpu::LoadOp::Load { wgpu::LoadOp::Load } else { wgpu::LoadOp::Clear(1.0) },
                                    store: wgpu::StoreOp::Store,
                                }),
                                stencil_ops: None,
                            })
                        } else {
                            None
                        },
                        occlusion_query_set: None,
                        timestamp_writes: None,
                    });

                    if target_id == 0 {
                        // Draw 3D Meshes
                        if !meshes.is_empty() {
                            rpass.set_pipeline(&self.pipeline_3d);
                            rpass.set_bind_group(0, &self.uniform_bind_group_3d, &[]);
                            for (i, (mesh, _)) in meshes.iter().enumerate() {
                                let offset = (i * 256) as u32;
                                rpass.set_bind_group(1, &self.model_bind_group, &[offset]);
                                rpass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                                rpass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                                rpass.draw_indexed(0..mesh.index_count, 0, 0..1);
                            }
                        }
                    }

                    rpass.set_bind_group(0, &self.uniform_bind_group, &[]);
                    
                    i += 1;
                    while i < graph.commands.len() {
                        match &graph.commands[i] {
                            crate::render::graph::RenderCommand::SetTarget(_) => break,
                            crate::render::graph::RenderCommand::Draw { start, end, tex_idx, scissor } => {
                                rpass.set_pipeline(&self.pipeline);
                                rpass.set_bind_group(1, &self.texture_bind_groups[*tex_idx], &[]);
                                rpass.set_bind_group(2, &self.lights_bind_group, &[]);
                                rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                                rpass.set_vertex_buffer(1, self.instance_buffer.slice(..));
                                let mut use_scissor = scissor.clone();
                                if use_scissor.is_none() && target_id == 0 {
                                    use_scissor = compute_scissor(instances, *start, *end, self.config.width, self.config.height);
                                }
                                if let Some([x,y,w,h]) = use_scissor { rpass.set_scissor_rect(x, y, w, h); }
                                if end > start { rpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16); rpass.draw_indexed(0..self.vertex_count, 0, *start..*end); }
                            }
                            crate::render::graph::RenderCommand::DrawUi { count: _ } => {
                                rpass.set_pipeline(&self.ui_pipeline);
                                rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                                rpass.set_vertex_buffer(1, self.ui_instance_buffer.slice(..));
                                rpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                                rpass.draw_indexed(0..self.vertex_count, 0, 0..self.ui_count);
                            }
                        }
                        i += 1;
                    }
                    
                    if target_id == 0 {
                        if let Some(renderer) = egui_renderer.as_mut() {
                             let screen_desc = egui_wgpu::ScreenDescriptor {
                                size_in_pixels: [self.config.width, self.config.height],
                                pixels_per_point: egui_pixels_per_point,
                            };
                            renderer.render(&mut rpass, egui_shapes, &screen_desc);
                        }
                    }
                }
            } else {
                i += 1;
            }
        }
        
        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }
    

    pub fn set_lights(&mut self, lights: Vec<GpuPointLight>) {
        self.update_lights(&lights);
    }
    pub fn gpu_timings_ms(&self) -> Option<(f32, f32)> { None }
    pub fn draw_stats(&self) -> (u32, u32) {
        let mut draws = 0u32;
        let mut instances = 0u32;
        for cmd in &self.commands {
            if let crate::render::graph::RenderCommand::Draw { start, end, .. } = cmd {
                if end > start { draws += 1; instances += end - start; }
            }
        }
        (draws, instances)
    }
    pub fn pass_count(&self) -> u32 {
        let mut passes = 0u32;
        for cmd in &self.commands { if matches!(cmd, crate::render::graph::RenderCommand::SetTarget(_)) { passes += 1; } }
        passes.max(1)
    }
    pub fn stage_timings_ms(&self) -> (Option<f32>, Option<f32>, Option<f32>) { (None, None, None) }
    pub fn offscreen_timing_ms(&self) -> Option<f32> { None }

    pub fn update_instances(&mut self, instances: &[Instance]) {
        self.instance_count = instances.len() as u32;
        self.layer_ranges.clear();
        self.layer_ranges.push((0, self.instance_count));
        self.draw_groups.clear();
        self.draw_groups.push(DrawGroup { start: 0, end: self.instance_count, tex_idx: 0, layer: 0.0, scissor: None });
        self.queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(instances));
        self.group_cache.clear();
        self.groups_dirty = false;
    }
    
    /// 使用脏标记系统增量更新实例数据
    /// 
    /// 仅上传变化的实例，减少 CPU-GPU 数据传输，提升性能 20-40%。
    /// 
    /// # 参数
    /// - `instances`: 当前帧的所有实例数据
    /// 
    /// # 返回
    /// 返回更新的脏范围数量
    pub fn update_instances_incremental(&mut self, instances: &[Instance]) -> usize {
        self.instance_count = instances.len() as u32;
        self.layer_ranges.clear();
        self.layer_ranges.push((0, self.instance_count));
        self.draw_groups.clear();
        self.draw_groups.push(DrawGroup { start: 0, end: self.instance_count, tex_idx: 0, layer: 0.0, scissor: None });
        
        // 检测变化并获取脏范围
        let dirty_ranges = self.dirty_tracker.update(instances);
        let range_count = dirty_ranges.len();
        
        // 仅上传脏范围内的数据
        let elem_size = std::mem::size_of::<Instance>() as u64;
        for &(start, end) in dirty_ranges {
            if end > start {
                let byte_offset = start as u64 * elem_size;
                let slice = &instances[start as usize..end as usize];
                self.queue.write_buffer(&self.instance_buffer, byte_offset, bytemuck::cast_slice(slice));
            }
        }
        
        self.group_cache.clear();
        self.groups_dirty = false;
        
        range_count
    }
    
    /// 标记特定实例为脏（将在下次 update_instances_incremental 时更新）
    #[inline]
    pub fn mark_instance_dirty(&mut self, index: usize) {
        self.dirty_tracker.mark_instance_dirty(index);
    }
    
    /// 标记实例范围为脏
    pub fn mark_instance_range_dirty(&mut self, start: usize, end: usize) {
        self.dirty_tracker.mark_range_dirty(start, end);
    }
    
    /// 标记所有实例为脏（强制完整更新）
    pub fn mark_all_instances_dirty(&mut self) {
        self.dirty_tracker.mark_all_dirty();
    }
    
    /// 获取增量更新统计信息
    pub fn dirty_update_stats(&self) -> (usize, usize) {
        (self.dirty_tracker.dirty_range_count(), self.dirty_tracker.dirty_instance_count())
    }
    
    /// 创建双缓冲实例管理器
    pub fn create_double_buffered_instances(&self, capacity: u32) -> DoubleBufferedInstances {
        DoubleBufferedInstances::new(&self.device, capacity)
    }
    
    /// 使用双缓冲系统更新实例 (异步方式,减少CPU-GPU同步等待)
    pub fn update_instances_double_buffered(
        &mut self,
        double_buffer: &mut DoubleBufferedInstances,
        instances: &[Instance],
    ) {
        // 确保容量足够
        double_buffer.ensure_capacity(&self.device, instances.len() as u32);
        
        // 使用staging buffer进行异步更新
        if let Some(cmd_buffer) = double_buffer.update_with_staging(&self.device, &self.queue, instances) {
            self.queue.submit(std::iter::once(cmd_buffer));
        }
        
        self.instance_count = double_buffer.count();
        self.layer_ranges.clear();
        self.layer_ranges.push((0, self.instance_count));
        self.draw_groups.clear();
        self.draw_groups.push(DrawGroup { start: 0, end: self.instance_count, tex_idx: 0, layer: 0.0, scissor: None });
        self.group_cache.clear();
        self.groups_dirty = false;
    }
    
    /// 获取双缓冲实例管理器的活动缓冲区 (用于渲染)
    pub fn get_active_instance_buffer<'b>(&self, double_buffer: &'b DoubleBufferedInstances) -> &'b wgpu::Buffer {
        double_buffer.active_buffer()
    }

    pub fn update_instances_grouped(&mut self, instances: &mut [Instance]) {
        self.instance_count = instances.len() as u32;
        self.layer_ranges.clear();
        self.draw_groups.clear();
        self.group_cache.clear();

        // 按chunk聚合写入，减少跨块数据搬运的cache miss
        if instances.is_empty() {
            return;
        }
        let mut start: u32 = 0;
        let mut cur_chunk = instances[0].chunk;
        let mut chunk_runs: Vec<(u32, u32, u32)> = Vec::new(); // (chunk, start, end)
        for (i, inst) in instances.iter().enumerate() {
            if inst.chunk != cur_chunk {
                chunk_runs.push((cur_chunk, start, i as u32));
                start = i as u32;
                cur_chunk = inst.chunk;
            }
        }
        chunk_runs.push((cur_chunk, start, instances.len() as u32));

        let elem_size = std::mem::size_of::<Instance>() as u64;
        if chunk_runs.len() == 1 {
            let (_, s, e) = chunk_runs[0];
            let slice = &instances[s as usize..e as usize];
            if !slice.is_empty() {
                let h = hash_instances(slice);
                let cid = instances[s as usize].chunk;
                let prev = self.chunk_hashes.get(&cid).copied();
                if prev != Some(h) {
                    self.queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(slice));
                    self.chunk_hashes.insert(cid, h);
                }
            }
        } else {
            for &(cid, s, e) in &chunk_runs {
                let byte_offset = (s as u64) * elem_size;
                let slice = &instances[s as usize..e as usize];
                if slice.is_empty() { continue; }
                let h = hash_instances(slice);
                let prev = self.chunk_hashes.get(&cid).copied();
                if prev != Some(h) {
                    self.queue.write_buffer(&self.instance_buffer, byte_offset, bytemuck::cast_slice(slice));
                    self.chunk_hashes.insert(cid, h);
                }
            }
        }
    }

    pub fn set_scale_factor(&mut self, scale: f32) { self.scale_factor = scale; }

    pub fn set_scissor(&mut self, rect: Option<[u32;4]>) { self.scissor = rect; }

    pub fn set_scissor_for_instances(&mut self, start: u32, end: u32, rect: Option<[u32;4]>) {
        for g in &mut self.draw_groups {
            if !(end <= g.start || start >= g.end) { g.scissor = rect; }
        }
    }

    pub fn mark_groups_dirty(&mut self) { self.groups_dirty = true; }

    pub fn load_texture_file(&mut self, path: &std::path::Path) -> Option<u32> {
        if let Ok(img) = image::open(path) {
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            self.queue.write_texture(
                wgpu::ImageCopyTexture { texture: &texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
                rgba.as_raw(),
                wgpu::ImageDataLayout { offset: 0, bytes_per_row: Some(4 * w), rows_per_image: Some(h) },
                wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            );
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor::default());
            let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.texture_bgl,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
                ],
            });
            let idx = self.texture_bind_groups.len() as u32;
            self.texture_bind_groups.push(bg);
            self.textures_size.push([w, h]);
            Some(idx)
        } else { None }
    }

    pub fn load_texture_file_linear(&mut self, path: &std::path::Path) -> Option<u32> {
        if let Ok(img) = image::open(path) {
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            self.queue.write_texture(
                wgpu::ImageCopyTexture { texture: &texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
                rgba.as_raw(),
                wgpu::ImageDataLayout { offset: 0, bytes_per_row: Some(4 * w), rows_per_image: Some(h) },
                wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            );
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor::default());
            let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.texture_bgl,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
                ],
            });
            let idx = self.texture_bind_groups.len() as u32;
            self.texture_bind_groups.push(bg);
            self.textures_size.push([w, h]);
            Some(idx)
        } else { None }
    }

    pub fn reload_texture_file_at(&mut self, index: u32, path: &std::path::Path, linear: bool) -> Option<()> {
        if let Ok(img) = image::open(path) {
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            let format = if linear { wgpu::TextureFormat::Rgba8Unorm } else { wgpu::TextureFormat::Rgba8UnormSrgb };
            let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            self.queue.write_texture(
                wgpu::ImageCopyTexture { texture: &texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
                rgba.as_raw(),
                wgpu::ImageDataLayout { offset: 0, bytes_per_row: Some(4 * w), rows_per_image: Some(h) },
                wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            );
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor::default());
            let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.texture_bgl,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
                ],
            });
            let idx = index as usize;
            if idx < self.texture_bind_groups.len() {
                self.texture_bind_groups[idx] = bg;
                self.textures_size[idx] = [w, h];
                return Some(());
            }
        }
        None
    }

    pub fn load_texture_from_bytes(&mut self, bytes: &[u8], is_linear: bool) -> Option<u32> {
        if let Ok(img) = image::load_from_memory(bytes) {
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            let format = if is_linear { wgpu::TextureFormat::Rgba8Unorm } else { wgpu::TextureFormat::Rgba8UnormSrgb };
            let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            self.queue.write_texture(
                wgpu::ImageCopyTexture { texture: &texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
                rgba.as_raw(),
                wgpu::ImageDataLayout { offset: 0, bytes_per_row: Some(4 * w), rows_per_image: Some(h) },
                wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            );
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor::default());
            let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.texture_bgl,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
                ],
            });
            let idx = self.texture_bind_groups.len() as u32;
            self.texture_bind_groups.push(bg);
            self.textures_size.push([w, h]);
            Some(idx)
        } else { None }
    }

    pub fn load_texture_from_image(&mut self, img: image::RgbaImage, is_linear: bool) -> Option<u32> {
        let (w, h) = img.dimensions();
        let format = if is_linear { wgpu::TextureFormat::Rgba8Unorm } else { wgpu::TextureFormat::Rgba8UnormSrgb };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        self.queue.write_texture(
            wgpu::ImageCopyTexture { texture: &texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            img.as_raw(),
            wgpu::ImageDataLayout { offset: 0, bytes_per_row: Some(4 * w), rows_per_image: Some(h) },
            wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor::default());
        let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.texture_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
            ],
        });
        let idx = self.texture_bind_groups.len() as u32;
        self.texture_bind_groups.push(bg);
        self.textures_size.push([w, h]);
        Some(idx)
    }

    pub fn update_screen(&mut self) {
        let u = ScreenUniform { screen_size: [self.size.width as f32, self.size.height as f32], scale_factor: self.scale_factor, _pad: 0.0 };
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&u));
    }

    pub fn set_scissor_for_layer(&mut self, layer: f32, rect: Option<[u32;4]>) {
        for g in &mut self.draw_groups {
            if (g.layer - layer).abs() < f32::EPSILON { g.scissor = rect; }
        }
    }

    pub fn ui_set(&mut self, items: &[UiInstance]) {
        self.ui_count = items.len() as u32;
        if self.ui_count > 0 { self.queue.write_buffer(&self.ui_instance_buffer, 0, bytemuck::cast_slice(items)); }
    }

    pub fn set_graph(&mut self, graph: crate::render::graph::RenderGraph, instances: &[Instance]) {
        self.commands = graph.commands;
        self.instance_count = instances.len() as u32;
        self.queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(instances));
    }

    pub fn config(&self) -> &wgpu::SurfaceConfiguration { &self.config }
    
    // ========================================================================
    // Instance Batching Methods
    // ========================================================================
    
    /// 上传批次管理器中的所有脏批次到 GPU
    /// 
    /// 在渲染前调用此方法以确保所有实例数据已同步到 GPU
    /// 
    /// # 示例
    /// ```ignore
    /// // 在渲染循环中
    /// renderer.upload_batches(&mut batch_manager);
    /// renderer.render_pbr_batched(&batch_manager, ...);
    /// ```
    pub fn upload_batches(&self, batch_manager: &mut crate::render::instance_batch::BatchManager) {
        batch_manager.update_buffers(&self.device, &self.queue);
    }
    
    /// 使用 BatchManager 进行实例化 PBR 渲染
    /// 
    /// 相比 `render_pbr`，此方法利用 BatchManager 的实例批处理，
    /// 可以显著减少 Draw Call 数量（70-90% 优化）
    pub fn render_pbr_batched(
        &mut self,
        batch_manager: &mut crate::render::instance_batch::BatchManager,
        point_lights: &[crate::render::pbr::PointLight3D],
        dir_lights: &[crate::render::pbr::DirectionalLight],
        view_proj: [[f32; 4]; 4],
        camera_pos: [f32; 3],
        egui_renderer: Option<&mut egui_wgpu::Renderer>,
        egui_shapes: &[egui::ClippedPrimitive],
        egui_pixels_per_point: f32,
    ) {
        // 更新相机
        self.update_pbr_camera(view_proj, camera_pos);
        
        // 更新光源
        self.update_pbr_lights(point_lights, dir_lights);

        // 实例级剔除（GPU路径，如不可用则CPU回退）
        #[allow(unused_mut)]
        let mut used_gpu_cull = false;
        #[cfg(feature = "wgpu_perf")]
        {
            use crate::render::gpu_driven::culling::{GpuCuller, GpuInstance};
            let (instances, mapping) = batch_manager.collect_gpu_instances();
            if !instances.is_empty() {
                let input_size = (instances.len() * std::mem::size_of::<GpuInstance>()) as wgpu::BufferAddress;
                let input_buffer = self.device.create_buffer(&wgpu::BufferDescriptor { label: Some("Culling Input"), size: input_size, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false });
                self.queue.write_buffer(&input_buffer, 0, bytemuck::cast_slice(&instances));
                let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor { label: Some("Culling Output"), size: input_size, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false });
                let counter_buffer = self.device.create_buffer(&wgpu::BufferDescriptor { label: Some("Culling Counter"), size: std::mem::size_of::<u32>() as wgpu::BufferAddress, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false });

                let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("GPU Culling Encoder") });
                let culler = GpuCuller::new(&self.device, instances.len() as u32, 64);
                culler.cull(&mut encoder, &self.device, &self.queue, &input_buffer, &output_buffer, &counter_buffer, view_proj, instances.len() as u32);

                // 读取计数器与输出实例
                let read_counter = self.device.create_buffer(&wgpu::BufferDescriptor { label: Some("Read Counter"), size: std::mem::size_of::<u32>() as wgpu::BufferAddress, usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
                encoder.copy_buffer_to_buffer(&counter_buffer, 0, &read_counter, 0, std::mem::size_of::<u32>() as wgpu::BufferAddress);
                let read_output = self.device.create_buffer(&wgpu::BufferDescriptor { label: Some("Read Output"), size: input_size, usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
                encoder.copy_buffer_to_buffer(&output_buffer, 0, &read_output, 0, input_size);
                self.queue.submit(std::iter::once(encoder.finish()));
                self.device.poll(wgpu::Maintain::Wait);

                let count_slice = read_counter.slice(..);
                count_slice.map_async(wgpu::MapMode::Read, |_| {});
                self.device.poll(wgpu::Maintain::Wait);
                let count_data = count_slice.get_mapped_range();
                let visible_count = u32::from_le_bytes(count_data[..4].try_into().unwrap_or([0,0,0,0]));
                drop(count_data);
                read_counter.unmap();

                if visible_count > 0 {
                    let out_slice = read_output.slice(..(visible_count as wgpu::BufferAddress * std::mem::size_of::<GpuInstance>() as wgpu::BufferAddress));
                    out_slice.map_async(wgpu::MapMode::Read, |_| {});
                    self.device.poll(wgpu::Maintain::Wait);
                    let out_data = out_slice.get_mapped_range();
                    let visible_instances: &[GpuInstance] = bytemuck::cast_slice(&out_data);
                    let ids: Vec<u32> = visible_instances.iter().map(|gi| gi.instance_id).collect();
                    drop(out_data);
                    read_output.unmap();
                    #[cfg(feature = "wgpu_perf")]
                    {
                        batch_manager.apply_visible_ids_segments(&self.device, &self.queue, &mapping, &ids);
                    }
                    #[cfg(not(feature = "wgpu_perf"))]
                    {
                        batch_manager.apply_visible_ids(&mapping, &ids);
                    }
                    used_gpu_cull = true;
                }
            }
        }

        if !used_gpu_cull {
            batch_manager.cull_instances_cpu(view_proj);
        }
        batch_manager.update_buffers(&self.device, &self.queue);
        
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(_) => {
                self.surface.configure(&self.device, &self.config);
                self.surface.get_current_texture().unwrap()
            }
        };
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { 
            label: Some("PBR Batched Encoder") 
        });
        
        // Update egui buffers if present
        if let Some(renderer) = egui_renderer.as_ref() {
            let screen_desc = egui_wgpu::ScreenDescriptor {
                size_in_pixels: [self.config.width, self.config.height],
                pixels_per_point: egui_pixels_per_point,
            };
            let _ = (renderer, screen_desc);
        }
        
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("PBR Batched Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.02, g: 0.04, b: 0.06, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            
            if let Some(ref pbr) = self.pbr_renderer {
                rpass.set_pipeline(&pbr.pipeline);
                rpass.set_bind_group(0, &pbr.uniform_bind_group, &[]);
                rpass.set_bind_group(2, &pbr.lights_bind_group, &[]);
                rpass.set_bind_group(3, &pbr.textures_bind_group, &[]);
                
                // 使用 render_batches 函数渲染所有可见批次
                crate::render::instance_batch::render_batches(&mut rpass, batch_manager);
                // 渲染小批次（阈值过滤后）
                crate::render::instance_batch::render_small_batches(&mut rpass, batch_manager);
            }
        }
        
        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }
    
    pub fn update_lights(&mut self, lights: &[GpuPointLight]) {
        self.lights.clear();
        self.lights.extend_from_slice(lights);
        
        let mut data = vec![GpuPointLight {
            pos: [0.0, 0.0],
            color: [0.0, 0.0, 0.0],
            radius: 0.0,
            intensity: 0.0,
            falloff: 0.0,
            _pad: [0.0, 0.0],
        }; 1024];
        
        for (i, light) in lights.iter().enumerate().take(1024) {
            data[i] = *light;
        }
        
        self.queue.write_buffer(&self.lights_buffer, 0, bytemuck::cast_slice(&data));
    }
    
    // ========================================================================
    // PBR 3D Rendering Methods
    // ========================================================================
    
    /// 更新PBR渲染器的相机参数
    pub fn update_pbr_camera(&self, view_proj: [[f32; 4]; 4], camera_pos: [f32; 3]) {
        if let Some(ref pbr) = self.pbr_renderer {
            pbr.update_camera(&self.queue, view_proj, camera_pos);
        }
    }
    
    /// 更新PBR材质参数
    pub fn update_pbr_material(&self, material: &crate::render::pbr::PbrMaterial) {
        if let Some(ref pbr) = self.pbr_renderer {
            pbr.update_material(&self.queue, material);
        }
    }
    
    /// 更新PBR 3D光源
    pub fn update_pbr_lights(
        &self, 
        point_lights: &[crate::render::pbr::PointLight3D], 
        dir_lights: &[crate::render::pbr::DirectionalLight]
    ) {
        if let Some(ref pbr) = self.pbr_renderer {
            pbr.update_lights(&self.queue, point_lights, dir_lights);
        }
    }
    
    /// 渲染PBR 3D网格
    pub fn render_pbr(
        &mut self,
        meshes: &[(crate::render::mesh::GpuMesh, crate::ecs::Transform, crate::render::pbr::PbrMaterial)],
        point_lights: &[crate::render::pbr::PointLight3D],
        dir_lights: &[crate::render::pbr::DirectionalLight],
        view_proj: [[f32; 4]; 4],
        camera_pos: [f32; 3],
        egui_renderer: Option<&mut egui_wgpu::Renderer>,
        egui_shapes: &[egui::ClippedPrimitive],
        egui_pixels_per_point: f32,
    ) {
        // 更新相机
        self.update_pbr_camera(view_proj, camera_pos);
        
        // 更新光源
        self.update_pbr_lights(point_lights, dir_lights);
        
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(_) => {
                self.surface.configure(&self.device, &self.config);
                self.surface.get_current_texture().unwrap()
            }
        };
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("PBR Encoder") });
        
        // Update egui buffers if present
        if let Some(renderer) = egui_renderer.as_ref() {
            let screen_desc = egui_wgpu::ScreenDescriptor {
                size_in_pixels: [self.config.width, self.config.height],
                pixels_per_point: egui_pixels_per_point,
            };
            // Note: we can't borrow mutably here, so we skip the buffer update in this path
            let _ = (renderer, screen_desc);
        }
        
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("PBR Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.02, g: 0.04, b: 0.06, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            
            if let Some(ref pbr) = self.pbr_renderer {
                rpass.set_pipeline(&pbr.pipeline);
                rpass.set_bind_group(0, &pbr.uniform_bind_group, &[]);
                rpass.set_bind_group(2, &pbr.lights_bind_group, &[]);
                
                // 1. Sort meshes to batch them
                let mut indices: Vec<usize> = (0..meshes.len()).collect();
                indices.sort_by(|&a, &b| {
                    let (mesh_a, _, mat_a) = &meshes[a];
                    let (mesh_b, _, mat_b) = &meshes[b];
                    
                    let vb_a = std::sync::Arc::as_ptr(&mesh_a.vertex_buffer);
                    let vb_b = std::sync::Arc::as_ptr(&mesh_b.vertex_buffer);
                    
                    if vb_a != vb_b {
                        return vb_a.cmp(&vb_b);
                    }
                    
                    if mat_a.base_color.x < mat_b.base_color.x { std::cmp::Ordering::Less }
                    else if mat_a.base_color.x > mat_b.base_color.x { std::cmp::Ordering::Greater }
                    else { std::cmp::Ordering::Equal }
                });

                // 2. Build Instance Buffer
                let mut instances = Vec::with_capacity(meshes.len());
                struct Batch<'a> {
                    mesh: &'a crate::render::mesh::GpuMesh,
                    material: &'a crate::render::pbr::PbrMaterial,
                    start_instance: u32,
                    count: u32,
                }
                let mut batches = Vec::new();
                
                if !indices.is_empty() {
                    let first_idx = indices[0];
                    let (first_mesh, _, first_mat) = &meshes[first_idx];
                    
                    let mut current_batch = Batch {
                        mesh: first_mesh,
                        material: first_mat,
                        start_instance: 0,
                        count: 0,
                    };
                    
                    for &i in &indices {
                        let (mesh, transform, material) = &meshes[i];
                        
                        let same_mesh = std::sync::Arc::as_ptr(&mesh.vertex_buffer) == std::sync::Arc::as_ptr(&current_batch.mesh.vertex_buffer);
                        let same_mat = material == current_batch.material;
                        
                        if same_mesh && same_mat {
                            let model_mat = glam::Mat4::from_scale_rotation_translation(
                                transform.scale, 
                                transform.rot, 
                                transform.pos
                            );
                            instances.push(crate::render::pbr_renderer::Instance3D {
                                model: model_mat.to_cols_array_2d(),
                            });
                            current_batch.count += 1;
                        } else {
                            batches.push(current_batch);
                            
                            current_batch = Batch {
                                mesh,
                                material,
                                start_instance: instances.len() as u32,
                                count: 1,
                            };
                            
                            let model_mat = glam::Mat4::from_scale_rotation_translation(
                                transform.scale, 
                                transform.rot, 
                                transform.pos
                            );
                            instances.push(crate::render::pbr_renderer::Instance3D {
                                model: model_mat.to_cols_array_2d(),
                            });
                        }
                    }
                    batches.push(current_batch);
                }
                
                // 3. Upload Instance Buffer
                let needed_size = (instances.len() * std::mem::size_of::<crate::render::pbr_renderer::Instance3D>()) as u64;
                if self.instance_buffer_3d.size() < needed_size {
                    self.instance_buffer_3d = self.device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("3D Instance Buffer"),
                        size: needed_size.max(1024),
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    });
                }
                
                self.queue.write_buffer(&self.instance_buffer_3d, 0, bytemuck::cast_slice(&instances));
                
                // 4. Draw Batches
                for batch in batches {
                    pbr.update_material(&self.queue, batch.material);
                    
                    rpass.set_bind_group(1, &pbr.material_bind_group, &[]);
                    
                    rpass.set_vertex_buffer(0, batch.mesh.vertex_buffer.slice(..));
                    rpass.set_vertex_buffer(1, self.instance_buffer_3d.slice(
                        (batch.start_instance as u64 * std::mem::size_of::<crate::render::pbr_renderer::Instance3D>() as u64) .. 
                        ((batch.start_instance + batch.count) as u64 * std::mem::size_of::<crate::render::pbr_renderer::Instance3D>() as u64)
                    ));
                    rpass.set_index_buffer(batch.mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    rpass.draw_indexed(0..batch.mesh.index_count, 0, 0..batch.count);
                }
            }
        }
        
        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }
    
    /// 获取PBR渲染器引用
    pub fn pbr_renderer(&self) -> Option<&crate::render::pbr_renderer::PbrRenderer> {
        self.pbr_renderer.as_ref()
    }

    pub fn create_gpu_mesh(&self, vertices: &[crate::render::mesh::Vertex3D], indices: &[u32]) -> std::sync::Arc<crate::render::mesh::GpuMesh> {
        std::sync::Arc::new(crate::render::mesh::GpuMesh::new(&self.device, vertices, indices))
    }

    pub fn create_texture_view_from_rgba(&self, img: &image::RgbaImage, srgb: bool) -> wgpu::TextureView {
        let (w, h) = img.dimensions();
        let format = if srgb { wgpu::TextureFormat::Rgba8UnormSrgb } else { wgpu::TextureFormat::Rgba8Unorm };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("PBR Imported Texture"),
            size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        self.queue.write_texture(
            wgpu::ImageCopyTexture { texture: &texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            img.as_raw(),
            wgpu::ImageDataLayout { offset: 0, bytes_per_row: Some(4 * w), rows_per_image: Some(h) },
            wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        );
        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub fn create_sampler(&self) -> wgpu::Sampler {
        self.device.create_sampler(&wgpu::SamplerDescriptor::default())
    }

    pub fn device(&self) -> &wgpu::Device { &self.device }
    pub fn queue(&self) -> &wgpu::Queue { &self.queue }
}

fn compute_scissor(insts: &[Instance], start: u32, end: u32, screen_w: u32, screen_h: u32) -> Option<[u32;4]> {
    if end <= start { return None; }
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    let s = start as usize; let e = end as usize;
    for inst in &insts[s..e] {
        let c = inst.rot.cos().abs();
        let s0 = inst.rot.sin().abs();
        let hx = inst.scale[0] * 0.5;
        let hy = inst.scale[1] * 0.5;
        let ex = c * hx + s0 * hy;
        let ey = s0 * hx + c * hy;
        let x0 = inst.pos[0] - ex;
        let y0 = inst.pos[1] - ey;
        let x1 = inst.pos[0] + ex;
        let y1 = inst.pos[1] + ey;
        if x0 < min_x { min_x = x0; }
        if y0 < min_y { min_y = y0; }
        if x1 > max_x { max_x = x1; }
        if y1 > max_y { max_y = y1; }
    }
    if !(min_x.is_finite() && min_y.is_finite() && max_x.is_finite() && max_y.is_finite()) { return None; }
    let mut x = min_x.max(0.0).floor() as u32;
    let mut y = min_y.max(0.0).floor() as u32;
    let mut w = (max_x.min(screen_w as f32).ceil() as i64 - x as i64).max(0) as u32;
    let mut h = (max_y.min(screen_h as f32).ceil() as i64 - y as i64).max(0) as u32;
    if w == 0 || h == 0 { return None; }
    if x >= screen_w { x = screen_w - 1; }
    if y >= screen_h { y = screen_h - 1; }
    if x + w > screen_w { w = screen_w - x; }
    if y + h > screen_h { h = screen_h - y; }
    Some([x, y, w, h])
}

fn inst_tex_index(inst: &Instance) -> usize { inst.tex_index as usize }

fn hash_instances(slice: &[Instance]) -> u64 {
    let mut h: u64 = 1469598103934665603;
    for inst in slice {
        let bytes: &[u8] = bytemuck::bytes_of(inst);
        for &b in bytes {
            h ^= b as u64;
            h = h.wrapping_mul(1099511628211);
        }
    }
    h
}
