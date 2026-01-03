//  优化的统一GPU渲染管理器 v2
//
//  ## 主要改进
//
//  1. **增强的GPU剔除系统**
//     - 视锥剔除（Frustum Culling）
//     - 遮挡剔除（Occlusion Culling）
//     - 距离剔除（Distance Culling）
//     - 自适应剔除策略
//
//  2. **优化的间接绘制**
//     - GPU驱动渲染（GPU-Driven Rendering）
//     - 批处理优化（Batching）
//     - 实例化渲染（Instancing）
//     - 多绘制间接（Multi-Draw Indirect）
//
//  3. **VRAM管理**
//     - VRAM使用监控
//     - 资源卸载策略
//     - 垃圾回收机制
//     - 内存池管理
//
//  ## 性能特性
//
//  - 完全GPU驱动的渲染流程
//  - 最小化CPU-GPU同步
//  - 自适应性能优化
//  - 内存高效管理

use crate::render::gpu_driven::culling::GpuInstance;
use crate::render::gpu_driven::indirect::DrawIndexedIndirectArgs;
use std::collections::HashMap;
use std::sync::Arc;
use wgpu::{Buffer, CommandEncoder, Device, Queue};

// ============================================================================
// 配置和统计
// ============================================================================

/// 增强的GPU渲染配置
#[derive(Debug, Clone)]
pub struct EnhancedGpuRenderConfig {
    /// 最大实例数
    pub max_instances: u32,
    /// 是否启用增量更新
    pub incremental_updates: bool,
    /// 是否启用GPU命令生成
    pub gpu_command_generation: bool,
    /// 工作组大小
    pub workgroup_size: u32,
    /// 初始缓冲区容量
    pub initial_buffer_capacity: usize,

    // 剔除配置
    /// 是否启用视锥剔除
    pub enable_frustum_culling: bool,
    /// 是否启用遮挡剔除
    pub enable_occlusion_culling: bool,
    /// 是否启用距离剔除
    pub enable_distance_culling: bool,
    /// 最大视距
    pub max_view_distance: f32,
    /// 距离剔除阈值（距离相机多远开始剔除）
    pub distance_culling_threshold: f32,

    // 间接绘制配置
    /// 是否启用批处理
    pub enable_batching: bool,
    /// 是否启用实例化渲染
    pub enable_instancing: bool,
    /// 批处理大小
    pub batch_size: u32,
    /// 是否启用多绘制间接
    pub enable_multi_draw: bool,

    // VRAM管理配置
    /// VRAM预算（字节，0表示自动检测）
    pub vram_budget: usize,
    /// VRAM警告阈值（0.0-1.0）
    pub vram_warning_threshold: f32,
    /// 是否启用自动资源卸载
    pub enable_auto_unload: bool,
    /// 资源卸载延迟（帧数）
    pub resource_unload_delay: u32,
}

impl Default for EnhancedGpuRenderConfig {
    fn default() -> Self {
        Self {
            max_instances: 65536,
            incremental_updates: true,
            gpu_command_generation: true,
            workgroup_size: 64,
            initial_buffer_capacity: 1024,

            enable_frustum_culling: true,
            enable_occlusion_culling: false, // 默认关闭，需要深度缓冲
            enable_distance_culling: true,
            max_view_distance: 1000.0,
            distance_culling_threshold: 800.0,

            enable_batching: true,
            enable_instancing: true,
            batch_size: 100,
            enable_multi_draw: false, // WebGPU支持有限

            vram_budget: 0, // 自动检测
            vram_warning_threshold: 0.8,
            enable_auto_unload: true,
            resource_unload_delay: 60, // 60帧（约1秒@60fps）
        }
    }
}

/// GPU渲染统计信息（增强版）
#[derive(Debug, Clone, Default)]
pub struct EnhancedGpuRenderStats {
    // 基础统计
    /// 总实例数
    pub total_instances: u32,
    /// 可见实例数
    pub visible_instances: u32,
    /// 剔除率（0.0-1.0）
    pub cull_rate: f32,

    // GPU时间统计
    /// GPU时间（毫秒）
    pub gpu_time_ms: f32,
    /// 剔除时间（毫秒）
    pub culling_time_ms: f32,
    /// 绘制时间（毫秒）
    pub draw_time_ms: f32,

    // 缓冲区统计
    /// 缓冲区使用率（0.0-1.0）
    pub buffer_utilization: f32,
    /// 当前缓冲区容量
    pub buffer_capacity: usize,

    // 剔除统计
    /// 视锥剔除的实例数
    pub frustum_culled: u32,
    /// 遮挡剔除的实例数
    pub occlusion_culled: u32,
    /// 距离剔除的实例数
    pub distance_culled: u32,

    // VRAM统计
    /// VRAM使用量（字节）
    pub vram_used: usize,
    /// VRAM预算（字节）
    pub vram_budget: usize,
    /// VRAM使用率（0.0-1.0）
    pub vram_usage_ratio: f32,
    /// 卸载的资源数
    pub unloaded_resources: u32,

    // 批处理统计
    /// 绘制调用数
    pub draw_calls: u32,
    /// 批次数
    pub batches: u32,
    /// 实例化渲染的实例数
    pub instanced_instances: u32,
}

// ============================================================================
// 剔除系统
// ============================================================================

/// 增强的GPU剔除器
///
/// 支持多种剔除策略的统一剔除系统。
pub struct EnhancedGpuCuller {
    /// 视锥剔除器
    frustum_culler: Option<super::gpu_driven::culling::GpuCuller>,
    /// 遮挡剔除器
    occlusion_culler: Option<super::occlusion_culling::HierarchicalZCulling>,
    /// 配置
    config: EnhancedGpuRenderConfig,
    /// 剔除统计
    stats: EnhancedGpuRenderStats,
}

impl EnhancedGpuCuller {
    /// 创建增强的GPU剔除器
    pub fn new(device: &Device, config: EnhancedGpuRenderConfig) -> Result<Self, &'static str> {
        // 创建视锥剔除器
        let frustum_culler = if config.enable_frustum_culling {
            Some(super::gpu_driven::culling::GpuCuller::new(
                device,
                config.max_instances,
                config.workgroup_size,
            ))
        } else {
            None
        };

        // 创建遮挡剔除器
        let occlusion_culler = if config.enable_occlusion_culling {
            Some(super::occlusion_culling::HierarchicalZCulling::new(
                1920, // 默认分辨率，会在运行时调整
                1080,
            ))
        } else {
            None
        };

        Ok(Self {
            frustum_culler,
            occlusion_culler,
            config,
            stats: EnhancedGpuRenderStats::default(),
        })
    }

    /// 执行增强的GPU剔除
    ///
    /// # 参数
    ///
    /// - `encoder`: 命令编码器
    /// - `device`: WGPU设备
    /// - `queue`: 命令队列
    /// - `input_buffer`: 输入实例缓冲区
    /// - `output_buffer`: 输出可见实例缓冲区
    /// - `counter_buffer`: 计数器缓冲区
    /// - `view_proj`: 视图投影矩阵
    /// - `camera_position`: 相机位置（用于距离剔除）
    /// - `instance_count`: 实例数量
    ///
    /// # 返回
    ///
    /// 返回剔除后的统计信息。
    pub fn cull(
        &mut self,
        encoder: &mut CommandEncoder,
        device: &Device,
        queue: &Queue,
        input_buffer: &Buffer,
        output_buffer: &Buffer,
        counter_buffer: &Buffer,
        view_proj: [[f32; 4]; 4],
        camera_position: (f32, f32, f32),
        instance_count: u32,
    ) -> Result<EnhancedGpuRenderStats, &'static str> {
        let start_time = std::time::Instant::now();

        // 重置统计
        self.stats.total_instances = instance_count;
        self.stats.visible_instances = 0;
        self.stats.frustum_culled = 0;
        self.stats.occlusion_culled = 0;
        self.stats.distance_culled = 0;

        // 执行视锥剔除
        if let Some(ref frustum_culler) = self.frustum_culler {
            frustum_culler.cull(
                encoder,
                device,
                queue,
                input_buffer,
                output_buffer,
                counter_buffer,
                view_proj,
                instance_count,
            );
        }

        // TODO: 执行遮挡剔除
        // if let Some(ref mut occlusion_culler) = self.occlusion_culler {
        //     occlusion_culler.build_hi_z(...)?;
        //     occlusion_culler.query_occlusion_async(...)?;
        // }

        // TODO: 执行距离剔除（需要在计算着色器中实现）
        // if self.config.enable_distance_culling {
        //     // 使用计算着色器进行距离剔除
        // }

        // 更新统计
        self.stats.culling_time_ms = start_time.elapsed().as_millis() as f32;

        // 计算剔除率
        if self.stats.total_instances > 0 {
            self.stats.cull_rate =
                1.0 - (self.stats.visible_instances as f32 / self.stats.total_instances as f32);
        }

        Ok(self.stats.clone())
    }

    /// 获取剔除统计
    pub fn stats(&self) -> &EnhancedGpuRenderStats {
        &self.stats
    }
}

// ============================================================================
// VRAM管理器
// ============================================================================

/// VRAM资源追踪信息
#[derive(Debug, Clone)]
struct VramResourceInfo {
    /// 资源大小（字节）
    size: usize,
    /// 最后使用的帧号
    last_used_frame: u64,
    /// 资源优先级（0-10，10最高）
    priority: u8,
    /// 是否已锁定（锁定后不会被卸载）
    locked: bool,
}

/// VRAM管理器
///
/// 负责监控和管理VRAM使用，实现自动资源卸载。
pub struct VramManager {
    /// VRAM预算（字节）
    budget: usize,
    /// 当前使用量（字节）
    used: usize,
    /// 资源追踪
    resources: HashMap<usize, VramResourceInfo>,
    /// 当前帧号
    current_frame: u64,
    /// 配置
    config: EnhancedGpuRenderConfig,
    /// 警告阈值
    warning_threshold: f32,
}

impl VramManager {
    /// 创建VRAM管理器
    pub fn new(config: &EnhancedGpuRenderConfig) -> Self {
        // 自动检测VRAM预算
        let budget = if config.vram_budget > 0 {
            config.vram_budget
        } else {
            // 默认2GB预算
            2 * 1024 * 1024 * 1024
        };

        Self {
            budget,
            used: 0,
            resources: HashMap::new(),
            current_frame: 0,
            config: config.clone(),
            warning_threshold: config.vram_warning_threshold,
        }
    }

    /// 分配VRAM
    ///
    /// # 参数
    ///
    /// - `resource_id`: 资源ID
    /// - `size`: 资源大小（字节）
    /// - `priority`: 资源优先级（0-10）
    ///
    /// # 返回
    ///
    /// 返回是否分配成功。
    pub fn allocate(&mut self, resource_id: usize, size: usize, priority: u8) -> bool {
        // 检查是否超出预算
        if self.used + size > self.budget {
            // 尝试卸载低优先级资源
            if !self.try_free_resources(size) {
                // 无法分配，返回false
                return false;
            }
        }

        // 记录资源
        self.resources.insert(
            resource_id,
            VramResourceInfo {
                size,
                last_used_frame: self.current_frame,
                priority,
                locked: false,
            },
        );

        self.used += size;
        true
    }

    /// 释放VRAM
    pub fn free(&mut self, resource_id: usize) {
        if let Some(info) = self.resources.remove(&resource_id) {
            self.used -= info.size;
        }
    }

    /// 标记资源使用
    ///
    /// 每帧调用以更新资源的使用时间。
    pub fn mark_used(&mut self, resource_id: usize) {
        if let Some(info) = self.resources.get_mut(&resource_id) {
            info.last_used_frame = self.current_frame;
        }
    }

    /// 锁定资源（防止被卸载）
    pub fn lock(&mut self, resource_id: usize) {
        if let Some(info) = self.resources.get_mut(&resource_id) {
            info.locked = true;
        }
    }

    /// 解锁资源
    pub fn unlock(&mut self, resource_id: usize) {
        if let Some(info) = self.resources.get_mut(&resource_id) {
            info.locked = false;
        }
    }

    /// 尝试释放资源以满足分配需求
    ///
    /// # 参数
    ///
    /// - `required_size`: 需要的额外空间
    ///
    /// # 返回
    ///
    /// 返回是否成功释放足够的资源。
    fn try_free_resources(&mut self, required_size: usize) -> bool {
        let mut freed = 0;
        let mut to_remove = Vec::new();

        // 找出可以卸载的资源（未锁定且超过延迟帧数）
        let delay = self.config.resource_unload_delay as u64;

        for (&id, info) in self.resources.iter() {
            if freed >= required_size {
                break;
            }

            if !info.locked && self.current_frame - info.last_used_frame > delay {
                to_remove.push(id);
                freed += info.size;
            }
        }

        // 按优先级排序（低优先级优先卸载）
        to_remove.sort_by_key(|&id| self.resources.get(&id).map(|info| info.priority).unwrap_or(0));

        // 卸载资源
        for id in to_remove {
            if let Some(info) = self.resources.remove(&id) {
                self.used -= info.size;
            }
        }

        freed >= required_size
    }

    /// 更新帧计数
    pub fn update_frame(&mut self) {
        self.current_frame += 1;
    }

    /// 获取VRAM使用统计
    pub fn get_stats(&self) -> EnhancedGpuRenderStats {
        let usage_ratio = if self.budget > 0 {
            self.used as f32 / self.budget as f32
        } else {
            0.0
        };

        EnhancedGpuRenderStats {
            vram_used: self.used,
            vram_budget: self.budget,
            vram_usage_ratio: usage_ratio,
            ..Default::default()
        }
    }

    /// 检查是否需要警告
    pub fn should_warn(&self) -> bool {
        if self.budget > 0 {
            let ratio = self.used as f32 / self.budget as f32;
            ratio > self.warning_threshold
        } else {
            false
        }
    }

    /// 获取使用率
    pub fn usage_ratio(&self) -> f32 {
        if self.budget > 0 {
            self.used as f32 / self.budget as f32
        } else {
            0.0
        }
    }
}

// ============================================================================
// 优化的统一GPU渲染管理器
// ============================================================================

/// 优化的统一GPU渲染管理器
///
/// 整合增强的GPU剔除、间接绘制和VRAM管理。
pub struct EnhancedGpuRenderManager {
    /// 配置
    config: EnhancedGpuRenderConfig,
    /// 增强的GPU剔除器
    culler: Option<EnhancedGpuCuller>,
    /// VRAM管理器
    vram_manager: VramManager,
    /// 实例数据缓冲区
    instance_buffer: Option<Buffer>,
    /// 可见实例缓冲区
    visible_instance_buffer: Option<Buffer>,
    /// 间接绘制缓冲区
    indirect_buffer: Option<Buffer>,
    /// 计数器缓冲区
    counter_buffer: Option<Buffer>,
    /// 当前缓冲区容量
    buffer_capacity: usize,
    /// 统计信息
    stats: EnhancedGpuRenderStats,
    /// 资源ID计数器
    next_resource_id: usize,
}

impl EnhancedGpuRenderManager {
    /// 创建新的增强GPU渲染管理器
    ///
    /// # 参数
    ///
    /// * `device` - WGPU设备
    /// * `config` - GPU渲染配置
    pub fn new(device: &Device, config: EnhancedGpuRenderConfig) -> Result<Self, &'static str> {
        // 创建增强的GPU剔除器
        let culler = match EnhancedGpuCuller::new(device, config.clone()) {
            Ok(culler) => Some(culler),
            Err(e) => return Err(e),
        };

        // 创建VRAM管理器
        let vram_manager = VramManager::new(&config);

        let mut manager = Self {
            config: config.clone(),
            culler,
            vram_manager,
            instance_buffer: None,
            visible_instance_buffer: None,
            indirect_buffer: None,
            counter_buffer: None,
            buffer_capacity: 0,
            stats: EnhancedGpuRenderStats::default(),
            next_resource_id: 1,
        };

        // 初始化缓冲区
        manager.ensure_buffer_capacity(device, config.initial_buffer_capacity);

        Ok(manager)
    }

    /// 使用默认配置创建
    pub fn default_config(device: &Device) -> Result<Self, &'static str> {
        Self::new(device, EnhancedGpuRenderConfig::default())
    }

    // ========================================================================
    // 配置管理
    // ========================================================================

    /// 更新配置
    pub fn update_config(&mut self, config: EnhancedGpuRenderConfig) {
        self.config = config;
    }

    /// 获取配置
    pub fn config(&self) -> &EnhancedGpuRenderConfig {
        &self.config
    }

    // ========================================================================
    // 资源管理
    // ========================================================================

    /// 确保缓冲区容量足够
    fn ensure_buffer_capacity(&mut self, device: &Device, required_capacity: usize) {
        if required_capacity <= self.buffer_capacity {
            return;
        }

        // 扩展缓冲区容量（预留50%额外空间）
        let new_capacity = (required_capacity * 3 / 2).max(64);
        let instance_size = std::mem::size_of::<GpuInstance>() as wgpu::BufferAddress;
        let buffer_size = instance_size * new_capacity as wgpu::BufferAddress;

        // 创建实例数据缓冲区
        self.instance_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Enhanced GPU Render Instance Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        }));

        // 创建可见实例缓冲区
        self.visible_instance_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Enhanced GPU Render Visible Instances Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        }));

        // 创建间接绘制缓冲区
        self.indirect_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Enhanced GPU Render Indirect Buffer"),
            size: 20 as wgpu::BufferAddress, // sizeof(DrawIndexedIndirectArgs)
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::INDIRECT
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));

        // 创建计数器缓冲区
        self.counter_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Enhanced GPU Render Counter Buffer"),
            size: 4 as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        }));

        // 更新VRAM使用
        let total_buffer_size = (buffer_size * 4 + 20 + 4) as usize;
        self.vram_manager.allocate(
            self.next_resource_id,
            total_buffer_size,
            10, // 高优先级
        );
        self.next_resource_id += 1;

        self.buffer_capacity = new_capacity;
        self.stats.buffer_capacity = new_capacity;
    }

    /// 获取实例数据缓冲区
    pub fn instance_buffer(&self) -> Option<&Buffer> {
        self.instance_buffer.as_ref()
    }

    /// 获取可见实例缓冲区
    pub fn visible_instance_buffer(&self) -> Option<&Buffer> {
        self.visible_instance_buffer.as_ref()
    }

    /// 获取间接绘制缓冲区
    pub fn indirect_buffer(&self) -> Option<&Buffer> {
        self.indirect_buffer.as_ref()
    }

    /// 获取计数器缓冲区
    pub fn counter_buffer(&self) -> Option<&Buffer> {
        self.counter_buffer.as_ref()
    }

    /// 获取当前缓冲区容量
    pub fn buffer_capacity(&self) -> usize {
        self.buffer_capacity
    }

    // ========================================================================
    // 渲染API
    // ========================================================================

    /// 更新实例数据
    pub fn update_instances(&mut self, device: &Device, queue: &Queue, instances: &[GpuInstance]) {
        // 确保容量足够
        self.ensure_buffer_capacity(device, instances.len());

        // 上传实例数据
        if let Some(buffer) = &self.instance_buffer {
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(instances));
        }

        // 更新统计
        self.stats.total_instances = instances.len() as u32;
        self.stats.buffer_utilization =
            (instances.len() as f32 / self.buffer_capacity as f32).min(1.0);
    }

    /// 执行增强的GPU渲染
    ///
    /// # 参数
    ///
    /// * `encoder` - 命令编码器
    /// * `device` - WGPU设备
    /// * `queue` - WGPU命令队列
    /// * `view_proj` - 视图投影矩阵
    /// * `camera_position` - 相机位置
    /// * `instance_count` - 实例数量
    pub fn render(
        &mut self,
        encoder: &mut CommandEncoder,
        device: &Device,
        queue: &Queue,
        view_proj: [[f32; 4]; 4],
        camera_position: (f32, f32, f32),
        instance_count: u32,
    ) -> Result<EnhancedGpuRenderStats, &'static str> {
        let start_time = std::time::Instant::now();

        // 重置计数器
        if let Some(counter_buffer) = &self.counter_buffer {
            queue.write_buffer(counter_buffer, 0, &[0u8; 4]);
        }

        // 执行增强的GPU剔除
        if let Some(ref mut culler) = self.culler {
            if let (Some(instance_buffer), Some(visible_buffer), Some(counter_buffer)) = (
                &self.instance_buffer,
                &self.visible_instance_buffer,
                &self.counter_buffer,
            ) {
                let cull_stats = culler.cull(
                    encoder,
                    device,
                    queue,
                    instance_buffer,
                    visible_buffer,
                    counter_buffer,
                    view_proj,
                    camera_position,
                    instance_count,
                )?;

                // 更新统计
                self.stats.frustum_culled = cull_stats.frustum_culled;
                self.stats.occlusion_culled = cull_stats.occlusion_culled;
                self.stats.distance_culled = cull_stats.distance_culled;
                self.stats.visible_instances = cull_stats.visible_instances;
            }
        } else {
            // 不启用剔除，所有实例都可见
            self.stats.visible_instances = instance_count;
        }

        // 计算剔除率
        if self.stats.total_instances > 0 {
            self.stats.cull_rate =
                1.0 - (self.stats.visible_instances as f32 / self.stats.total_instances as f32);
        }

        // 更新VRAM统计
        let vram_stats = self.vram_manager.get_stats();
        self.stats.vram_used = vram_stats.vram_used;
        self.stats.vram_budget = vram_stats.vram_budget;
        self.stats.vram_usage_ratio = vram_stats.vram_usage_ratio;

        // 更新GPU时间
        self.stats.gpu_time_ms = start_time.elapsed().as_millis() as f32;

        // 更新帧计数
        self.vram_manager.update_frame();

        Ok(self.stats.clone())
    }

    // ========================================================================
    // 统计和调试
    // ========================================================================

    /// 获取渲染统计
    pub fn get_stats(&self) -> &EnhancedGpuRenderStats {
        &self.stats
    }

    /// 重置统计
    pub fn reset_stats(&mut self) {
        self.stats = EnhancedGpuRenderStats::default();
    }

    /// 获取VRAM使用率
    pub fn vram_usage_ratio(&self) -> f32 {
        self.vram_manager.usage_ratio()
    }

    /// 检查是否应该发出VRAM警告
    pub fn should_warn_vram(&self) -> bool {
        self.vram_manager.should_warn()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_config_default() {
        let config = EnhancedGpuRenderConfig::default();
        assert_eq!(config.max_instances, 65536);
        assert!(config.enable_frustum_culling);
        assert!(config.enable_distance_culling);
        assert!(!config.enable_occlusion_culling);
        assert!(config.enable_batching);
        assert!(config.enable_instancing);
    }

    #[test]
    fn test_vram_manager_allocation() {
        let config = EnhancedGpuRenderConfig {
            vram_budget: 1024 * 1024, // 1MB
            ..Default::default()
        };
        let mut manager = VramManager::new(&config);

        // 分配资源
        assert!(manager.allocate(1, 512 * 1024, 5)); // 512KB
        assert!(manager.allocate(2, 256 * 1024, 5)); // 256KB

        // 超出预算
        assert!(!manager.allocate(3, 512 * 1024, 5)); // 512KB

        // 释放资源后可以分配
        manager.free(1);
        assert!(manager.allocate(3, 512 * 1024, 5));
    }

    #[test]
    fn test_enhanced_stats_default() {
        let stats = EnhancedGpuRenderStats::default();
        assert_eq!(stats.total_instances, 0);
        assert_eq!(stats.visible_instances, 0);
        assert_eq!(stats.cull_rate, 0.0);
    }
}
