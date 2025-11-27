//! 粒子发射器组件和 GPU 粒子系统
//!
//! 支持大规模粒子模拟，完全在 GPU 上执行。

use bevy_ecs::prelude::*;
use glam::{Vec3, Vec4};
use std::ops::Range;

// ============================================================================
// 粒子发射器组件
// ============================================================================

/// 粒子发射器配置
#[derive(Clone)]
pub struct ParticleEmitterConfig {
    /// 最大粒子数
    pub max_particles: u32,
    /// 每秒发射数量
    pub emission_rate: f32,
    /// 发射持续时间（None = 无限）
    pub duration: Option<f32>,
    /// 是否循环
    pub looping: bool,
    /// 预热时间
    pub prewarm_time: f32,
    /// 发射形状
    pub shape: ParticleShape,
}

impl Default for ParticleEmitterConfig {
    fn default() -> Self {
        Self {
            max_particles: 10000,
            emission_rate: 100.0,
            duration: None,
            looping: true,
            prewarm_time: 0.0,
            shape: ParticleShape::Point,
        }
    }
}

/// 发射形状
#[derive(Clone)]
pub enum ParticleShape {
    /// 点发射
    Point,
    /// 球形发射
    Sphere { radius: f32 },
    /// 半球发射
    Hemisphere { radius: f32 },
    /// 圆锥发射
    Cone { angle: f32, radius: f32 },
    /// 盒子发射
    Box { half_extents: Vec3 },
    /// 圆形发射
    Circle { radius: f32 },
    /// 边缘发射
    Edge { length: f32 },
}

impl Default for ParticleShape {
    fn default() -> Self {
        Self::Point
    }
}

/// 粒子发射器组件
#[derive(Component)]
pub struct ParticleEmitter {
    /// 配置
    pub config: ParticleEmitterConfig,
    /// 粒子生命周期范围（秒）
    pub lifetime: Range<f32>,
    /// 初始速度范围
    pub initial_velocity: Range<Vec3>,
    /// 速度随机性（0-1）
    pub velocity_randomness: f32,
    /// 重力
    pub gravity: Vec3,
    /// 阻力系数
    pub drag: f32,
    /// 初始颜色
    pub start_color: Vec4,
    /// 结束颜色
    pub end_color: Vec4,
    /// 颜色渐变
    pub color_gradient: Option<ColorGradient>,
    /// 初始大小范围
    pub start_size: Range<f32>,
    /// 结束大小（如果设置）
    pub end_size: Option<f32>,
    /// 大小随生命周期
    pub size_over_lifetime: Option<SizeOverLifetime>,
    /// 初始旋转范围（弧度）
    pub start_rotation: Range<f32>,
    /// 旋转速度
    pub rotation_speed: f32,
    /// 是否启用
    pub enabled: bool,
    /// 当前累积发射时间
    pub emission_accumulator: f32,
    /// 当前运行时间
    pub elapsed_time: f32,
}

impl Default for ParticleEmitter {
    fn default() -> Self {
        Self {
            config: ParticleEmitterConfig::default(),
            lifetime: 1.0..3.0,
            initial_velocity: Vec3::new(-1.0, 2.0, -1.0)..Vec3::new(1.0, 5.0, 1.0),
            velocity_randomness: 0.2,
            gravity: Vec3::new(0.0, -9.81, 0.0),
            drag: 0.0,
            start_color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            end_color: Vec4::new(1.0, 1.0, 1.0, 0.0),
            color_gradient: None,
            start_size: 0.1..0.3,
            end_size: Some(0.05),
            size_over_lifetime: None,
            start_rotation: 0.0..std::f32::consts::TAU,
            rotation_speed: 0.0,
            enabled: true,
            emission_accumulator: 0.0,
            elapsed_time: 0.0,
        }
    }
}

impl ParticleEmitter {
    pub fn new(max_particles: u32) -> Self {
        Self {
            config: ParticleEmitterConfig {
                max_particles,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// 设置发射速率
    pub fn with_emission_rate(mut self, rate: f32) -> Self {
        self.config.emission_rate = rate;
        self
    }

    /// 设置生命周期
    pub fn with_lifetime(mut self, min: f32, max: f32) -> Self {
        self.lifetime = min..max;
        self
    }

    /// 设置重力
    pub fn with_gravity(mut self, gravity: Vec3) -> Self {
        self.gravity = gravity;
        self
    }

    /// 设置初始速度
    pub fn with_velocity(mut self, min: Vec3, max: Vec3) -> Self {
        self.initial_velocity = min..max;
        self
    }

    /// 设置颜色
    pub fn with_colors(mut self, start: Vec4, end: Vec4) -> Self {
        self.start_color = start;
        self.end_color = end;
        self
    }

    /// 设置大小
    pub fn with_size(mut self, start_min: f32, start_max: f32, end: f32) -> Self {
        self.start_size = start_min..start_max;
        self.end_size = Some(end);
        self
    }

    /// 设置发射形状
    pub fn with_shape(mut self, shape: ParticleShape) -> Self {
        self.config.shape = shape;
        self
    }

    /// 计算本帧应发射的粒子数
    pub fn particles_to_emit(&mut self, delta_time: f32) -> u32 {
        if !self.enabled {
            return 0;
        }

        self.elapsed_time += delta_time;
        self.emission_accumulator += self.config.emission_rate * delta_time;

        let count = self.emission_accumulator.floor() as u32;
        self.emission_accumulator -= count as f32;
        count
    }

    /// 重置发射器
    pub fn reset(&mut self) {
        self.emission_accumulator = 0.0;
        self.elapsed_time = 0.0;
    }
}

// ============================================================================
// 颜色渐变
// ============================================================================

/// 颜色停止点
#[derive(Clone, Copy)]
pub struct ColorStop {
    /// 时间点（0-1）
    pub time: f32,
    /// 颜色
    pub color: Vec4,
}

/// 颜色渐变
#[derive(Clone)]
pub struct ColorGradient {
    /// 颜色停止点（按时间排序）
    pub stops: Vec<ColorStop>,
}

impl ColorGradient {
    pub fn new() -> Self {
        Self { stops: Vec::new() }
    }

    pub fn add_stop(mut self, time: f32, color: Vec4) -> Self {
        self.stops.push(ColorStop { time, color });
        self.stops.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
        self
    }

    /// 采样颜色
    pub fn sample(&self, t: f32) -> Vec4 {
        if self.stops.is_empty() {
            return Vec4::ONE;
        }
        if self.stops.len() == 1 {
            return self.stops[0].color;
        }

        let t = t.clamp(0.0, 1.0);

        // 找到两个相邻的停止点
        for i in 0..self.stops.len() - 1 {
            let a = &self.stops[i];
            let b = &self.stops[i + 1];
            if t >= a.time && t <= b.time {
                let local_t = (t - a.time) / (b.time - a.time);
                return a.color.lerp(b.color, local_t);
            }
        }

        self.stops.last().unwrap().color
    }
}

impl Default for ColorGradient {
    fn default() -> Self {
        Self::new()
            .add_stop(0.0, Vec4::new(1.0, 1.0, 1.0, 1.0))
            .add_stop(1.0, Vec4::new(1.0, 1.0, 1.0, 0.0))
    }
}

// ============================================================================
// 大小随生命周期
// ============================================================================

/// 大小曲线类型
#[derive(Clone)]
pub enum SizeOverLifetime {
    /// 线性变化
    Linear { start: f32, end: f32 },
    /// 曲线
    Curve { points: Vec<(f32, f32)> },
    /// 随机在两条曲线之间
    RandomBetweenCurves {
        min_points: Vec<(f32, f32)>,
        max_points: Vec<(f32, f32)>,
    },
}

impl SizeOverLifetime {
    /// 采样大小
    pub fn sample(&self, t: f32) -> f32 {
        match self {
            Self::Linear { start, end } => start + (end - start) * t,
            Self::Curve { points } => sample_curve(points, t),
            Self::RandomBetweenCurves { min_points, max_points } => {
                let min = sample_curve(min_points, t);
                let max = sample_curve(max_points, t);
                (min + max) * 0.5 // 简化处理
            }
        }
    }
}

/// 曲线采样
fn sample_curve(points: &[(f32, f32)], t: f32) -> f32 {
    if points.is_empty() {
        return 1.0;
    }
    if points.len() == 1 {
        return points[0].1;
    }

    let t = t.clamp(0.0, 1.0);

    for i in 0..points.len() - 1 {
        let (t0, v0) = points[i];
        let (t1, v1) = points[i + 1];
        if t >= t0 && t <= t1 {
            let local_t = (t - t0) / (t1 - t0);
            return v0 + (v1 - v0) * local_t;
        }
    }

    points.last().unwrap().1
}

// ============================================================================
// GPU 粒子数据结构
// ============================================================================

/// GPU 粒子结构（对应 WGSL struct）
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuParticle {
    /// 位置
    pub position: [f32; 3],
    /// 生命周期
    pub lifetime: f32,
    /// 速度
    pub velocity: [f32; 3],
    /// 当前年龄
    pub age: f32,
    /// 颜色
    pub color: [f32; 4],
    /// 大小
    pub size: f32,
    /// 旋转
    pub rotation: f32,
    /// 旋转速度
    pub rotation_speed: f32,
    /// 存活标记（1.0 = 存活，0.0 = 死亡）
    pub alive: f32,
}

impl Default for GpuParticle {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            lifetime: 0.0,
            velocity: [0.0; 3],
            age: 0.0,
            color: [1.0, 1.0, 1.0, 1.0],
            size: 1.0,
            rotation: 0.0,
            rotation_speed: 0.0,
            alive: 0.0,
        }
    }
}

/// GPU 粒子系统 Uniform
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ParticleSystemUniforms {
    /// 发射器位置
    pub emitter_position: [f32; 3],
    /// 时间增量
    pub delta_time: f32,
    /// 重力
    pub gravity: [f32; 3],
    /// 阻力
    pub drag: f32,
    /// 本帧发射数量
    pub emit_count: u32,
    /// 当前时间
    pub time: f32,
    /// 随机种子
    pub random_seed: f32,
    /// 填充
    pub _padding: f32,
}

// ============================================================================
// GPU 粒子系统
// ============================================================================

/// GPU 粒子系统
pub struct GpuParticleSystem {
    /// 粒子缓冲区
    pub particle_buffer: wgpu::Buffer,
    /// 存活列表缓冲区
    pub alive_list_buffer: wgpu::Buffer,
    /// 死亡列表缓冲区
    pub dead_list_buffer: wgpu::Buffer,
    /// 计数器缓冲区（存活数、死亡数、发射数）
    pub counter_buffer: wgpu::Buffer,
    /// Uniform 缓冲区
    pub uniform_buffer: wgpu::Buffer,
    /// 发射 Compute Pipeline
    pub emit_pipeline: Option<wgpu::ComputePipeline>,
    /// 更新 Compute Pipeline
    pub update_pipeline: Option<wgpu::ComputePipeline>,
    /// 渲染 Pipeline
    pub render_pipeline: Option<wgpu::RenderPipeline>,
    /// Bind Group Layout
    pub bind_group_layout: Option<wgpu::BindGroupLayout>,
    /// Bind Group
    pub bind_group: Option<wgpu::BindGroup>,
    /// 最大粒子数
    pub max_particles: u32,
    /// 统计信息
    pub stats: ParticleSystemStats,
}

/// 粒子系统统计
#[derive(Default, Clone, Copy)]
pub struct ParticleSystemStats {
    /// 当前存活粒子数
    pub alive_count: u32,
    /// 总发射数
    pub total_emitted: u64,
    /// 本帧发射数
    pub frame_emitted: u32,
    /// 模拟时间（ms）
    pub simulation_time_ms: f32,
    /// 渲染时间（ms）
    pub render_time_ms: f32,
}

impl GpuParticleSystem {
    /// 创建新的 GPU 粒子系统
    pub fn new(device: &wgpu::Device, max_particles: u32) -> Self {
        let particle_size = std::mem::size_of::<GpuParticle>() as u64;
        let particle_buffer_size = particle_size * max_particles as u64;

        // 创建粒子缓冲区
        let particle_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Particle Buffer"),
            size: particle_buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        // 存活列表（索引）
        let alive_list_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Alive List Buffer"),
            size: (max_particles * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // 死亡列表（索引）
        let dead_list_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Dead List Buffer"),
            size: (max_particles * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // 计数器（alive_count, dead_count, emit_count, dispatch_count）
        let counter_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Counter Buffer"),
            size: 16,
            usage: wgpu::BufferUsages::STORAGE 
                | wgpu::BufferUsages::COPY_DST 
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::INDIRECT,
            mapped_at_creation: false,
        });

        // Uniform 缓冲区
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Particle System Uniforms"),
            size: std::mem::size_of::<ParticleSystemUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            particle_buffer,
            alive_list_buffer,
            dead_list_buffer,
            counter_buffer,
            uniform_buffer,
            emit_pipeline: None,
            update_pipeline: None,
            render_pipeline: None,
            bind_group_layout: None,
            bind_group: None,
            max_particles,
            stats: ParticleSystemStats::default(),
        }
    }

    /// 初始化（重置死亡列表为所有粒子）
    pub fn initialize(&self, queue: &wgpu::Queue) {
        // 初始化死亡列表为 [0, 1, 2, ..., max_particles-1]
        let dead_list: Vec<u32> = (0..self.max_particles).collect();
        queue.write_buffer(&self.dead_list_buffer, 0, bytemuck::cast_slice(&dead_list));

        // 初始化计数器（alive=0, dead=max, emit=0, dispatch=0）
        let counters = [0u32, self.max_particles, 0u32, 0u32];
        queue.write_buffer(&self.counter_buffer, 0, bytemuck::cast_slice(&counters));
    }

    /// 更新 Uniform
    pub fn update_uniforms(
        &mut self,
        queue: &wgpu::Queue,
        emitter_position: Vec3,
        gravity: Vec3,
        drag: f32,
        emit_count: u32,
        delta_time: f32,
        time: f32,
    ) {
        let uniforms = ParticleSystemUniforms {
            emitter_position: emitter_position.to_array(),
            delta_time,
            gravity: gravity.to_array(),
            drag,
            emit_count,
            time,
            random_seed: rand::random::<f32>(),
            _padding: 0.0,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    /// 重置系统
    pub fn reset(&mut self, queue: &wgpu::Queue) {
        self.initialize(queue);
        self.stats = ParticleSystemStats::default();
    }
}

// ============================================================================
// ECS 系统
// ============================================================================

/// 粒子发射器更新系统
pub fn particle_emitter_update_system(
    time: Res<crate::ecs::Time>,
    mut emitters: Query<&mut ParticleEmitter>,
) {
    let delta = time.delta_seconds;

    for mut emitter in emitters.iter_mut() {
        if !emitter.enabled {
            continue;
        }

        // 检查持续时间
        if let Some(duration) = emitter.config.duration {
            if emitter.elapsed_time >= duration && !emitter.config.looping {
                emitter.enabled = false;
                continue;
            }
        }

        // 计算发射数（实际发射在 GPU compute shader 中执行）
        let _emit_count = emitter.particles_to_emit(delta);
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_particle_emitter_default() {
        let emitter = ParticleEmitter::default();
        assert_eq!(emitter.config.max_particles, 10000);
        assert!(emitter.enabled);
    }

    #[test]
    fn test_color_gradient_sample() {
        let gradient = ColorGradient::default();
        
        let color_0 = gradient.sample(0.0);
        assert!((color_0.w - 1.0).abs() < 0.001); // Alpha = 1 at start
        
        let color_1 = gradient.sample(1.0);
        assert!(color_1.w.abs() < 0.001); // Alpha = 0 at end
        
        let color_half = gradient.sample(0.5);
        assert!((color_half.w - 0.5).abs() < 0.001); // Alpha = 0.5 at middle
    }

    #[test]
    fn test_size_over_lifetime() {
        let size = SizeOverLifetime::Linear { start: 1.0, end: 0.0 };
        
        assert!((size.sample(0.0) - 1.0).abs() < 0.001);
        assert!((size.sample(0.5) - 0.5).abs() < 0.001);
        assert!(size.sample(1.0).abs() < 0.001);
    }

    #[test]
    fn test_particles_to_emit() {
        let mut emitter = ParticleEmitter::new(1000);
        emitter.config.emission_rate = 100.0;
        
        // 0.01 秒应该发射 1 个粒子
        let count = emitter.particles_to_emit(0.01);
        assert_eq!(count, 1);
    }
}
