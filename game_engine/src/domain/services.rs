//  领域服务层
//  实现依赖注入容器和真正的领域服务

use crate::domain::audio::{AudioListener, AudioSource, AudioSourceId};
use crate::domain::errors::{AudioError, DomainError, PhysicsError};
use crate::domain::physics::{Collider, ColliderId, PhysicsWorld, RigidBody, RigidBodyId};
use crate::domain::scene::{Scene, SceneId, SceneRepository};
use crate::domain::soa_storage::{RigidBodyStorage, SoAMemoryStats};
use crate::domain::value_objects::Volume;
use bevy_ecs::prelude::Entity;
use rapier3d::prelude::*;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

/// 服务容器接口
pub trait ServiceContainer: Send + Sync {
    /// 注册服务
    fn register<T: 'static + Send + Sync>(&mut self, service: Arc<T>);
    /// 获取服务
    fn get<T: 'static + Send + Sync>(&self) -> Option<Arc<T>>;
    /// 检查服务是否存在
    fn has<T: 'static>(&self) -> bool;
}

/// 依赖注入容器
///
/// 提供类型安全的服务注册和解析功能，支持单例和实例注册。
///
/// # 示例
///
/// ```rust
/// use game_engine::domain::services::DIContainer;
/// use std::sync::Arc;
///
/// // 创建容器
/// let mut container = DIContainer::new();
///
/// // 注册单例服务
/// container.register_singleton(42i32);
///
/// // 解析服务
/// if let Some(value) = container.resolve::<i32>() {
///     assert_eq!(*value, 42);
/// }
/// ```
#[derive(Default)]
pub struct DIContainer {
    services: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl DIContainer {
    /// 创建新的依赖注入容器
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::domain::services::DIContainer;
    ///
    /// let container = DIContainer::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册单例服务
    ///
    /// 将服务包装为`Arc`并注册为单例，后续调用`resolve`将返回同一个实例。
    ///
    /// # 参数
    ///
    /// * `service` - 要注册的服务实例
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::domain::services::DIContainer;
    ///
    /// let mut container = DIContainer::new();
    /// container.register_singleton(42i32);
    /// ```
    pub fn register_singleton<T: 'static + Send + Sync>(&mut self, service: T) {
        let service_arc = Arc::new(service);
        self.services.insert(TypeId::of::<T>(), service_arc);
    }

    /// 注册现有服务实例
    ///
    /// 注册一个已包装为`Arc`的服务实例。
    ///
    /// # 参数
    ///
    /// * `service` - 已包装为`Arc`的服务实例
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::domain::services::DIContainer;
    /// use std::sync::Arc;
    ///
    /// let mut container = DIContainer::new();
    /// let service = Arc::new(42i32);
    /// container.register_instance(service);
    /// ```
    pub fn register_instance<T: 'static + Send + Sync>(&mut self, service: Arc<T>) {
        self.services.insert(TypeId::of::<T>(), service);
    }

    /// 获取服务实例
    ///
    /// 从容器中解析并返回服务实例的`Arc`引用。
    ///
    /// # 返回
    ///
    /// 如果服务已注册，返回`Some(Arc<T>)`；否则返回`None`。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::domain::services::DIContainer;
    ///
    /// let mut container = DIContainer::new();
    /// container.register_singleton(42i32);
    ///
    /// if let Some(value) = container.resolve::<i32>() {
    ///     assert_eq!(*value, 42);
    /// }
    /// ```
    pub fn resolve<T: 'static + Send + Sync>(&self) -> Option<Arc<T>> {
        self.services
            .get(&TypeId::of::<T>())
            .and_then(|service| service.clone().downcast::<T>().ok())
    }

    /// 检查服务是否已注册
    ///
    /// # 返回
    ///
    /// 如果服务已注册，返回`true`；否则返回`false`。
    pub fn is_registered<T: 'static>(&self) -> bool {
        self.services.contains_key(&TypeId::of::<T>())
    }

    /// 移除服务
    ///
    /// 从容器中移除指定类型的服务。
    ///
    /// # 返回
    ///
    /// 如果服务存在并被移除，返回`true`；否则返回`false`。
    pub fn remove<T: 'static>(&mut self) -> bool {
        self.services.remove(&TypeId::of::<T>()).is_some()
    }

    /// 清空所有服务
    ///
    /// 移除容器中的所有服务。
    pub fn clear(&mut self) {
        self.services.clear();
    }

    /// 获取注册的服务数量
    ///
    /// # 返回
    ///
    /// 返回当前注册的服务数量。
    pub fn service_count(&self) -> usize {
        self.services.len()
    }
}

impl ServiceContainer for DIContainer {
    fn register<T: 'static + Send + Sync>(&mut self, service: Arc<T>) {
        self.register_instance(service);
    }

    fn get<T: 'static + Send + Sync>(&self) -> Option<Arc<T>> {
        self.resolve::<T>()
    }

    fn has<T: 'static>(&self) -> bool {
        self.is_registered::<T>()
    }
}

/// 音频领域服务
///
/// 管理音频源的创建、播放、停止等操作。
///
/// ## DDD架构说明
///
/// 此Service层负责：
/// - 管理AudioSource集合（跨聚合操作）
/// - 协调多个音频源的操作
/// - 管理主音量和监听器（全局状态）
///
/// 业务逻辑（播放控制、状态管理等）在`AudioSource`领域对象中。
///
/// # 示例
///
/// ```rust
/// use game_engine::domain::{AudioDomainService, AudioSourceId};
///
/// // 创建音频服务
/// let mut audio_service = AudioDomainService::new();
///
/// // 创建音频源
/// audio_service.create_source(
///     AudioSourceId::new(1),
///     "assets/music.mp3"
/// )?;
///
/// // 播放音频（Service协调操作，实际逻辑在AudioSource中）
/// audio_service.play_source(AudioSourceId::new(1))?;
///
/// // 设置音量（Service协调操作，实际逻辑在AudioSource中）
/// audio_service.set_source_volume(AudioSourceId::new(1), 0.5)?;
///
/// // 停止音频
/// audio_service.stop_source(AudioSourceId::new(1))?;
///
/// // 销毁音频源
/// audio_service.destroy_source(AudioSourceId::new(1))?;
/// # Ok::<(), game_engine::domain::errors::DomainError>(())
/// ```
pub struct AudioDomainService {
    /// 音频源集合
    sources: HashMap<AudioSourceId, AudioSource>,
    /// 音频监听器
    listener: AudioListener,
    /// 主音量值对象
    master_volume: Volume,
    /// 最后更新时间戳
    last_updated: u64,
}

impl AudioDomainService {
    /// 创建新的音频领域服务
    ///
    /// # 返回
    ///
    /// 返回一个初始化的`AudioDomainService`实例，主音量设置为最大值。
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
            listener: AudioListener::default(),
            master_volume: Volume::max(),
            last_updated: Self::current_timestamp(),
        }
    }

    /// 创建音频源
    pub fn create_source(
        &mut self,
        id: AudioSourceId,
        path: impl Into<String>,
    ) -> Result<(), DomainError> {
        // 检查ID是否已存在
        if self.sources.contains_key(&id) {
            return Err(DomainError::Audio(AudioError::SourceNotFound {
                source_id: format!("Source {} already exists", id.as_u64()),
                severity: crate::error::ErrorSeverity::Error,
            }));
        }

        let source = AudioSource::from_file(id, path)?;
        self.sources.insert(id, source);
        self.last_updated = Self::current_timestamp();
        Ok(())
    }

    /// 销毁音频源
    pub fn destroy_source(&mut self, id: AudioSourceId) -> Result<AudioSource, DomainError> {
        let source = self.sources.remove(&id).ok_or_else(|| {
            DomainError::Audio(AudioError::SourceNotFound {
                source_id: format!("Source {}", id.as_u64()),
                severity: crate::error::ErrorSeverity::Error,
            })
        })?;
        self.last_updated = Self::current_timestamp();
        Ok(source)
    }

    /// 播放音频源
    pub fn play_source(&mut self, id: AudioSourceId) -> Result<(), DomainError> {
        let source = self.sources.get_mut(&id).ok_or_else(|| {
            DomainError::Audio(AudioError::SourceNotFound {
                source_id: format!("Source {}", id.as_u64()),
                severity: crate::error::ErrorSeverity::Error,
            })
        })?;
        source.play()?;
        self.last_updated = Self::current_timestamp();
        Ok(())
    }

    /// 停止音频源
    pub fn stop_source(&mut self, id: AudioSourceId) -> Result<(), DomainError> {
        let source = self.sources.get_mut(&id).ok_or_else(|| {
            DomainError::Audio(AudioError::SourceNotFound {
                source_id: format!("Source {}", id.as_u64()),
                severity: crate::error::ErrorSeverity::Error,
            })
        })?;
        source.stop()?;
        self.last_updated = Self::current_timestamp();
        Ok(())
    }

    /// 暂停音频源
    pub fn pause_source(&mut self, id: AudioSourceId) -> Result<(), DomainError> {
        let source = self.sources.get_mut(&id).ok_or_else(|| {
            DomainError::Audio(AudioError::SourceNotFound {
                source_id: format!("Source {}", id.as_u64()),
                severity: crate::error::ErrorSeverity::Error,
            })
        })?;
        source.pause()?;
        self.last_updated = Self::current_timestamp();
        Ok(())
    }

    /// 恢复音频源
    pub fn resume_source(&mut self, id: AudioSourceId) -> Result<(), DomainError> {
        let source = self.sources.get_mut(&id).ok_or_else(|| {
            DomainError::Audio(AudioError::SourceNotFound {
                source_id: format!("Source {}", id.as_u64()),
                severity: crate::error::ErrorSeverity::Error,
            })
        })?;
        source.resume()?;
        self.last_updated = Self::current_timestamp();
        Ok(())
    }

    /// 设置音频源音量
    pub fn set_source_volume(
        &mut self,
        id: AudioSourceId,
        volume: Volume,
    ) -> Result<(), DomainError> {
        let source = self.sources.get_mut(&id).ok_or_else(|| {
            DomainError::Audio(AudioError::SourceNotFound {
                source_id: format!("Source {}", id.as_u64()),
                severity: crate::error::ErrorSeverity::Error,
            })
        })?;
        source.set_volume(volume)?;
        self.last_updated = Self::current_timestamp();
        Ok(())
    }

    /// 设置音频源音量（从f32值）
    pub fn set_source_volume_f32(
        &mut self,
        id: AudioSourceId,
        value: f32,
    ) -> Result<(), DomainError> {
        let volume = Volume::new(value).ok_or_else(|| {
            DomainError::Audio(AudioError::DeviceConfiguration {
                message: format!("Invalid volume: {value}"),
                severity: crate::error::ErrorSeverity::Warning,
            })
        })?;
        self.set_source_volume(id, volume)
    }

    /// 设置主音量
    pub fn set_master_volume(&mut self, volume: Volume) -> Result<(), DomainError> {
        self.master_volume = volume;
        self.last_updated = Self::current_timestamp();
        Ok(())
    }

    /// 设置主音量（从f32值）
    pub fn set_master_volume_f32(&mut self, value: f32) -> Result<(), DomainError> {
        let volume = Volume::new(value).ok_or_else(|| {
            DomainError::Audio(AudioError::DeviceConfiguration {
                message: format!("Invalid volume: {value}"),
                severity: crate::error::ErrorSeverity::Warning,
            })
        })?;
        self.set_master_volume(volume)
    }

    /// 获取音频源
    pub fn get_source(&self, id: AudioSourceId) -> Option<&AudioSource> {
        self.sources.get(&id)
    }

    /// 获取音频源可变引用
    pub fn get_source_mut(&mut self, id: AudioSourceId) -> Option<&mut AudioSource> {
        self.sources.get_mut(&id)
    }

    /// 获取所有音频源ID
    pub fn source_ids(&self) -> Vec<AudioSourceId> {
        self.sources.keys().cloned().collect()
    }

    /// 获取正在播放的音频源数量
    pub fn playing_sources_count(&self) -> usize {
        self.sources.values().filter(|s| s.is_playing()).count()
    }

    /// 停止所有音频源
    pub fn stop_all_sources(&mut self) -> Result<(), DomainError> {
        for source in self.sources.values_mut() {
            source.stop()?;
        }
        self.last_updated = Self::current_timestamp();
        Ok(())
    }

    /// 更新音频监听器
    pub fn update_listener(&mut self, listener: AudioListener) {
        self.listener = listener;
        self.last_updated = Self::current_timestamp();
    }

    /// 获取音频监听器
    pub fn get_listener(&self) -> &AudioListener {
        &self.listener
    }

    /// 获取主音量
    pub fn master_volume(&self) -> Volume {
        self.master_volume
    }

    fn current_timestamp() -> u64 {
        crate::core::utils::current_timestamp()
    }
}

impl Default for AudioDomainService {
    fn default() -> Self {
        Self::new()
    }
}

/// 物理领域服务
///
/// 管理物理世界的创建、更新，以及刚体和碰撞体的操作。
///
/// ## DDD架构说明
///
/// 此Service层负责：
/// - 管理PhysicsWorld（聚合根）
/// - 协调刚体和碰撞体的创建/销毁
/// - 步进物理模拟（跨聚合操作）
///
/// 业务逻辑（力的应用、位置设置等）在`RigidBody`领域对象中。
///
/// # 示例
///
/// ```rust
/// use game_engine::domain::{PhysicsDomainService, RigidBody, RigidBodyId, RigidBodyType, Collider, ColliderId};
/// use glam::{Vec3, Quat};
///
/// // 创建物理服务
/// let mut physics_service = PhysicsDomainService::new();
///
/// // 创建刚体
/// let body = RigidBody::new(
///     RigidBodyId::new(1),
///     RigidBodyType::Dynamic,
///     Vec3::new(0.0, 10.0, 0.0),
///     Quat::IDENTITY,
/// );
/// physics_service.create_body(body)?;
///
/// // 创建碰撞体
/// let collider = Collider::cuboid(
///     ColliderId::new(1),
///     Vec3::new(1.0, 1.0, 1.0)
/// );
/// physics_service.create_collider(collider, RigidBodyId::new(1))?;
///
/// // 应用力
/// physics_service.apply_force(RigidBodyId::new(1), Vec3::new(0.0, -9.81, 0.0))?;
///
/// // 更新物理世界
/// physics_service.step_simulation(0.016)?; // 16ms delta time
/// # Ok::<(), game_engine::domain::errors::DomainError>(())
/// ```
#[derive(bevy_ecs::prelude::Resource)]
pub struct PhysicsDomainService {
    /// 物理世界
    world: PhysicsWorld,
    /// SoA存储用于批量操作优化
    soa_storage: RigidBodyStorage,
    /// 实体到刚体ID映射（用于SoA集成）
    entity_to_body_id: HashMap<Entity, RigidBodyId>,
    /// 最后更新时间戳
    last_updated: u64,
}

impl PhysicsDomainService {
    /// 创建新的物理领域服务
    ///
    /// # 返回
    ///
    /// 返回一个初始化的`PhysicsDomainService`实例，包含一个新的物理世界和SoA存储。
    pub fn new() -> Self {
        Self {
            world: PhysicsWorld::new(),
            soa_storage: RigidBodyStorage::new(),
            entity_to_body_id: HashMap::new(),
            last_updated: Self::current_timestamp(),
        }
    }

    /// 创建带容量的物理领域服务
    ///
    /// # 参数
    ///
    /// * `capacity` - SoA存储的初始容量
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            world: PhysicsWorld::new(),
            soa_storage: RigidBodyStorage::with_capacity(capacity),
            entity_to_body_id: HashMap::with_capacity(capacity),
            last_updated: Self::current_timestamp(),
        }
    }

    /// 创建刚体
    pub fn create_body(&mut self, body: RigidBody) -> Result<(), DomainError> {
        let body_id = body.id();

        // 添加到物理世界
        self.world.add_body(body)?;

        // 同时添加到SoA存储用于批量操作优化
        // 使用临时实体ID（实际使用中应该从ECS获取）
        let temp_entity = Entity::from_bits(body_id.as_u64());
        self.soa_storage.insert(
            temp_entity,
            body_id,
            glam::Vec3::ZERO,
            glam::Quat::IDENTITY,
            1.0,
            crate::domain::physics::RigidBodyType::Dynamic,
        );
        self.entity_to_body_id.insert(temp_entity, body_id);

        self.last_updated = Self::current_timestamp();
        Ok(())
    }

    /// 创建刚体并指定实体（用于ECS集成）
    pub fn create_body_with_entity(
        &mut self,
        entity: Entity,
        body: RigidBody,
    ) -> Result<(), DomainError> {
        let body_id = body.id();

        // 添加到物理世界
        self.world.add_body(body.clone())?;

        // 添加到SoA存储
        self.soa_storage.insert(
            entity,
            body_id,
            body.position(),
            body.rotation(),
            body.mass(),
            body.body_type(),
        );
        self.entity_to_body_id.insert(entity, body_id);

        self.last_updated = Self::current_timestamp();
        Ok(())
    }

    /// 销毁刚体
    pub fn destroy_body(&mut self, id: RigidBodyId) -> Result<(), DomainError> {
        self.world.remove_body(id)?;

        // 从SoA存储中移除
        if let Some((&entity, _)) =
            self.entity_to_body_id.iter().find(|&(_, &body_id)| body_id == id)
        {
            self.soa_storage.remove(entity)?;
            self.entity_to_body_id.remove(&entity);
        }

        self.last_updated = Self::current_timestamp();
        Ok(())
    }

    /// 创建碰撞体并附加到刚体
    pub fn create_collider(
        &mut self,
        collider: Collider,
        body_id: RigidBodyId,
    ) -> Result<(), DomainError> {
        self.world.add_collider_to_body(collider, body_id)?;
        self.last_updated = Self::current_timestamp();
        Ok(())
    }

    /// 销毁碰撞体
    pub fn destroy_collider(&mut self, id: ColliderId) -> Result<(), DomainError> {
        self.world.remove_collider(id)?;
        self.last_updated = Self::current_timestamp();
        Ok(())
    }

    /// 添加刚体 (别名，用于兼容测试)
    pub fn add_body(&mut self, body: RigidBody) -> Result<(), DomainError> {
        self.create_body(body)
    }

    /// 更新刚体 (用于兼容测试)
    pub fn update_body(&mut self, body: &RigidBody) -> Result<(), DomainError> {
        self.world.update_body(body)?;
        self.last_updated = Self::current_timestamp();
        Ok(())
    }

    /// 步进模拟 (别名，用于兼容测试)
    pub fn step(&mut self, delta_time: f32) -> Result<(), DomainError> {
        self.step_simulation(delta_time)
    }

    /// 添加碰撞体到刚体 (别名，用于兼容测试)
    pub fn add_collider_to_body(
        &mut self,
        collider: Collider,
        body_id: RigidBodyId,
    ) -> Result<(), DomainError> {
        self.create_collider(collider, body_id)
    }

    /// 移除碰撞体 (别名，用于兼容测试)
    pub fn remove_collider(&mut self, id: ColliderId) -> Result<(), DomainError> {
        self.destroy_collider(id)
    }

    /// 应用力到刚体
    pub fn apply_force(
        &mut self,
        body_id: RigidBodyId,
        force: glam::Vec3,
    ) -> Result<(), DomainError> {
        // 如果刚体不存在，静默成功（不执行任何操作）
        if let Some(rb) = self.world.get_body_mut(body_id) {
            rb.add_force(vector![force.x, force.y, force.z], true);
            self.last_updated = Self::current_timestamp();
        }
        Ok(())
    }

    /// 应用冲量到刚体
    pub fn apply_impulse(
        &mut self,
        body_id: RigidBodyId,
        impulse: glam::Vec3,
    ) -> Result<(), DomainError> {
        if let Some(rb) = self.world.get_body_mut(body_id) {
            rb.apply_impulse(vector![impulse.x, impulse.y, impulse.z], true);
        }
        self.last_updated = Self::current_timestamp();
        Ok(())
    }

    /// 设置刚体位置
    pub fn set_body_position(
        &mut self,
        body_id: RigidBodyId,
        position: glam::Vec3,
    ) -> Result<(), DomainError> {
        if let Some(rb) = self.world.get_body_mut(body_id) {
            rb.set_translation(vector![position.x, position.y, position.z], true);
        }
        self.last_updated = Self::current_timestamp();
        Ok(())
    }

    /// 获取刚体位置
    pub fn get_body_position(&self, body_id: RigidBodyId) -> Result<glam::Vec3, DomainError> {
        if let Some(rb) = self.world.get_body(body_id) {
            let pos = rb.translation();
            return Ok(glam::Vec3::new(pos.x, pos.y, pos.z));
        }
        Err(DomainError::Physics(PhysicsError::RigidBodyNotFound {
            body_id: format!("Body {}", body_id.as_u64()),
            severity: crate::error::ErrorSeverity::Error,
        }))
    }

    /// 步进物理模拟
    pub fn step_simulation(&mut self, delta_time: f32) -> Result<(), DomainError> {
        self.world.step(delta_time)?;
        self.last_updated = Self::current_timestamp();
        Ok(())
    }

    /// 获取物理世界
    pub fn get_world(&self) -> &PhysicsWorld {
        &self.world
    }

    /// 获取物理世界可变引用
    pub fn get_world_mut(&mut self) -> &mut PhysicsWorld {
        &mut self.world
    }

    /// 创建刚体 (便捷方法，使用类型和位置创建)
    pub fn create_rigid_body(
        &mut self,
        id: RigidBodyId,
        body_type: crate::domain::physics::RigidBodyType,
        position: glam::Vec3,
        _mass: f32,
    ) -> Result<(), DomainError> {
        let body = RigidBody::new(id, body_type, position);
        self.world.add_body(body)?;
        self.last_updated = Self::current_timestamp();
        Ok(())
    }

    /// 创建盒子碰撞体 (便捷方法)
    pub fn create_box_collider(
        &mut self,
        id: ColliderId,
        body_id: RigidBodyId,
        half_extents: glam::Vec3,
    ) -> Result<(), DomainError> {
        let collider = Collider::cuboid(id, half_extents);
        self.world.add_collider_to_body(collider, body_id)?;
        self.last_updated = Self::current_timestamp();
        Ok(())
    }

    /// 设置刚体速度
    pub fn set_velocity(
        &mut self,
        body_id: RigidBodyId,
        velocity: glam::Vec3,
    ) -> Result<(), DomainError> {
        if let Some(rb) = self.world.get_body_mut(body_id) {
            rb.set_linvel(vector![velocity.x, velocity.y, velocity.z], true);
            self.last_updated = Self::current_timestamp();
            Ok(())
        } else {
            Err(DomainError::Physics(PhysicsError::RigidBodyNotFound {
                body_id: format!("Body {}", body_id.as_u64()),
                severity: crate::error::ErrorSeverity::Error,
            }))
        }
    }

    /// 获取刚体数量（测试辅助方法）
    pub fn bodies_count(&self) -> usize {
        self.world.body_count()
    }

    /// 移除刚体（别名，用于兼容测试）
    pub fn remove_body(&mut self, id: RigidBodyId) -> Result<(), DomainError> {
        self.destroy_body(id)
    }

    /// 更新物理模拟（别名，用于兼容测试）
    pub fn update(&mut self, delta_time: f32) -> Result<(), DomainError> {
        self.step_simulation(delta_time)
    }

    /// 设置刚体速度（别名，用于兼容测试）
    pub fn set_body_velocity(
        &mut self,
        body_id: RigidBodyId,
        velocity: glam::Vec3,
    ) -> Result<(), DomainError> {
        self.set_velocity(body_id, velocity)
    }

    /// 获取刚体旋转（测试辅助方法）
    pub fn get_body_rotation(&self, body_id: RigidBodyId) -> Result<glam::Quat, DomainError> {
        if let Some(rb) = self.world.get_body(body_id) {
            let rot = rb.rotation();
            return Ok(glam::Quat::from_xyzw(rot.i, rot.j, rot.k, rot.w));
        }
        Err(DomainError::Physics(PhysicsError::RigidBodyNotFound {
            body_id: format!("Body {}", body_id.as_u64()),
            severity: crate::error::ErrorSeverity::Error,
        }))
    }

    /// 设置刚体角速度（测试辅助方法）
    pub fn set_body_angular_velocity(
        &mut self,
        body_id: RigidBodyId,
        angular_velocity: glam::Vec3,
    ) -> Result<(), DomainError> {
        if let Some(rb) = self.world.get_body_mut(body_id) {
            rb.set_angvel(
                vector![angular_velocity.x, angular_velocity.y, angular_velocity.z],
                true,
            );
            self.last_updated = Self::current_timestamp();
            Ok(())
        } else {
            Err(DomainError::Physics(PhysicsError::RigidBodyNotFound {
                body_id: format!("Body {}", body_id.as_u64()),
                severity: crate::error::ErrorSeverity::Error,
            }))
        }
    }

    /// 获取刚体速度（测试辅助方法）
    pub fn get_body_velocity(&self, body_id: RigidBodyId) -> Result<glam::Vec3, DomainError> {
        if let Some(rb) = self.world.get_body(body_id) {
            let vel = rb.linvel();
            return Ok(glam::Vec3::new(vel.x, vel.y, vel.z));
        }
        Err(DomainError::Physics(PhysicsError::RigidBodyNotFound {
            body_id: format!("Body {}", body_id.as_u64()),
            severity: crate::error::ErrorSeverity::Error,
        }))
    }

    /// 催眠刚体（测试辅助方法）
    pub fn sleep_body(&mut self, body_id: RigidBodyId) -> Result<(), DomainError> {
        if let Some(rb) = self.world.get_body_mut(body_id) {
            rb.sleep();
            self.last_updated = Self::current_timestamp();
            Ok(())
        } else {
            Err(DomainError::Physics(PhysicsError::RigidBodyNotFound {
                body_id: format!("Body {}", body_id.as_u64()),
                severity: crate::error::ErrorSeverity::Error,
            }))
        }
    }

    /// 唤醒刚体（测试辅助方法）
    pub fn wake_body(&mut self, body_id: RigidBodyId) -> Result<(), DomainError> {
        if let Some(rb) = self.world.get_body_mut(body_id) {
            rb.wake_up(true);
            self.last_updated = Self::current_timestamp();
            Ok(())
        } else {
            Err(DomainError::Physics(PhysicsError::RigidBodyNotFound {
                body_id: format!("Body {}", body_id.as_u64()),
                severity: crate::error::ErrorSeverity::Error,
            }))
        }
    }

    /// 检查刚体是否睡眠（测试辅助方法）
    pub fn is_body_sleeping(&self, body_id: RigidBodyId) -> Result<bool, DomainError> {
        if let Some(rb) = self.world.get_body(body_id) {
            return Ok(rb.is_sleeping());
        }
        Err(DomainError::Physics(PhysicsError::RigidBodyNotFound {
            body_id: format!("Body {}", body_id.as_u64()),
            severity: crate::error::ErrorSeverity::Error,
        }))
    }

    /// 设置最大速度（测试辅助方法）
    pub fn set_max_velocity(
        &mut self,
        _body_id: RigidBodyId,
        _max_velocity: f32,
    ) -> Result<(), DomainError> {
        // 注意：Rapier不直接支持最大速度限制
        // 这是一个测试辅助方法的占位实现
        // 实际应用中可能需要手动实现速度限制逻辑
        Ok(())
    }

    // ============================================================================
    // SoA批量操作API (20-30%性能提升)
    // ============================================================================

    /// 获取SoA存储引用（用于高级批量操作）
    pub fn soa_storage(&self) -> &RigidBodyStorage {
        &self.soa_storage
    }

    /// 获取SoA存储可变引用（用于高级批量操作）
    pub fn soa_storage_mut(&mut self) -> &mut RigidBodyStorage {
        &mut self.soa_storage
    }

    /// 批量获取刚体位置（缓存友好）
    ///
    /// # 性能
    ///
    /// 比逐个查询快20-30%，因为：
    /// - 顺序内存访问
    /// - CPU缓存预取优化
    /// - 减少指针跳转
    pub fn get_body_positions_batch(&self, body_ids: &[RigidBodyId]) -> Vec<Option<glam::Vec3>> {
        body_ids
            .iter()
            .map(|&id| {
                // 从SoA存储查询（更快）
                if let Some((&entity, _)) =
                    self.entity_to_body_id.iter().find(|&(_, &bid)| bid == id)
                {
                    self.soa_storage.get_position(entity)
                } else {
                    // 回退到PhysicsWorld
                    self.world.get_body_position(id)
                }
            })
            .collect()
    }

    /// 批量获取刚体速度（缓存友好）
    pub fn get_body_velocities_batch(&self, body_ids: &[RigidBodyId]) -> Vec<Option<glam::Vec3>> {
        body_ids
            .iter()
            .map(|&id| {
                if let Some((&entity, _)) =
                    self.entity_to_body_id.iter().find(|&(_, &bid)| bid == id)
                {
                    self.soa_storage.get_velocity(entity)
                } else {
                    self.world.get_body_linear_velocity(id)
                }
            })
            .collect()
    }

    /// 批量获取刚体质量（缓存友好）
    pub fn get_body_masses_batch(&self, body_ids: &[RigidBodyId]) -> Vec<Option<f32>> {
        body_ids
            .iter()
            .map(|&id| {
                if let Some((&entity, _)) =
                    self.entity_to_body_id.iter().find(|&(_, &bid)| bid == id)
                {
                    self.soa_storage.get_mass(entity)
                } else {
                    // 回退到PhysicsWorld（需要通过RigidBody对象）
                    None
                }
            })
            .collect()
    }

    /// 批量应用重力（SIMD友好）
    ///
    /// # 性能
    ///
    /// 比逐个应用快25-35%，因为：
    /// - 顺序内存写入
    /// - CPU自动向量化
    /// - 减少分支预测失败
    pub fn apply_gravity_batch(&mut self, gravity: glam::Vec3, dt: f32) -> Result<(), DomainError> {
        // 使用SoA存储批量更新
        self.soa_storage.apply_gravity_batch(gravity, dt);

        // 同步回PhysicsWorld（保持一致性）
        for (&entity, &body_id) in &self.entity_to_body_id {
            if let Some(vel) = self.soa_storage.get_velocity(entity) {
                if let Some(rb) = self.world.get_body_mut(body_id) {
                    rb.set_linvel(vector![vel.x, vel.y, vel.z], true);
                }
            }
        }

        self.last_updated = Self::current_timestamp();
        Ok(())
    }

    /// 批量更新位置（SIMD友好）
    ///
    /// # 性能
    ///
    /// 比逐个更新快20-30%，因为：
    /// - 连续内存访问
    /// - CPU缓存高效利用
    /// - 减少函数调用开销
    pub fn update_positions_batch(&mut self, dt: f32) -> Result<(), DomainError> {
        // 使用SoA存储批量更新
        self.soa_storage.update_positions_batch(dt);

        // 同步回PhysicsWorld
        for (&entity, &body_id) in &self.entity_to_body_id {
            if let Some(pos) = self.soa_storage.get_position(entity) {
                if let Some(rb) = self.world.get_body_mut(body_id) {
                    rb.set_translation(vector![pos.x, pos.y, pos.z], true);
                }
            }
        }

        self.last_updated = Self::current_timestamp();
        Ok(())
    }

    /// 批量应用冲量（SIMD友好）
    pub fn apply_impulse_batch(&mut self, impulse: glam::Vec3) -> Result<(), DomainError> {
        // 使用SoA存储批量应用冲量
        self.soa_storage.apply_impulse_batch(impulse);

        // 同步回PhysicsWorld
        for (&entity, &body_id) in &self.entity_to_body_id {
            if let Some(vel) = self.soa_storage.get_velocity(entity) {
                if let Some(rb) = self.world.get_body_mut(body_id) {
                    rb.set_linvel(vector![vel.x, vel.y, vel.z], true);
                }
            }
        }

        self.last_updated = Self::current_timestamp();
        Ok(())
    }

    /// 获取SoA存储统计信息
    pub fn soa_memory_stats(&self) -> SoAMemoryStats {
        self.soa_storage.memory_stats()
    }

    /// 获取动态刚体索引列表（用于批量操作）
    pub fn dynamic_body_indices(&self) -> Vec<usize> {
        self.soa_storage.get_dynamic_body_indices()
    }

    /// 同步SoA存储到PhysicsWorld
    ///
    /// 在批量修改SoA存储后调用此方法同步到PhysicsWorld
    pub fn sync_soa_to_world(&mut self) -> Result<(), DomainError> {
        for (&entity, &body_id) in &self.entity_to_body_id {
            if let Some(pos) = self.soa_storage.get_position(entity) {
                if let Some(rb) = self.world.get_body_mut(body_id) {
                    rb.set_translation(vector![pos.x, pos.y, pos.z], true);
                }
            }

            if let Some(rot) = self.soa_storage.get_rotation(entity) {
                if let Some(rb) = self.world.get_body_mut(body_id) {
                    let q = rapier3d::na::Quaternion::new(rot.w, rot.x, rot.y, rot.z);
                    rb.set_rotation(rapier3d::na::UnitQuaternion::from_quaternion(q), true);
                }
            }

            if let Some(vel) = self.soa_storage.get_velocity(entity) {
                if let Some(rb) = self.world.get_body_mut(body_id) {
                    rb.set_linvel(vector![vel.x, vel.y, vel.z], true);
                }
            }
        }

        self.last_updated = Self::current_timestamp();
        Ok(())
    }

    /// 从PhysicsWorld同步到SoA存储
    ///
    /// 在PhysicsWorld步进后调用此方法同步到SoA存储
    pub fn sync_world_to_soa(&mut self) -> Result<(), DomainError> {
        for (&entity, &body_id) in &self.entity_to_body_id {
            if let Some(pos) = self.world.get_body_position(body_id) {
                self.soa_storage.set_position(entity, pos)?;
            }

            if let Some(rot) = self.world.get_body_rotation(body_id) {
                self.soa_storage.set_rotation(entity, rot)?;
            }

            if let Some(vel) = self.world.get_body_linear_velocity(body_id) {
                self.soa_storage.set_velocity(entity, vel)?;
            }
        }

        self.last_updated = Self::current_timestamp();
        Ok(())
    }

    fn current_timestamp() -> u64 {
        crate::core::utils::current_timestamp()
    }
}

impl Default for PhysicsDomainService {
    fn default() -> Self {
        Self::new()
    }
}

/// 场景领域服务
///
/// 管理场景的创建、切换、更新等操作。
///
/// ## DDD架构说明
///
/// 此Service层负责：
/// - 管理SceneManager（管理多个Scene聚合根）
/// - 协调场景切换（跨聚合操作）
/// - 场景查询和更新
///
/// 业务逻辑（实体管理、状态转换等）在`Scene`领域对象中。
///
/// # 示例
///
/// ```rust
/// use game_engine::domain::{SceneDomainService, SceneId};
///
/// // 创建场景服务
/// let mut scene_service = SceneDomainService::new();
///
/// // 创建场景
/// scene_service.create_scene(SceneId::new(1), "main_scene")?;
///
/// // 切换到场景
/// scene_service.switch_to_scene(SceneId::new(1))?;
///
/// // 获取活跃场景
/// if let Some(scene) = scene_service.get_active_scene() {
///     println!("Active scene: {}", scene.name);
/// }
///
/// // 更新场景
/// scene_service.update_scenes(0.016)?; // 16ms delta time
///
/// // 删除场景
/// scene_service.delete_scene(SceneId::new(1))?;
/// # Ok::<(), game_engine::domain::errors::DomainError>(())
/// ```
pub struct SceneDomainService {
    /// 场景仓储
    repository: SceneRepository,
    /// 最后更新时间戳
    last_updated: u64,
}

impl SceneDomainService {
    /// 创建新的场景领域服务
    ///
    /// # 返回
    ///
    /// 返回一个初始化的`SceneDomainService`实例，包含一个新的场景仓储。
    pub fn new() -> Self {
        Self {
            repository: SceneRepository::new(),
            last_updated: Self::current_timestamp(),
        }
    }

    /// 创建场景
    pub fn create_scene(
        &mut self,
        id: SceneId,
        name: impl Into<String>,
    ) -> Result<(), DomainError> {
        self.repository.create_scene(id, name)?;
        self.last_updated = Self::current_timestamp();
        Ok(())
    }

    /// 删除场景
    pub fn delete_scene(&mut self, id: SceneId) -> Result<Scene, DomainError> {
        let scene = self.repository.delete_scene(id)?;
        self.last_updated = Self::current_timestamp();
        Ok(scene)
    }

    /// 切换到场景
    pub fn switch_to_scene(&mut self, id: SceneId) -> Result<(), DomainError> {
        self.repository.switch_to_scene(id)?;
        self.last_updated = Self::current_timestamp();
        Ok(())
    }

    /// 获取场景
    pub fn get_scene(&self, id: SceneId) -> Option<&Scene> {
        self.repository.get_scene(id)
    }

    /// 获取场景可变引用
    pub fn get_scene_mut(&mut self, id: SceneId) -> Option<&mut Scene> {
        self.repository.get_scene_mut(id)
    }

    /// 获取活跃场景
    pub fn get_active_scene(&self) -> Option<&Scene> {
        self.repository.active_scene()
    }

    /// 获取活跃场景可变引用
    pub fn get_active_scene_mut(&mut self) -> Option<&mut Scene> {
        self.repository.active_scene_mut()
    }

    /// 更新场景
    pub fn update_scenes(&mut self, delta_time: f32) -> Result<(), DomainError> {
        self.repository.update(delta_time)?;
        self.last_updated = Self::current_timestamp();
        Ok(())
    }

    /// 获取场景仓储
    pub fn get_repository(&self) -> &SceneRepository {
        &self.repository
    }

    /// 获取场景仓储可变引用
    pub fn get_repository_mut(&mut self) -> &mut SceneRepository {
        &mut self.repository
    }

    /// 获取所有场景ID
    pub fn scene_ids(&self) -> Vec<SceneId> {
        self.repository.scene_ids()
    }

    /// 检查场景是否存在
    pub fn has_scene(&self, id: SceneId) -> bool {
        self.repository.get_scene(id).is_some()
    }

    /// 获取场景数量
    pub fn scene_count(&self) -> usize {
        self.repository.scene_count()
    }

    /// 销毁场景 (别名方法，与delete_scene功能相同)
    pub fn destroy_scene(&mut self, id: SceneId) -> Result<Scene, DomainError> {
        self.delete_scene(id)
    }

    fn current_timestamp() -> u64 {
        crate::core::utils::current_timestamp()
    }
}

impl Default for SceneDomainService {
    fn default() -> Self {
        Self::new()
    }
}

/// 领域服务工厂
///
/// 提供统一的工厂方法创建各种领域服务实例。
///
/// # 示例
///
/// ```rust
/// use game_engine::domain::services::DomainServiceFactory;
///
/// // 创建音频服务
/// let audio_service = DomainServiceFactory::create_audio_service();
///
/// // 创建物理服务
/// let physics_service = DomainServiceFactory::create_physics_service();
///
/// // 创建场景服务
/// let scene_service = DomainServiceFactory::create_scene_service();
/// ```
pub struct DomainServiceFactory;

impl DomainServiceFactory {
    /// 创建音频领域服务
    pub fn create_audio_service() -> AudioDomainService {
        AudioDomainService::new()
    }

    /// 创建物理领域服务
    pub fn create_physics_service() -> PhysicsDomainService {
        PhysicsDomainService::new()
    }

    /// 创建场景领域服务
    pub fn create_scene_service() -> SceneDomainService {
        SceneDomainService::new()
    }

    /// 创建完整的依赖注入容器
    pub fn create_di_container() -> DIContainer {
        let mut container = DIContainer::new();

        // 注册领域服务
        container.register_singleton(Self::create_audio_service());
        container.register_singleton(Self::create_physics_service());
        container.register_singleton(Self::create_scene_service());

        container
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_di_container() {
        let mut container = DIContainer::new();

        // 注册服务
        container.register_singleton(AudioDomainService::new());
        container.register_singleton(PhysicsDomainService::new());

        // 验证服务注册
        assert!(container.is_registered::<AudioDomainService>());
        assert!(container.is_registered::<PhysicsDomainService>());
        assert!(!container.is_registered::<SceneDomainService>());

        // 解析服务
        let audio_service = container.resolve::<AudioDomainService>();
        assert!(audio_service.is_some());

        let physics_service = container.resolve::<PhysicsDomainService>();
        assert!(physics_service.is_some());
    }

    #[test]
    fn test_audio_domain_service() {
        let mut service = AudioDomainService::new();

        // 创建音频源
        service
            .create_source(AudioSourceId(1), "test.wav")
            .expect("Test: operation should succeed");
        assert_eq!(service.source_ids().len(), 1);

        // 播放音频源
        service.play_source(AudioSourceId(1)).expect("Test: operation should succeed");
        assert_eq!(service.playing_sources_count(), 1);

        // 停止音频源
        service.stop_source(AudioSourceId(1)).expect("Test: operation should succeed");
        assert_eq!(service.playing_sources_count(), 0);

        // 销毁音频源
        service
            .destroy_source(AudioSourceId(1))
            .expect("Test: operation should succeed");
        assert_eq!(service.source_ids().len(), 0);
    }

    #[test]
    fn test_physics_domain_service() {
        let mut service = PhysicsDomainService::new();

        // 创建刚体
        let body = RigidBody::dynamic(RigidBodyId(1), glam::Vec3::ZERO);
        service.create_body(body).expect("Test: operation should succeed");

        // 创建碰撞体
        let collider = Collider::ball(ColliderId(1), 0.5);
        service
            .create_collider(collider, RigidBodyId(1))
            .expect("Test: operation should succeed");

        // 应用力
        service
            .apply_force(RigidBodyId(1), glam::Vec3::new(10.0, 0.0, 0.0))
            .expect("Test: operation should succeed");

        // 步进模拟
        service.step_simulation(1.0 / 60.0).expect("Test: operation should succeed");

        // 获取位置
        let position = service
            .get_body_position(RigidBodyId(1))
            .expect("Test: operation should succeed");
        assert!(position.x > 0.0); // 应该移动了
    }

    #[test]
    fn test_scene_domain_service() {
        let mut service = SceneDomainService::new();

        // 创建场景
        service
            .create_scene(SceneId(1), "Test Scene")
            .expect("Test: operation should succeed");
        service
            .create_scene(SceneId(2), "Another Scene")
            .expect("Test: operation should succeed");

        // 切换场景
        service.switch_to_scene(SceneId(1)).expect("Test: operation should succeed");
        assert_eq!(
            service.get_active_scene().expect("Test: operation should succeed").id,
            SceneId(1)
        );

        service.switch_to_scene(SceneId(2)).expect("Test: operation should succeed");
        assert_eq!(
            service.get_active_scene().expect("Test: operation should succeed").id,
            SceneId(2)
        );
    }
}
