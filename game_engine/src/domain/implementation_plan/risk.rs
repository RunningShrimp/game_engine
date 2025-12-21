//  风险管理领域对象
// 
//  该模块实现了风险管理的核心业务逻辑，包括风险的识别、
//  状态跟踪和缓解措施管理。

use crate::domain::implementation_plan::errors::{ImplementationPlanError, RiskError};
use serde::{Deserialize, Serialize};

/// 风险唯一标识符
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RiskId(pub u64);

impl RiskId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for RiskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Risk({})", self.0)
    }
}

/// 风险级别枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskLevel {
    /// 低风险
    Low,
    /// 中风险
    Medium,
    /// 高风险
    High,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::Low => write!(f, "Low"),
            RiskLevel::Medium => write!(f, "Medium"),
            RiskLevel::High => write!(f, "High"),
        }
    }
}

/// 风险状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskStatus {
    /// 已识别
    Identified,
    /// 正在缓解
    Mitigating,
    /// 已缓解
    Mitigated,
    /// 已发生
    Occurred,
}

impl std::fmt::Display for RiskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskStatus::Identified => write!(f, "Identified"),
            RiskStatus::Mitigating => write!(f, "Mitigating"),
            RiskStatus::Mitigated => write!(f, "Mitigated"),
            RiskStatus::Occurred => write!(f, "Occurred"),
        }
    }
}

/// 缓解措施
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MitigationMeasure {
    /// 措施ID
    pub id: String,
    /// 措施描述
    pub description: String,
    /// 负责人
    pub owner: Option<String>,
    /// 截止日期
    pub due_date: Option<u64>,
    /// 是否已完成
    pub completed: bool,
    /// 创建时间
    pub created_at: u64,
}

impl MitigationMeasure {
    /// 创建新的缓解措施
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            description: description.into(),
            owner: None,
            due_date: None,
            completed: false,
            created_at: Self::current_timestamp(),
        }
    }

    /// 设置负责人
    pub fn with_owner(mut self, owner: impl Into<String>) -> Self {
        self.owner = Some(owner.into());
        self
    }

    /// 设置截止日期
    pub fn with_due_date(mut self, due_date: u64) -> Self {
        self.due_date = Some(due_date);
        self
    }

    /// 标记为完成
    pub fn complete(&mut self) {
        self.completed = true;
    }

    /// 检查是否过期
    pub fn is_overdue(&self) -> bool {
        if let Some(due_date) = self.due_date {
            !self.completed && Self::current_timestamp() > due_date
        } else {
            false
        }
    }

    /// 获取当前时间戳
    fn current_timestamp() -> u64 {
        crate::core::utils::current_timestamp()
    }
}

/// 风险 - 聚合根
///
/// 封装风险的所有属性和行为，确保业务规则在边界内执行。
///
/// ## 聚合边界
///
/// **包含**：
/// - `RiskId`：风险唯一标识符
/// - `name`：风险名称
/// - `description`：风险描述
/// - `level`：风险级别
/// - `status`：风险状态
/// - `impact_assessment`：影响评估（可选）
/// - `mitigation_measures`：缓解措施列表
/// - `identified_at`：识别时间戳
/// - `updated_at`：最后更新时间戳
///
/// **不包含**：
/// - 风险历史记录（基础设施层）
/// - 风险通知（基础设施层）
///
/// ## 业务规则
///
/// 1. 风险ID创建后不可变
/// 2. 已发生的风险不能修改状态
/// 3. 风险名称和描述不能为空
/// 4. 缓解措施必须有描述
///
/// ## 不变性约束
///
/// - `RiskId`：创建后不可变
/// - `identified_at`：创建后不可变
/// - `status`：只能通过聚合根方法修改（`start_mitigation`, `mitigate`, `occur`）
#[derive(Debug, Clone)]
pub struct Risk {
    /// 风险ID
    pub id: RiskId,
    /// 风险名称
    pub name: String,
    /// 风险描述
    pub description: String,
    /// 风险级别
    pub level: RiskLevel,
    /// 风险状态
    pub status: RiskStatus,
    /// 影响评估
    pub impact_assessment: Option<String>,
    /// 缓解措施列表
    pub mitigation_measures: Vec<MitigationMeasure>,
    /// 识别时间戳
    pub identified_at: u64,
    /// 最后更新时间戳
    pub updated_at: u64,
}

impl Risk {
    /// 创建新风险
    pub fn new(
        id: RiskId,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Result<Self, ImplementationPlanError> {
        let name = name.into();
        let description = description.into();

        if name.trim().is_empty() {
            return Err(ImplementationPlanError::Risk(RiskError::InvalidParameter(
                "Risk name cannot be empty".to_string(),
            )));
        }

        if description.trim().is_empty() {
            return Err(ImplementationPlanError::Risk(RiskError::InvalidParameter(
                "Risk description cannot be empty".to_string(),
            )));
        }

        let now = Self::current_timestamp();
        Ok(Self {
            id,
            name,
            description,
            level: RiskLevel::Medium,
            status: RiskStatus::Identified,
            impact_assessment: None,
            mitigation_measures: Vec::new(),
            identified_at: now,
            updated_at: now,
        })
    }

    /// 设置风险级别
    pub fn with_level(mut self, level: RiskLevel) -> Self {
        self.level = level;
        self.updated_at = Self::current_timestamp();
        self
    }

    /// 设置影响评估
    pub fn with_impact_assessment(mut self, assessment: impl Into<String>) -> Self {
        self.impact_assessment = Some(assessment.into());
        self.updated_at = Self::current_timestamp();
        self
    }

    /// 添加缓解措施
    pub fn add_mitigation_measure(
        &mut self,
        measure: MitigationMeasure,
    ) -> Result<(), ImplementationPlanError> {
        if measure.description.trim().is_empty() {
            return Err(ImplementationPlanError::Risk(RiskError::InvalidParameter(
                "Mitigation measure description cannot be empty".to_string(),
            )));
        }

        self.mitigation_measures.push(measure);
        self.updated_at = Self::current_timestamp();
        Ok(())
    }

    /// 移除缓解措施
    pub fn remove_mitigation_measure(
        &mut self,
        measure_id: &str,
    ) -> Result<(), ImplementationPlanError> {
        if let Some(pos) = self
            .mitigation_measures
            .iter()
            .position(|m| m.id == measure_id)
        {
            self.mitigation_measures.remove(pos);
            self.updated_at = Self::current_timestamp();
            Ok(())
        } else {
            Err(ImplementationPlanError::Risk(RiskError::InvalidParameter(
                format!("Mitigation measure {} not found", measure_id),
            )))
        }
    }

    /// 开始缓解
    pub fn start_mitigation(&mut self) -> Result<(), ImplementationPlanError> {
        match self.status {
            RiskStatus::Identified => {
                self.status = RiskStatus::Mitigating;
                self.updated_at = Self::current_timestamp();
                Ok(())
            }
            RiskStatus::Mitigating => Ok(()), // 已经是缓解中状态
            RiskStatus::Mitigated => Err(ImplementationPlanError::Risk(
                RiskError::InvalidStatusTransition {
                    from: RiskStatus::Mitigated.to_string(),
                    to: RiskStatus::Mitigating.to_string(),
                },
            )),
            RiskStatus::Occurred => Err(ImplementationPlanError::Risk(
                RiskError::InvalidStatusTransition {
                    from: RiskStatus::Occurred.to_string(),
                    to: RiskStatus::Mitigating.to_string(),
                },
            )),
        }
    }

    /// 完成缓解
    pub fn mitigate(&mut self) -> Result<(), ImplementationPlanError> {
        match self.status {
            RiskStatus::Identified | RiskStatus::Mitigating => {
                self.status = RiskStatus::Mitigated;
                self.updated_at = Self::current_timestamp();
                Ok(())
            }
            RiskStatus::Mitigated => Ok(()), // 已经是已缓解状态
            RiskStatus::Occurred => Err(ImplementationPlanError::Risk(
                RiskError::InvalidStatusTransition {
                    from: RiskStatus::Occurred.to_string(),
                    to: RiskStatus::Mitigated.to_string(),
                },
            )),
        }
    }

    /// 风险发生
    pub fn occur(&mut self) -> Result<(), ImplementationPlanError> {
        match self.status {
            RiskStatus::Identified | RiskStatus::Mitigating => {
                self.status = RiskStatus::Occurred;
                self.updated_at = Self::current_timestamp();
                Ok(())
            }
            RiskStatus::Mitigated => Err(ImplementationPlanError::Risk(
                RiskError::InvalidStatusTransition {
                    from: RiskStatus::Mitigated.to_string(),
                    to: RiskStatus::Occurred.to_string(),
                },
            )),
            RiskStatus::Occurred => Ok(()), // 已经是已发生状态
        }
    }

    /// 计算缓解进度（0.0 到 1.0）
    pub fn calculate_mitigation_progress(&self) -> f32 {
        if self.mitigation_measures.is_empty() {
            return if self.status == RiskStatus::Mitigated {
                1.0
            } else {
                0.0
            };
        }

        let completed_count = self
            .mitigation_measures
            .iter()
            .filter(|measure| measure.completed)
            .count();

        completed_count as f32 / self.mitigation_measures.len() as f32
    }

    /// 检查是否可以完成缓解（所有措施已完成）
    pub fn can_complete_mitigation(&self) -> bool {
        self.mitigation_measures
            .iter()
            .all(|measure| measure.completed)
    }

    /// 检查是否有过期的缓解措施
    pub fn has_overdue_measures(&self) -> bool {
        self.mitigation_measures
            .iter()
            .any(|measure| measure.is_overdue())
    }

    /// 获取当前时间戳
    fn current_timestamp() -> u64 {
        crate::core::utils::current_timestamp()
    }
}

/// 风险管理器
pub struct RiskManager {
    risks: std::collections::HashMap<RiskId, Risk>,
    next_id: u64,
}

impl RiskManager {
    /// 创建新的风险管理器
    pub fn new() -> Self {
        Self {
            risks: std::collections::HashMap::new(),
            next_id: 1,
        }
    }

    /// 创建风险
    pub fn create_risk(
        &mut self,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Result<RiskId, ImplementationPlanError> {
        let id = RiskId::new(self.next_id);
        self.next_id += 1;

        let risk = Risk::new(id, name, description)?;
        self.risks.insert(id, risk);
        Ok(id)
    }

    /// 获取风险
    pub fn get_risk(&self, id: &RiskId) -> Result<&Risk, ImplementationPlanError> {
        self.risks.get(id).ok_or_else(|| {
            ImplementationPlanError::Risk(RiskError::RiskNotFound(format!("{}", id)))
        })
    }

    /// 获取风险的可变引用
    pub fn get_risk_mut(&mut self, id: &RiskId) -> Result<&mut Risk, ImplementationPlanError> {
        self.risks.get_mut(id).ok_or_else(|| {
            ImplementationPlanError::Risk(RiskError::RiskNotFound(format!("{}", id)))
        })
    }

    /// 删除风险
    pub fn delete_risk(&mut self, id: &RiskId) -> Result<(), ImplementationPlanError> {
        if self.risks.remove(id).is_some() {
            Ok(())
        } else {
            Err(ImplementationPlanError::Risk(RiskError::RiskNotFound(
                format!("{}", id),
            )))
        }
    }

    /// 获取所有风险
    pub fn get_all_risks(&self) -> Vec<&Risk> {
        self.risks.values().collect()
    }

    /// 获取按状态过滤的风险
    pub fn get_risks_by_status(&self, status: RiskStatus) -> Vec<&Risk> {
        self.risks
            .values()
            .filter(|risk| risk.status == status)
            .collect()
    }

    /// 获取按级别过滤的风险
    pub fn get_risks_by_level(&self, level: RiskLevel) -> Vec<&Risk> {
        self.risks
            .values()
            .filter(|risk| risk.level == level)
            .collect()
    }

    /// 获取高风险项目
    pub fn get_high_priority_risks(&self) -> Vec<&Risk> {
        self.risks
            .values()
            .filter(|risk| risk.level == RiskLevel::High && risk.status != RiskStatus::Mitigated)
            .collect()
    }

    /// 获取有过期缓解措施的风险
    pub fn get_risks_with_overdue_measures(&self) -> Vec<&Risk> {
        self.risks
            .values()
            .filter(|risk| risk.has_overdue_measures())
            .collect()
    }

    /// 计算总体风险缓解进度
    pub fn calculate_overall_mitigation_progress(&self) -> f32 {
        if self.risks.is_empty() {
            return 1.0; // 没有风险，进度为100%
        }

        let total_progress: f32 = self
            .risks
            .values()
            .map(|risk| risk.calculate_mitigation_progress())
            .sum();

        total_progress / self.risks.len() as f32
    }
}

impl Default for RiskManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_creation() {
        let risk = Risk::new(RiskId(1), "Test Risk", "Test description").unwrap();
        assert_eq!(risk.id, RiskId(1));
        assert_eq!(risk.name, "Test Risk");
        assert_eq!(risk.description, "Test description");
        assert_eq!(risk.level, RiskLevel::Medium);
        assert_eq!(risk.status, RiskStatus::Identified);
    }

    #[test]
    fn test_risk_creation_empty_name() {
        let result = Risk::new(RiskId(1), "", "Test description");
        assert!(matches!(
            result,
            Err(ImplementationPlanError::Risk(RiskError::InvalidParameter(
                _
            )))
        ));
    }

    #[test]
    fn test_risk_creation_empty_description() {
        let result = Risk::new(RiskId(1), "Test Risk", "");
        assert!(matches!(
            result,
            Err(ImplementationPlanError::Risk(RiskError::InvalidParameter(
                _
            )))
        ));
    }

    #[test]
    fn test_risk_with_level() {
        let risk = Risk::new(RiskId(1), "Test Risk", "Test description")
            .unwrap()
            .with_level(RiskLevel::High);
        assert_eq!(risk.level, RiskLevel::High);
    }

    #[test]
    fn test_risk_with_impact_assessment() {
        let risk = Risk::new(RiskId(1), "Test Risk", "Test description")
            .unwrap()
            .with_impact_assessment("High impact");
        assert_eq!(risk.impact_assessment, Some("High impact".to_string()));
    }

    #[test]
    fn test_risk_add_mitigation_measure() {
        let mut risk = Risk::new(RiskId(1), "Test Risk", "Test description").unwrap();
        let measure = MitigationMeasure::new("Test measure");

        risk.add_mitigation_measure(measure).unwrap();
        assert_eq!(risk.mitigation_measures.len(), 1);
    }

    #[test]
    fn test_risk_add_empty_mitigation_measure() {
        let mut risk = Risk::new(RiskId(1), "Test Risk", "Test description").unwrap();
        let measure = MitigationMeasure::new("");

        let result = risk.add_mitigation_measure(measure);
        assert!(matches!(
            result,
            Err(ImplementationPlanError::Risk(RiskError::InvalidParameter(
                _
            )))
        ));
    }

    #[test]
    fn test_risk_start_mitigation() {
        let mut risk = Risk::new(RiskId(1), "Test Risk", "Test description").unwrap();

        risk.start_mitigation().unwrap();
        assert_eq!(risk.status, RiskStatus::Mitigating);
    }

    #[test]
    fn test_risk_mitigate() {
        let mut risk = Risk::new(RiskId(1), "Test Risk", "Test description").unwrap();

        risk.start_mitigation().unwrap();
        risk.mitigate().unwrap();
        assert_eq!(risk.status, RiskStatus::Mitigated);
    }

    #[test]
    fn test_risk_occur() {
        let mut risk = Risk::new(RiskId(1), "Test Risk", "Test description").unwrap();

        risk.occur().unwrap();
        assert_eq!(risk.status, RiskStatus::Occurred);
    }

    #[test]
    fn test_risk_invalid_status_transitions() {
        let mut risk = Risk::new(RiskId(1), "Test Risk", "Test description").unwrap();

        // 已缓解的风险不能发生
        risk.mitigate().unwrap();
        assert!(risk.occur().is_err());

        // 已发生风险不能开始缓解
        let mut risk2 = Risk::new(RiskId(2), "Test Risk 2", "Test description").unwrap();
        risk2.occur().unwrap();
        assert!(risk2.start_mitigation().is_err());
    }

    #[test]
    fn test_risk_calculate_mitigation_progress() {
        let mut risk = Risk::new(RiskId(1), "Test Risk", "Test description").unwrap();

        // 没有缓解措施
        assert_eq!(risk.calculate_mitigation_progress(), 0.0);

        // 添加缓解措施
        let mut measure1 = MitigationMeasure::new("Measure 1");
        let mut measure2 = MitigationMeasure::new("Measure 2");
        risk.add_mitigation_measure(measure1.clone()).unwrap();
        risk.add_mitigation_measure(measure2.clone()).unwrap();

        assert_eq!(risk.calculate_mitigation_progress(), 0.0);

        // 完成一个措施
        risk.mitigation_measures[0].complete();
        assert_eq!(risk.calculate_mitigation_progress(), 0.5);

        // 完成所有措施
        risk.mitigation_measures[1].complete();
        assert_eq!(risk.calculate_mitigation_progress(), 1.0);
    }

    #[test]
    fn test_risk_can_complete_mitigation() {
        let mut risk = Risk::new(RiskId(1), "Test Risk", "Test description").unwrap();

        // 没有缓解措施
        assert!(risk.can_complete_mitigation());

        // 添加缓解措施
        let measure1 = MitigationMeasure::new("Measure 1");
        let measure2 = MitigationMeasure::new("Measure 2");
        risk.add_mitigation_measure(measure1).unwrap();
        risk.add_mitigation_measure(measure2).unwrap();

        assert!(!risk.can_complete_mitigation());

        // 完成所有措施
        risk.mitigation_measures[0].complete();
        risk.mitigation_measures[1].complete();
        assert!(risk.can_complete_mitigation());
    }

    #[test]
    fn test_mitigation_measure_creation() {
        let measure = MitigationMeasure::new("Test measure");
        assert_eq!(measure.description, "Test measure");
        assert!(!measure.completed);
        assert!(measure.owner.is_none());
    }

    #[test]
    fn test_mitigation_measure_with_owner() {
        let measure = MitigationMeasure::new("Test measure").with_owner("John Doe");
        assert_eq!(measure.owner, Some("John Doe".to_string()));
    }

    #[test]
    fn test_mitigation_measure_with_due_date() {
        let measure = MitigationMeasure::new("Test measure").with_due_date(1234567890);
        assert_eq!(measure.due_date, Some(1234567890));
    }

    #[test]
    fn test_mitigation_measure_complete() {
        let mut measure = MitigationMeasure::new("Test measure");
        measure.complete();
        assert!(measure.completed);
    }

    #[test]
    fn test_risk_manager_create_risk() {
        let mut manager = RiskManager::new();

        let id = manager
            .create_risk("Test Risk", "Test description")
            .unwrap();
        let risk = manager.get_risk(&id).unwrap();
        assert_eq!(risk.name, "Test Risk");
    }

    #[test]
    fn test_risk_manager_get_risks_by_status() {
        let mut manager = RiskManager::new();

        let id1 = manager.create_risk("Risk 1", "Description 1").unwrap();
        let id2 = manager.create_risk("Risk 2", "Description 2").unwrap();

        manager
            .get_risk_mut(&id1)
            .unwrap()
            .start_mitigation()
            .unwrap();

        let identified = manager.get_risks_by_status(RiskStatus::Identified);
        let mitigating = manager.get_risks_by_status(RiskStatus::Mitigating);

        assert_eq!(identified.len(), 1);
        assert_eq!(mitigating.len(), 1);
    }

    #[test]
    fn test_risk_manager_get_risks_by_level() {
        let mut manager = RiskManager::new();

        let id1 = manager.create_risk("Risk 1", "Description 1").unwrap();
        let id2 = manager.create_risk("Risk 2", "Description 2").unwrap();

        manager.get_risk_mut(&id1).unwrap().level = RiskLevel::High;

        let high_risks = manager.get_risks_by_level(RiskLevel::High);
        let medium_risks = manager.get_risks_by_level(RiskLevel::Medium);

        assert_eq!(high_risks.len(), 1);
        assert_eq!(medium_risks.len(), 1);
    }

    #[test]
    fn test_risk_manager_calculate_overall_mitigation_progress() {
        let mut manager = RiskManager::new();

        // 空管理器
        assert_eq!(manager.calculate_overall_mitigation_progress(), 1.0);

        let id1 = manager.create_risk("Risk 1", "Description 1").unwrap();
        let id2 = manager.create_risk("Risk 2", "Description 2").unwrap();

        // 验证两个风险都已创建
        assert!(manager.get_risk(&id1).is_ok());
        assert!(manager.get_risk(&id2).is_ok());

        // 添加缓解措施并完成一个
        let mut measure = MitigationMeasure::new("Measure");
        measure.complete();
        manager
            .get_risk_mut(&id1)
            .unwrap()
            .add_mitigation_measure(measure)
            .unwrap();

        assert_eq!(manager.calculate_overall_mitigation_progress(), 0.5); // (1.0 + 0.0) / 2
    }

    #[test]
    fn test_risk_id_display() {
        let id = RiskId::new(42);
        assert_eq!(format!("{}", id), "Risk(42)");
    }

    #[test]
    fn test_risk_level_display() {
        assert_eq!(format!("{}", RiskLevel::Low), "Low");
        assert_eq!(format!("{}", RiskLevel::Medium), "Medium");
        assert_eq!(format!("{}", RiskLevel::High), "High");
    }

    #[test]
    fn test_risk_status_display() {
        assert_eq!(format!("{}", RiskStatus::Identified), "Identified");
        assert_eq!(format!("{}", RiskStatus::Mitigating), "Mitigating");
        assert_eq!(format!("{}", RiskStatus::Mitigated), "Mitigated");
        assert_eq!(format!("{}", RiskStatus::Occurred), "Occurred");
    }
}
