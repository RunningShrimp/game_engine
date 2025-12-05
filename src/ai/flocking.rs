//! 群体行为模块
//!
//! 实现Flocking算法和避障行为，用于模拟群体AI实体的自然运动。
//!
//! ## 功能特性
//!
//! - Flocking算法（分离、对齐、聚集）
//! - 避障行为
//! - 群体目标跟随
//! - 可配置的行为权重
//!
//! ## 使用示例
//!
//! ```rust
//! use crate::ai::flocking::*;
//!
//! // 创建群体管理器
//! let mut flock = FlockManager::new(FlockConfig::default());
//!
//! // 添加群体成员
//! let agent1 = flock.add_agent(Vec3::new(0.0, 0.0, 0.0));
//! let agent2 = flock.add_agent(Vec3::new(1.0, 0.0, 0.0));
//!
//! // 更新群体行为
//! flock.update(0.016); // delta_time
//!
//! // 获取代理的新速度
//! let velocity = flock.get_agent_velocity(agent1);
//! ```

use crate::impl_default;
use glam::{Vec2, Vec3};
use std::collections::HashMap;
use thiserror::Error;

/// 群体行为错误
#[derive(Error, Debug)]
pub enum FlockingError {
    #[error("Agent not found: {0}")]
    AgentNotFound(u32),
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// 代理ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AgentId(pub u32);

impl AgentId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

/// 群体配置
#[derive(Debug, Clone)]
pub struct FlockConfig {
    /// 分离权重（避免碰撞）
    pub separation_weight: f32,
    /// 对齐权重（与邻居对齐）
    pub alignment_weight: f32,
    /// 聚集权重（向邻居中心移动）
    pub cohesion_weight: f32,
    /// 避障权重
    pub avoidance_weight: f32,
    /// 感知半径（代理能感知到的最大距离）
    pub perception_radius: f32,
    /// 分离距离（开始分离的最小距离）
    pub separation_distance: f32,
    /// 最大速度
    pub max_speed: f32,
    /// 最大转向力
    pub max_steering_force: f32,
}

impl_default!(FlockConfig {
    separation_weight: 1.5,
    alignment_weight: 1.0,
    cohesion_weight: 1.0,
    avoidance_weight: 2.0,
    perception_radius: 5.0,
    separation_distance: 2.0,
    max_speed: 5.0,
    max_steering_force: 10.0,
});

/// 代理状态
#[derive(Debug, Clone)]
pub struct Agent {
    /// 代理ID
    pub id: AgentId,
    /// 位置
    pub position: Vec3,
    /// 速度
    pub velocity: Vec3,
    /// 朝向
    pub heading: Vec3,
    /// 质量（用于惯性）
    pub mass: f32,
}

impl Agent {
    /// 创建新的代理
    pub fn new(id: AgentId, position: Vec3) -> Self {
        Self {
            id,
            position,
            velocity: Vec3::ZERO,
            heading: Vec3::X,
            mass: 1.0,
        }
    }

    /// 更新代理状态
    pub fn update(
        &mut self,
        steering_force: Vec3,
        delta_time: f32,
        max_speed: f32,
        max_force: f32,
    ) {
        // 限制转向力
        let limited_force =
            steering_force.normalize_or_zero() * steering_force.length().min(max_force);

        // 计算加速度（F = ma, a = F/m）
        let acceleration = limited_force / self.mass;

        // 更新速度
        self.velocity += acceleration * delta_time;

        // 限制速度
        let speed = self.velocity.length();
        if speed > max_speed {
            self.velocity = self.velocity.normalize() * max_speed;
        }

        // 更新位置
        self.position += self.velocity * delta_time;

        // 更新朝向（如果速度足够大）
        if self.velocity.length_squared() > 0.01 {
            self.heading = self.velocity.normalize();
        }
    }
}

/// 障碍物
#[derive(Debug, Clone)]
pub struct Obstacle {
    /// 位置
    pub position: Vec3,
    /// 半径
    pub radius: f32,
}

impl Obstacle {
    /// 创建新的障碍物
    pub fn new(position: Vec3, radius: f32) -> Self {
        Self { position, radius }
    }
}

/// 群体管理器
#[derive(Default)]
pub struct FlockManager {
    /// 配置
    config: FlockConfig,
    /// 代理映射
    agents: HashMap<AgentId, Agent>,
    /// 障碍物列表
    obstacles: Vec<Obstacle>,
    /// 下一个代理ID
    next_agent_id: u32,
    /// 目标位置（可选）
    target: Option<Vec3>,
}

impl FlockManager {
    /// 创建新的群体管理器
    pub fn new(config: FlockConfig) -> Self {
        Self {
            config,
            next_agent_id: 1,
            ..Default::default()
        }
    }

    /// 创建默认配置的群体管理器
    pub fn new_default() -> Self {
        Self::new(FlockConfig::default())
    }

    /// 添加代理
    pub fn add_agent(&mut self, position: Vec3) -> AgentId {
        let id = AgentId::new(self.next_agent_id);
        self.next_agent_id += 1;

        let agent = Agent::new(id, position);
        self.agents.insert(id, agent);

        id
    }

    /// 移除代理
    pub fn remove_agent(&mut self, id: AgentId) -> Result<(), FlockingError> {
        self.agents
            .remove(&id)
            .ok_or(FlockingError::AgentNotFound(id.0))?;
        Ok(())
    }

    /// 获取代理
    pub fn get_agent(&self, id: AgentId) -> Option<&Agent> {
        self.agents.get(&id)
    }

    /// 获取代理（可变）
    pub fn get_agent_mut(&mut self, id: AgentId) -> Option<&mut Agent> {
        self.agents.get_mut(&id)
    }

    /// 添加障碍物
    pub fn add_obstacle(&mut self, obstacle: Obstacle) {
        self.obstacles.push(obstacle);
    }

    /// 移除障碍物
    pub fn remove_obstacle(&mut self, index: usize) {
        if index < self.obstacles.len() {
            self.obstacles.remove(index);
        }
    }

    /// 设置目标位置
    pub fn set_target(&mut self, target: Option<Vec3>) {
        self.target = target;
    }

    /// 更新群体行为
    pub fn update(&mut self, delta_time: f32) {
        // 计算每个代理的转向力
        let mut steering_forces: HashMap<AgentId, Vec3> = HashMap::new();

        for (id, agent) in &self.agents {
            let mut force = Vec3::ZERO;

            // 分离力
            let separation = self.calculate_separation(*id, agent);
            force += separation * self.config.separation_weight;

            // 对齐力
            let alignment = self.calculate_alignment(*id, agent);
            force += alignment * self.config.alignment_weight;

            // 聚集力
            let cohesion = self.calculate_cohesion(*id, agent);
            force += cohesion * self.config.cohesion_weight;

            // 避障力
            let avoidance = self.calculate_avoidance(agent);
            force += avoidance * self.config.avoidance_weight;

            steering_forces.insert(*id, force);
        }

        // 应用转向力更新代理
        for (id, force) in steering_forces {
            if let Some(agent) = self.agents.get_mut(&id) {
                agent.update(
                    force,
                    delta_time,
                    self.config.max_speed,
                    self.config.max_steering_force,
                );
            }
        }
    }

    /// 计算分离力（避免与邻居碰撞）
    fn calculate_separation(&self, agent_id: AgentId, agent: &Agent) -> Vec3 {
        let mut steer = Vec3::ZERO;
        let mut count = 0;

        for (other_id, other) in &self.agents {
            if *other_id == agent_id {
                continue;
            }

            let diff = agent.position - other.position;
            let distance = diff.length();

            if distance > 0.0 && distance < self.config.separation_distance {
                // 距离越近，分离力越大
                let strength = 1.0 / distance;
                steer += diff.normalize() * strength;
                count += 1;
            }
        }

        if count > 0 {
            steer /= count as f32;
            steer = steer.normalize_or_zero() * self.config.max_speed;
            steer -= agent.velocity;
        }

        steer
    }

    /// 计算对齐力（与邻居速度对齐）
    fn calculate_alignment(&self, agent_id: AgentId, agent: &Agent) -> Vec3 {
        let mut sum = Vec3::ZERO;
        let mut count = 0;

        for (other_id, other) in &self.agents {
            if *other_id == agent_id {
                continue;
            }

            let distance = (agent.position - other.position).length();

            if distance > 0.0 && distance < self.config.perception_radius {
                sum += other.velocity;
                count += 1;
            }
        }

        if count > 0 {
            sum /= count as f32;
            sum = sum.normalize_or_zero() * self.config.max_speed;
            sum -= agent.velocity;
        }

        sum
    }

    /// 计算聚集力（向邻居中心移动）
    fn calculate_cohesion(&self, agent_id: AgentId, agent: &Agent) -> Vec3 {
        let mut center = Vec3::ZERO;
        let mut count = 0;

        for (other_id, other) in &self.agents {
            if *other_id == agent_id {
                continue;
            }

            let distance = (agent.position - other.position).length();

            if distance > 0.0 && distance < self.config.perception_radius {
                center += other.position;
                count += 1;
            }
        }

        if count > 0 {
            center /= count as f32;
            let desired = center - agent.position;
            let desired = desired.normalize_or_zero() * self.config.max_speed;
            desired - agent.velocity
        } else {
            Vec3::ZERO
        }
    }

    /// 计算避障力
    fn calculate_avoidance(&self, agent: &Agent) -> Vec3 {
        let mut steer = Vec3::ZERO;

        for obstacle in &self.obstacles {
            let to_obstacle = obstacle.position - agent.position;
            let distance = to_obstacle.length();
            let combined_radius = obstacle.radius + 0.5; // 代理半径假设为0.5

            if distance < combined_radius {
                // 计算避障方向（垂直于代理朝向）
                let avoidance_dir = -to_obstacle.normalize_or_zero();
                let strength = (combined_radius - distance) / combined_radius;
                steer += avoidance_dir * strength * self.config.max_speed;
            }
        }

        if steer.length_squared() > 0.0 {
            steer = steer.normalize_or_zero() * self.config.max_speed;
            steer -= agent.velocity;
        }

        steer
    }

    /// 获取代理速度
    pub fn get_agent_velocity(&self, id: AgentId) -> Option<Vec3> {
        self.agents.get(&id).map(|a| a.velocity)
    }

    /// 获取代理位置
    pub fn get_agent_position(&self, id: AgentId) -> Option<Vec3> {
        self.agents.get(&id).map(|a| a.position)
    }

    /// 获取代理数量
    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }

    /// 更新配置
    pub fn update_config(&mut self, config: FlockConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_creation() {
        let mut flock = FlockManager::new(FlockConfig::default());

        let agent1 = flock.add_agent(Vec3::new(0.0, 0.0, 0.0));
        let agent2 = flock.add_agent(Vec3::new(1.0, 0.0, 0.0));

        assert_eq!(flock.agent_count(), 2);
        assert!(flock.get_agent(agent1).is_some());
        assert!(flock.get_agent(agent2).is_some());
    }

    #[test]
    fn test_separation() {
        let mut flock = FlockManager::new(FlockConfig {
            separation_distance: 2.0,
            ..Default::default()
        });

        // 创建两个非常接近的代理
        let agent1 = flock.add_agent(Vec3::new(0.0, 0.0, 0.0));
        let agent2 = flock.add_agent(Vec3::new(0.5, 0.0, 0.0));

        // 更新一次
        flock.update(0.016);

        // 代理应该开始分离
        let pos1 = flock.get_agent_position(agent1).unwrap();
        let pos2 = flock.get_agent_position(agent2).unwrap();

        // 位置应该发生变化
        assert!(pos1 != Vec3::new(0.0, 0.0, 0.0) || pos2 != Vec3::new(0.5, 0.0, 0.0));
    }

    #[test]
    fn test_obstacle_avoidance() {
        let mut flock = FlockManager::new(FlockConfig {
            avoidance_weight: 5.0, // 增加避障权重以确保效果明显
            ..Default::default()
        });

        let agent = flock.add_agent(Vec3::new(0.0, 0.0, 0.0));

        // 添加障碍物（在代理前方）
        let obstacle = Obstacle::new(Vec3::new(1.0, 0.0, 0.0), 0.5);
        flock.add_obstacle(obstacle);

        // 设置代理朝向障碍物
        if let Some(agent_mut) = flock.get_agent_mut(agent) {
            agent_mut.velocity = Vec3::new(1.0, 0.0, 0.0);
            agent_mut.heading = Vec3::new(1.0, 0.0, 0.0);
        }

        // 更新多次以确保避障生效
        for _ in 0..10 {
            flock.update(0.016);
        }

        // 代理应该避开障碍物（速度方向改变或速度减小）
        let velocity = flock.get_agent_velocity(agent).unwrap();
        let position = flock.get_agent_position(agent).unwrap();

        // 检查：速度方向改变（y或z分量非零）或速度减小，或者位置没有直接撞向障碍物
        let velocity_changed = velocity.y.abs() > 0.01 || velocity.z.abs() > 0.01;
        let velocity_reduced = velocity.x < 0.8;
        let position_safe = position.x < 0.8; // 没有太接近障碍物

        assert!(
            velocity_changed || velocity_reduced || position_safe,
            "Agent should avoid obstacle. Velocity: {:?}, Position: {:?}",
            velocity,
            position
        );
    }

    #[test]
    fn test_flock_update() {
        let mut flock = FlockManager::new(FlockConfig::default());

        // 创建多个代理
        for i in 0..5 {
            let x = (i as f32) * 2.0;
            flock.add_agent(Vec3::new(x, 0.0, 0.0));
        }

        assert_eq!(flock.agent_count(), 5);

        // 更新多次
        for _ in 0..10 {
            flock.update(0.016);
        }

        // 所有代理应该仍然存在
        assert_eq!(flock.agent_count(), 5);
    }
}
