//! 增强的群体智能系统
//!
//! 提供更复杂的群体行为：
//! - 分层群体（多个子群体）
//! - 领导者跟随
//! - 路径跟随
//! - 群体目标
//! - 动态行为权重

use crate::ai::flocking::{Agent, AgentId, FlockConfig, FlockingError};
use crate::impl_default;
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 增强的群体配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedFlockConfig {
    /// 基础配置
    pub base_config: FlockConfig,
    /// 是否启用领导者跟随
    pub enable_leader_following: bool,
    /// 领导者跟随权重
    pub leader_follow_weight: f32,
    /// 是否启用路径跟随
    pub enable_path_following: bool,
    /// 路径跟随权重
    pub path_follow_weight: f32,
    /// 是否启用群体目标
    pub enable_group_goal: bool,
    /// 群体目标权重
    pub group_goal_weight: f32,
    /// 分层群体数量
    pub sub_flock_count: usize,
}

impl_default!(EnhancedFlockConfig {
    base_config: FlockConfig::default(),
    enable_leader_following: true,
    leader_follow_weight: 1.5,
    enable_path_following: false,
    path_follow_weight: 1.0,
    enable_group_goal: true,
    group_goal_weight: 1.0,
    sub_flock_count: 1,
});

/// 增强的群体管理器
pub struct EnhancedFlockManager {
    config: EnhancedFlockConfig,
    /// 代理映射
    agents: HashMap<AgentId, Agent>,
    /// 子群体映射（代理ID -> 子群体ID）
    sub_flocks: HashMap<AgentId, usize>,
    /// 领导者映射（子群体ID -> 领导者ID）
    leaders: HashMap<usize, AgentId>,
    /// 路径点列表
    path_points: Vec<Vec3>,
    /// 当前路径点索引
    current_path_index: usize,
    /// 群体目标
    group_goal: Option<Vec3>,
    /// 下一个代理ID
    next_agent_id: u32,
}

impl EnhancedFlockManager {
    /// 创建新的增强群体管理器
    pub fn new(config: EnhancedFlockConfig) -> Self {
        Self {
            config,
            agents: HashMap::new(),
            sub_flocks: HashMap::new(),
            leaders: HashMap::new(),
            path_points: Vec::new(),
            current_path_index: 0,
            group_goal: None,
            next_agent_id: 1,
        }
    }

    /// 添加代理到指定子群体
    pub fn add_agent_to_flock(&mut self, position: Vec3, sub_flock_id: usize) -> AgentId {
        let id = AgentId::new(self.next_agent_id);
        self.next_agent_id += 1;

        let agent = Agent::new(id, position);
        self.agents.insert(id, agent);
        self.sub_flocks.insert(id, sub_flock_id);

        // 如果这是子群体的第一个代理，设为领导者
        if !self.leaders.contains_key(&sub_flock_id) {
            self.leaders.insert(sub_flock_id, id);
        }

        id
    }

    /// 添加代理（默认子群体0）
    pub fn add_agent(&mut self, position: Vec3) -> AgentId {
        self.add_agent_to_flock(position, 0)
    }

    /// 设置子群体领导者
    pub fn set_leader(&mut self, sub_flock_id: usize, agent_id: AgentId) -> Result<(), FlockingError> {
        if !self.agents.contains_key(&agent_id) {
            return Err(FlockingError::AgentNotFound(agent_id.0));
        }
        if self.sub_flocks.get(&agent_id) != Some(&sub_flock_id) {
            return Err(FlockingError::InvalidConfig(
                "Agent does not belong to this sub-flock".to_string(),
            ));
        }
        self.leaders.insert(sub_flock_id, agent_id);
        Ok(())
    }

    /// 设置路径点
    pub fn set_path(&mut self, path: Vec<Vec3>) {
        self.path_points = path;
        self.current_path_index = 0;
    }

    /// 设置群体目标
    pub fn set_group_goal(&mut self, goal: Option<Vec3>) {
        self.group_goal = goal;
    }

    /// 更新群体行为
    pub fn update(&mut self, delta_time: f32) {
        let mut steering_forces: HashMap<AgentId, Vec3> = HashMap::new();

        for (id, agent) in &self.agents {
            let mut force = Vec3::ZERO;

            // 基础Flocking行为
            let separation = self.calculate_separation(*id, agent);
            force += separation * self.config.base_config.separation_weight;

            let alignment = self.calculate_alignment(*id, agent);
            force += alignment * self.config.base_config.alignment_weight;

            let cohesion = self.calculate_cohesion(*id, agent);
            force += cohesion * self.config.base_config.cohesion_weight;

            // 领导者跟随
            if self.config.enable_leader_following {
                if let Some(leader_force) = self.calculate_leader_following(*id, agent) {
                    force += leader_force * self.config.leader_follow_weight;
                }
            }

            // 路径跟随
            if self.config.enable_path_following && !self.path_points.is_empty() {
                if let Some(path_force) = self.calculate_path_following(agent) {
                    force += path_force * self.config.path_follow_weight;
                }
            }

            // 群体目标
            if self.config.enable_group_goal {
                if let Some(goal_force) = self.calculate_group_goal(agent) {
                    force += goal_force * self.config.group_goal_weight;
                }
            }

            steering_forces.insert(*id, force);
        }

        // 应用转向力
        for (id, force) in steering_forces {
            if let Some(agent) = self.agents.get_mut(&id) {
                agent.update(
                    force,
                    delta_time,
                    self.config.base_config.max_speed,
                    self.config.base_config.max_steering_force,
                );
            }
        }

        // 更新路径点
        if self.config.enable_path_following && !self.path_points.is_empty() {
            self.update_path_index();
        }
    }

    /// 计算领导者跟随力
    fn calculate_leader_following(&self, agent_id: AgentId, agent: &Agent) -> Option<Vec3> {
        let sub_flock_id = self.sub_flocks.get(&agent_id)?;
        let leader_id = self.leaders.get(sub_flock_id)?;

        // 代理不跟随自己
        if *leader_id == agent_id {
            return None;
        }

        let leader = self.agents.get(leader_id)?;
        let to_leader = leader.position - agent.position;
        let distance = to_leader.length();

        if distance > 0.0 && distance < self.config.base_config.perception_radius * 2.0 {
            let desired = to_leader.normalize_or_zero() * self.config.base_config.max_speed;
            Some(desired - agent.velocity)
        } else {
            None
        }
    }

    /// 计算路径跟随力
    fn calculate_path_following(&self, agent: &Agent) -> Option<Vec3> {
        if self.current_path_index >= self.path_points.len() {
            return None;
        }

        let target = self.path_points[self.current_path_index];
        let to_target = target - agent.position;
        let distance = to_target.length();

        if distance < 1.0 {
            // 到达当前路径点，继续下一个
            return None;
        }

        let desired = to_target.normalize_or_zero() * self.config.base_config.max_speed;
        Some(desired - agent.velocity)
    }

    /// 计算群体目标力
    fn calculate_group_goal(&self, agent: &Agent) -> Option<Vec3> {
        let goal = self.group_goal?;
        let to_goal = goal - agent.position;
        let distance = to_goal.length();

        if distance < 0.5 {
            return None; // 已到达目标
        }

        let desired = to_goal.normalize_or_zero() * self.config.base_config.max_speed;
        Some(desired - agent.velocity)
    }

    /// 更新路径点索引
    fn update_path_index(&mut self) {
        if self.path_points.is_empty() {
            return;
        }

        // 检查是否有代理接近当前路径点
        let current_target = self.path_points[self.current_path_index];
        let mut all_close = true;

        for agent in self.agents.values() {
            let distance = (agent.position - current_target).length();
            if distance > 2.0 {
                all_close = false;
                break;
            }
        }

        if all_close && self.current_path_index < self.path_points.len() - 1 {
            self.current_path_index += 1;
        }
    }

    /// 计算分离力（从基础FlockManager复制）
    fn calculate_separation(&self, agent_id: AgentId, agent: &Agent) -> Vec3 {
        let mut steer = Vec3::ZERO;
        let mut count = 0;

        let sub_flock_id = self.sub_flocks.get(&agent_id).copied();

        for (other_id, other) in &self.agents {
            if *other_id == agent_id {
                continue;
            }

            // 只考虑同子群体的代理
            if let Some(sub_id) = sub_flock_id {
                if self.sub_flocks.get(other_id) != Some(&sub_id) {
                    continue;
                }
            }

            let diff = agent.position - other.position;
            let distance = diff.length();

            if distance > 0.0 && distance < self.config.base_config.separation_distance {
                let strength = 1.0 / distance;
                steer += diff.normalize() * strength;
                count += 1;
            }
        }

        if count > 0 {
            steer /= count as f32;
            steer = steer.normalize_or_zero() * self.config.base_config.max_speed;
            steer -= agent.velocity;
        }

        steer
    }

    /// 计算对齐力（从基础FlockManager复制）
    fn calculate_alignment(&self, agent_id: AgentId, agent: &Agent) -> Vec3 {
        let mut sum = Vec3::ZERO;
        let mut count = 0;

        let sub_flock_id = self.sub_flocks.get(&agent_id).copied();

        for (other_id, other) in &self.agents {
            if *other_id == agent_id {
                continue;
            }

            if let Some(sub_id) = sub_flock_id {
                if self.sub_flocks.get(other_id) != Some(&sub_id) {
                    continue;
                }
            }

            let distance = (agent.position - other.position).length();

            if distance > 0.0 && distance < self.config.base_config.perception_radius {
                sum += other.velocity;
                count += 1;
            }
        }

        if count > 0 {
            sum /= count as f32;
            sum = sum.normalize_or_zero() * self.config.base_config.max_speed;
            sum -= agent.velocity;
        }

        sum
    }

    /// 计算聚集力（从基础FlockManager复制）
    fn calculate_cohesion(&self, agent_id: AgentId, agent: &Agent) -> Vec3 {
        let mut center = Vec3::ZERO;
        let mut count = 0;

        let sub_flock_id = self.sub_flocks.get(&agent_id).copied();

        for (other_id, other) in &self.agents {
            if *other_id == agent_id {
                continue;
            }

            if let Some(sub_id) = sub_flock_id {
                if self.sub_flocks.get(other_id) != Some(&sub_id) {
                    continue;
                }
            }

            let distance = (agent.position - other.position).length();

            if distance > 0.0 && distance < self.config.base_config.perception_radius {
                center += other.position;
                count += 1;
            }
        }

        if count > 0 {
            center /= count as f32;
            let desired = center - agent.position;
            let desired = desired.normalize_or_zero() * self.config.base_config.max_speed;
            desired - agent.velocity
        } else {
            Vec3::ZERO
        }
    }

    /// 获取代理
    pub fn get_agent(&self, id: AgentId) -> Option<&Agent> {
        self.agents.get(&id)
    }

    /// 获取代理数量
    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_flock_manager() {
        let config = EnhancedFlockConfig::default();
        let mut manager = EnhancedFlockManager::new(config);

        let agent1 = manager.add_agent(Vec3::new(0.0, 0.0, 0.0));
        let agent2 = manager.add_agent(Vec3::new(1.0, 0.0, 0.0));

        assert_eq!(manager.agent_count(), 2);
        assert!(manager.get_agent(agent1).is_some());
    }
}

