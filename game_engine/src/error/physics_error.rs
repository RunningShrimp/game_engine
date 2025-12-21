//  物理系统错误类型
// 
//  定义了物理系统相关的所有错误类型，包括刚体、碰撞体、约束等。

use crate::error::{ErrorCategory, ErrorSeverity};
use thiserror::Error;

/// 物理系统错误
///
/// 涵盖了物理模拟中的所有可能的错误情况，
/// 从刚体创建到约束求解。
#[derive(Error, Debug, Clone)]
pub enum PhysicsError {
    /// 刚体创建错误
    #[error("Rigid body creation failed: {message}")]
    RigidBodyCreation {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 刚体未找到
    #[error("Rigid body not found: {body_id}")]
    RigidBodyNotFound {
        /// 刚体ID
        body_id: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 无效刚体参数
    #[error("Invalid rigid body parameter: {parameter} = {value}")]
    InvalidRigidBodyParameter {
        /// 参数名称
        parameter: String,
        /// 参数值
        value: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 碰撞体创建错误
    #[error("Collider creation failed: {message}")]
    ColliderCreation {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 碰撞体未找到
    #[error("Collider not found: {collider_id}")]
    ColliderNotFound {
        /// 碰撞体ID
        collider_id: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 无效碰撞体参数
    #[error("Invalid collider parameter: {parameter} = {value}")]
    InvalidColliderParameter {
        /// 参数名称
        parameter: String,
        /// 参数值
        value: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 约束创建错误
    #[error("Joint creation failed: {message}")]
    JointCreation {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 约束未找到
    #[error("Joint not found: {joint_id}")]
    JointNotFound {
        /// 约束ID
        joint_id: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 物理世界未初始化
    #[error("Physics world not initialized")]
    WorldNotInitialized {
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 物理模拟错误
    #[error("Physics simulation error: {message}")]
    Simulation {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 物理配置错误
    #[error("Physics configuration error: {message}")]
    Configuration {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 碰撞检测错误
    #[error("Collision detection error: {message}")]
    CollisionDetection {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 接触求解错误
    #[error("Contact solving error: {message}")]
    ContactSolving {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 力/冲量应用错误
    #[error("Force/Impulse application error: {message}")]
    ForceApplication {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 位置/旋转设置错误
    #[error("Transform setting error: {message}")]
    TransformSetting {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 速度设置错误
    #[error("Velocity setting error: {message}")]
    VelocitySetting {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 质量属性错误
    #[error("Mass properties error: {message}")]
    MassProperties {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 材质属性错误
    #[error("Material properties error: {message}")]
    MaterialProperties {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 查询错误（射线、形状等）
    #[error("Physics query error: {message}")]
    Query {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 广播/收集错误
    #[error("Broad phase error: {message}")]
    BroadPhase {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 窄相位错误
    #[error("Narrow phase error: {message}")]
    NarrowPhase {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 物理世界边界错误
    #[error("World boundary error: {message}")]
    WorldBoundary {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 并发访问错误
    #[error("Concurrent access error: {message}")]
    ConcurrentAccess {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 通用物理错误
    #[error("Physics error: {message}")]
    General {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },
}

impl PhysicsError {
    /// 创建刚体创建错误
    pub fn rigid_body_creation(message: impl Into<String>) -> Self {
        Self::RigidBodyCreation {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建刚体未找到错误
    pub fn rigid_body_not_found(body_id: impl Into<String>) -> Self {
        Self::RigidBodyNotFound {
            body_id: body_id.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建无效刚体参数错误
    pub fn invalid_rigid_body_parameter(
        parameter: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        Self::InvalidRigidBodyParameter {
            parameter: parameter.into(),
            value: value.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建碰撞体创建错误
    pub fn collider_creation(message: impl Into<String>) -> Self {
        Self::ColliderCreation {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建碰撞体未找到错误
    pub fn collider_not_found(collider_id: impl Into<String>) -> Self {
        Self::ColliderNotFound {
            collider_id: collider_id.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建约束创建错误
    pub fn joint_creation(message: impl Into<String>) -> Self {
        Self::JointCreation {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建物理世界未初始化错误
    pub fn world_not_initialized() -> Self {
        Self::WorldNotInitialized {
            severity: ErrorSeverity::Critical,
        }
    }

    /// 创建物理模拟错误
    pub fn simulation(message: impl Into<String>) -> Self {
        Self::Simulation {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建物理配置错误
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建碰撞检测错误
    pub fn collision_detection(message: impl Into<String>) -> Self {
        Self::CollisionDetection {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建力应用错误
    pub fn force_application(message: impl Into<String>) -> Self {
        Self::ForceApplication {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建变换设置错误
    pub fn transform_setting(message: impl Into<String>) -> Self {
        Self::TransformSetting {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建速度设置错误
    pub fn velocity_setting(message: impl Into<String>) -> Self {
        Self::VelocitySetting {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建质量属性错误
    pub fn mass_properties(message: impl Into<String>) -> Self {
        Self::MassProperties {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建查询错误
    pub fn query(message: impl Into<String>) -> Self {
        Self::Query {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建通用物理错误
    pub fn general(message: impl Into<String>) -> Self {
        Self::General {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建带有严重级别的通用物理错误
    pub fn general_with_severity(message: impl Into<String>, severity: ErrorSeverity) -> Self {
        Self::General {
            message: message.into(),
            severity,
        }
    }

    /// 获取错误的严重级别
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            PhysicsError::RigidBodyCreation { severity, .. }
            | PhysicsError::RigidBodyNotFound { severity, .. }
            | PhysicsError::InvalidRigidBodyParameter { severity, .. }
            | PhysicsError::ColliderCreation { severity, .. }
            | PhysicsError::ColliderNotFound { severity, .. }
            | PhysicsError::InvalidColliderParameter { severity, .. }
            | PhysicsError::JointCreation { severity, .. }
            | PhysicsError::JointNotFound { severity, .. }
            | PhysicsError::WorldNotInitialized { severity, .. }
            | PhysicsError::Simulation { severity, .. }
            | PhysicsError::Configuration { severity, .. }
            | PhysicsError::CollisionDetection { severity, .. }
            | PhysicsError::ContactSolving { severity, .. }
            | PhysicsError::ForceApplication { severity, .. }
            | PhysicsError::TransformSetting { severity, .. }
            | PhysicsError::VelocitySetting { severity, .. }
            | PhysicsError::MassProperties { severity, .. }
            | PhysicsError::MaterialProperties { severity, .. }
            | PhysicsError::Query { severity, .. }
            | PhysicsError::BroadPhase { severity, .. }
            | PhysicsError::NarrowPhase { severity, .. }
            | PhysicsError::WorldBoundary { severity, .. }
            | PhysicsError::ConcurrentAccess { severity, .. }
            | PhysicsError::General { severity, .. } => *severity,
        }
    }

    /// 检查错误是否可恢复
    pub fn is_recoverable(&self) -> bool {
        match self {
            // 严重错误通常不可恢复
            PhysicsError::WorldNotInitialized { severity, .. } => {
                *severity < ErrorSeverity::Critical
            }

            // 参数错误通常可恢复（可以通过修正参数）
            PhysicsError::InvalidRigidBodyParameter { .. }
            | PhysicsError::InvalidColliderParameter { .. }
            | PhysicsError::MassProperties { .. }
            | PhysicsError::MaterialProperties { .. } => true,

            // 未找到错误通常可恢复（可以检查存在性）
            PhysicsError::RigidBodyNotFound { .. }
            | PhysicsError::ColliderNotFound { .. }
            | PhysicsError::JointNotFound { .. } => true,

            // 查询错误通常可恢复
            PhysicsError::Query { .. } => true,

            // 其他错误需要根据严重级别判断
            _ => self.severity() < ErrorSeverity::Critical,
        }
    }

    /// 获取错误分类
    pub fn category(&self) -> ErrorCategory {
        ErrorCategory::Physics
    }

    /// 检查是否为刚体相关错误
    pub fn is_rigid_body_related(&self) -> bool {
        matches!(
            self,
            PhysicsError::RigidBodyCreation { .. }
                | PhysicsError::RigidBodyNotFound { .. }
                | PhysicsError::InvalidRigidBodyParameter { .. }
                | PhysicsError::ForceApplication { .. }
                | PhysicsError::TransformSetting { .. }
                | PhysicsError::VelocitySetting { .. }
                | PhysicsError::MassProperties { .. }
        )
    }

    /// 检查是否为碰撞体相关错误
    pub fn is_collider_related(&self) -> bool {
        matches!(
            self,
            PhysicsError::ColliderCreation { .. }
                | PhysicsError::ColliderNotFound { .. }
                | PhysicsError::InvalidColliderParameter { .. }
                | PhysicsError::MaterialProperties { .. }
        )
    }

    /// 检查是否为约束相关错误
    pub fn is_joint_related(&self) -> bool {
        matches!(
            self,
            PhysicsError::JointCreation { .. } | PhysicsError::JointNotFound { .. }
        )
    }

    /// 检查是否为模拟相关错误
    pub fn is_simulation_related(&self) -> bool {
        matches!(
            self,
            PhysicsError::Simulation { .. }
                | PhysicsError::CollisionDetection { .. }
                | PhysicsError::ContactSolving { .. }
                | PhysicsError::BroadPhase { .. }
                | PhysicsError::NarrowPhase { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physics_error_creation() {
        let err = PhysicsError::rigid_body_not_found("body_123");
        assert_eq!(err.severity(), ErrorSeverity::Error);
        assert!(err.is_rigid_body_related());
        assert!(err.is_recoverable());
    }

    #[test]
    fn test_physics_error_severity() {
        let critical_err = PhysicsError::world_not_initialized();
        assert_eq!(critical_err.severity(), ErrorSeverity::Critical);
        assert!(!critical_err.is_recoverable());

        let normal_err = PhysicsError::general("Temporary physics issue");
        assert_eq!(normal_err.severity(), ErrorSeverity::Error);
        assert!(normal_err.is_recoverable());
    }

    #[test]
    fn test_physics_error_categories() {
        let collider_err = PhysicsError::collider_not_found("collider_456");
        assert!(collider_err.is_collider_related());

        let joint_err = PhysicsError::joint_creation("Invalid joint type");
        assert!(joint_err.is_joint_related());

        let sim_err = PhysicsError::simulation("Time step too large");
        assert!(sim_err.is_simulation_related());
    }

    #[test]
    fn test_invalid_parameter_error() {
        let err = PhysicsError::invalid_rigid_body_parameter("mass", "-1.0");
        assert_eq!(err.severity(), ErrorSeverity::Error);
        assert!(err.is_rigid_body_related());
        assert!(err.is_recoverable());
    }
}
