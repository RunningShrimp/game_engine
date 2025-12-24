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
}

impl ClothSoftBody {
    /// 创建矩形布料
    pub fn new_rectangular(
        width: usize,
        height: usize,
        spacing: f32,
        mass: f32,
    ) -> Self {
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

        Self {
            particles,
            structural_springs,
            shear_springs,
            bending_springs,
            width,
            height,
            spacing,
            gravity: Vec3A::new(0.0, -9.81, 0.0),
            air_damping: 0.99,
        }
    }

    /// 更新布料物理
    pub fn update(&mut self, dt: f32) {
        // 应用重力
        for particle in &mut self.particles {
            if !particle.fixed {
                particle.velocity += self.gravity * dt;
            }
        }

        // 应用弹簧力
        Self::apply_spring_forces_to_particles(
            &mut self.particles,
            &self.structural_springs,
            dt,
        );
        Self::apply_spring_forces_to_particles(
            &mut self.particles,
            &self.shear_springs,
            dt,
        );
        Self::apply_spring_forces_to_particles(
            &mut self.particles,
            &self.bending_springs,
            dt,
        );

        // 更新位置
        for particle in &mut self.particles {
            if !particle.fixed {
                particle.position += particle.velocity * dt;
                particle.velocity *= self.air_damping;
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
        let mass = parameters.rest_density * (4.0 / 3.0) * std::f32::consts::PI
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
                    let position = Vec3A::new(
                        x as f32 * spacing,
                        y as f32 * spacing,
                        z as f32 * spacing,
                    );
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
            self.spatial_hash.entry(cell).or_insert_with(Vec::new).push(idx);
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
                                let distance = (self.particles[idx].position
                                    - particle.position)
                                    .length();
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
            self.particles[i].pressure = self.parameters.gas_constant
                * (density - self.parameters.rest_density);
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
                    let pressure_gradient = self.sph_kernel_gradient(
                        distance,
                        self.parameters.smoothing_radius,
                    );
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
        Self { time_step, substeps }
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
    fn test_fluid_creation() {
        let params = SphParameters::default();
        let fluid = FluidSoftBody::new(100, params);
        assert_eq!(fluid.particles.len(), 100);
    }
}

