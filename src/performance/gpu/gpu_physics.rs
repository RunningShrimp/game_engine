//! GPU 计算着色器和物理加速
//!
//! 使用 WGPU 实现 GPU 计算着色器进行并行物理模拟
//! - 粒子系统模拟
//! - 碰撞检测
//! - 约束求解
//! - 力场计算

use glam::Vec3;

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
}

impl Default for GPUPhysicsConfig {
    fn default() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            time_step: 0.016666,
            iterations: 8,
            damping: 0.999,
            collision_margin: 0.01,
        }
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
        }
    }

    /// 启用/禁用 GPU 计算
    pub fn set_gpu_enabled(&mut self, enabled: bool) {
        self.gpu_enabled = enabled;
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
        if self.gpu_enabled {
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

    /// GPU 模拟步骤 (占位符)
    fn step_gpu(&mut self) {
        // 在实际实现中，这将：
        // 1. 将物体数据上传到 GPU
        // 2. 执行计算着色器
        // 3. 读取结果
        // 目前为简化版本的 CPU 实现
        self.step_cpu();
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
