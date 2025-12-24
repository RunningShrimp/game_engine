//  GPU 计算着色器和物理加速
//
//  使用 WGPU 实现 GPU 计算着色器进行并行物理模拟
//  - 粒子系统模拟
//  - 碰撞检测
//  - 约束求解
//  - 力场计算

use glam::Vec3;
use std::sync::Arc;

/// GPU 物理计算着色器源代码
const GPU_PHYSICS_SHADER: &str = r#"
// GPU 物理计算着色器

struct PhysicsBody {
    position: vec3<f32>,
    inv_mass: f32,
    velocity: vec3<f32>,
    angular_velocity: f32,
    force: vec3<f32>,
    _padding: f32,
}

struct SimParams {
    gravity: vec3<f32>,
    time_step: f32,
    damping: f32,
    body_count: u32,
    _padding: vec2<f32>,
}

@group(0) @binding(0) var<storage, read_write> bodies: array<PhysicsBody>;
@group(0) @binding(1) var<uniform> params: SimParams;

@compute @workgroup_size(64)
fn integrate(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if (idx >= params.body_count) {
        return;
    }

    var body = bodies[idx];
    
    // 只更新动态物体（inv_mass > 0）
    if (body.inv_mass > 0.0) {
        // 计算加速度
        let acceleration = (body.force + params.gravity) * body.inv_mass;
        
        // 更新速度（半隐式欧拉积分）
        body.velocity = body.velocity + acceleration * params.time_step;
        
        // 应用阻尼
        body.velocity = body.velocity * params.damping;
        
        // 更新位置
        body.position = body.position + body.velocity * params.time_step;
        
        // 清除力累积
        body.force = vec3<f32>(0.0, 0.0, 0.0);
    }
    
    bodies[idx] = body;
}
"#;

/// GPU 物理体结构体
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GPUPhysicsBody {
    /// 位置 (世界坐标)
    pub position: Vec3,
    /// 倒数质量 (1/mass, 0 表示固定)
    pub inv_mass: f32,
    /// 速度
    pub velocity: Vec3,
    /// 角速度
    pub angular_velocity: f32,
    /// 累积力
    pub force: Vec3,
    /// 填充
    pub _padding0: f32,
}

/// GPU 物理体结构体（GPU 格式，用于着色器）
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GPUPhysicsBodyGPU {
    /// 位置 (世界坐标)
    position: [f32; 3],
    /// 倒数质量 (1/mass, 0 表示固定)
    inv_mass: f32,
    /// 速度
    velocity: [f32; 3],
    /// 角速度
    angular_velocity: f32,
    /// 累积力
    force: [f32; 3],
    /// 填充
    _padding: f32,
}

/// GPU 碰撞约束
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GPUConstraint {
    /// 约束类型 (0=距离, 1=球形, 2=胶囊体)
    pub constraint_type: u32,
    /// 第一个物体索引
    pub body_a_idx: u32,
    /// 第二个物体索引
    pub body_b_idx: u32,
    /// 约束参数 (距离等)
    pub param: f32,
    /// 累积脉冲
    pub impulse: f32,
    /// 填充
    pub _padding: [f32; 3],
}

/// GPU 碰撞信息
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GPUCollisionInfo {
    /// 第一个物体索引
    pub body_a_idx: u32,
    /// 第二个物体索引
    pub body_b_idx: u32,
    /// 碰撞法线
    pub normal: Vec3,
    /// 碰撞深度
    pub depth: f32,
    /// 碰撞点 A
    pub contact_point_a: Vec3,
    /// 填充
    pub _padding0: f32,
    /// 碰撞点 B
    pub contact_point_b: Vec3,
    /// 填充
    pub _padding1: f32,
}

/// GPU 物理模拟器配置
#[derive(Debug, Clone)]
pub struct GPUPhysicsConfig {
    /// 重力加速度
    pub gravity: Vec3,
    /// 时间步长 (秒)
    pub time_step: f32,
    /// 迭代次数
    pub iterations: u32,
    /// 阻尼系数
    pub damping: f32,
    /// 碰撞裕度
    pub collision_margin: f32,
    /// 工作组大小
    pub workgroup_size: u32,
}

impl Default for GPUPhysicsConfig {
    fn default() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            time_step: 0.016666,
            iterations: 8,
            damping: 0.999,
            collision_margin: 0.01,
            workgroup_size: 64,
        }
    }
}

/// GPU 模拟参数（用于着色器 uniform）
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct SimParams {
    gravity: [f32; 3],
    time_step: f32,
    damping: f32,
    body_count: u32,
    _padding: [f32; 2],
}

/// GPU 物理资源 - 管理 GPU 缓冲区和管线
pub struct GPUPhysicsResources {
    /// 物理体缓冲区
    body_buffer: wgpu::Buffer,
    /// 模拟参数 uniform 缓冲区
    params_buffer: wgpu::Buffer,
    /// 绑定组布局
    bind_group_layout: wgpu::BindGroupLayout,
    /// 绑定组
    bind_group: wgpu::BindGroup,
    /// 计算管线
    pipeline: wgpu::ComputePipeline,
    /// 最大物体数量
    max_bodies: u32,
}

impl GPUPhysicsResources {
    /// 创建 GPU 物理资源
    pub fn new(device: &wgpu::Device, max_bodies: u32) -> Self {
        // 创建物理体缓冲区
        let body_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Physics Body Buffer"),
            size: (max_bodies as u64) * std::mem::size_of::<GPUPhysicsBody>() as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // 创建参数 uniform 缓冲区
        let params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Physics Params Buffer"),
            size: std::mem::size_of::<SimParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // 创建绑定组布局
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Physics Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // 创建绑定组
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Physics Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: body_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: params_buffer.as_entire_binding(),
                },
            ],
        });

        // 编译着色器
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Physics Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(GPU_PHYSICS_SHADER.into()),
        });

        // 创建管线布局
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Physics Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // 创建计算管线
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Physics Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: Some("integrate"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        Self {
            body_buffer,
            params_buffer,
            bind_group_layout,
            bind_group,
            pipeline,
            max_bodies,
        }
    }

    /// 调整缓冲区大小并重新创建绑定组
    pub fn resize(&mut self, device: &wgpu::Device, new_max_bodies: u32) {
        self.max_bodies = new_max_bodies;
        // 重新创建缓冲区
        self.body_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Physics Body Buffer"),
            size: (new_max_bodies as u64) * std::mem::size_of::<GPUPhysicsBody>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        // 重新创建绑定组
        self.recreate_bind_group(device);
    }

    /// 重新创建绑定组（当缓冲区大小改变时）
    fn recreate_bind_group(&mut self, device: &wgpu::Device) {
        self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Physics Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.body_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.params_buffer.as_entire_binding(),
                },
            ],
        });
    }
}

/// GPU 物理模拟器
pub struct GPUPhysicsSimulator {
    /// 配置
    config: GPUPhysicsConfig,
    /// 物理体数据
    bodies: Vec<GPUPhysicsBody>,
    /// 约束数据
    constraints: Vec<GPUConstraint>,
    /// 碰撞信息
    collisions: Vec<GPUCollisionInfo>,
    /// 是否启用 GPU 计算
    gpu_enabled: bool,
    /// GPU 资源（可选）
    gpu_resources: Option<GPUPhysicsResources>,
    /// wgpu 设备引用
    device: Option<Arc<wgpu::Device>>,
    /// wgpu 队列引用
    queue: Option<Arc<wgpu::Queue>>,
}

impl GPUPhysicsSimulator {
    /// 创建新的 GPU 物理模拟器
    pub fn new() -> Self {
        Self {
            config: GPUPhysicsConfig::default(),
            bodies: Vec::new(),
            constraints: Vec::new(),
            collisions: Vec::new(),
            gpu_enabled: false,
            gpu_resources: None,
            device: None,
            queue: None,
        }
    }
}

impl Default for GPUPhysicsSimulator {
    fn default() -> Self {
        Self::new()
    }
}

impl GPUPhysicsSimulator {
    /// 创建并带有配置
    pub fn with_config(config: GPUPhysicsConfig) -> Self {
        Self {
            config,
            bodies: Vec::new(),
            constraints: Vec::new(),
            collisions: Vec::new(),
            gpu_enabled: false,
            gpu_resources: None,
            device: None,
            queue: None,
        }
    }

    /// 初始化 GPU 资源
    pub fn initialize_gpu(
        &mut self,
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        max_bodies: u32,
    ) {
        let resources = GPUPhysicsResources::new(&device, max_bodies);
        self.gpu_resources = Some(resources);
        self.device = Some(device);
        self.queue = Some(queue);
        self.gpu_enabled = true;
    }

    /// 启用/禁用 GPU 计算
    pub fn set_gpu_enabled(&mut self, enabled: bool) {
        if enabled && self.gpu_resources.is_none() {
            // 如果没有初始化 GPU 资源，回退到 CPU
            self.gpu_enabled = false;
        } else {
            self.gpu_enabled = enabled;
        }
    }

    /// 检查 GPU 是否可用
    pub fn is_gpu_available(&self) -> bool {
        self.gpu_resources.is_some()
    }

    /// 添加物理体
    pub fn add_body(&mut self, position: Vec3, mass: f32) -> usize {
        let body = GPUPhysicsBody {
            position,
            inv_mass: if mass > 0.0 { 1.0 / mass } else { 0.0 },
            velocity: Vec3::ZERO,
            angular_velocity: 0.0,
            force: Vec3::ZERO,
            _padding0: 0.0,
        };
        self.bodies.push(body);
        self.bodies.len() - 1
    }

    /// 添加约束
    pub fn add_constraint(&mut self, constraint_type: u32, body_a: u32, body_b: u32, param: f32) {
        let constraint = GPUConstraint {
            constraint_type,
            body_a_idx: body_a,
            body_b_idx: body_b,
            param,
            impulse: 0.0,
            _padding: [0.0; 3],
        };
        self.constraints.push(constraint);
    }

    /// 对物体施加力
    pub fn apply_force(&mut self, body_idx: usize, force: Vec3) {
        if body_idx < self.bodies.len() {
            self.bodies[body_idx].force += force;
        }
    }

    /// 执行单步物理模拟
    pub fn step(&mut self) {
        if self.gpu_enabled && self.gpu_resources.is_some() {
            self.step_gpu();
        } else {
            self.step_cpu();
        }
    }

    /// CPU 模拟步骤
    fn step_cpu(&mut self) {
        // 应用力和重力
        for body in &mut self.bodies {
            if body.inv_mass > 0.0 {
                let acceleration = (body.force + self.config.gravity) * body.inv_mass;
                body.velocity += acceleration * self.config.time_step;
                body.velocity *= self.config.damping;
                body.position += body.velocity * self.config.time_step;
                body.force = Vec3::ZERO;
            }
        }

        // 约束求解
        for _ in 0..self.config.iterations {
            self.solve_constraints();
        }
    }

    /// GPU 模拟步骤 - 使用 wgpu 计算着色器
    fn step_gpu(&mut self) {
        let (device, queue, resources) = match (&self.device, &self.queue, &self.gpu_resources) {
            (Some(d), Some(q), Some(r)) => (d, q, r),
            _ => {
                // 回退到 CPU 实现
                self.step_cpu();
                return;
            }
        };

        if self.bodies.is_empty() {
            return;
        }

        let body_count = self.bodies.len() as u32;

        // 检查缓冲区大小是否足够
        if body_count > resources.max_bodies {
            // 缓冲区太小，回退到 CPU
            self.step_cpu();
            return;
        }

        // 准备物理体数据（转换为 GPU 格式）
        let gpu_bodies: Vec<GPUPhysicsBodyGPU> = self
            .bodies
            .iter()
            .map(|b| GPUPhysicsBodyGPU {
                position: [b.position.x, b.position.y, b.position.z],
                inv_mass: b.inv_mass,
                velocity: [b.velocity.x, b.velocity.y, b.velocity.z],
                angular_velocity: b.angular_velocity,
                force: [b.force.x, b.force.y, b.force.z],
                _padding: 0.0,
            })
            .collect();

        // 上传物理体数据到 GPU
        queue.write_buffer(&resources.body_buffer, 0, bytemuck::cast_slice(&gpu_bodies));

        // 准备模拟参数
        let params = SimParams {
            gravity: [
                self.config.gravity.x,
                self.config.gravity.y,
                self.config.gravity.z,
            ],
            time_step: self.config.time_step,
            damping: self.config.damping,
            body_count,
            _padding: [0.0; 2],
        };

        // 上传参数到 GPU
        queue.write_buffer(&resources.params_buffer, 0, bytemuck::bytes_of(&params));

        // 创建命令编码器
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Physics Compute Encoder"),
        });

        // 创建计算通道
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Physics Compute Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&resources.pipeline);
            compute_pass.set_bind_group(0, &resources.bind_group, &[]);

            // 计算工作组数量
            let workgroups =
                (body_count + self.config.workgroup_size - 1) / self.config.workgroup_size;
            compute_pass.dispatch_workgroups(workgroups, 1, 1);
        }

        // 创建暂存缓冲区用于读取结果
        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Physics Staging Buffer"),
            size: (body_count as u64) * std::mem::size_of::<GPUPhysicsBodyGPU>() as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        // 复制结果到暂存缓冲区
        encoder.copy_buffer_to_buffer(
            &resources.body_buffer,
            0,
            &staging_buffer,
            0,
            (body_count as u64) * std::mem::size_of::<GPUPhysicsBodyGPU>() as u64,
        );

        // 提交命令
        queue.submit(std::iter::once(encoder.finish()));

        // 读取结果（同步方式）
        let buffer_slice = staging_buffer.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });

        if receiver.recv().ok().and_then(|r| r.ok()).is_some() {
            let data = buffer_slice.get_mapped_range();
            let gpu_results: &[GPUPhysicsBodyGPU] = bytemuck::cast_slice(&data);

            // 更新本地物理体数据
            for (i, gpu_body) in gpu_results.iter().enumerate() {
                if i < self.bodies.len() {
                    self.bodies[i].position = Vec3::new(
                        gpu_body.position[0],
                        gpu_body.position[1],
                        gpu_body.position[2],
                    );
                    self.bodies[i].velocity = Vec3::new(
                        gpu_body.velocity[0],
                        gpu_body.velocity[1],
                        gpu_body.velocity[2],
                    );
                    self.bodies[i].force =
                        Vec3::new(gpu_body.force[0], gpu_body.force[1], gpu_body.force[2]);
                }
            }
        }

        // 约束求解仍在 CPU 上执行（可以在未来扩展到 GPU）
        for _ in 0..self.config.iterations {
            self.solve_constraints();
        }
    }

    /// 约束求解
    fn solve_constraints(&mut self) {
        let constraints = self.constraints.clone();
        for mut constraint in constraints {
            match constraint.constraint_type {
                0 => self.solve_distance_constraint(&mut constraint),
                1 => self.solve_sphere_constraint(&mut constraint),
                2 => self.solve_capsule_constraint(&mut constraint),
                _ => {}
            }
        }
    }

    /// 求解距离约束
    fn solve_distance_constraint(&mut self, constraint: &mut GPUConstraint) {
        let a_idx = constraint.body_a_idx as usize;
        let b_idx = constraint.body_b_idx as usize;

        if a_idx >= self.bodies.len() || b_idx >= self.bodies.len() {
            return;
        }

        let pos_a = self.bodies[a_idx].position;
        let pos_b = self.bodies[b_idx].position;
        let inv_mass_a = self.bodies[a_idx].inv_mass;
        let inv_mass_b = self.bodies[b_idx].inv_mass;

        let delta = pos_b - pos_a;
        let dist = delta.length();
        let target_dist = constraint.param;

        if dist < 0.001 {
            return;
        }

        let diff = dist - target_dist;
        let correction = delta.normalize() * diff * 0.5;

        if inv_mass_a > 0.0 {
            self.bodies[a_idx].position += correction * inv_mass_a;
        }
        if inv_mass_b > 0.0 {
            self.bodies[b_idx].position -= correction * inv_mass_b;
        }
    }

    /// 求解球形约束 (固定球体)
    fn solve_sphere_constraint(&mut self, constraint: &mut GPUConstraint) {
        let a_idx = constraint.body_a_idx as usize;

        if a_idx >= self.bodies.len() {
            return;
        }

        // 约束参数编码为: 位置 (Vec3) 和半径
        // 简化: 固定到原点
        if self.bodies[a_idx].inv_mass > 0.0 {
            self.bodies[a_idx].position = Vec3::ZERO;
        }
    }

    /// 求解胶囊体约束
    fn solve_capsule_constraint(&mut self, constraint: &mut GPUConstraint) {
        // 简化实现: 类似于距离约束
        self.solve_distance_constraint(constraint);
    }

    /// 检测碰撞 (简化的碰撞检测)
    pub fn detect_collisions(&mut self) {
        self.collisions.clear();

        let n = self.bodies.len();
        for i in 0..n {
            for j in (i + 1)..n {
                let dist = (self.bodies[j].position - self.bodies[i].position).length();
                let min_dist = 1.0; // 假设最小碰撞距离

                if dist < min_dist {
                    let normal = (self.bodies[j].position - self.bodies[i].position).normalize();
                    let collision = GPUCollisionInfo {
                        body_a_idx: i as u32,
                        body_b_idx: j as u32,
                        normal,
                        depth: min_dist - dist,
                        contact_point_a: self.bodies[i].position + normal * 0.5,
                        _padding0: 0.0,
                        contact_point_b: self.bodies[j].position - normal * 0.5,
                        _padding1: 0.0,
                    };
                    self.collisions.push(collision);
                }
            }
        }
    }

    /// 获取物体
    pub fn get_bodies(&self) -> &[GPUPhysicsBody] {
        &self.bodies
    }

    /// 获取约束
    pub fn get_constraints(&self) -> &[GPUConstraint] {
        &self.constraints
    }

    /// 获取碰撞信息
    pub fn get_collisions(&self) -> &[GPUCollisionInfo] {
        &self.collisions
    }

    /// 获取物体位置
    pub fn get_body_position(&self, idx: usize) -> Option<Vec3> {
        self.bodies.get(idx).map(|b| b.position)
    }

    /// 获取物体速度
    pub fn get_body_velocity(&self, idx: usize) -> Option<Vec3> {
        self.bodies.get(idx).map(|b| b.velocity)
    }
}

/// GPU 粒子系统
pub struct GPUParticleSystem {
    /// 粒子位置
    positions: Vec<Vec3>,
    /// 粒子速度
    velocities: Vec<Vec3>,
    /// 粒子生命周期
    lifetimes: Vec<f32>,
    /// 最大粒子数
    max_particles: usize,
    /// 重力
    gravity: Vec3,
}

impl GPUParticleSystem {
    /// 创建新的粒子系统
    pub fn new(max_particles: usize) -> Self {
        Self {
            positions: Vec::with_capacity(max_particles),
            velocities: Vec::with_capacity(max_particles),
            lifetimes: Vec::with_capacity(max_particles),
            max_particles,
            gravity: Vec3::new(0.0, -9.81, 0.0),
        }
    }

    /// 发射粒子
    pub fn emit(&mut self, position: Vec3, velocity: Vec3, lifetime: f32) {
        if self.positions.len() < self.max_particles {
            self.positions.push(position);
            self.velocities.push(velocity);
            self.lifetimes.push(lifetime);
        }
    }

    /// 更新粒子系统
    pub fn update(&mut self, dt: f32) {
        let mut to_remove = Vec::new();

        for (i, lifetime) in self.lifetimes.iter_mut().enumerate() {
            *lifetime -= dt;

            if *lifetime <= 0.0 {
                to_remove.push(i);
            } else {
                // 应用重力和空气阻力
                self.velocities[i] += self.gravity * dt;
                self.velocities[i] *= 0.99; // 阻力

                self.positions[i] += self.velocities[i] * dt;
            }
        }

        // 移除已死亡的粒子
        for &i in to_remove.iter().rev() {
            self.positions.remove(i);
            self.velocities.remove(i);
            self.lifetimes.remove(i);
        }
    }

    /// 获取粒子数量
    pub fn particle_count(&self) -> usize {
        self.positions.len()
    }

    /// 获取粒子位置
    pub fn get_positions(&self) -> &[Vec3] {
        &self.positions
    }

    /// 获取粒子速度
    pub fn get_velocities(&self) -> &[Vec3] {
        &self.velocities
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_physics_simulator() {
        let mut sim = GPUPhysicsSimulator::new();

        let body_a = sim.add_body(Vec3::new(0.0, 0.0, 0.0), 1.0);
        let body_b = sim.add_body(Vec3::new(1.0, 0.0, 0.0), 1.0);

        assert_eq!(sim.get_bodies().len(), 2);
        assert_eq!(body_a, 0);
        assert_eq!(body_b, 1);
    }

    #[test]
    fn test_apply_force() {
        let mut sim = GPUPhysicsSimulator::new();
        let body_idx = sim.add_body(Vec3::ZERO, 1.0);

        sim.apply_force(body_idx, Vec3::new(10.0, 0.0, 0.0));
        sim.step();

        let vel = sim.get_body_velocity(body_idx).unwrap();
        assert!(vel.x > 0.0); // 受力影响
    }

    #[test]
    fn test_constraints() {
        let mut sim = GPUPhysicsSimulator::new();
        let body_a = sim.add_body(Vec3::new(0.0, 0.0, 0.0), 1.0);
        let body_b = sim.add_body(Vec3::new(2.0, 0.0, 0.0), 1.0);

        sim.add_constraint(0, body_a as u32, body_b as u32, 1.0); // 距离约束

        assert_eq!(sim.get_constraints().len(), 1);
    }

    #[test]
    fn test_collision_detection() {
        let mut sim = GPUPhysicsSimulator::new();
        let body_a = sim.add_body(Vec3::new(0.0, 0.0, 0.0), 1.0);
        let body_b = sim.add_body(Vec3::new(0.5, 0.0, 0.0), 1.0); // 很近

        // 验证身体已添加
        assert!(body_a >= 0);
        assert!(body_b >= 0);
        assert_ne!(body_a, body_b); // 应该是不同的ID

        sim.detect_collisions();

        assert!(sim.get_collisions().len() > 0);
    }

    #[test]
    fn test_gpu_particle_system() {
        let mut particles = GPUParticleSystem::new(100);

        particles.emit(Vec3::ZERO, Vec3::new(0.0, 10.0, 0.0), 2.0);
        particles.emit(Vec3::new(1.0, 0.0, 0.0), Vec3::new(1.0, 5.0, 0.0), 1.5);

        assert_eq!(particles.particle_count(), 2);

        particles.update(0.016);
        assert_eq!(particles.particle_count(), 2);

        particles.update(2.0);
        assert_eq!(particles.particle_count(), 0); // 所有粒子已死亡
    }
}
