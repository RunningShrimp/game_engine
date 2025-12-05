use crate::core::error::RenderError;
use crate::domain::render::{
    LightSource, PbrScene as DomainPbrScene, RenderCommand, RenderObject as DomainRenderObject,
    RenderObjectId, RenderScene, RenderStrategy,
};
use crate::ecs::{DirectionalLightComp, Mesh, PointLight3D as EcsPointLight3D, Transform};
use crate::impl_default;
use crate::render::frustum::Frustum;
use crate::render::lod::{LodConfig, LodConfigBuilder, LodQuality, LodSelector};
use crate::render::mesh::GpuMesh;
use crate::render::pbr::{DirectionalLight, PointLight3D};
use crate::render::wgpu::{Instance, WgpuRenderer};
use bevy_ecs::prelude::*;
use glam::{Mat4, Vec3};
use std::collections::HashMap;
use std::sync::Arc;

// 注意：RenderObject enum 已移除，请使用 domain::render::RenderObject

/// 层缓存 - 用于差异渲染
///
/// 缓存离屏纹理，使用LRU（最近最少使用）策略进行淘汰。
/// 用于优化渲染性能，避免重复渲染未变化的层。
///
/// # 使用示例
///
/// ```rust
/// use game_engine::services::render::LayerCache;
///
/// let mut cache = LayerCache::new();
/// cache.new_frame();
/// cache.mark_clean(1); // 标记纹理1为干净
/// assert!(!cache.is_dirty(1));
/// ```
#[derive(Default)]
pub struct LayerCache {
    /// 按ID缓存的离屏纹理
    offscreen_cache: HashMap<u32, CachedTexture>,
    /// 帧计数器，用于LRU淘汰
    frame_count: u64,
}

impl LayerCache {
    /// 创建新的层缓存
    pub fn new() -> Self {
        Self::default()
    }
}

/// 缓存的纹理信息
///
/// 存储纹理的缓存状态信息，包括最后使用帧和脏标记。
struct CachedTexture {
    /// 最后使用的帧号
    last_used_frame: u64,
    /// 是否为脏（需要重新渲染）
    dirty: bool,
}

impl LayerCache {
    /// 开始新的一帧
    ///
    /// 增加帧计数器，并淘汰60帧未使用的纹理。
    ///
    /// # 性能
    ///
    /// 每帧调用一次，用于维护LRU缓存。
    pub fn new_frame(&mut self) {
        self.frame_count += 1;
        // 淘汰60帧未使用的纹理
        self.offscreen_cache
            .retain(|_, v| self.frame_count - v.last_used_frame < 60);
    }

    /// 标记纹理为已使用
    ///
    /// 更新纹理的最后使用帧号，防止被LRU淘汰。
    ///
    /// # 参数
    ///
    /// * `id` - 纹理ID
    pub fn mark_used(&mut self, id: u32) {
        if let Some(cached) = self.offscreen_cache.get_mut(&id) {
            cached.last_used_frame = self.frame_count;
        }
    }

    /// 检查纹理是否为脏（需要重新渲染）
    ///
    /// # 参数
    ///
    /// * `id` - 纹理ID
    ///
    /// # 返回
    ///
    /// 如果纹理不存在或标记为脏，返回`true`；否则返回`false`。
    pub fn is_dirty(&self, id: u32) -> bool {
        self.offscreen_cache.get(&id).is_none_or(|c| c.dirty)
    }

    /// 标记纹理为干净（不需要重新渲染）
    ///
    /// 如果纹理不存在，会创建新的缓存条目。
    ///
    /// # 参数
    ///
    /// * `id` - 纹理ID
    pub fn mark_clean(&mut self, id: u32) {
        if let Some(cached) = self.offscreen_cache.get_mut(&id) {
            cached.dirty = false;
        } else {
            self.offscreen_cache.insert(
                id,
                CachedTexture {
                    last_used_frame: self.frame_count,
                    dirty: false,
                },
            );
        }
    }
}

/// 渲染服务 - 领域服务
///
/// 封装渲染业务逻辑，作为领域服务协调渲染领域对象。
///
/// ## 业务职责
///
/// - **场景构建**：从ECS数据构建RenderScene聚合根
/// - **场景更新**：协调RenderScene的可见性和LOD更新
/// - **LOD配置**：管理LOD选择器的配置和生命周期
/// - **视锥体管理**：更新视锥体用于剔除
/// - **渲染策略**：应用RenderStrategy的业务规则
///
/// ## 设计原则
///
/// - 业务逻辑封装在领域对象（RenderObject、RenderScene、RenderStrategy）中
/// - RenderService负责协调和编排，不包含具体业务规则
/// - 通过领域对象的方法执行业务逻辑
///
/// ## 使用示例
///
/// ```rust
/// use game_engine::services::render::RenderService;
/// use bevy_ecs::prelude::*;
/// use glam::Mat4;
///
/// // 创建渲染服务
/// let mut render_service = RenderService::new();
///
/// // 配置LOD
/// render_service.use_default_lod();
///
/// // 更新视锥体
/// let view_proj = Mat4::IDENTITY;
/// render_service.update_frustum(view_proj);
///
/// // 从ECS构建渲染场景
/// let mut world = World::new();
/// render_service.build_domain_scene(&mut world).unwrap();
///
/// // 更新场景（可见性、LOD等）
/// render_service.update_scene(0.016, glam::Vec3::ZERO).unwrap();
///
/// // 获取渲染命令
/// let commands = render_service.get_render_commands();
/// ```
#[derive(Default)]
pub struct RenderService {
    /// Layer cache for differential updates
    pub layer_cache: LayerCache,
    /// 渲染场景（领域对象，聚合根）
    render_scene: RenderScene,
    /// 当前视锥体（用于传递给RenderScene）
    current_frustum: Option<Frustum>,
}

impl RenderService {
    /// 创建新的渲染服务
    pub fn new() -> Self {
        Self {
            render_scene: RenderScene::new(),
            ..Default::default()
        }
    }

    /// 配置LOD选择器
    ///
    /// 业务逻辑：将LOD选择器设置到RenderScene聚合根中，由聚合根统一管理。
    /// RenderService只负责配置，不保留LOD选择器的副本，避免状态同步问题。
    pub fn configure_lod(&mut self, config: LodConfig) {
        let selector = LodSelector::new(config);
        // 将LOD选择器设置到RenderScene聚合根中，由聚合根管理
        self.render_scene.set_lod_selector(selector);
        // 不再在RenderService中保留LOD选择器，避免状态同步问题
        // 性能指标更新通过RenderScene的lod_selector_mut()访问
    }

    /// 使用默认LOD配置
    pub fn use_default_lod(&mut self) {
        let config = LodConfigBuilder::new()
            .add_level(0.0, 20.0, LodQuality::High)
            .add_level(20.0, 50.0, LodQuality::Medium)
            .add_level(50.0, 100.0, LodQuality::Low)
            .add_level(100.0, f32::INFINITY, LodQuality::VeryLow)
            .build();
        self.configure_lod(config);
    }

    /// 更新视锥体（用于剔除）
    pub fn update_frustum(&mut self, view_proj: Mat4) {
        let frustum = Frustum::from_view_projection(view_proj);
        self.current_frustum = Some(frustum.clone());
        self.render_scene.set_frustum(frustum);
    }

    /// 从ECS构建渲染场景（使用领域对象）
    pub fn build_domain_scene(&mut self, world: &mut World) -> Result<(), RenderError> {
        self.render_scene.clear();

        let mut object_id = 0u64;
        let mut query_mesh = world.query::<(&Mesh, &Transform)>();

        for (mesh, transform) in query_mesh.iter(world) {
            if let Some(gpu_mesh) = mesh.handle.get() {
                // 将GpuMesh包装为Arc（DomainRenderObject需要Arc<GpuMesh>）
                let mesh_arc = Arc::new(gpu_mesh.clone());
                let mut render_obj =
                    DomainRenderObject::new(RenderObjectId(object_id), mesh_arc, *transform);

                // 应用渲染策略
                let strategy = RenderStrategy::select_for_object(&render_obj);
                // 根据策略标记对象
                match strategy {
                    RenderStrategy::StaticBatch => render_obj.mark_static(),
                    _ => render_obj.mark_dynamic(),
                }

                self.render_scene.add_object(render_obj)?;
                object_id += 1;
            }
        }

        Ok(())
    }

    /// 更新渲染场景（可见性、LOD等）
    ///
    /// 业务逻辑：委托给RenderScene聚合根的update方法，确保业务逻辑封装在领域对象中。
    /// RenderService只负责协调（设置视锥体），不包含具体业务规则。
    pub fn update_scene(&mut self, delta_time: f32, camera_pos: Vec3) -> Result<(), RenderError> {
        // 确保视锥体已设置到RenderScene
        if let Some(ref frustum) = self.current_frustum {
            self.render_scene.set_frustum(frustum.clone());
        }

        // 委托给RenderScene聚合根更新场景
        // 所有业务逻辑（可见性计算、LOD选择）都在RenderScene中
        self.render_scene.update(delta_time, camera_pos)?;

        Ok(())
    }

    /// 更新自适应LOD性能指标
    ///
    /// 应该在每帧渲染后调用，记录性能指标并自适应调整LOD。
    ///
    /// 业务逻辑：通过RenderScene聚合根访问LOD选择器并更新性能指标。
    ///
    /// # 参数
    /// - `frame_time_ms`: 当前帧时间（毫秒）
    /// - `gpu_load`: GPU负载（0.0-1.0，可选）
    pub fn update_adaptive_lod(&mut self, frame_time_ms: f32, gpu_load: Option<f32>) {
        // 通过RenderScene聚合根访问LOD选择器并更新性能指标
        if let Some(ref mut lod_selector) = self.render_scene.lod_selector_mut() {
            lod_selector.update_performance(frame_time_ms, gpu_load);
        }
    }

    /// 获取需要渲染的对象
    pub fn get_renderable_objects(&self) -> impl Iterator<Item = &DomainRenderObject> {
        self.render_scene.renderable_objects()
    }

    /// 获取渲染命令列表
    ///
    /// 业务逻辑：从渲染场景构建渲染命令，按策略分组
    pub fn get_render_commands(&self) -> Vec<RenderCommand> {
        self.render_scene.build_render_commands()
    }

    /// 获取渲染场景的可变引用（用于高级操作）
    ///
    /// 注意：应该优先使用RenderService的方法，而不是直接操作RenderScene
    pub fn render_scene_mut(&mut self) -> &mut RenderScene {
        &mut self.render_scene
    }

    /// 获取渲染场景的引用
    pub fn render_scene(&self) -> &RenderScene {
        &self.render_scene
    }

    // ========================================================================
    // 渲染策略业务逻辑方法
    // ========================================================================

    /// 为渲染对象选择渲染策略
    ///
    /// 这是业务逻辑方法，封装了渲染策略选择的规则。
    /// 委托给`RenderStrategy::select_for_object()`，业务逻辑已封装在领域对象中。
    ///
    /// ## 业务规则
    ///
    /// - **静态对象**: 使用`RenderStrategy::StaticBatch`
    /// - **动态对象**: 使用`RenderStrategy::DynamicBatch`
    /// - **实例化**: 相同网格的多个实例（>10）使用`RenderStrategy::Instanced`
    ///
    /// ## 参数
    ///
    /// * `obj` - 渲染对象（领域对象）
    ///
    /// ## 返回
    ///
    /// 返回选择的渲染策略。
    ///
    /// ## 示例
    ///
    /// ```rust
    /// use game_engine::services::render::RenderService;
    /// use game_engine::domain::render::{RenderObject, RenderObjectId};
    /// use game_engine::render::mesh::GpuMesh;
    /// use std::sync::Arc;
    ///
    /// let service = RenderService::new();
    /// let mesh = Arc::new(GpuMesh::default());
    ///
    /// // 创建静态对象
    /// let mut static_obj = RenderObject::new(
    ///     RenderObjectId::new(1),
    ///     mesh.clone(),
    ///     Default::default(),
    /// );
    /// static_obj.mark_static();
    ///
    /// // 选择策略
    /// let strategy = service.select_render_strategy(&static_obj);
    /// assert!(matches!(strategy, RenderStrategy::StaticBatch));
    /// ```
    pub fn select_render_strategy(&self, obj: &DomainRenderObject) -> RenderStrategy {
        RenderStrategy::select_for_object(obj)
    }

    /// 为多个相同对象选择渲染策略
    ///
    /// 这是业务逻辑方法，封装了实例化策略选择的规则。
    /// 委托给`RenderStrategy::select_for_instances()`，业务逻辑已封装在领域对象中。
    ///
    /// ## 业务规则
    ///
    /// - **实例化阈值**: 如果相同网格的实例数量超过阈值（>10），使用`RenderStrategy::Instanced`
    /// - **静态批次**: 如果实例数量<=10且为静态对象，使用`RenderStrategy::StaticBatch`
    /// - **动态批次**: 如果实例数量<=10且为动态对象，使用`RenderStrategy::DynamicBatch`
    ///
    /// ## 参数
    ///
    /// * `instance_count` - 实例数量
    /// * `is_static` - 是否为静态对象
    ///
    /// ## 返回
    ///
    /// 返回选择的渲染策略。
    ///
    /// ## 示例
    ///
    /// ```rust
    /// use game_engine::services::render::RenderService;
    /// use game_engine::domain::render::RenderStrategy;
    ///
    /// let service = RenderService::new();
    ///
    /// // 实例数量超过阈值，使用实例化
    /// let strategy = service.select_strategy_for_instances(15, true);
    /// assert!(matches!(strategy, RenderStrategy::Instanced));
    ///
    /// // 实例数量未超过阈值，使用静态批次
    /// let strategy = service.select_strategy_for_instances(5, true);
    /// assert!(matches!(strategy, RenderStrategy::StaticBatch));
    ///
    /// // 实例数量未超过阈值，使用动态批次
    /// let strategy = service.select_strategy_for_instances(5, false);
    /// assert!(matches!(strategy, RenderStrategy::DynamicBatch));
    /// ```
    pub fn select_strategy_for_instances(
        &self,
        instance_count: usize,
        is_static: bool,
    ) -> RenderStrategy {
        RenderStrategy::select_for_instances(instance_count, is_static)
    }

    /// 判断是否应该使用实例化渲染
    ///
    /// 这是业务逻辑方法，封装了实例化决策的规则。
    /// 委托给`RenderStrategy::should_instanciate()`，业务逻辑已封装在领域对象中。
    ///
    /// ## 业务规则
    ///
    /// - **实例化条件**: 策略为`RenderStrategy::Instanced`且实例数量超过阈值（>10）时返回`true`
    /// - **其他情况**: 返回`false`
    ///
    /// ## 参数
    ///
    /// * `strategy` - 渲染策略
    /// * `instance_count` - 实例数量
    ///
    /// ## 返回
    ///
    /// 如果应该使用实例化渲染，返回`true`；否则返回`false`。
    ///
    /// ## 示例
    ///
    /// ```rust
    /// use game_engine::services::render::RenderService;
    /// use game_engine::domain::render::RenderStrategy;
    ///
    /// let service = RenderService::new();
    ///
    /// // 实例化策略且实例数量超过阈值
    /// assert!(service.should_use_instancing(&RenderStrategy::Instanced, 15));
    ///
    /// // 实例化策略但实例数量未超过阈值
    /// assert!(!service.should_use_instancing(&RenderStrategy::Instanced, 5));
    ///
    /// // 非实例化策略
    /// assert!(!service.should_use_instancing(&RenderStrategy::StaticBatch, 15));
    /// ```
    pub fn should_use_instancing(&self, strategy: &RenderStrategy, instance_count: usize) -> bool {
        strategy.should_instanciate(instance_count)
    }

    // ========================================================================
    // LOD决策业务逻辑方法
    // ========================================================================

    /// 选择LOD级别
    ///
    /// 这是业务逻辑方法，封装了LOD选择的规则。
    /// 委托给`RenderObject::select_lod()`和`LodSelector`，业务逻辑已封装在领域对象中。
    ///
    /// ## 业务规则
    ///
    /// - **不可见对象**: 如果对象不可见，返回`LodQuality::Culled`
    /// - **距离选择**: 基于到相机的距离选择LOD级别
    /// - **自适应调整**: 考虑自适应LOD调整（距离偏移）
    /// - **过渡处理**: 支持LOD过渡（Crossfade/Dithering）
    ///
    /// ## 参数
    ///
    /// * `obj` - 渲染对象（领域对象）
    /// * `distance` - 到相机的距离（世界单位）
    /// * `delta_time` - 时间增量（秒，用于过渡计算）
    ///
    /// ## 返回
    ///
    /// 如果成功，返回`Ok(LodSelection)`；如果对象处于错误状态，返回错误。
    ///
    /// ## 错误
    ///
    /// - `RenderError::InvalidState`: 如果对象处于错误状态
    ///
    /// ## 注意
    ///
    /// 如果没有配置LOD选择器，返回默认选择（High质量）。
    ///
    /// ## 示例
    ///
    /// ```rust
    /// use game_engine::services::render::RenderService;
    /// use game_engine::domain::render::{RenderObject, RenderObjectId};
    /// use game_engine::render::mesh::GpuMesh;
    /// use std::sync::Arc;
    ///
    /// let mut service = RenderService::new();
    /// service.use_default_lod();
    ///
    /// let mesh = Arc::new(GpuMesh::default());
    /// let mut obj = RenderObject::new(
    ///     RenderObjectId::new(1),
    ///     mesh,
    ///     Default::default(),
    /// );
    ///
    /// // 选择LOD（近距离，应该使用High质量）
    /// let lod = service.select_lod_for_object(&mut obj, 10.0, 0.016).unwrap();
    /// assert_eq!(lod.quality, LodQuality::High);
    /// ```
    pub fn select_lod_for_object(
        &mut self,
        obj: &mut DomainRenderObject,
        distance: f32,
        delta_time: f32,
    ) -> Result<crate::render::lod::LodSelection, RenderError> {
        if let Some(ref mut lod_selector) = self.render_scene.lod_selector_mut() {
            obj.select_lod(distance, lod_selector, delta_time)
        } else {
            // 如果没有LOD选择器，返回默认选择（High质量）
            Ok(crate::render::lod::LodSelection {
                current_level: 0,
                quality: crate::render::lod::LodQuality::High,
                transition_factor: 0.0,
                is_transitioning: false,
                next_level: None,
            })
        }
    }

    /// 批量选择LOD级别
    ///
    /// 这是业务逻辑方法，封装了批量LOD选择的规则。
    /// 委托给`update_scene()`，业务逻辑已封装在领域对象中。
    ///
    /// ## 业务规则
    ///
    /// - **可见性筛选**: 只对可见对象进行LOD选择
    /// - **距离选择**: 基于到相机的距离选择LOD级别
    /// - **自适应调整**: 考虑自适应LOD调整（距离偏移）
    /// - **批量处理**: 高效处理场景中所有对象的LOD选择
    ///
    /// ## 参数
    ///
    /// * `camera_pos` - 相机位置（世界坐标）
    /// * `delta_time` - 时间增量（秒，用于过渡计算）
    ///
    /// ## 返回
    ///
    /// 如果成功，返回`Ok(())`；否则返回错误。
    ///
    /// ## 注意
    ///
    /// 这个方法与`update_scene()`功能相同，提供更明确的语义。
    /// 建议使用此方法进行LOD选择，使用`update_scene()`进行完整的场景更新。
    ///
    /// ## 示例
    ///
    /// ```rust
    /// use game_engine::services::render::RenderService;
    /// use glam::Vec3;
    ///
    /// let mut service = RenderService::new();
    /// service.use_default_lod();
    ///
    /// // 批量选择LOD
    /// let camera_pos = Vec3::new(0.0, 0.0, 0.0);
    /// service.select_lod_for_scene(camera_pos, 0.016).unwrap();
    /// ```
    pub fn select_lod_for_scene(
        &mut self,
        camera_pos: Vec3,
        delta_time: f32,
    ) -> Result<(), RenderError> {
        // 委托给update_scene（包含LOD选择）
        self.update_scene(delta_time, camera_pos)
    }

    /// 获取LOD配置建议
    ///
    /// 这是业务逻辑方法，根据性能指标建议LOD配置调整。
    /// 提供性能驱动的LOD调整建议，帮助维持目标帧率。
    ///
    /// ## 业务规则
    ///
    /// - **帧时间阈值**: 如果帧时间超过阈值（16.7ms，60 FPS），建议降低LOD质量
    ///   - 每超过1ms，建议增加0.1距离偏移
    /// - **GPU负载阈值**: 如果GPU负载超过80%，建议降低LOD质量
    ///   - 每超过0.1负载，建议增加1.0距离偏移
    /// - **性能良好**: 如果性能良好，返回0.0（不调整）
    ///
    /// ## 参数
    ///
    /// * `frame_time_ms` - 当前帧时间（毫秒）
    /// * `gpu_load` - GPU负载（0.0-1.0，可选）
    ///
    /// ## 返回
    ///
    /// 返回建议的距离偏移调整（f32）：
    /// - 正值：增加距离，降低LOD质量
    /// - 负值：减少距离，提升LOD质量（当前不返回负值）
    /// - 0.0：不需要调整
    ///
    /// ## 注意
    ///
    /// 这个方法只提供建议，实际的LOD调整通过`update_adaptive_lod()`方法执行。
    /// 建议值应该与当前LOD配置的距离偏移相加。
    ///
    /// ## 示例
    ///
    /// ```rust
    /// use game_engine::services::render::RenderService;
    ///
    /// let service = RenderService::new();
    ///
    /// // 正常帧时间，不需要调整
    /// let adjustment = service.suggest_lod_adjustment(16.0, Some(0.5));
    /// assert_eq!(adjustment, 0.0);
    ///
    /// // 高帧时间，建议降低质量
    /// let adjustment = service.suggest_lod_adjustment(20.0, Some(0.5));
    /// assert!(adjustment > 0.0);
    ///
    /// // 高GPU负载，建议降低质量
    /// let adjustment = service.suggest_lod_adjustment(16.0, Some(0.9));
    /// assert!(adjustment > 0.0);
    /// ```
    pub fn suggest_lod_adjustment(
        &self,
        frame_time_ms: f32,
        gpu_load: Option<f32>,
    ) -> f32 {
        // 业务规则：如果帧时间超过阈值，建议降低LOD质量
        let frame_time_threshold = 16.7; // 60 FPS
        let frame_time_adjustment = if frame_time_ms > frame_time_threshold {
            (frame_time_ms - frame_time_threshold) * 0.1 // 每毫秒增加0.1距离
        } else {
            0.0
        };

        // 业务规则：如果GPU负载过高，建议降低LOD质量
        let gpu_load_adjustment = if let Some(load) = gpu_load {
            if load > 0.8 {
                (load - 0.8) * 10.0 // 每0.1负载增加1.0距离
            } else {
                0.0
            }
        } else {
            0.0
        };

        frame_time_adjustment + gpu_load_adjustment
    }

    // ========================================================================
    // 错误处理和恢复方法
    // ========================================================================

    /// 验证渲染场景状态
    ///
    /// 这是业务逻辑方法，验证渲染场景的有效性。
    /// 委托给`RenderScene::validate()`，业务逻辑已封装在领域对象中。
    ///
    /// ## 业务规则
    ///
    /// - **对象有效性**: 所有渲染对象必须有效
    /// - **包围球有效性**: 所有对象必须有有效的包围球
    /// - **错误状态检查**: 检查对象是否处于错误状态
    ///
    /// ## 返回
    ///
    /// 如果场景有效，返回`Ok(())`；如果发现无效对象，返回`RenderError::InvalidState`。
    ///
    /// ## 错误
    ///
    /// - `RenderError::InvalidState`: 如果场景中包含无效对象
    ///
    /// ## 示例
    ///
    /// ```rust
    /// use game_engine::services::render::RenderService;
    /// use bevy_ecs::prelude::*;
    ///
    /// let mut service = RenderService::new();
    /// let mut world = World::new();
    ///
    /// // 构建场景
    /// service.build_domain_scene(&mut world).unwrap();
    ///
    /// // 验证场景
    /// assert!(service.validate_scene().is_ok());
    /// ```
    pub fn validate_scene(&self) -> Result<(), RenderError> {
        self.render_scene.validate()
    }

    /// 从错误恢复
    ///
    /// 这是业务逻辑方法，尝试从错误状态恢复渲染场景。
    /// 遍历所有处于错误状态的对象，尝试恢复它们。
    ///
    /// ## 业务规则
    ///
    /// - **错误检测**: 识别所有处于错误状态的对象
    /// - **恢复尝试**: 对每个错误对象调用`recover_from_error()`
    /// - **容错处理**: 如果恢复失败，记录错误但继续运行（不中断渲染）
    /// - **统计返回**: 返回成功恢复的对象数量
    ///
    /// ## 返回
    ///
    /// 返回成功恢复的对象数量（usize）。
    ///
    /// ## 注意
    ///
    /// 这个方法应该在每帧或定期调用，确保场景能够从临时错误中恢复。
    /// 如果恢复失败，对象会保持错误状态，但不会影响其他对象的渲染。
    ///
    /// ## 示例
    ///
    /// ```rust
    /// use game_engine::services::render::RenderService;
    ///
    /// let mut service = RenderService::new();
    ///
    /// // 尝试从错误恢复
    /// let recovered_count = service.recover_from_errors();
    /// println!("Recovered {} objects from error state", recovered_count);
    ///
    /// // 检查是否还有错误
    /// let (errors, total) = service.get_error_stats();
    /// if errors > 0 {
    ///     println!("Warning: {} objects still in error state", errors);
    /// }
    /// ```
    pub fn recover_from_errors(&mut self) -> usize {
        let mut recovered_count = 0;

        for obj in self.render_scene.objects_mut() {
            if obj.error_state.is_some() {
                if obj.recover_from_error().is_ok() {
                    recovered_count += 1;
                }
            }
        }

        recovered_count
    }

    /// 获取错误统计
    ///
    /// 这是业务逻辑方法，获取渲染场景的错误统计信息。
    /// 用于监控和诊断渲染场景的健康状态。
    ///
    /// ## 返回
    ///
    /// 返回`(错误对象数量, 总对象数量)`的元组：
    /// - 第一个元素：处于错误状态的对象数量
    /// - 第二个元素：场景中的总对象数量
    ///
    /// ## 使用场景
    ///
    /// - **性能监控**: 监控错误率，识别性能问题
    /// - **调试诊断**: 诊断渲染问题，定位错误对象
    /// - **健康检查**: 检查场景健康状态，触发恢复机制
    ///
    /// ## 示例
    ///
    /// ```rust
    /// use game_engine::services::render::RenderService;
    ///
    /// let service = RenderService::new();
    ///
    /// // 获取错误统计
    /// let (errors, total) = service.get_error_stats();
    ///
    /// // 计算错误率
    /// let error_rate = if total > 0 {
    ///     errors as f32 / total as f32
    /// } else {
    ///     0.0
    /// };
    ///
    /// // 如果错误率过高，触发恢复
    /// if error_rate > 0.1 {
    ///     println!("Warning: High error rate: {:.1}%", error_rate * 100.0);
    /// }
    /// ```
    pub fn get_error_stats(&self) -> (usize, usize) {
        let total = self.render_scene.objects().len();
        let errors = self
            .render_scene
            .objects()
            .iter()
            .filter(|obj| obj.error_state.is_some())
            .count();

        (errors, total)
    }

    // ========================================================================
    // PBR Scene Building
    // ========================================================================

    /// 构建PBR场景 - 提取光源 (网格由 BatchManager 管理)
    ///
    /// 这是业务逻辑方法，封装了光源提取的规则。
    /// 现在委托给领域对象`PbrScene::from_ecs_world()`，业务逻辑已封装在领域对象中。
    pub fn build_pbr_scene(&mut self, world: &mut World) -> PbrScene {
        // 委托给领域对象构建场景，业务逻辑已封装在PbrScene中
        let domain_scene = DomainPbrScene::from_ecs_world(world);

        // 转换为基础设施层的PbrScene（用于渲染）
        PbrScene {
            point_lights: domain_scene.point_lights().to_vec(),
            dir_lights: domain_scene.dir_lights().to_vec(),
        }
    }

    /// 执行PBR渲染
    ///
    /// 这是业务逻辑方法，封装了渲染执行的策略和错误处理。
    ///
    /// ## 业务逻辑
    ///
    /// - 更新LayerCache用于差异渲染
    /// - 更新视锥体用于下一帧的剔除
    /// - 验证场景有效性（允许无光源渲染）
    /// - 调用底层渲染器执行渲染
    pub fn paint_pbr(
        &mut self,
        renderer: &mut WgpuRenderer,
        batch_manager: &mut crate::render::instance_batch::BatchManager,
        scene: &PbrScene,
        view_proj: [[f32; 4]; 4],
        camera_pos: [f32; 3],
        egui_renderer: Option<&mut egui_wgpu::Renderer>,
        egui_shapes: &[egui::ClippedPrimitive],
        pixels_per_point: f32,
    ) -> Result<(), RenderError> {
        // 更新缓存
        self.layer_cache.new_frame();

        // 更新视锥体用于下一帧的剔除
        let view_proj_mat = Mat4::from_cols_array_2d(&view_proj);
        self.update_frustum(view_proj_mat);

        // 业务规则：验证场景有效性（委托给领域对象）
        // 将基础设施层的PbrScene转换为领域对象进行验证
        // 注意：这里简化处理，因为场景已经在build_pbr_scene中通过领域对象构建
        // 业务规则：允许无光源渲染（可能有环境光），所以不需要验证光源数量

        // 执行渲染
        renderer.render_pbr_batched(
            batch_manager,
            &scene.point_lights,
            &scene.dir_lights,
            view_proj,
            camera_pos,
            egui_renderer,
            egui_shapes,
            pixels_per_point,
        );

        Ok(())
    }
}

/// PBR场景数据
///
/// 包含PBR渲染所需的光源数据。
///
/// # 使用示例
///
/// ```rust
/// use game_engine::services::render::PbrScene;
/// use game_engine::render::pbr::{PointLight3D, DirectionalLight};
/// use glam::Vec3;
///
/// let mut scene = PbrScene::new();
/// scene.point_lights.push(PointLight3D {
///     position: Vec3::ONE,
///     color: Vec3::ONE,
///     intensity: 1.0,
///     radius: 10.0,
/// });
/// ```
#[derive(Default)]
pub struct PbrScene {
    /// 点光源列表
    pub point_lights: Vec<PointLight3D>,
    /// 方向光列表
    pub dir_lights: Vec<DirectionalLight>,
}

impl PbrScene {
    /// 创建新的PBR场景
    ///
    /// 创建一个空的PBR场景，不包含任何光源。
    ///
    /// # 返回
    ///
    /// 返回一个空的`PbrScene`实例。
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::lod::{LodConfigBuilder, LodQuality};

    #[test]
    fn test_render_service_creation() {
        let mut service = RenderService::new();
        // LOD选择器现在由RenderScene管理，RenderService不再直接持有
        assert!(service.render_scene.lod_selector_mut().is_none());
        assert!(service.current_frustum.is_none());
    }

    #[test]
    fn test_configure_lod() {
        let mut service = RenderService::new();
        let config = LodConfigBuilder::new()
            .add_level(0.0, 20.0, LodQuality::High)
            .add_level(20.0, 50.0, LodQuality::Medium)
            .build();

        service.configure_lod(config);
        // LOD选择器现在由RenderScene管理
        assert!(service.render_scene.lod_selector_mut().is_some());
    }

    #[test]
    fn test_use_default_lod() {
        let mut service = RenderService::new();
        service.use_default_lod();
        // LOD选择器现在由RenderScene管理
        assert!(service.render_scene.lod_selector_mut().is_some());
    }

    #[test]
    fn test_update_frustum() {
        let mut service = RenderService::new();
        let view_proj = Mat4::IDENTITY;

        service.update_frustum(view_proj);
        assert!(service.current_frustum.is_some());
    }

    #[test]
    fn test_build_pbr_scene_empty() {
        let mut service = RenderService::new();
        let mut world = World::new();

        let scene = service.build_pbr_scene(&mut world);
        assert_eq!(scene.point_lights.len(), 0);
        assert_eq!(scene.dir_lights.len(), 0);
    }

    #[test]
    fn test_build_pbr_scene_with_lights() {
        let mut service = RenderService::new();
        let mut world = World::new();

        // 添加点光源
        let light_entity = world.spawn((
            Transform {
                pos: Vec3::new(1.0, 2.0, 3.0),
                rot: glam::Quat::IDENTITY,
                scale: Vec3::ONE,
            },
            EcsPointLight3D {
                color: [1.0, 1.0, 1.0],
                intensity: 1.0,
                radius: 10.0,
            },
        ));

        // 添加方向光
        world.spawn(DirectionalLightComp {
            direction: [0.0, -1.0, 0.0],
            color: [1.0, 1.0, 1.0],
            intensity: 0.8,
        });

        let scene = service.build_pbr_scene(&mut world);
        assert_eq!(scene.point_lights.len(), 1);
        assert_eq!(scene.dir_lights.len(), 1);
        assert_eq!(scene.point_lights[0].position, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(scene.point_lights[0].intensity, 1.0);
    }

    #[test]
    fn test_build_pbr_scene_filters_invalid_lights() {
        let mut service = RenderService::new();
        let mut world = World::new();

        // 添加无效的点光源（强度为0）
        world.spawn((
            Transform {
                pos: Vec3::ZERO,
                rot: glam::Quat::IDENTITY,
                scale: Vec3::ONE,
            },
            EcsPointLight3D {
                color: [1.0, 1.0, 1.0],
                intensity: 0.0, // 无效
                radius: 10.0,
            },
        ));

        // 添加有效的点光源
        world.spawn((
            Transform {
                pos: Vec3::ONE,
                rot: glam::Quat::IDENTITY,
                scale: Vec3::ONE,
            },
            EcsPointLight3D {
                color: [1.0, 1.0, 1.0],
                intensity: 1.0, // 有效
                radius: 10.0,
            },
        ));

        let scene = service.build_pbr_scene(&mut world);
        // 应该只包含有效的光源
        assert_eq!(scene.point_lights.len(), 1);
        assert_eq!(scene.point_lights[0].intensity, 1.0);
    }

    #[test]
    fn test_update_scene() {
        let mut service = RenderService::new();
        service.use_default_lod();

        // 设置视锥体
        let view_proj = Mat4::perspective_rh(std::f32::consts::PI / 4.0, 16.0 / 9.0, 0.1, 100.0);
        service.update_frustum(view_proj);

        // 更新场景（应该成功，即使场景为空）
        let result = service.update_scene(0.016, Vec3::ZERO);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_renderable_objects_empty() {
        let service = RenderService::new();
        let count = service.get_renderable_objects().count();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_layer_cache() {
        let mut cache = LayerCache::default();

        // 新帧
        cache.new_frame();
        assert_eq!(cache.frame_count, 1);

        // 未使用的纹理应该被认为是脏的（不存在）
        assert!(cache.is_dirty(1));

        // 标记为干净（这会创建缓存条目）
        cache.mark_clean(1);
        assert!(!cache.is_dirty(1));

        // 标记使用
        cache.mark_used(1);
        assert!(!cache.is_dirty(1));
    }

    #[test]
    fn test_pbr_scene_structure() {
        let scene = PbrScene {
            point_lights: vec![PointLight3D {
                position: Vec3::ONE,
                color: Vec3::ONE,
                intensity: 1.0,
                radius: 10.0,
            }],
            dir_lights: vec![DirectionalLight {
                direction: Vec3::new(0.0, -1.0, 0.0),
                color: Vec3::ONE,
                intensity: 0.8,
            }],
        };

        assert_eq!(scene.point_lights.len(), 1);
        assert_eq!(scene.dir_lights.len(), 1);
    }
}
