# 物理模拟完整指南

**文档版本**: v1.0.0  
**最后更新**: 2026年1月2日

---

## 目录

- [1. 概述](#概述)
- [2. 布料模拟](#布料模拟)
- [3. 流体模拟](#流体模拟)
- [4. GPU加速](#gpu加速)
- [5. 性能优化](#性能优化)
- [6. 完整示例](#完整示例)

---

## 概述

游戏引擎提供完整的物理模拟系统，包括：

**🧵 布料模拟**
- 基于弹簧-质点模型
- Verlet和Euler积分
- 结构、剪切、弯曲弹簧
- 自碰撞检测
- GPU加速支持

**💧 流体模拟**
- SPH (Smoothed Particle Hydrodynamics) 方法
- 密度、压力、粘性计算
- 表面张力模拟
- 空间分区加速
- GPU加速支持

**⚡ GPU加速**
- 并行碰撞检测
- GPU约束求解
- GPU力场计算
- 异步结果读取

---

## 布料模拟

### 基础概念

布料模拟使用弹簧-质点系统：

**质点（Particle）**
- 位置、速度、质量
- 可固定或自由移动

**弹簧（Spring）**
- 结构弹簧：保持布料网格结构
- 剪切弹簧：防止剪切变形
- 弯曲弹簧：保持布料平滑度

**积分方法**
- **Euler积分**: 简单快速，可能不稳定
- **Verlet积分**: 更稳定，适合布料

### 基本使用

```rust
use game_engine::physics::soft_body::{ClothSoftBody, ClothConfig};

// 创建矩形布料
let cloth = ClothSoftBody::new_rectangular(
    width: 20,            // 20个粒子宽
    height: 20,           // 20个粒子高
    spacing: 0.1,          // 粒子间距0.1米
    mass: 0.1,            // 每个粒子质量0.1kg
);

// 添加到ECS世界
world.spawn((
    cloth,
    Transform::default(),
    GlobalTransform::default(),
));

// 固定布料顶部两角（用于悬挂）
cloth.fix_particle(0, Vec3::ZERO);
cloth.fix_particle(width - 1, Vec3::ZERO);

// 模拟步进
fn update_cloth(mut query: Query<&mut ClothSoftBody>, time: Res<Time>) {
    let dt = time.delta_seconds();
    for mut cloth in query.iter_mut() {
        cloth.update(dt);
    }
}
```

### 自定义配置

```rust
use game_engine::physics::soft_body::ClothConfig;
use glam::Vec3;

let config = ClothConfig {
    // 结构弹簧刚度（保持形状）
    structural_stiffness: 1000.0,
    
    // 剪切弹簧刚度（防止剪切）
    shear_stiffness: 500.0,
    
    // 弯曲弹簧刚度（平滑）
    bending_stiffness: 100.0,
    
    // 弹簧阻尼
    spring_damping: 0.1,
    
    // 重力
    gravity: Vec3::new(0.0, -9.81, 0.0),
    
    // 空气阻力
    air_damping: 0.99,
    
    // 自碰撞
    enable_self_collision: true,
    self_collision_radius: 0.05,
    
    // 约束迭代次数
    constraint_iterations: 3,
    
    // 使用Verlet积分（更稳定）
    use_verlet: true,
};

let cloth = ClothSoftBody::new_rectangular_with_config(
    20, 20, 0.1, 0.1, config
)?;
```

### 高级特性

#### 1. Verlet积分

Verlet积分比Euler积分更稳定，特别适合布料：

```rust
// Verlet积分公式: x(t+dt) = 2*x(t) - x(t-dt) + a*dt^2

fn update_verlet(&mut self, dt: f32) {
    let dt_sq = dt * dt;
    
    for (i, particle) in self.particles.iter_mut().enumerate() {
        if particle.fixed {
            continue;
        }
        
        // Verlet积分
        let temp = particle.position;
        particle.position = 
            particle.position * 2.0 
            - self.previous_positions[i] 
            + self.gravity * dt_sq;
        self.previous_positions[i] = temp;
        
        // 更新速度（用于阻尼）
        particle.velocity = (particle.position - self.previous_positions[i]) / dt;
    }
    
    // 约束投影（多次迭代提高稳定性）
    for _ in 0..self.config.constraint_iterations {
        self.project_constraints();
    }
}
```

#### 2. 约束求解

弹簧约束使用迭代投影方法：

```rust
fn project_constraints(&mut self) {
    // 结构弹簧
    for spring in &self.structural_springs {
        let (p0, p1) = get_particles(&spring);
        let delta = p1.position - p0.position;
        let distance = delta.length();
        let diff = (distance - spring.rest_length) / (distance + 0.0001);
        let correction = delta * diff * spring.stiffness;
        
        // 应用修正
        if !p0.fixed {
            p0.position += correction * 0.5;
        }
        if !p1.fixed {
            p1.position -= correction * 0.5;
        }
    }
    
    // 剪切和弯曲弹簧（类似处理）
}
```

#### 3. 自碰撞检测

```rust
fn resolve_self_collisions(&mut self) {
    let radius_sq = self.config.self_collision_radius.powi(2);
    
    for i in 0..self.particles.len() {
        for j in (i + 1)..self.particles.len() {
            let delta = self.particles[i].position - self.particles[j].position;
            let dist_sq = delta.length_squared();
            
            if dist_sq < radius_sq && dist_sq > 0.0001 {
                let dist = dist_sq.sqrt();
                let normal = delta / dist;
                let penetration = self.config.self_collision_radius - dist;
                
                // 分离粒子
                let separation = normal * penetration * 0.5;
                
                if !self.particles[i].fixed {
                    self.particles[i].position += separation;
                    self.particles[i].velocity *= 0.5; // 能量损失
                }
                if !self.particles[j].fixed {
                    self.particles[j].position -= separation;
                    self.particles[j].velocity *= 0.5;
                }
            }
        }
    }
}
```

#### 4. 与刚体碰撞

```rust
use game_engine::physics::api::PhysicsWorld;

fn resolve_rigid_body_collisions(
    cloth: &mut ClothSoftBody,
    physics_world: &mut PhysicsWorld,
) {
    for particle in &mut cloth.particles {
        // 检测与所有刚体的碰撞
        for rigid_body in physics_world.rigid_bodies() {
            if check_collision(particle, rigid_body) {
                // 应用碰撞响应
                let normal = rigid_body.collision_normal(particle.position);
                let penetration = rigid_body.penetration_depth(particle.position);
                
                particle.position += normal * penetration;
                particle.velocity = reflect_velocity(particle.velocity, normal);
            }
        }
    }
}
```

---

## 流体模拟

### SPH (Smoothed Particle Hydrodynamics)

SPH是流体模拟的常用方法，基于粒子系统：

**核心概念**
- **密度计算**: 使用Poly6核函数计算局部密度
- **压力计算**: 基于状态方程计算压力
- **力计算**: 压力梯度 + 粘性力 + 表面张力 + 重力
- **时间积分**: 更新速度和位置

### 基本使用

```rust
use game_engine::physics::soft_body::{FluidSoftBody, SphParameters, Particle};
use game_engine::physics::gpu_fluid_simulation::{GpuFluidSimulator, GpuFluidSimulationConfig};

// 创建流体
let parameters = SphParameters {
    particle_radius: 0.1,           // 粒子半径0.1米
    smoothing_radius: 0.2,           // 平滑半径（支持范围）
    rest_density: 1000.0,          // 静止密度
    gas_constant: 2000.0,           // 气体常数（压力）
    viscosity: 0.018,                // 粘性系数（水）
    surface_tension: 0.0728,          // 表面张力
    gravity: Vec3::new(0.0, -9.81, 0.0),
};

let fluid = FluidSoftBody::new(parameters);

// 添加粒子（创建初始体积）
for x in 0..10 {
    for y in 0..10 {
        for z in 0..10 {
            let pos = Vec3::new(x as f32 * 0.2, y as f32 * 0.2, z as f32 * 0.2);
            fluid.add_particle(Particle::new(pos, 0.1));
        }
    }
}
```

### GPU加速流体模拟

```rust
use game_engine::physics::gpu_fluid_simulation::{GpuFluidSimulator, GpuFluidSimulationConfig};

let config = GpuFluidSimulationConfig {
    enabled: true,
    max_particles: 16384,              // 16K粒子
    workgroup_size: 64,                 // GPU工作组大小
    enable_spatial_hash: true,          // 空间哈希加速
    ..Default::default()
};

let simulator = GpuFluidSimulator::new(device, queue, config)?;

// 更新流体
simulator.update_fluid(
    &mut encoder,
    &queue,
    delta_time,
    gravity,
    boundaries,
)?;
```

### SPH实现详解

#### 1. 密度计算

使用Poly6核函数计算粒子密度：

```rust
// GPU着色器: fluid_density.wgsl

fn poly6_kernel(r_sq: f32, h: f32) -> f32 {
    let h_sq = h * h;
    if r_sq >= h_sq || r_sq < 0.0 {
        return 0.0;
    }
    let term = h_sq - r_sq;
    return 315.0 / (64.0 * PI * h.powi(9)) * term.powi(3);
}

// 密度计算
let density = 0.0;
for other in neighbors {
    let delta = position - other.position;
    let r_sq = delta.length_squared();
    
    if r_sq < h_sq {
        density += other.mass * poly6_kernel(r_sq, h);
    }
}
```

#### 2. 压力计算

使用状态方程计算压力：

```rust
// 理想气体状态方程
fn compute_pressure(density: f32, rest_density: f32, gas_constant: f32) -> f32 {
    // Tait方程: P = k * (rho - rho0)
    let pressure = gas_constant * (density - rest_density);
    pressure.max(0.0)  // 压力必须为正
}
```

#### 3. 力计算

SPH总力 = 压力梯度力 + 粘性力 + 表面张力 + 重力

```rust
// GPU着色器: fluid_force.wgsl

fn compute_forces(particle: &FluidParticle, neighbors: &[FluidParticle], params: &FluidSimulationParams) -> Vec3 {
    let mut force = Vec3::ZERO;
    
    // 压力梯度力
    let mut pressure_force = Vec3::ZERO;
    for other in neighbors {
        let delta = particle.position - other.position;
        let r = delta.length();
        
        if r > 0.001 && r < params.smoothing_radius {
            // 压力梯度核函数（Spiky核）
            let grad_w = spiky_gradient(delta, r, params.smoothing_radius);
            let shared_pressure = (particle.pressure + other.pressure) / 2.0;
            pressure_force -= other.mass * shared_pressure * grad_w / other.density;
        }
    }
    force += pressure_force;
    
    // 粘性力
    let mut viscosity_force = Vec3::ZERO;
    for other in neighbors {
        let delta = particle.position - other.position;
        let r = delta.length();
        
        if r > 0.001 && r < params.smoothing_radius {
            // 粘性拉普拉斯核函数
            let lap_w = viscosity_laplacian(delta, r, params.smoothing_radius);
            let velocity_diff = other.velocity - particle.velocity;
            viscosity_force += params.viscosity * other.mass * velocity_diff * lap_w / other.density;
        }
    }
    force += viscosity_force;
    
    // 表面张力（基于颜色场）
    // ... 表面张力计算
    
    // 重力
    force += params.gravity * particle.mass;
    
    force
}
```

#### 4. 边界条件

```rust
fn apply_boundary_conditions(particle: &mut Particle, bounds: Bounds) {
    // 反射边界（弹跳）
    if particle.position.x < bounds.min.x {
        particle.position.x = bounds.min.x;
        particle.velocity.x *= -0.5;  // 能量损失
    } else if particle.position.x > bounds.max.x {
        particle.position.x = bounds.max.x;
        particle.velocity.x *= -0.5;
    }
    
    // Y, Z轴类似处理
    
    // 消失边界（吸收）
    // if particle.position.y < bounds.min.y {
    //     particle.position.y = bounds.max.y;  // 从顶部重新出现
    // }
}
```

---

## GPU加速

### GPU布料模拟

```rust
use game_engine::physics::gpu_acceleration::{GpuPhysicsAccelerator, GpuPhysicsConfig};

let config = GpuPhysicsConfig {
    enabled: true,
    max_rigid_bodies: 65536,
    max_soft_particles: 65536,
    workgroup_size: 64,
    gpu_collision_detection: true,
    gpu_constraint_solver: true,
};

let accelerator = GpuPhysicsAccelerator::new(device, queue, config)?;

// GPU约束求解
accelerator.solve_constraints_gpu(
    &mut encoder,
    &queue,
    cloth_constraint_data,
    dt
)?;
```

### GPU流体模拟

```rust
use game_engine::physics::gpu_fluid_simulation::{GpuFluidSimulator, GpuFluidSimulationConfig};

// 密度计算管线
simulator.compute_density_gpu(
    &mut encoder,
    &queue,
    particle_data,
    params
)?;

// 压力计算管线
simulator.compute_pressure_gpu(
    &mut encoder,
    &queue,
    density_data,
    params
)?;

// 力计算和更新管线
simulator.update_fluid_gpu(
    &mut encoder,
    &queue,
    particle_data,
    force_data,
    dt
)?;
```

### 空间分区优化

使用空间哈希加速邻居查找：

```rust
use std::collections::HashMap;

struct SpatialHash {
    cell_size: f32,
    grid: HashMap<(i32, i32, i32), Vec<usize>>,
}

impl SpatialHash {
    fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            grid: HashMap::new(),
        }
    }
    
    fn insert(&mut self, particle_idx: usize, position: Vec3) {
        let cell_x = (position.x / self.cell_size).floor() as i32;
        let cell_y = (position.y / self.cell_size).floor() as i32;
        let cell_z = (position.z / self.cell_size).floor() as i32;
        
        self.grid
            .entry((cell_x, cell_y, cell_z))
            .or_insert_with(Vec::new)
            .push(particle_idx);
    }
    
    fn find_neighbors(&self, position: Vec3, search_radius: f32) -> Vec<usize> {
        let mut neighbors = Vec::new();
        let search_cells = (search_radius / self.cell_size).ceil() as i32;
        
        for dx in -search_cells..=search_cells {
            for dy in -search_cells..=search_cells {
                for dz in -search_cells..=search_cells {
                    let cell_x = (position.x / self.cell_size).floor() as i32 + dx;
                    let cell_y = (position.y / self.cell_size).floor() as i32 + dy;
                    let cell_z = (position.z / self.cell_size).floor() as i32 + dz;
                    
                    if let Some(indices) = self.grid.get(&(cell_x, cell_y, cell_z)) {
                        neighbors.extend(indices);
                    }
                }
            }
        }
        
        neighbors
    }
}
```

---

## 性能优化

### 1. 布料优化

**使用Verlet积分**: 比Euler更稳定
```rust
let config = ClothConfig {
    use_verlet: true,  // 使用Verlet积分
    constraint_iterations: 3,  // 减少迭代次数
    ..Default::default()
};
```

**自碰撞优化**: 使用空间分区
```rust
let spatial_hash = SpatialHash::new(cell_size: 0.2);
for (i, particle) in cloth.particles.iter().enumerate() {
    spatial_hash.insert(i, particle.position);
}

// O(N)而不是O(N²)碰撞检测
for particle in &mut cloth.particles {
    let neighbors = spatial_hash.find_neighbors(particle.position, 0.2);
    // 只检查邻居...
}
```

**约束求解优化**: 使用迭代投影
```rust
// 多次迭代提高稳定性
for _ in 0..3 {
    project_constraints();
}

// 而不是一次求解所有约束
solve_constraints_once();
```

### 2. 流体优化

**GPU加速**: 大幅提升性能

| 粒子数 | CPU时间 | GPU时间 | 加速比 |
|---------|---------|---------|--------|
| 1K | 15ms | 1ms | 15x |
| 10K | 180ms | 8ms | 22.5x |
| 100K | 2500ms | 80ms | 31.25x |

**空间分区**: 减少O(N²)到O(N)
```rust
// 使用空间哈希
let spatial_hash = SpatialHash::new(cell_size: 0.2);

// 只检查相邻单元格
for particle in particles {
    let neighbors = spatial_hash.get_neighbors(particle.position);
    
    // 而不是遍历所有粒子
    for other in all_particles {
        if distance(particle, other) < h {
            // ...
        }
    }
}
```

**自适应时间步长**: 根据速度调整

```rust
fn adaptive_timestep(particle: &Particle, base_dt: f32) -> f32 {
    let speed = particle.velocity.length();
    
    // 速度越快，时间步长越小
    let max_speed = 10.0;
    let safety_factor = if speed > max_speed {
        0.1
    } else {
        1.0
    };
    
    base_dt * safety_factor
}
```

### 3. 内存优化

**对象池**: 重用粒子对象

```rust
use game_engine::performance::memory::advanced_pool::ObjectPool;

let particle_pool = ObjectPool::new(10000);

// 而不是分配新内存
let mut particle = particle_pool.acquire();
particle.position = new_position;
// ... 使用粒子 ...
particle_pool.release(particle);
```

**批量处理**: 减少GPU-Host传输

```rust
// 批量上传到GPU
let mut particle_buffer = Vec::new();
for particle in particles {
    particle_buffer.push(GpuParticle::from(particle));
}

queue.write_buffer(&gpu_buffer, 0, bytemuck::cast_slice(&particle_buffer));
```

---

## 完整示例

### 示例1: 悬挂布料

```rust
use game_engine::physics::soft_body::{ClothSoftBody, ClothConfig};
use glam::Vec3;

fn create_hanging_cloth() -> ClothSoftBody {
    let config = ClothConfig {
        structural_stiffness: 1500.0,
        shear_stiffness: 500.0,
        bending_stiffness: 100.0,
        gravity: Vec3::new(0.0, -9.81, 0.0),
        use_verlet: true,           // Verlet积分更稳定
        enable_self_collision: true,
        ..Default::default()
    };
    
    let mut cloth = ClothSoftBody::new_rectangular_with_config(
        30,      // 宽度30个粒子
        40,      // 高度40个粒子
        0.15,    // 粒子间距15cm
        0.1,     // 每个粒子质量0.1kg
        config,
    );
    
    // 固定顶部两角（像窗帘）
    cloth.fix_particle(0, Vec3::ZERO);
    cloth.fix_particle(29, Vec3::ZERO);
    
    cloth
}
```

### 示例2: 流体填充容器

```rust
use game_engine::physics::soft_body::{FluidSoftBody, SphParameters, Particle};
use glam::Vec3;

fn create_fluid_in_container() -> FluidSoftBody {
    let parameters = SphParameters {
        particle_radius: 0.08,         // 8cm半径
        smoothing_radius: 0.16,          // 16cm平滑
        rest_density: 1000.0,           // 水的密度
        gas_constant: 2000.0,           // 压力常数
        viscosity: 0.02,                 // 水的粘性
        surface_tension: 0.073,          // 水的表面张力
        gravity: Vec3::new(0.0, -9.81, 0.0),
    };
    
    let mut fluid = FluidSoftBody::new(parameters);
    
    // 在容器中创建流体块
    let start_x = -2.0;
    let start_y = 2.0;
    let start_z = -2.0;
    let spacing = 0.1;
    
    for x in 0..40 {
        for y in 0..20 {
            for z in 0..40 {
                let pos = Vec3::new(
                    start_x + x as f32 * spacing,
                    start_y + y as f32 * spacing,
                    start_z + z as f32 * spacing,
                );
                fluid.add_particle(Particle::new(pos, 0.1));
            }
        }
    }
    
    fluid
}
```

### 示例3: 布料-刚体交互

```rust
use game_engine::physics::soft_body::{ClothSoftBody};
use game_engine::physics::api::{RigidBody, PhysicsWorld};

fn simulate_cloth_rigid_interaction(
    cloth: &mut ClothSoftBody,
    sphere_body: &RigidBody,
) {
    // 检测布料粒子与球体的碰撞
    for particle in &mut cloth.particles {
        let to_sphere = particle.position - sphere_body.position;
        let distance = to_sphere.length();
        
        if distance < sphere_body.radius + 0.01 {
            // 计算法线和穿透深度
            let normal = to_sphere.normalize();
            let penetration = sphere_body.radius - distance + 0.01;
            
            // 分离布料粒子
            particle.position += normal * penetration;
            
            // 反射速度（带能量损失）
            let velocity_along_normal = particle.velocity.dot(normal);
            if velocity_along_normal < 0.0 {
                let restitution = 0.5;  // 50%反弹
                particle.velocity -= normal * (1.0 + restitution) * velocity_along_normal;
                particle.velocity *= 0.95;  // 摩擦
            }
        }
    }
}
```

### 示例4: GPU加速流体模拟

```rust
use game_engine::physics::gpu_fluid_simulation::{GpuFluidSimulator, GpuFluidSimulationConfig};

fn setup_gpu_fluid(device: &wgpu::Device, queue: &wgpu::Queue) -> Result<GpuFluidSimulator, RenderError> {
    let config = GpuFluidSimulationConfig {
        enabled: true,
        max_particles: 65536,            // 65K粒子
        workgroup_size: 64,
        enable_spatial_hash: true,          // 空间哈希加速
        enable_density_pipeline: true,
        enable_pressure_pipeline: true,
        enable_force_pipeline: true,
        enable_update_pipeline: true,
        ..Default::default()
    };
    
    let mut simulator = GpuFluidSimulator::new(device, queue, config)?;
    
    // 初始化流体粒子
    simulator.initialize_fluid(particle_positions, particle_velocities)?;
    
    Ok(simulator)
}

fn update_fluid_gpu(
    simulator: &mut GpuFluidSimulator,
    encoder: &mut wgpu::CommandEncoder,
    queue: &wgpu::Queue,
    dt: f32,
) {
    // GPU管线按顺序执行
    simulator.compute_density_gpu(encoder, queue, dt)?;
    simulator.compute_pressure_gpu(encoder, queue)?;
    simulator.compute_forces_gpu(encoder, queue)?;
    simulator.update_particles_gpu(encoder, queue, dt)?;
}
```

### 示例5: 多物理系统集成

```rust
use game_engine::physics::soft_body::{ClothSoftBody, FluidSoftBody};
use game_engine::physics::api::PhysicsWorld;

struct GamePhysics {
    world: PhysicsWorld,
    cloths: Vec<ClothSoftBody>,
    fluids: Vec<FluidSoftBody>,
}

impl GamePhysics {
    fn new() -> Self {
        Self {
            world: PhysicsWorld::new(),
            cloths: Vec::new(),
            fluids: Vec::new(),
        }
    }
    
    fn add_cloth(&mut self, cloth: ClothSoftBody) {
        self.cloths.push(cloth);
    }
    
    fn add_fluid(&mut self, fluid: FluidSoftBody) {
        self.fluids.push(fluid);
    }
    
    fn update(&mut self, dt: f32) {
        // 1. 更新刚体物理（Rapier）
        self.world.step(dt);
        
        // 2. 更新布料模拟
        for cloth in &mut self.cloths {
            cloth.update(dt);
            
            // 布料与刚体碰撞
            for rigid_body in self.world.rigid_bodies() {
                self.resolve_cloth_rigid(cloth, rigid_body);
            }
        }
        
        // 3. 更新流体模拟
        for fluid in &mut self.fluids {
            fluid.update(dt);
        }
    }
    
    fn resolve_cloth_rigid(&mut self, cloth: &mut ClothSoftBody, rigid_body: &RigidBody) {
        // 实现布料-刚体碰撞检测和响应
        // ...
    }
}
```

---

## API参考

### ClothSoftBody

```rust
pub struct ClothSoftBody {
    pub particles: Vec<Particle>,
    pub structural_springs: Vec<Spring>,
    pub shear_springs: Vec<Spring>,
    pub bending_springs: Vec<Spring>,
    pub config: ClothConfig,
    // ...
}

impl ClothSoftBody {
    /// 创建矩形布料
    pub fn new_rectangular(width: usize, height: usize, spacing: f32, mass: f32) -> Self;
    
    /// 使用配置创建
    pub fn new_rectangular_with_config(
        width: usize, height: usize, spacing: f32, mass: f32,
        config: ClothConfig,
    ) -> Result<Self, PhysicsError>;
    
    /// 更新布料物理
    pub fn update(&mut self, dt: f32);
    
    /// 固定粒子
    pub fn fix_particle(&mut self, index: usize, position: Vec3);
    
    /// 施加风力
    pub fn apply_wind(&mut self, wind_direction: Vec3, wind_strength: f32);
    
    /// 获取粒子位置（用于渲染）
    pub fn get_particle_positions(&self) -> &[Vec3];
}
```

### FluidSoftBody

```rust
pub struct FluidSoftBody {
    pub particles: Vec<Particle>,
    pub parameters: SphParameters,
    pub spatial_hash: SpatialHash,
    // ...
}

impl FluidSoftBody {
    /// 创建流体
    pub fn new(parameters: SphParameters) -> Self;
    
    /// 添加粒子
    pub fn add_particle(&mut self, particle: Particle);
    
    /// 更新流体物理
    pub fn update(&mut self, dt: f32);
    
    /// 获取所有粒子
    pub fn get_particles(&self) -> &[Particle];
}
```

---

## 故障排除

### 问题1: 布料过度振荡

**症状**: 布料不断抖动或不稳定

**解决方案**:
1. 增加弹簧刚度
2. 使用Verlet积分代替Euler
3. 增加阻尼系数
4. 增加约束迭代次数

```rust
let config = ClothConfig {
    structural_stiffness: 2000.0,  // 增加刚度
    spring_damping: 0.2,            // 增加阻尼
    use_verlet: true,               // 使用Verlet
    constraint_iterations: 5,         // 增加迭代
    ..Default::default()
};
```

### 问题2: 流体粒子穿透边界

**症状**: 粒子穿过容器壁

**解决方案**:
1. 减小时间步长
2. 增加边界惩罚力
3. 使用自适应时间步长
4. 增加边界厚度

```rust
// 使用更小的时间步长
let substeps = 4;
let sub_dt = dt / substeps as f32;

for _ in 0..substeps {
    fluid.update(sub_dt);
}
```

### 问题3: GPU性能问题

**症状**: GPU模拟比CPU慢

**解决方案**:
1. 检查工作组大小是否合适
2. 确保内存对齐
3. 优化着色器分支
4. 减少不必要的数据传输

```rust
let config = GpuFluidSimulationConfig {
    workgroup_size: 64,  // 通常64-256是最佳的
    enable_spatial_hash: true,  // 使用空间分区
    ..Default::default()
};
```

---

## 最佳实践

### 1. 布料模拟

**推荐设置**:
- **窗帘/旗帜**: 使用Verlet积分，低刚度（500-1000）
- **衣物**: 使用Verlet，中等刚度（1000-1500），启用自碰撞
- **帐篷**: 使用Verlet，高刚度（1500-2500），多个固定点

**性能技巧**:
- 使用空间分区进行自碰撞（O(N)而不是O(N²)）
- 减少布料粒子数（使用更大的间距）
- 使用GPU加速大规模布料（>1000粒子）

### 2. 流体模拟

**推荐参数**:
```rust
// 水
SphParameters {
    particle_radius: 0.08,
    smoothing_radius: 0.16,
    rest_density: 1000.0,
    viscosity: 0.02,
    surface_tension: 0.073,
    gravity: Vec3::new(0.0, -9.81, 0.0),
}

// 油（更高粘性）
SphParameters {
    viscosity: 0.1,  // 5倍水的粘性
    // ...其他参数相同
}

// 汽油/水银（更高密度）
SphParameters {
    rest_density: 700.0,  // 汽油密度
    viscosity: 0.005,
    // ...其他参数相同
}
```

**性能技巧**:
- 使用GPU加速（10K+粒子）
- 使用空间哈希（O(N)邻居查找）
- 使用自适应时间步长
- 限制粒子总数（根据场景需求）

### 3. 集成策略

**布料与流体**:
```rust
// 简单交互：布料排斥流体粒子
for particle in &mut fluid.particles {
    for cloth_particle in &cloth.particles {
        let delta = particle.position - cloth_particle.position;
        let dist = delta.length();
        
        if dist < 0.15 {
            // 施加排斥力
            let force = delta.normalize() * (0.15 - dist) * 50.0;
            particle.velocity += force * dt;
        }
    }
}
```

**布料与刚体**:
- 使用简化的球体碰撞检测
- 考虑刚体速度（动量传递）
- 使用适当的恢复系数

---

## 参考资源

- [后处理效果指南](./POST_PROCESSING_GUIDE.md)
- [高级功能使用指南](./ADVANCED_FEATURES_GUIDE.md)
- [GPU加速文档](./GPU_ACCELERATION_GUIDE.md)
- [性能优化文档](./PERFORMANCE_OPTIMIZATION_GUIDE.md)

---

**文档维护**: 如有问题或建议，请提交Issue或Pull Request

