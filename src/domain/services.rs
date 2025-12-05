//! 领域服务层
//! 实现依赖注入容器和真正的领域服务

use crate::domain::audio::{AudioListener, AudioSource, AudioSourceId};
use crate::domain::errors::{AudioError, DomainError, PhysicsError};
use crate::domain::physics::{Collider, ColliderId, PhysicsWorld, RigidBody, RigidBodyId};
use crate::domain::scene::{Scene, SceneId, SceneManager};
use crate::domain::value_objects::Volume;
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
/// // 播放音频
/// audio_service.play_source(AudioSourceId::new(1))?;
///
/// // 设置音量
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
        let source = AudioSource::from_file(id, path)?;
        self.sources.insert(id, source);
        self.last_updated = Self::current_timestamp();
        Ok(())
    }

    /// 销毁音频源
    pub fn destroy_source(&mut self, id: AudioSourceId) -> Result<AudioSource, DomainError> {
        let source = self.sources.remove(&id).ok_or_else(|| {
            DomainError::Audio(AudioError::SourceNotFound(format!(
                "Source {}",
                id.as_u64()
            )))
        })?;
        self.last_updated = Self::current_timestamp();
        Ok(source)
    }

    /// 播放音频源
    pub fn play_source(&mut self, id: AudioSourceId) -> Result<(), DomainError> {
        let source = self.sources.get_mut(&id).ok_or_else(|| {
            DomainError::Audio(AudioError::SourceNotFound(format!(
                "Source {}",
                id.as_u64()
            )))
        })?;
        source.play()?;
        self.last_updated = Self::current_timestamp();
        Ok(())
    }

    /// 停止音频源
    pub fn stop_source(&mut self, id: AudioSourceId) -> Result<(), DomainError> {
        let source = self.sources.get_mut(&id).ok_or_else(|| {
            DomainError::Audio(AudioError::SourceNotFound(format!(
                "Source {}",
                id.as_u64()
            )))
        })?;
        source.stop()?;
        self.last_updated = Self::current_timestamp();
        Ok(())
    }

    /// 暂停音频源
    pub fn pause_source(&mut self, id: AudioSourceId) -> Result<(), DomainError> {
        let source = self.sources.get_mut(&id).ok_or_else(|| {
            DomainError::Audio(AudioError::SourceNotFound(format!(
                "Source {}",
                id.as_u64()
            )))
        })?;
        source.pause()?;
        self.last_updated = Self::current_timestamp();
        Ok(())
    }

    /// 恢复音频源
    pub fn resume_source(&mut self, id: AudioSourceId) -> Result<(), DomainError> {
        let source = self.sources.get_mut(&id).ok_or_else(|| {
            DomainError::Audio(AudioError::SourceNotFound(format!(
                "Source {}",
                id.as_u64()
            )))
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
            DomainError::Audio(AudioError::SourceNotFound(format!(
                "Source {}",
                id.as_u64()
            )))
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
        let volume = Volume::new(value)
            .ok_or_else(|| DomainError::Audio(AudioError::InvalidVolume(value)))?;
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
        let volume = Volume::new(value)
            .ok_or_else(|| DomainError::Audio(AudioError::InvalidVolume(value)))?;
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

    fn current_timestamp() -> u64 {
        crate::core::utils::current_timestamp()
    }
}

/// 物理领域服务
///
/// 管理物理世界的创建、更新，以及刚体和碰撞体的操作。
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
    /// 最后更新时间戳
    last_updated: u64,
}

impl PhysicsDomainService {
    /// 创建新的物理领域服务
    ///
    /// # 返回
    ///
    /// 返回一个初始化的`PhysicsDomainService`实例，包含一个新的物理世界。
    pub fn new() -> Self {
        Self {
            world: PhysicsWorld::new(),
            last_updated: Self::current_timestamp(),
        }
    }

    /// 创建刚体
    pub fn create_body(&mut self, body: RigidBody) -> Result<(), DomainError> {
        self.world.add_body(body)?;
        self.last_updated = Self::current_timestamp();
        Ok(())
    }

    /// 销毁刚体
    pub fn destroy_body(&mut self, id: RigidBodyId) -> Result<(), DomainError> {
        self.world.remove_body(id)?;
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

    /// 应用力到刚体
    pub fn apply_force(
        &mut self,
        body_id: RigidBodyId,
        force: glam::Vec3,
    ) -> Result<(), DomainError> {
        if let Some(handle) = self.world.body_handles.get(&body_id) {
            if let Some(rb) = self.world.rigid_body_set.get_mut(*handle) {
                rb.add_force(vector![force.x, force.y, force.z], true);
            }
        }
        self.last_updated = Self::current_timestamp();
        Ok(())
    }

    /// 应用冲量到刚体
    pub fn apply_impulse(
        &mut self,
        body_id: RigidBodyId,
        impulse: glam::Vec3,
    ) -> Result<(), DomainError> {
        if let Some(handle) = self.world.body_handles.get(&body_id) {
            if let Some(rb) = self.world.rigid_body_set.get_mut(*handle) {
                rb.apply_impulse(vector![impulse.x, impulse.y, impulse.z], true);
            }
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
        if let Some(handle) = self.world.body_handles.get(&body_id) {
            if let Some(rb) = self.world.rigid_body_set.get_mut(*handle) {
                rb.set_translation(vector![position.x, position.y, position.z], true);
            }
        }
        self.last_updated = Self::current_timestamp();
        Ok(())
    }

    /// 获取刚体位置
    pub fn get_body_position(&self, body_id: RigidBodyId) -> Result<glam::Vec3, DomainError> {
        if let Some(handle) = self.world.body_handles.get(&body_id) {
            if let Some(rb) = self.world.rigid_body_set.get(*handle) {
                let pos = rb.translation();
                return Ok(glam::Vec3::new(pos.x, pos.y, pos.z));
            }
        }
        Err(DomainError::Physics(PhysicsError::BodyNotFound(format!(
            "Body {}",
            body_id.as_u64()
        ))))
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

    fn current_timestamp() -> u64 {
        crate::core::utils::current_timestamp()
    }
}

/// 场景领域服务
///
/// 管理场景的创建、切换、更新等操作。
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
    /// 场景管理器
    manager: SceneManager,
    /// 最后更新时间戳
    last_updated: u64,
}

impl SceneDomainService {
    /// 创建新的场景领域服务
    ///
    /// # 返回
    ///
    /// 返回一个初始化的`SceneDomainService`实例，包含一个新的场景管理器。
    pub fn new() -> Self {
        Self {
            manager: SceneManager::new(),
            last_updated: Self::current_timestamp(),
        }
    }

    /// 创建场景
    pub fn create_scene(
        &mut self,
        id: SceneId,
        name: impl Into<String>,
    ) -> Result<(), DomainError> {
        self.manager.create_scene(id, name)?;
        self.last_updated = Self::current_timestamp();
        Ok(())
    }

    /// 删除场景
    pub fn delete_scene(&mut self, id: SceneId) -> Result<Scene, DomainError> {
        let scene = self.manager.delete_scene(id)?;
        self.last_updated = Self::current_timestamp();
        Ok(scene)
    }

    /// 切换到场景
    pub fn switch_to_scene(&mut self, id: SceneId) -> Result<(), DomainError> {
        self.manager.switch_to_scene(id)?;
        self.last_updated = Self::current_timestamp();
        Ok(())
    }

    /// 获取场景
    pub fn get_scene(&self, id: SceneId) -> Option<&Scene> {
        self.manager.get_scene(id)
    }

    /// 获取场景可变引用
    pub fn get_scene_mut(&mut self, id: SceneId) -> Option<&mut Scene> {
        self.manager.get_scene_mut(id)
    }

    /// 获取活跃场景
    pub fn get_active_scene(&self) -> Option<&Scene> {
        self.manager.active_scene()
    }

    /// 获取活跃场景可变引用
    pub fn get_active_scene_mut(&mut self) -> Option<&mut Scene> {
        self.manager.active_scene_mut()
    }

    /// 更新场景
    pub fn update_scenes(&mut self, delta_time: f32) -> Result<(), DomainError> {
        self.manager.update(delta_time)?;
        self.last_updated = Self::current_timestamp();
        Ok(())
    }

    /// 获取场景管理器
    pub fn get_manager(&self) -> &SceneManager {
        &self.manager
    }

    /// 获取场景管理器可变引用
    pub fn get_manager_mut(&mut self) -> &mut SceneManager {
        &mut self.manager
    }

    /// 获取所有场景ID
    pub fn scene_ids(&self) -> Vec<SceneId> {
        self.manager.scene_ids()
    }

    fn current_timestamp() -> u64 {
        crate::core::utils::current_timestamp()
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
        service.create_source(AudioSourceId(1), "test.wav").unwrap();
        assert_eq!(service.source_ids().len(), 1);

        // 播放音频源
        service.play_source(AudioSourceId(1)).unwrap();
        assert_eq!(service.playing_sources_count(), 1);

        // 停止音频源
        service.stop_source(AudioSourceId(1)).unwrap();
        assert_eq!(service.playing_sources_count(), 0);

        // 销毁音频源
        service.destroy_source(AudioSourceId(1)).unwrap();
        assert_eq!(service.source_ids().len(), 0);
    }

    #[test]
    fn test_physics_domain_service() {
        let mut service = PhysicsDomainService::new();

        // 创建刚体
        let body = RigidBody::dynamic(RigidBodyId(1), glam::Vec3::ZERO);
        service.create_body(body).unwrap();

        // 创建碰撞体
        let collider = Collider::ball(ColliderId(1), 0.5);
        service.create_collider(collider, RigidBodyId(1)).unwrap();

        // 应用力
        service
            .apply_force(RigidBodyId(1), glam::Vec3::new(10.0, 0.0, 0.0))
            .unwrap();

        // 步进模拟
        service.step_simulation(1.0 / 60.0).unwrap();

        // 获取位置
        let position = service.get_body_position(RigidBodyId(1)).unwrap();
        assert!(position.x > 0.0); // 应该移动了
    }

    #[test]
    fn test_scene_domain_service() {
        let mut service = SceneDomainService::new();

        // 创建场景
        service.create_scene(SceneId(1), "Test Scene").unwrap();
        service.create_scene(SceneId(2), "Another Scene").unwrap();

        // 切换场景
        service.switch_to_scene(SceneId(1)).unwrap();
        assert_eq!(service.get_active_scene().unwrap().id, SceneId(1));

        service.switch_to_scene(SceneId(2)).unwrap();
        assert_eq!(service.get_active_scene().unwrap().id, SceneId(2));
    }
}
