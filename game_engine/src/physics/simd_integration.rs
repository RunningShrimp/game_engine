/// SIMD优化的物理系统集成
///
/// 提供SIMD加速的物理计算函数，集成game_engine_simd库。
use bevy_ecs::prelude::*;
use glam::{Mat4, Vec3};

// 条件性导入SIMD支持
#[cfg(feature = "simd")]
use game_engine_simd::{SimdBackend, batch::PhysicsIntegrator, batch::TransformBatchUpdater};

// 导入velocity组件包装器
use super::velocity_components::{GlobalTransform, InverseMass, Position, Velocity};

/// SIMD优化的物理组件
#[derive(Component, Resource, Debug, Clone)]
pub struct SimdPhysicsState {
    /// 是否启用SIMD优化
    pub enabled: bool,
    /// SIMD后端类型
    pub backend: SimdBackendType,
    /// 性能统计
    pub stats: SimdPerformanceStats,
}

/// SIMD后端类型
#[derive(Debug, Clone, Copy, Default)]
pub enum SimdBackendType {
    #[default]
    Scalar,
    Sse2,
    Avx2,
    Neon,
}

#[cfg(feature = "simd")]
impl From<SimdBackend> for SimdBackendType {
    fn from(backend: SimdBackend) -> Self {
        match backend {
            SimdBackend::Scalar => SimdBackendType::Scalar,
            SimdBackend::Sse2 => SimdBackendType::Sse2,
            SimdBackend::Sse41 => SimdBackendType::Sse2, // Map Sse41 to Sse2
            SimdBackend::Avx => SimdBackendType::Avx2,   // Map Avx to Avx2
            SimdBackend::Avx2 => SimdBackendType::Avx2,
            SimdBackend::Avx512 => SimdBackendType::Avx2, // Map Avx512 to Avx2
            SimdBackend::Neon => SimdBackendType::Neon,
            _ => SimdBackendType::Scalar, // Fallback for unknown backends
        }
    }
}

impl Default for SimdPhysicsState {
    fn default() -> Self {
        Self {
            enabled: true,
            backend: {
                #[cfg(feature = "simd")]
                {
                    SimdBackend::best_available().into()
                }
                #[cfg(not(feature = "simd"))]
                {
                    SimdBackendType::Scalar
                }
            },
            stats: SimdPerformanceStats::default(),
        }
    }
}

/// SIMD性能统计
#[derive(Debug, Clone, Default)]
pub struct SimdPerformanceStats {
    /// 速度更新次数
    pub velocity_updates: usize,
    /// 位置更新次数
    pub position_updates: usize,
    /// 变换更新次数
    pub transform_updates: usize,
    /// 总处理时间（微秒）
    pub total_processing_time_us: u64,
}

/// 批量物理积分数据
pub struct PhysicsIntegrateBatch {
    /// 实体IDs
    pub entities: Vec<Entity>,
    /// 速度数据 [vx, vy, vz, _]
    pub velocities: Vec<[f32; 4]>,
    /// 力数据 [fx, fy, fz, _]
    pub forces: Vec<[f32; 4]>,
    /// 逆质量 (1/mass)
    pub inverse_masses: Vec<f32>,
    /// 位置数据 [x, y, z, _]
    pub positions: Vec<[f32; 4]>,
}

impl PhysicsIntegrateBatch {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entities: Vec::with_capacity(capacity),
            velocities: Vec::with_capacity(capacity),
            forces: Vec::with_capacity(capacity),
            inverse_masses: Vec::with_capacity(capacity),
            positions: Vec::with_capacity(capacity),
        }
    }

    pub fn push(
        &mut self,
        entity: Entity,
        velocity: Vec3,
        force: Vec3,
        inverse_mass: f32,
        position: Vec3,
    ) {
        self.entities.push(entity);
        self.velocities.push([velocity.x, velocity.y, velocity.z, 0.0]);
        self.forces.push([force.x, force.y, force.z, 0.0]);
        self.inverse_masses.push(inverse_mass);
        self.positions.push([position.x, position.y, position.z, 0.0]);
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    pub fn clear(&mut self) {
        self.entities.clear();
        self.velocities.clear();
        self.forces.clear();
        self.inverse_masses.clear();
        self.positions.clear();
    }
}

/// SIMD批量物理积分系统
///
/// 使用SIMD加速批量物理积分计算
pub fn simd_physics_integrate_system(
    mut query: Query<(Entity, &mut Velocity, &mut Position, &InverseMass)>,
    mut simd_state: ResMut<SimdPhysicsState>,
) {
    if !simd_state.enabled {
        return;
    }

    // 收集物理数据
    let mut batch = PhysicsIntegrateBatch::with_capacity(1024);

    for (entity, velocity, position, inverse_mass) in query.iter_mut() {
        if inverse_mass.0 > 0.0 {
            // 假设力为0（简化示例）
            let force = Vec3::ZERO;
            batch.push(entity, velocity.0, force, inverse_mass.0, position.0);
        }
    }

    if batch.is_empty() {
        return;
    }

    // 使用SIMD更新速度和位置
    {
        let dt = 0.016; // 固定时间步长

        #[cfg(feature = "simd")]
        {
            // 更新速度
            let vel_result = PhysicsIntegrator::update_velocities_simd(
                &mut batch.velocities,
                &batch.forces,
                &batch.inverse_masses,
                dt,
            );

            // 更新位置
            let pos_result = PhysicsIntegrator::update_positions_simd(
                &mut batch.positions,
                &batch.velocities,
                dt,
            );

            // 更新统计信息
            simd_state.stats.velocity_updates += vel_result.count;
            simd_state.stats.position_updates += pos_result.count;
            simd_state.stats.total_processing_time_us +=
                vel_result.processing_time_us + pos_result.processing_time_us;
        }

        #[cfg(not(feature = "simd"))]
        {
            // 标量实现（向后兼容）
            for i in 0..batch.len() {
                // 简单的Euler积分
                batch.velocities[i][0] += batch.forces[i][0] * batch.inverse_masses[i] * dt;
                batch.velocities[i][1] += batch.forces[i][1] * batch.inverse_masses[i] * dt;
                batch.velocities[i][2] += batch.forces[i][2] * batch.inverse_masses[i] * dt;

                batch.positions[i][0] += batch.velocities[i][0] * dt;
                batch.positions[i][1] += batch.velocities[i][1] * dt;
                batch.positions[i][2] += batch.velocities[i][2] * dt;
            }

            // 更新统计信息
            simd_state.stats.velocity_updates += batch.len();
            simd_state.stats.position_updates += batch.len();
        }
    }

    // 将结果写回ECS组件
    for (i, entity) in batch.entities.iter().enumerate() {
        if let Ok((_, mut velocity, mut position, _)) = query.get_mut(*entity) {
            velocity.0.x = batch.velocities[i][0];
            velocity.0.y = batch.velocities[i][1];
            velocity.0.z = batch.velocities[i][2];

            position.0.x = batch.positions[i][0];
            position.0.y = batch.positions[i][1];
            position.0.z = batch.positions[i][2];
        }
    }
}

/// 批量变换数据
pub struct TransformUpdateBatch {
    /// 实体IDs
    pub entities: Vec<Entity>,
    /// 本地变换矩阵（列主序 4x4）
    pub local_transforms: Vec<[[f32; 4]; 4]>,
    /// 父变换矩阵（列主序 4x4）
    pub parent_transforms: Vec<[[f32; 4]; 4]>,
}

impl TransformUpdateBatch {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entities: Vec::with_capacity(capacity),
            local_transforms: Vec::with_capacity(capacity),
            parent_transforms: Vec::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, entity: Entity, local: &Mat4, parent: &Mat4) {
        self.entities.push(entity);
        self.local_transforms.push(mat4_to_array(local));
        self.parent_transforms.push(mat4_to_array(parent));
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    pub fn clear(&mut self) {
        self.entities.clear();
        self.local_transforms.clear();
        self.parent_transforms.clear();
    }
}

/// Mat4转数组
fn mat4_to_array(mat: &glam::Mat4) -> [[f32; 4]; 4] {
    [
        [mat.x_axis.x, mat.x_axis.y, mat.x_axis.z, mat.x_axis.w],
        [mat.y_axis.x, mat.y_axis.y, mat.y_axis.z, mat.y_axis.w],
        [mat.z_axis.x, mat.z_axis.y, mat.z_axis.z, mat.z_axis.w],
        [mat.w_axis.x, mat.w_axis.y, mat.w_axis.z, mat.w_axis.w],
    ]
}

/// SIMD批量变换更新系统
///
/// 使用SIMD加速批量变换矩阵计算
pub fn simd_transform_update_system(
    mut query: Query<(Entity, &GlobalTransform, &ParentTransform)>,
    mut simd_state: ResMut<SimdPhysicsState>,
) {
    if !simd_state.enabled {
        return;
    }

    // 收集变换数据
    let mut batch = TransformUpdateBatch::with_capacity(1024);

    for (entity, local_transform, parent_transform) in query.iter_mut() {
        batch.push(entity, &local_transform.0, &parent_transform.0);
    }

    if batch.is_empty() {
        return;
    }

    // 使用SIMD批量更新变换
    #[cfg(feature = "simd")]
    {
        let mut results = vec![[[0.0; 4]; 4]; batch.len()];

        let result = TransformBatchUpdater::update_transforms_batch(
            &batch.local_transforms,
            &batch.parent_transforms,
            &mut results,
        );

        simd_state.stats.transform_updates += result.count;
        simd_state.stats.total_processing_time_us += result.processing_time_us;
    }

    #[cfg(not(feature = "simd"))]
    {
        // 标量实现（计数但不实际处理）
        simd_state.stats.transform_updates += batch.len();
    }
}

/// 父变换组件
#[derive(Component, Debug, Clone)]
pub struct ParentTransform(pub glam::Mat4);

/// SIMD性能监控资源
#[derive(Resource, Debug, Clone)]
pub struct SimdPerformanceMonitor {
    /// 是否启用性能监控
    pub enabled: bool,
    /// 历史性能数据
    pub history: Vec<SimdPerformanceStats>,
}

impl Default for SimdPerformanceMonitor {
    fn default() -> Self {
        Self {
            enabled: true,
            history: Vec::with_capacity(1000),
        }
    }
}

/// SIMD性能监控系统
///
/// 每帧记录SIMD性能统计数据
pub fn simd_performance_monitor_system(
    simd_state: Res<SimdPhysicsState>,
    mut monitor: ResMut<SimdPerformanceMonitor>,
) {
    if !monitor.enabled {
        return;
    }

    // 记录当前帧的性能数据
    monitor.history.push(simd_state.stats.clone());

    // 限制历史数据大小
    if monitor.history.len() > 1000 {
        monitor.history.remove(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simd_backend_type() {
        let backend = SimdBackendType::default();
        // 默认应该是Scalar或者根据平台检测
        match backend {
            SimdBackendType::Scalar => (),
            _ => (),
        }
    }

    #[test]
    fn test_physics_integrate_batch() {
        let mut batch = PhysicsIntegrateBatch::with_capacity(10);
        assert_eq!(batch.len(), 0);
        assert!(batch.is_empty());

        let entity = Entity::from_bits(0);
        batch.push(
            entity,
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::ZERO,
            1.0,
            Vec3::ZERO,
        );

        assert_eq!(batch.len(), 1);
        assert!(!batch.is_empty());
    }

    #[test]
    fn test_transform_update_batch() {
        let mut batch = TransformUpdateBatch::with_capacity(10);
        assert_eq!(batch.len(), 0);
        assert!(batch.is_empty());

        let entity = Entity::from_bits(0);
        let local = glam::Mat4::IDENTITY;
        let parent = glam::Mat4::IDENTITY;

        batch.push(entity, &local, &parent);

        assert_eq!(batch.len(), 1);
        assert!(!batch.is_empty());
    }
}
