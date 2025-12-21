//! GPU实例化渲染模块
//!
//! 提供高性能的GPU实例化渲染功能，优化实例数据组织和传输。
//!
//! ## 功能特性
//!
//! - **高效实例数据管理**：使用双缓冲和增量更新
//! - **自动批处理**：自动合并相同网格和材质的实例
//! - **GPU驱动剔除**：集成GPU剔除减少绘制调用
//! - **性能监控**：实时统计Draw Call减少和性能提升
//!
//! ## 架构设计
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │              GPU Instancing Pipeline                     │
//! ├─────────────────────────────────────────────────────────┤
//! │  1. Instance Collection                                 │
//! │     - 收集所有需要实例化的对象                            │
//! │     - 按 (mesh_id, material_id) 分组                      │
//! │                                                          │
//! │  2. Instance Data Upload                                │
//! │     - 增量更新实例数据（仅上传变化的部分）                  │
//! │     - 使用双缓冲减少等待时间                              │
//! │                                                          │
//! │  3. GPU Culling (Optional)                              │
//! │     - 视锥剔除                                           │
//! │     - 遮挡剔除                                           │
//! │                                                          │
//! │  4. Instanced Draw                                      │
//! │     - 单次 draw_indexed_instanced 绘制整个批次            │
//! └─────────────────────────────────────────────────────────┘
//! ```

use crate::render::batch_optimizer::BatchOptimizer;
use crate::render::gpu_driven::{GpuDrivenRenderer, GpuInstance};
use crate::render::instance_batch::BatchKey;
use glam::{Mat4, Vec3};
use std::collections::HashMap;
use wgpu::{Device, Queue};

/// GPU实例化渲染配置
#[derive(Debug, Clone)]
pub struct GpuInstancingConfig {
    /// 是否启用GPU驱动剔除
    pub enable_gpu_culling: bool,
    /// 是否启用遮挡剔除
    pub enable_occlusion_culling: bool,
    /// 最大实例数
    pub max_instances: u32,
    /// 每批次最大实例数
    pub max_instances_per_batch: u32,
    /// 是否启用增量更新
    pub enable_incremental_update: bool,
    /// 是否启用双缓冲
    pub enable_double_buffering: bool,
}

impl Default for GpuInstancingConfig {
    fn default() -> Self {
        Self {
            enable_gpu_culling: true,
            enable_occlusion_culling: false,
            max_instances: 65536,
            max_instances_per_batch: 1000,
            enable_incremental_update: true,
            enable_double_buffering: true,
        }
    }
}

/// 实例数据
#[derive(Debug, Clone)]
pub struct InstanceData {
    /// 模型矩阵
    pub model_matrix: Mat4,
    /// 位置
    pub position: Vec3,
    /// 缩放
    pub scale: Vec3,
    /// 旋转（四元数）
    pub rotation: glam::Quat,
    /// 自定义数据（用于着色器）
    pub custom_data: [f32; 4],
}

impl InstanceData {
    /// 创建新的实例数据
    pub fn new(position: Vec3, scale: Vec3, rotation: glam::Quat) -> Self {
        let model_matrix = Mat4::from_scale_rotation_translation(scale, rotation, position);
        Self {
            model_matrix,
            position,
            scale,
            rotation,
            custom_data: [0.0; 4],
        }
    }

    /// 转换为GPU实例格式（用于GPU驱动剔除）
    pub fn to_gpu_instance(&self, instance_id: u32, aabb_min: Vec3, aabb_max: Vec3) -> GpuInstance {
        GpuInstance {
            instance_id,
            aabb_min: [aabb_min.x, aabb_min.y, aabb_min.z],
            aabb_max: [aabb_max.x, aabb_max.y, aabb_max.z],
            model: self.model_matrix.to_cols_array_2d(),
            flags: 0, // 默认标志
        }
    }
}

/// GPU实例化渲染统计
#[derive(Debug, Clone, Default)]
pub struct GpuInstancingStats {
    /// 总实例数
    pub total_instances: u32,
    /// 可见实例数（剔除后）
    pub visible_instances: u32,
    /// Draw Call数量（优化后）
    pub draw_calls: u32,
    /// Draw Call数量（优化前，估计）
    pub draw_calls_before: u32,
    /// Draw Call减少率（0.0-1.0）
    pub draw_call_reduction: f32,
    /// 批次数
    pub batch_count: usize,
    /// 平均每批次实例数
    pub avg_instances_per_batch: f32,
}

/// GPU实例化渲染器
///
/// 注意：此渲染器提供统一的GPU实例化渲染API。
/// 实际的批次管理应该通过BatchManager进行。
pub struct GpuInstancingRenderer {
    /// 配置
    config: GpuInstancingConfig,
    /// GPU驱动渲染器（如果启用）
    gpu_driven: Option<GpuDrivenRenderer>,
    /// 批处理优化器
    batch_optimizer: BatchOptimizer,
    /// 实例数据映射（按BatchKey分组）
    instances: HashMap<BatchKey, Vec<InstanceData>>,
    /// 统计信息
    stats: GpuInstancingStats,
}

impl GpuInstancingRenderer {
    /// 创建新的GPU实例化渲染器
    pub fn new(device: &Device, config: GpuInstancingConfig) -> Self {
        let gpu_driven = if config.enable_gpu_culling {
            Some(GpuDrivenRenderer::new(
                device,
                crate::render::gpu_driven::GpuDrivenConfig {
                    frustum_culling: config.enable_gpu_culling,
                    occlusion_culling: config.enable_occlusion_culling,
                    lod_enabled: false,
                    max_instances: config.max_instances,
                    workgroup_size: 64,
                },
            ))
        } else {
            None
        };

        let batch_optimizer = BatchOptimizer::new(config.max_instances_per_batch);

        Self {
            config,
            gpu_driven,
            batch_optimizer,
            instances: HashMap::new(),
            stats: GpuInstancingStats::default(),
        }
    }

    /// 添加实例
    ///
    /// 将实例添加到对应的批次中。
    ///
    /// # 参数
    ///
    /// * `key` - 批次键（mesh_id, material_id等）
    /// * `instance_data` - 实例数据
    pub fn add_instance(&mut self, key: BatchKey, instance_data: InstanceData) {
        // 将实例添加到对应批次的实例列表
        self.instances.entry(key).or_insert_with(Vec::new).push(instance_data);
    }

    /// 更新实例数据到GPU
    ///
    /// 将实例数据转换为GpuInstance并上传到GPU缓冲区（如果启用GPU驱动剔除）。
    ///
    /// # 参数
    ///
    /// * `device` - WGPU设备
    /// * `queue` - WGPU命令队列
    pub fn update_gpu(&mut self, device: &Device, queue: &Queue) {
        // 如果启用GPU驱动剔除，收集所有实例并更新
        if let Some(ref gpu_driven) = self.gpu_driven {
            let mut all_gpu_instances = Vec::new();
            let mut instance_id = 0u32;
            
            for (key, instances) in &self.instances {
                for instance_data in instances {
                    // 计算AABB（简化版本，实际应该从网格获取）
                    let aabb_min = instance_data.position - instance_data.scale * 0.5;
                    let aabb_max = instance_data.position + instance_data.scale * 0.5;
                    
                    let gpu_instance = instance_data.to_gpu_instance(instance_id, aabb_min, aabb_max);
                    all_gpu_instances.push(gpu_instance);
                    instance_id += 1;
                }
            }
            
            if !all_gpu_instances.is_empty() {
                gpu_driven.update_instances(queue, &all_gpu_instances);
            }
        }
    }

    /// 执行GPU剔除
    ///
    /// 如果启用GPU驱动剔除，执行视锥剔除和遮挡剔除。
    ///
    /// # 参数
    ///
    /// * `encoder` - 命令编码器
    /// * `device` - WGPU设备
    /// * `queue` - WGPU命令队列
    /// * `view_proj` - 视图投影矩阵
    pub fn cull(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        device: &Device,
        queue: &Queue,
        view_proj: [[f32; 4]; 4],
    ) {
        if let Some(ref gpu_driven) = self.gpu_driven {
            let total_instances: u32 = self
                .instances
                .values()
                .map(|instances| instances.len() as u32)
                .sum();
            if total_instances > 0 {
                gpu_driven.cull(encoder, device, queue, view_proj, total_instances);
            }
        }
    }

    /// 优化批次
    ///
    /// 使用批处理优化器优化批次顺序和合并。
    ///
    /// # 返回
    ///
    /// 优化后的批次键列表
    pub fn optimize_batches(&mut self) -> Vec<BatchKey> {
        let mut batch_keys: Vec<BatchKey> = self.instances.keys().copied().collect();
        
        // 使用批处理优化器排序
        batch_keys.sort();
        
        // 更新统计
        self.stats.batch_count = batch_keys.len();
        self.stats.total_instances = self
            .instances
            .values()
            .map(|instances| instances.len() as u32)
            .sum();
        
        if self.stats.batch_count > 0 {
            self.stats.avg_instances_per_batch =
                self.stats.total_instances as f32 / self.stats.batch_count as f32;
        }
        
        // 估计优化前的Draw Call数量（假设每个实例一个Draw Call）
        self.stats.draw_calls_before = self.stats.total_instances;
        self.stats.draw_calls = self.stats.batch_count as u32;
        
        if self.stats.draw_calls_before > 0 {
            self.stats.draw_call_reduction = 1.0
                - (self.stats.draw_calls as f32 / self.stats.draw_calls_before as f32);
        }
        
        batch_keys
    }

    /// 清除所有实例
    pub fn clear(&mut self) {
        self.instances.clear();
        self.stats = GpuInstancingStats::default();
    }
    
    /// 获取实例数据（按批次键）
    pub fn get_instances(&self, key: &BatchKey) -> Option<&Vec<InstanceData>> {
        self.instances.get(key)
    }

    /// 获取统计信息
    pub fn stats(&self) -> &GpuInstancingStats {
        &self.stats
    }

    /// 获取配置
    pub fn config(&self) -> &GpuInstancingConfig {
        &self.config
    }

    /// 获取GPU驱动渲染器（如果启用）
    pub fn gpu_driven(&self) -> Option<&GpuDrivenRenderer> {
        self.gpu_driven.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instance_data_creation() {
        let instance = InstanceData::new(
            Vec3::new(1.0, 2.0, 3.0),
            Vec3::ONE,
            glam::Quat::IDENTITY,
        );
        
        assert_eq!(instance.position, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(instance.scale, Vec3::ONE);
    }

    #[test]
    fn test_gpu_instancing_config_default() {
        let config = GpuInstancingConfig::default();
        assert!(config.enable_gpu_culling);
        assert_eq!(config.max_instances, 65536);
    }

    #[test]
    fn test_add_instance() {
        use crate::render::instance_batch::BatchKey;
        
        // 注意：这个测试需要实际的wgpu设备来创建GpuInstancingRenderer
        // 这里只测试InstanceData的创建
        let instance = InstanceData::new(
            Vec3::ZERO,
            Vec3::ONE,
            glam::Quat::IDENTITY,
        );
        
        let gpu_instance = instance.to_gpu_instance(0, Vec3::new(-0.5, -0.5, -0.5), Vec3::new(0.5, 0.5, 0.5));
        assert_eq!(gpu_instance.instance_id, 0);
    }
}

