//! 软体物理系统
//!
//! 提供布料和流体模拟功能，基于粒子系统和弹簧-质点模型。
//!
//! ## 布料模拟
//!
//! 使用弹簧-质点系统模拟布料：
//! - 结构弹簧：保持布料形状
//! - 剪切弹簧：防止剪切变形
//! - 弯曲弹簧：保持平滑度
//!
//! ## 流体模拟
//!
//! 使用SPH（Smoothed Particle Hydrodynamics）方法模拟流体：
//! - 密度计算
//! - 压力计算
//! - 粘性计算
//! - 表面张力

use bevy_ecs::prelude::*;
use glam::Vec3A;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 软体类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SoftBodyType {
    /// 布料
    Cloth,
    /// 流体
    Fluid,
    /// 软体（通用）
    SoftBody,
}

/// 粒子
#[derive(Debug, Clone)]
pub struct Particle {
    /// 位置
    pub position: Vec3A,
    /// 速度
    pub velocity: Vec3A,
    /// 质量
    pub mass: f32,
    /// 密度（用于流体）
    pub density: f32,
    /// 压力（用于流体）
    pub pressure: f32,
    /// 是否固定（用于布料约束）
    pub fixed: bool,
}

impl Particle {
    pub fn new(position: Vec3A, mass: f32) -> Self {
        Self {
            position,
            velocity: Vec3A::ZERO,
            mass,
            density: 0.0,
            pressure: 0.0,
            fixed: false,
        }
    }

    pub fn fixed(position: Vec3A, mass: f32) -> Self {
        Self {
            position,
            velocity: Vec3A::ZERO,
            mass,
            density: 0.0,
            pressure: 0.0,
            fixed: true,
        }
    }
}

/// 弹簧连接
#[derive(Debug, Clone)]
pub struct Spring {
    /// 粒子索引对
    pub particles: (usize, usize),
    /// 静止长度
    pub rest_length: f32,
    /// 弹簧常数
    pub stiffness: f32,
    /// 阻尼系数
    pub damping: f32,
}

impl Spring {
    pub fn new(particle_a: usize, particle_b: usize, rest_length: f32, stiffness: f32) -> Self {
        Self {
            particles: (particle_a, particle_b),
            rest_length,
            stiffness,
            damping: 0.1,
        }
    }
}

/// 布料配置
#[derive(Debug, Clone)]
pub struct ClothConfig {
    /// 结构弹簧刚度
    pub structural_stiffness: f32,
    /// 剪切弹簧刚度
    pub shear_stiffness: f32,
    /// 弯曲弹簧刚度
    pub bending_stiffness: f32,
    /// 弹簧阻尼系数
    pub spring_damping: f32,
    /// 重力
    pub gravity: Vec3A,
    /// 空气阻力
    pub air_damping: f32,
    /// 启用自碰撞
    pub enable_self_collision: bool,
    /// 自碰撞半径
    pub self_collision_radius: f32,
    /// 约束迭代次数
    pub constraint_iterations: usize,
    /// 使用Verlet积分
    pub use_verlet: bool,
}

impl Default for ClothConfig {
    fn default() -> Self {
        Self {
            structural_stiffness: 1000.0,
            shear_stiffness: 500.0,
            bending_stiffness: 100.0,
            spring_damping: 0.1,
            gravity: Vec3A::new(0.0, -9.81, 0.0),
            air_damping: 0.99,
            enable_self_collision: false,
            self_collision_radius: 0.05,
            constraint_iterations: 3,
            use_verlet: false,
        }
    }
}

/// 布料软体
#[derive(Debug, Clone)]
pub struct ClothSoftBody {
    /// 粒子
    pub particles: Vec<Particle>,
    /// 结构弹簧（连接相邻粒子）
    pub structural_springs: Vec<Spring>,
    /// 剪切弹簧（连接对角线粒子）
    pub shear_springs: Vec<Spring>,
    /// 弯曲弹簧（连接间隔粒子）
    pub bending_springs: Vec<Spring>,
    /// 宽度（粒子数）
    pub width: usize,
    /// 高度（粒子数）
    pub height: usize,
    /// 粒子间距
    pub spacing: f32,
    /// 重力
    pub gravity: Vec3A,
    /// 空气阻力
    pub air_damping: f32,
    /// 配置
    pub config: ClothConfig,
    /// 上一帧位置（用于Verlet积分）
    pub previous_positions: Vec<Vec3A>,
}

impl ClothSoftBody {
    /// 创建矩形布料
    pub fn new_rectangular(width: usize, height: usize, spacing: f32, mass: f32) -> Self {
        let mut particles = Vec::new();
        let mut structural_springs = Vec::new();
        let mut shear_springs = Vec::new();
        let mut bending_springs = Vec::new();

        // 创建粒子网格
        for y in 0..height {
            for x in 0..width {
                let position = Vec3A::new(
                    x as f32 * spacing,
                    height as f32 * spacing - y as f32 * spacing,
                    0.0,
                );
                let particle = if y == 0 {
                    // 顶部固定
                    Particle::fixed(position, mass)
                } else {
                    Particle::new(position, mass)
                };
                particles.push(particle);
            }
        }

        // 创建结构弹簧（水平和垂直）
        for y in 0..height {
            for x in 0..width {
                let idx = y * width + x;

                // 水平连接
                if x < width - 1 {
                    let rest_length = spacing;
                    structural_springs.push(Spring::new(idx, idx + 1, rest_length, 1000.0));
                }

                // 垂直连接
                if y < height - 1 {
                    let rest_length = spacing;
                    structural_springs.push(Spring::new(idx, idx + width, rest_length, 1000.0));
                }
            }
        }

        // 创建剪切弹簧（对角线）
        for y in 0..height - 1 {
            for x in 0..width - 1 {
                let idx = y * width + x;
                let rest_length = spacing * 2.0_f32.sqrt();

                // 左上到右下
                shear_springs.push(Spring::new(idx, idx + width + 1, rest_length, 500.0));
                // 右上到左下
                shear_springs.push(Spring::new(idx + 1, idx + width, rest_length, 500.0));
            }
        }

        // 创建弯曲弹簧（间隔粒子）
        for y in 0..height {
            for x in 0..width {
                let idx = y * width + x;

                // 水平弯曲
                if x < width - 2 {
                    let rest_length = spacing * 2.0;
                    bending_springs.push(Spring::new(idx, idx + 2, rest_length, 100.0));
                }

                // 垂直弯曲
                if y < height - 2 {
                    let rest_length = spacing * 2.0;
                    bending_springs.push(Spring::new(idx, idx + width * 2, rest_length, 100.0));
                }
            }
        }

        let config = ClothConfig::default();
        let previous_positions = particles.iter().map(|p| p.position).collect();

        Self {
            particles,
            structural_springs,
            shear_springs,
            bending_springs,
            width,
            height,
            spacing,
            gravity: config.gravity,
            air_damping: config.air_damping,
            config,
            previous_positions,
        }
    }

    /// 使用自定义配置创建矩形布料
    pub fn new_rectangular_with_config(
        width: usize,
        height: usize,
        spacing: f32,
        mass: f32,
        config: ClothConfig,
    ) -> Self {
        let mut cloth = Self::new_rectangular(width, height, spacing, mass);
        cloth.config = config;
        cloth.gravity = cloth.config.gravity;
        cloth.air_damping = cloth.config.air_damping;

        // 更新弹簧参数
        for spring in &mut cloth.structural_springs {
            spring.stiffness = cloth.config.structural_stiffness;
            spring.damping = cloth.config.spring_damping;
        }
        for spring in &mut cloth.shear_springs {
            spring.stiffness = cloth.config.shear_stiffness;
            spring.damping = cloth.config.spring_damping;
        }
        for spring in &mut cloth.bending_springs {
            spring.stiffness = cloth.config.bending_stiffness;
            spring.damping = cloth.config.spring_damping;
        }

        cloth
    }

    /// 更新布料物理
    pub fn update(&mut self, dt: f32) {
        if self.config.use_verlet {
            self.update_verlet(dt);
        } else {
            self.update_euler(dt);
        }

        // 自碰撞检测
        if self.config.enable_self_collision {
            self.resolve_self_collisions();
        }
    }

    /// 使用欧拉积分更新
    fn update_euler(&mut self, dt: f32) {
        // 应用重力
        for particle in &mut self.particles {
            if !particle.fixed {
                particle.velocity += self.gravity * dt;
            }
        }

        // 应用弹簧力
        Self::apply_spring_forces_to_particles(&mut self.particles, &self.structural_springs, dt);
        Self::apply_spring_forces_to_particles(&mut self.particles, &self.shear_springs, dt);
        Self::apply_spring_forces_to_particles(&mut self.particles, &self.bending_springs, dt);

        // 更新位置
        for particle in &mut self.particles {
            if !particle.fixed {
                particle.position += particle.velocity * dt;
                particle.velocity *= self.air_damping;
            }
        }
    }

    /// 使用Verlet积分更新（更稳定）
    fn update_verlet(&mut self, dt: f32) {
        let dt_sq = dt * dt;

        // 确保previous_positions已初始化
        if self.previous_positions.len() != self.particles.len() {
            self.previous_positions = self.particles.iter().map(|p| p.position).collect();
        }

        for (i, particle) in self.particles.iter_mut().enumerate() {
            if particle.fixed {
                self.previous_positions[i] = particle.position;
                continue;
            }

            // 计算加速度（主要是重力，约束通过投影解决）
            let acceleration = self.gravity;

            // Verlet积分: x(t+dt) = 2*x(t) - x(t-dt) + a*dt^2
            let temp = particle.position;
            particle.position =
                particle.position * 2.0 - self.previous_positions[i] + acceleration * dt_sq;
            self.previous_positions[i] = temp;

            // 更新速度（用于阻尼和显示）
            particle.velocity = (particle.position - self.previous_positions[i]) / dt;
        }

        // 应用弹簧约束（约束投影方法）
        for _ in 0..self.config.constraint_iterations {
            self.project_constraints();
        }

        // 应用阻尼
        for particle in &mut self.particles {
            if !particle.fixed {
                particle.velocity *= self.air_damping;
            }
        }
    }

    /// 约束投影（用于Verlet积分）
    fn project_constraints(&mut self) {
        // 收集所有修正值
        let mut corrections: Vec<(usize, Vec3A)> = Vec::new();

        // 投影结构弹簧约束
        for spring in &self.structural_springs {
            if let Some((idx_a, correction_a, idx_b, correction_b)) =
                self.calculate_spring_constraint_correction(spring)
            {
                corrections.push((idx_a, correction_a));
                corrections.push((idx_b, correction_b));
            }
        }

        // 投影剪切弹簧约束
        for spring in &self.shear_springs {
            if let Some((idx_a, correction_a, idx_b, correction_b)) =
                self.calculate_spring_constraint_correction(spring)
            {
                corrections.push((idx_a, correction_a));
                corrections.push((idx_b, correction_b));
            }
        }

        // 投影弯曲弹簧约束
        for spring in &self.bending_springs {
            if let Some((idx_a, correction_a, idx_b, correction_b)) =
                self.calculate_spring_constraint_correction(spring)
            {
                corrections.push((idx_a, correction_a));
                corrections.push((idx_b, correction_b));
            }
        }

        // 应用所有修正值
        for (idx, correction) in corrections {
            if idx < self.particles.len() {
                self.particles[idx].position += correction;
            }
        }
    }

    /// 计算弹簧约束的修正值
    fn calculate_spring_constraint_correction(
        &self,
        spring: &Spring,
    ) -> Option<(usize, Vec3A, usize, Vec3A)> {
        let (idx_a, idx_b) = spring.particles;
        if idx_a >= self.particles.len() || idx_b >= self.particles.len() {
            return None;
        }

        let particle_a = &self.particles[idx_a];
        let particle_b = &self.particles[idx_b];

        if particle_a.fixed && particle_b.fixed {
            return None;
        }

        let delta = particle_b.position - particle_a.position;
        let length = delta.length();

        if length < 0.0001 {
            return None;
        }

        let diff = (length - spring.rest_length) / length;
        let correction = delta * diff * 0.5;

        if !particle_a.fixed && !particle_b.fixed {
            // 两个粒子都可移动，按质量分配
            let total_mass = particle_a.mass + particle_b.mass;
            let ratio_a = particle_b.mass / total_mass;
            let ratio_b = particle_a.mass / total_mass;

            Some((idx_a, correction * ratio_a, idx_b, -correction * ratio_b))
        } else if !particle_a.fixed {
            Some((idx_a, correction, idx_b, Vec3A::ZERO))
        } else if !particle_b.fixed {
            Some((idx_a, Vec3A::ZERO, idx_b, -correction))
        } else {
            None
        }
    }

    /// 解决自碰撞
    fn resolve_self_collisions(&mut self) {
        let radius_sq = self.config.self_collision_radius * self.config.self_collision_radius;

        for i in 0..self.particles.len() {
            if self.particles[i].fixed {
                continue;
            }

            for j in (i + 1)..self.particles.len() {
                if self.particles[j].fixed {
                    continue;
                }

                let delta = self.particles[j].position - self.particles[i].position;
                let distance_sq = delta.length_squared();

                if distance_sq < radius_sq && distance_sq > 0.0001 {
                    let distance = distance_sq.sqrt();
                    let direction = delta / distance;
                    let overlap = self.config.self_collision_radius - distance;

                    // 分离粒子
                    let correction = direction * overlap * 0.5;
                    self.particles[i].position -= correction;
                    self.particles[j].position += correction;
                }
            }
        }
    }

    /// 检测与球体的碰撞
    pub fn collide_with_sphere(&mut self, center: Vec3A, radius: f32) {
        let radius_sq = radius * radius;

        for particle in &mut self.particles {
            if particle.fixed {
                continue;
            }

            let delta = particle.position - center;
            let distance_sq = delta.length_squared();

            if distance_sq < radius_sq {
                let distance = distance_sq.sqrt();
                let direction = if distance > 0.0001 {
                    delta / distance
                } else {
                    Vec3A::Y // 默认向上
                };

                // 将粒子推到球体表面
                particle.position = center + direction * radius;

                // 更新速度（反弹）
                let normal_velocity = particle.velocity.dot(direction);
                if normal_velocity < 0.0 {
                    particle.velocity -= direction * normal_velocity * 2.0;
                }
            }
        }
    }

    /// 检测与平面的碰撞
    pub fn collide_with_plane(&mut self, point: Vec3A, normal: Vec3A) {
        let normal = normal.normalize();

        for particle in &mut self.particles {
            if particle.fixed {
                continue;
            }

            let delta = particle.position - point;
            let distance = delta.dot(normal);

            if distance < 0.0 {
                // 粒子在平面下方，推回
                particle.position -= normal * distance;

                // 更新速度（反弹）
                let normal_velocity = particle.velocity.dot(normal);
                if normal_velocity < 0.0 {
                    particle.velocity -= normal * normal_velocity * 2.0;
                }
            }
        }
    }

    fn apply_spring_forces_to_particles(particles: &mut [Particle], springs: &[Spring], dt: f32) {
        // 先收集所有需要应用的力
        let mut forces: Vec<Vec3A> = vec![Vec3A::ZERO; particles.len()];

        for spring in springs {
            let (idx_a, idx_b) = spring.particles;
            if idx_a >= particles.len() || idx_b >= particles.len() {
                continue;
            }

            let particle_a = &particles[idx_a];
            let particle_b = &particles[idx_b];

            let delta = particle_b.position - particle_a.position;
            let length = delta.length();
            let direction = if length > 0.0 {
                delta / length
            } else {
                Vec3A::ZERO
            };

            let displacement = length - spring.rest_length;
            let force = direction * displacement * spring.stiffness;

            // 收集力
            if !particle_a.fixed {
                forces[idx_a] += force / particle_a.mass;
            }

            if !particle_b.fixed {
                forces[idx_b] -= force / particle_b.mass;
            }

            // 应用阻尼
            let relative_velocity = particle_b.velocity - particle_a.velocity;
            let damping_force = direction * relative_velocity.dot(direction) * spring.damping;

            if !particle_a.fixed {
                forces[idx_a] += damping_force / particle_a.mass;
            }

            if !particle_b.fixed {
                forces[idx_b] -= damping_force / particle_b.mass;
            }
        }

        // 一次性应用所有力
        for (idx, force) in forces.iter().enumerate() {
            if !particles[idx].fixed {
                particles[idx].velocity += *force * dt;
            }
        }
    }
}

/// SPH流体参数
#[derive(Debug, Clone)]
pub struct SphParameters {
    /// 粒子半径
    pub particle_radius: f32,
    /// 平滑半径
    pub smoothing_radius: f32,
    /// 静止密度
    pub rest_density: f32,
    /// 气体常数（用于压力计算）
    pub gas_constant: f32,
    /// 粘性系数
    pub viscosity: f32,
    /// 表面张力系数
    pub surface_tension: f32,
    /// 重力
    pub gravity: Vec3A,
}

impl Default for SphParameters {
    fn default() -> Self {
        Self {
            particle_radius: 0.1,
            smoothing_radius: 0.2,
            rest_density: 1000.0,
            gas_constant: 2000.0,
            viscosity: 0.018,
            surface_tension: 0.0728,
            gravity: Vec3A::new(0.0, -9.81, 0.0),
        }
    }
}

/// 流体软体
#[derive(Debug, Clone)]
pub struct FluidSoftBody {
    /// 粒子
    pub particles: Vec<Particle>,
    /// SPH参数
    pub parameters: SphParameters,
    /// 空间分区（用于加速邻居查找）
    pub spatial_hash: HashMap<(i32, i32, i32), Vec<usize>>,
    /// 空间分区单元格大小
    pub cell_size: f32,
}

impl FluidSoftBody {
    /// 创建流体
    pub fn new(particle_count: usize, parameters: SphParameters) -> Self {
        let mut particles = Vec::new();
        let mass = parameters.rest_density
            * (4.0 / 3.0)
            * std::f32::consts::PI
            * parameters.particle_radius.powi(3);

        // 创建初始粒子分布（简单立方体）
        let side = (particle_count as f32).cbrt().ceil() as usize;
        let spacing = parameters.particle_radius * 2.1;

        for z in 0..side {
            for y in 0..side {
                for x in 0..side {
                    if particles.len() >= particle_count {
                        break;
                    }
                    let position =
                        Vec3A::new(x as f32 * spacing, y as f32 * spacing, z as f32 * spacing);
                    particles.push(Particle::new(position, mass));
                }
                if particles.len() >= particle_count {
                    break;
                }
            }
            if particles.len() >= particle_count {
                break;
            }
        }

        let cell_size = parameters.smoothing_radius * 2.0;

        Self {
            particles,
            parameters,
            spatial_hash: HashMap::new(),
            cell_size,
        }
    }

    /// 更新流体物理
    pub fn update(&mut self, dt: f32) {
        // 更新空间分区
        self.update_spatial_hash();

        // 计算密度和压力
        self.compute_density_and_pressure();

        // 计算力
        self.compute_forces(dt);

        // 更新位置和速度
        for particle in &mut self.particles {
            particle.position += particle.velocity * dt;
        }
    }

    fn update_spatial_hash(&mut self) {
        self.spatial_hash.clear();

        for (idx, particle) in self.particles.iter().enumerate() {
            let cell = (
                (particle.position.x / self.cell_size).floor() as i32,
                (particle.position.y / self.cell_size).floor() as i32,
                (particle.position.z / self.cell_size).floor() as i32,
            );
            self.spatial_hash.entry(cell).or_default().push(idx);
        }
    }

    fn get_neighbors(&self, particle_idx: usize) -> Vec<usize> {
        let particle = &self.particles[particle_idx];
        let cell = (
            (particle.position.x / self.cell_size).floor() as i32,
            (particle.position.y / self.cell_size).floor() as i32,
            (particle.position.z / self.cell_size).floor() as i32,
        );

        let mut neighbors = Vec::new();

        // 检查相邻单元格
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    let neighbor_cell = (cell.0 + dx, cell.1 + dy, cell.2 + dz);
                    if let Some(indices) = self.spatial_hash.get(&neighbor_cell) {
                        for &idx in indices {
                            if idx != particle_idx {
                                let distance =
                                    (self.particles[idx].position - particle.position).length();
                                if distance < self.parameters.smoothing_radius {
                                    neighbors.push(idx);
                                }
                            }
                        }
                    }
                }
            }
        }

        neighbors
    }

    fn compute_density_and_pressure(&mut self) {
        for i in 0..self.particles.len() {
            let mut density = 0.0;
            let neighbors = self.get_neighbors(i);

            for &j in &neighbors {
                let distance = (self.particles[j].position - self.particles[i].position).length();
                density += self.sph_kernel(distance, self.parameters.smoothing_radius)
                    * self.particles[j].mass;
            }

            self.particles[i].density = density;
            // 计算压力（使用状态方程）
            self.particles[i].pressure =
                self.parameters.gas_constant * (density - self.parameters.rest_density);
        }
    }

    fn compute_forces(&mut self, dt: f32) {
        for i in 0..self.particles.len() {
            let mut pressure_force = Vec3A::ZERO;
            let mut viscosity_force = Vec3A::ZERO;
            let neighbors = self.get_neighbors(i);

            for &j in &neighbors {
                let delta = self.particles[j].position - self.particles[i].position;
                let distance = delta.length();

                if distance > 0.0 {
                    let direction = delta / distance;

                    // 压力力
                    let pressure_gradient =
                        self.sph_kernel_gradient(distance, self.parameters.smoothing_radius);
                    let pressure_contribution = -direction
                        * (self.particles[i].pressure + self.particles[j].pressure)
                        / (2.0 * self.particles[j].density)
                        * pressure_gradient
                        * self.particles[j].mass;
                    pressure_force += pressure_contribution;

                    // 粘性力
                    let viscosity_contribution = (self.particles[j].velocity
                        - self.particles[i].velocity)
                        * self.parameters.viscosity
                        / self.particles[j].density
                        * self.sph_kernel_laplacian(distance, self.parameters.smoothing_radius)
                        * self.particles[j].mass;
                    viscosity_force += viscosity_contribution;
                }
            }

            // 应用力
            let density = self.particles[i].density;
            let total_force = pressure_force + viscosity_force + self.parameters.gravity;
            self.particles[i].velocity += total_force / density * dt;
        }
    }

    /// SPH核函数（三次样条）
    fn sph_kernel(&self, distance: f32, radius: f32) -> f32 {
        let q = distance / radius;
        if q >= 1.0 {
            return 0.0;
        }

        let sigma = 8.0 / (std::f32::consts::PI * radius.powi(3));
        if q < 0.5 {
            sigma * (6.0 * q.powi(3) - 6.0 * q.powi(2) + 1.0)
        } else {
            sigma * 2.0 * (1.0 - q).powi(3)
        }
    }

    /// SPH核函数梯度
    fn sph_kernel_gradient(&self, distance: f32, radius: f32) -> f32 {
        let q = distance / radius;
        if q >= 1.0 {
            return 0.0;
        }

        let sigma = 8.0 / (std::f32::consts::PI * radius.powi(3));
        if q < 0.5 {
            sigma * (18.0 * q.powi(2) - 12.0 * q) / radius
        } else {
            -sigma * 6.0 * (1.0 - q).powi(2) / radius
        }
    }

    /// SPH核函数拉普拉斯
    fn sph_kernel_laplacian(&self, distance: f32, radius: f32) -> f32 {
        let q = distance / radius;
        if q >= 1.0 {
            return 0.0;
        }

        let sigma = 8.0 / (std::f32::consts::PI * radius.powi(3));
        if q < 0.5 {
            sigma * (36.0 * q - 12.0) / radius.powi(2)
        } else {
            sigma * 12.0 * (1.0 - q) / radius.powi(2)
        }
    }
}

/// 软体物理组件
#[derive(Component, Debug, Clone)]
pub struct SoftBodyComponent {
    /// 软体类型
    pub body_type: SoftBodyType,
    /// 布料（如果类型是Cloth）
    pub cloth: Option<ClothSoftBody>,
    /// 流体（如果类型是Fluid）
    pub fluid: Option<FluidSoftBody>,
    /// 是否启用
    pub enabled: bool,
}

impl SoftBodyComponent {
    pub fn new_cloth(cloth: ClothSoftBody) -> Self {
        Self {
            body_type: SoftBodyType::Cloth,
            cloth: Some(cloth),
            fluid: None,
            enabled: true,
        }
    }

    pub fn new_fluid(fluid: FluidSoftBody) -> Self {
        Self {
            body_type: SoftBodyType::Fluid,
            cloth: None,
            fluid: Some(fluid),
            enabled: true,
        }
    }
}

/// 软体物理系统资源
#[derive(Resource, Default)]
pub struct SoftBodyPhysicsWorld {
    /// 时间步长
    pub time_step: f32,
    /// 子步数（用于稳定性）
    pub substeps: u32,
}

impl SoftBodyPhysicsWorld {
    pub fn new() -> Self {
        Self {
            time_step: 1.0 / 60.0,
            substeps: 1,
        }
    }

    pub fn with_substeps(time_step: f32, substeps: u32) -> Self {
        Self {
            time_step,
            substeps,
        }
    }
}

/// 软体物理更新系统
pub fn soft_body_physics_system(
    mut query: Query<&mut SoftBodyComponent>,
    world: Res<SoftBodyPhysicsWorld>,
) {
    let dt = world.time_step / world.substeps as f32;

    for _ in 0..world.substeps {
        for mut soft_body in query.iter_mut() {
            if !soft_body.enabled {
                continue;
            }

            match soft_body.body_type {
                SoftBodyType::Cloth => {
                    if let Some(ref mut cloth) = soft_body.cloth {
                        cloth.update(dt);
                    }
                }
                SoftBodyType::Fluid => {
                    if let Some(ref mut fluid) = soft_body.fluid {
                        fluid.update(dt);
                    }
                }
                SoftBodyType::SoftBody => {
                    // 通用软体（待实现）
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloth_creation() {
        let cloth = ClothSoftBody::new_rectangular(10, 10, 0.1, 0.1);
        assert_eq!(cloth.particles.len(), 100);
        assert!(!cloth.structural_springs.is_empty());
    }

    #[test]
    fn test_cloth_with_config() {
        let mut config = ClothConfig::default();
        config.structural_stiffness = 2000.0;
        config.use_verlet = true;

        let cloth = ClothSoftBody::new_rectangular_with_config(10, 10, 0.1, 0.1, config);
        assert_eq!(cloth.particles.len(), 100);
        assert_eq!(cloth.config.structural_stiffness, 2000.0);
    }

    #[test]
    fn test_cloth_verlet_integration() {
        let mut config = ClothConfig::default();
        config.use_verlet = true;
        let mut cloth = ClothSoftBody::new_rectangular_with_config(5, 5, 0.1, 0.1, config);

        // 记录初始位置
        let initial_pos = cloth.particles[10].position;

        // 更新几次
        for _ in 0..10 {
            cloth.update(0.016);
        }

        // 粒子应该移动了（由于重力）
        assert_ne!(cloth.particles[10].position, initial_pos);
    }

    #[test]
    fn test_cloth_sphere_collision() {
        let mut cloth = ClothSoftBody::new_rectangular(5, 5, 0.1, 0.1);

        // 将粒子放在球体内
        cloth.particles[10].position = Vec3A::new(0.0, 0.0, 0.0);

        // 碰撞检测
        cloth.collide_with_sphere(Vec3A::ZERO, 0.5);

        // 粒子应该在球体表面
        let distance = cloth.particles[10].position.length();
        assert!((distance - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_cloth_plane_collision() {
        let mut cloth = ClothSoftBody::new_rectangular(5, 5, 0.1, 0.1);

        // 将粒子放在平面下方
        cloth.particles[10].position = Vec3A::new(0.0, -1.0, 0.0);

        // 碰撞检测（平面在y=0）
        cloth.collide_with_plane(Vec3A::ZERO, Vec3A::Y);

        // 粒子应该在平面上方
        assert!(cloth.particles[10].position.y >= 0.0);
    }

    #[test]
    fn test_fluid_creation() {
        let params = SphParameters::default();
        let fluid = FluidSoftBody::new(100, params);
        assert_eq!(fluid.particles.len(), 100);
    }
}
