//! # NPC系统集成
//!
//! 本模块提供LLM服务与现有NPC系统的集成。
//!
//! ## 使用示例
//!
//! ```rust
//! use game_engine::ai::npc::{IntelligentNPC, HybridMode};
//!
//! let mut npc = IntelligentNPC::new(entity_id);
//! npc.set_llm_service(Some(openai_adapter));
//! npc.set_hybrid_mode(HybridMode::Hybrid);
//!
//! let action = npc.decide().await?;
//! ```

use super::service::{Action, ActionType, AIService, NPCContext, Situation, NPCStatus};
use super::BehaviorTree;
use bevy_ecs::entity::Entity;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// 混合AI模式
///
/// 定义传统AI和LLM的协作模式。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum HybridMode {
    /// 仅使用传统AI（行为树、状态机）
    TraditionalOnly,
    /// 仅使用LLM决策
    LLMOnly,
    /// 混合模式：根据复杂度选择
    Hybrid,
    /// 自适应模式：根据性能和效果动态调整
    Adaptive,
}

/// 智能NPC
///
/// 结合传统AI和LLM的NPC系统。
pub struct IntelligentNPC {
    /// NPC实体ID
    entity_id: Entity,
    /// 基础AI（行为树）
    base_ai: Option<BehaviorTree>,
    /// LLM服务
    llm_service: Option<Arc<dyn AIService>>,
    /// 混合模式
    hybrid_mode: HybridMode,
    /// 当前情境
    current_situation: Option<Situation>,
    /// 性能统计
    performance_stats: PerformanceStats,
    /// NPC配置
    config: NPCConfig,
}

/// NPC配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NPCConfig {
    /// 是否启用LLM
    pub enable_llm: bool,
    /// LLM调用延迟阈值（秒）
    pub llm_latency_threshold: f32,
    /// 复杂度阈值（超过此值使用LLM）
    pub complexity_threshold: f32,
    /// 自适应调整间隔（秒）
    pub adaptive_adjustment_interval: u32,
    /// 最小信任度（低于此值回退到传统AI）
    pub min_confidence: f32,
}

impl Default for NPCConfig {
    fn default() -> Self {
        Self {
            enable_llm: true,
            llm_latency_threshold: 2.0,
            complexity_threshold: 0.6,
            adaptive_adjustment_interval: 60,
            min_confidence: 0.5,
        }
    }
}

/// 性能统计
#[derive(Debug, Clone, Default)]
pub struct PerformanceStats {
    /// LLM调用次数
    pub llm_calls: u32,
    /// 传统AI调用次数
    pub traditional_calls: u32,
    /// 平均LLM延迟（毫秒）
    pub avg_llm_latency: f32,
    /// 平均置信度
    pub avg_confidence: f32,
    /// 失败次数
    pub failures: u32,
}

impl IntelligentNPC {
    /// 创建新的智能NPC
    ///
    /// # 参数
    ///
    /// - `entity_id`: NPC的实体ID
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::ai::npc::IntelligentNPC;
    /// use bevy_ecs::entity::Entity;
    ///
    /// let npc = IntelligentNPC::new(Entity::from_raw(1));
    /// ```
    pub fn new(entity_id: Entity) -> Self {
        Self {
            entity_id,
            base_ai: None,
            llm_service: None,
            hybrid_mode: HybridMode::Hybrid,
            current_situation: None,
            performance_stats: PerformanceStats::default(),
            config: NPCConfig::default(),
        }
    }

    /// 设置行为树
    pub fn with_behavior_tree(mut self, behavior_tree: BehaviorTree) -> Self {
        self.base_ai = Some(behavior_tree);
        self
    }

    /// 设置LLM服务
    pub fn with_llm_service(mut self, service: Arc<dyn AIService>) -> Self {
        self.llm_service = Some(service);
        self
    }

    /// 设置混合模式
    pub fn with_hybrid_mode(mut self, mode: HybridMode) -> Self {
        self.hybrid_mode = mode;
        self
    }

    /// 设置NPC配置
    pub fn with_config(mut self, config: NPCConfig) -> Self {
        self.config = config;
        self
    }

    /// 设置LLM服务
    pub fn set_llm_service(&mut self, service: Option<Arc<dyn AIService>>) {
        self.llm_service = service;
    }

    /// 设置混合模式
    pub fn set_hybrid_mode(&mut self, mode: HybridMode) {
        self.hybrid_mode = mode;
    }

    /// 更新当前情境
    pub fn update_situation(&mut self, situation: Situation) {
        self.current_situation = Some(situation);
    }

    /// 决策NPC行为
    ///
    /// 根据当前模式和情境选择合适的AI策略。
    pub async fn decide(&mut self) -> Action {
        match self.hybrid_mode {
            HybridMode::TraditionalOnly => self.traditional_decide(),
            HybridMode::LLMOnly => {
                let situation = self.current_situation.as_ref().expect("No situation set").clone();
                self.llm_decide(&situation).await
            }
            HybridMode::Hybrid => {
                let situation = self.current_situation.as_ref().expect("No situation set").clone();
                self.hybrid_decide(&situation).await
            }
            HybridMode::Adaptive => {
                let situation = self.current_situation.as_ref().expect("No situation set").clone();
                self.adaptive_decide(&situation).await
            }
        }
    }

    /// 传统AI决策
    fn traditional_decide(&mut self) -> Action {
        self.performance_stats.traditional_calls += 1;

        // 简化实现：返回默认动作
        // 实际应用中会执行行为树
        Action {
            action_type: ActionType::Wait,
            parameters: HashMap::new(),
            priority: 0.5,
            estimated_duration: None,
        }
    }

    /// LLM决策
    async fn llm_decide(&mut self, situation: &Situation) -> Action {
        if let Some(service) = &self.llm_service {
            let start = std::time::Instant::now();

            match service.decide_action(situation).await {
                Ok(action) => {
                    let latency = start.elapsed().as_secs_f32() * 1000.0;
                    self.update_llm_stats(latency, 1.0);
                    action
                }
                Err(e) => {
                    log::error!("LLM decision failed: {}", e);
                    self.performance_stats.failures += 1;
                    self.traditional_decide()
                }
            }
        } else {
            self.traditional_decide()
        }
    }

    /// 混合决策
    ///
    /// 根据复杂度选择使用传统AI或LLM。
    async fn hybrid_decide(&mut self, situation: &Situation) -> Action {
        let complexity = self.calculate_complexity(situation);

        if complexity >= self.config.complexity_threshold {
            self.llm_decide(situation).await
        } else {
            self.traditional_decide()
        }
    }

    /// 自适应决策
    ///
    /// 根据性能统计动态选择AI策略。
    async fn adaptive_decide(&mut self, situation: &Situation) -> Action {
        // 检查LLM性能
        let use_llm = if self.performance_stats.llm_calls == 0 {
            true // 还没有统计数据，尝试LLM
        } else {
            let avg_latency = self.performance_stats.avg_llm_latency;
            let avg_confidence = self.performance_stats.avg_confidence;

            avg_latency < (self.config.llm_latency_threshold * 1000.0)
                && avg_confidence >= self.config.min_confidence
        };

        if use_llm && self.llm_service.is_some() {
            self.llm_decide(situation).await
        } else {
            self.traditional_decide()
        }
    }

    /// 计算情境复杂度
    ///
    /// 返回0.0-1.0之间的值，表示当前情境的复杂度。
    fn calculate_complexity(&self, situation: &Situation) -> f32 {
        let mut complexity = 0.0f32;

        // 实体数量影响复杂度
        complexity += (situation.nearby_entities.len() as f32 * 0.1).min(0.3);

        // 威胁数量影响复杂度
        complexity += (situation.perceived_threats.len() as f32 * 0.15).min(0.3);

        // 可用动作数量影响复杂度
        complexity += (situation.available_actions.len() as f32 * 0.05).min(0.2);

        // NPC状态影响复杂度
        match situation.npc_status {
            NPCStatus::InCombat => complexity += 0.2,
            NPCStatus::Wounded => complexity += 0.1,
            _ => {}
        };

        complexity.min(1.0)
    }

    /// 更新LLM统计信息
    fn update_llm_stats(&mut self, latency_ms: f32, confidence: f32) {
        self.performance_stats.llm_calls += 1;

        // 更新平均延迟
        let total_calls = self.performance_stats.llm_calls as f32;
        self.performance_stats.avg_llm_latency =
            (self.performance_stats.avg_llm_latency * (total_calls - 1.0) + latency_ms) / total_calls;

        // 更新平均置信度
        self.performance_stats.avg_confidence =
            (self.performance_stats.avg_confidence * (total_calls - 1.0) + confidence) / total_calls;
    }

    /// 生成NPC对话
    pub async fn generate_dialogue(&self, context: &NPCContext) -> Result<String, super::service::AIError> {
        if let Some(service) = &self.llm_service {
            service.generate_dialogue(context).await
        } else {
            Ok("I have nothing to say.".to_string())
        }
    }

    /// 获取性能统计
    pub fn get_performance_stats(&self) -> &PerformanceStats {
        &self.performance_stats
    }

    /// 重置性能统计
    pub fn reset_performance_stats(&mut self) {
        self.performance_stats = PerformanceStats::default();
    }
}

/// NPC管理器
///
/// 管理多个智能NPC实例。
pub struct NPCManager {
    npcs: HashMap<Entity, IntelligentNPC>,
    default_llm_service: Option<Arc<dyn AIService>>,
}

impl NPCManager {
    /// 创建新的NPC管理器
    pub fn new() -> Self {
        Self {
            npcs: HashMap::new(),
            default_llm_service: None,
        }
    }

    /// 设置默认LLM服务
    pub fn set_default_llm_service(&mut self, service: Arc<dyn AIService>) {
        self.default_llm_service = Some(service);
    }

    /// 添加NPC
    pub fn add_npc(&mut self, npc: IntelligentNPC) {
        let entity_id = npc.entity_id;
        self.npcs.insert(entity_id, npc);
    }

    /// 获取NPC
    pub fn get_npc(&mut self, entity_id: Entity) -> Option<&mut IntelligentNPC> {
        self.npcs.get_mut(&entity_id)
    }

    /// 移除NPC
    pub fn remove_npc(&mut self, entity_id: Entity) -> Option<IntelligentNPC> {
        self.npcs.remove(&entity_id)
    }

    /// 批量更新NPC情境
    pub fn update_situations(&mut self, situations: HashMap<Entity, Situation>) {
        for (entity_id, situation) in situations {
            if let Some(npc) = self.npcs.get_mut(&entity_id) {
                npc.update_situation(situation);
            }
        }
    }

    /// 批量决策所有NPC
    pub async fn decide_all(&mut self) -> HashMap<Entity, Action> {
        let mut actions = HashMap::new();

        for (entity_id, npc) in &mut self.npcs {
            // 确保NPC有LLM服务
            if npc.llm_service.is_none() {
                if let Some(service) = &self.default_llm_service {
                    npc.llm_service = Some(Arc::clone(service));
                }
            }

            let action = npc.decide().await;
            actions.insert(*entity_id, action);
        }

        actions
    }

    /// 获取所有NPC的性能统计
    pub fn get_all_stats(&self) -> HashMap<Entity, &PerformanceStats> {
        self.npcs
            .iter()
            .map(|(id, npc)| (*id, npc.get_performance_stats()))
            .collect()
    }
}

impl Default for NPCManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::service::{ActionType, NPCContext, PlayerState, EnvironmentState};
    use std::collections::HashMap;

    fn create_test_situation() -> Situation {
        Situation {
            nearby_entities: vec![],
            current_goal: Some("Patrol".to_string()),
            available_actions: vec![Action {
                action_type: ActionType::Wait,
                parameters: HashMap::new(),
                priority: 0.5,
                estimated_duration: None,
            }],
            time_constraints: None,
            npc_status: NPCStatus::Idle,
            perceived_threats: vec![],
            resources: HashMap::new(),
        }
    }

    #[test]
    fn test_intelligent_npc_creation() {
        let entity_id = Entity::from_raw(1);
        let npc = IntelligentNPC::new(entity_id);

        assert_eq!(npc.entity_id, entity_id);
        assert_eq!(npc.hybrid_mode, HybridMode::Hybrid);
    }

    #[test]
    fn test_with_hybrid_mode() {
        let npc = IntelligentNPC::new(Entity::from_raw(1))
            .with_hybrid_mode(HybridMode::LLMOnly);

        assert_eq!(npc.hybrid_mode, HybridMode::LLMOnly);
    }

    #[test]
    fn test_calculate_complexity() {
        let npc = IntelligentNPC::new(Entity::from_raw(1));
        let situation = create_test_situation();

        let complexity = npc.calculate_complexity(&situation);
        assert!(complexity >= 0.0 && complexity <= 1.0);
    }

    #[test]
    fn test_npc_manager() {
        let mut manager = NPCManager::new();
        let npc = IntelligentNPC::new(Entity::from_raw(1));

        manager.add_npc(npc);
        assert_eq!(manager.npcs.len(), 1);

        let entity_id = Entity::from_raw(1);
        assert!(manager.get_npc(entity_id).is_some());

        manager.remove_npc(entity_id);
        assert_eq!(manager.npcs.len(), 0);
    }

    #[test]
    fn test_npc_config_default() {
        let config = NPCConfig::default();
        assert_eq!(config.enable_llm, true);
        assert_eq!(config.llm_latency_threshold, 2.0);
        assert_eq!(config.complexity_threshold, 0.6);
    }
}
