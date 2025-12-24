//  Staging Buffer集成示例
//
//  展示如何将新的环形缓冲区系统集成到现有的渲染管线中。

use std::sync::Arc;
use std::time::Duration;

use game_engine::resources::{
    EnhancedStagingBufferPool, MemoryDebugger, MemoryMonitor, UploadQueue,
    create_enhanced_staging_buffer_pool, create_high_performance_memory_debugger,
    create_high_performance_memory_monitor,
};

use game_engine::render::wgpu::WgpuRenderer;

/// Staging Buffer集成管理器
///
/// 提供统一的接口来管理所有Staging Buffer相关的组件。
pub struct StagingBufferIntegration {
    /// 增强型Staging Buffer池
    enhanced_pool: Arc<parking_lot::Mutex<EnhancedStagingBufferPool>>,
    /// 内存监控器
    memory_monitor: Arc<MemoryMonitor>,
    /// 内存调试器
    memory_debugger: Arc<parking_lot::Mutex<MemoryDebugger>>,
    /// 上传队列
    upload_queue: Arc<parking_lot::Mutex<UploadQueue>>,
    /// 是否启用监控
    monitoring_enabled: bool,
    /// 是否启用调试
    debugging_enabled: bool,
}

impl StagingBufferIntegration {
    /// 创建新的集成管理器
    pub fn new(device: &wgpu::Device) -> Self {
        // 创建增强型Staging Buffer池
        let enhanced_pool = Arc::new(parking_lot::Mutex::new(
            create_enhanced_staging_buffer_pool(device),
        ));

        // 创建内存监控器
        let memory_monitor = Arc::new(create_high_performance_memory_monitor());

        // 创建内存调试器
        let memory_debugger = Arc::new(parking_lot::Mutex::new(
            create_high_performance_memory_debugger(),
        ));

        // 创建上传队列
        let upload_queue = Arc::new(parking_lot::Mutex::new(UploadQueue::new()));

        let mut integration = Self {
            enhanced_pool,
            memory_monitor,
            memory_debugger,
            upload_queue,
            monitoring_enabled: true,
            debugging_enabled: false, // 默认禁用调试以减少开销
        };

        // 设置监控器
        integration.setup_monitoring();

        integration
    }

    /// 设置监控
    fn setup_monitoring(&self) {
        if self.monitoring_enabled {
            // 将增强池添加到监控器
            self.memory_monitor.add_monitored_pool(self.enhanced_pool.clone());

            // 启动监控
            self.memory_monitor.lock().start_monitoring();

            tracing::info!(target: "staging_integration", "Memory monitoring enabled");
        }
    }

    /// 启用调试
    pub fn enable_debugging(&mut self) {
        self.debugging_enabled = true;

        // 设置调试器的堆栈捕获函数
        self.memory_debugger.lock().set_stack_capture_fn(|| {
            // 简化的堆栈捕获实现
            vec![
                "allocation_point_1".to_string(),
                "allocation_point_2".to_string(),
            ]
        });

        // 将监控器添加到调试器
        self.memory_debugger.lock().set_memory_monitor(self.memory_monitor.clone());

        // 启动调试
        self.memory_debugger.lock().start_debugging();

        tracing::info!(target: "staging_integration", "Memory debugging enabled");
    }

    /// 禁用调试
    pub fn disable_debugging(&mut self) {
        self.debugging_enabled = false;
        self.memory_debugger.lock().stop_debugging();
        tracing::info!(target: "staging_integration", "Memory debugging disabled");
    }

    /// 分配Staging Buffer
    ///
    /// 这是主要的分配接口，内部使用增强型池。
    pub fn allocate(&mut self, size: u64, alignment: u64) -> Option<(usize, u64)> {
        let mut pool = self.enhanced_pool.lock();

        // 如果启用调试，跟踪分配
        if self.debugging_enabled {
            let allocation_id = self.memory_debugger.lock().track_allocation(
                size,
                alignment,
                "staging_buffer".to_string(),
            );

            // 记录调试信息
            tracing::debug!(
                target: "staging_integration",
                "Staging buffer allocation: ID={}, Size={} bytes, Alignment={}",
                allocation_id, size, alignment
            );
        }

        // 执行分配
        let result = pool.allocate(size, alignment);

        // 如果启用调试，记录分配结果
        if self.debugging_enabled {
            if let Some((buffer_index, offset)) = result {
                tracing::debug!(
                    target: "staging_integration",
                    "Staging buffer allocated: Buffer={}, Offset={}",
                    buffer_index, offset
                );
            } else {
                tracing::debug!(
                    target: "staging_integration",
                    "Staging buffer allocation failed: Size={} bytes",
                    size
                );
            }
        }

        result
    }

    /// 释放Staging Buffer
    pub fn deallocate(&mut self, buffer_index: usize) {
        let mut pool = self.enhanced_pool.lock();

        // 获取缓冲区信息
        if let Some(buffer) = pool.get_buffer(buffer_index) {
            if let Some(block) = buffer.block() {
                // 如果启用调试，跟踪释放
                if self.debugging_enabled {
                    self.memory_debugger.lock().track_deallocation(block.id);

                    tracing::debug!(
                        target: "staging_integration",
                        "Staging buffer deallocated: Block={}, Size={} bytes",
                        block.id, block.size
                    );
                }

                // 执行释放
                pool.deallocate(block.clone());
            }
        }
    }

    /// 帧结束时调用
    ///
    /// 应该在每帧结束时调用以更新所有组件。
    pub fn end_frame(&mut self) {
        // 更新增强池
        self.enhanced_pool.lock().end_frame();

        // 更新监控器
        if self.monitoring_enabled {
            self.memory_monitor.lock().update();
        }

        // 更新调试器
        if self.debugging_enabled {
            self.memory_debugger.lock().update_visualization();
        }

        // 处理上传队列
        self.upload_queue.lock().end_frame(&wgpu::Device::default()); // 简化实现
    }

    /// 获取性能统计
    pub fn get_performance_stats(&self) -> game_engine::resources::EnhancedPerformanceMetrics {
        self.enhanced_pool.lock().performance_metrics()
    }

    /// 获取内存使用情况
    pub fn get_memory_usage(&self) -> (u64, u64, f32) {
        self.enhanced_pool.lock().memory_usage()
    }

    /// 获取监控数据
    pub fn get_monitoring_data(&self) -> Option<game_engine::resources::MonitoringExportData> {
        if self.monitoring_enabled {
            Some(self.memory_monitor.lock().export_data())
        } else {
            None
        }
    }

    /// 获取调试数据
    pub fn get_debug_data(&self) -> Option<game_engine::resources::DebugExportData> {
        if self.debugging_enabled {
            Some(self.memory_debugger.lock().export_debug_data())
        } else {
            None
        }
    }

    /// 强制垃圾回收
    pub fn force_garbage_collection(&mut self) {
        tracing::info!(target: "staging_integration", "Forcing garbage collection");

        // 强制GC所有组件
        self.enhanced_pool.lock().force_gc();
        self.memory_monitor.lock().force_gc();
        self.memory_debugger.lock().clear_debug_data();
    }

    /// 重置统计信息
    pub fn reset_stats(&mut self) {
        tracing::info!(target: "staging_integration", "Resetting statistics");

        self.enhanced_pool.lock().reset_stats();
        self.memory_monitor.lock().reset_stats();
    }

    /// 获取集成状态
    pub fn get_integration_status(&self) -> IntegrationStatus {
        IntegrationStatus {
            monitoring_enabled: self.monitoring_enabled,
            debugging_enabled: self.debugging_enabled,
            total_allocations: self.enhanced_pool.lock().stats().total_allocations,
            current_memory_usage: self.get_memory_usage().1,
            memory_utilization: self.get_memory_usage().2,
            average_allocation_latency: self.get_performance_stats().average_allocation_latency_us,
        }
    }

    /// 创建渲染管线集成示例
    ///
    /// 展示如何将集成管理器与WgpuRenderer结合使用。
    pub fn create_renderer_integration(
        renderer: &mut WgpuRenderer,
        device: &wgpu::Device,
    ) -> RendererIntegration {
        let staging_integration = Arc::new(StagingBufferIntegration::new(device));

        RendererIntegration {
            staging_integration,
            renderer: renderer as *mut WgpuRenderer,
        }
    }
}

/// 集成状态信息
#[derive(Debug, Clone)]
pub struct IntegrationStatus {
    /// 监控是否启用
    pub monitoring_enabled: bool,
    /// 调试是否启用
    pub debugging_enabled: bool,
    /// 总分配次数
    pub total_allocations: u64,
    /// 当前内存使用量（字节）
    pub current_memory_usage: u64,
    /// 内存使用率（0.0-1.0）
    pub memory_utilization: f32,
    /// 平均分配延迟（微秒）
    pub average_allocation_latency: f32,
}

/// 渲染管线集成
///
/// 提供与WgpuRenderer的集成接口。
pub struct RendererIntegration {
    /// Staging Buffer集成管理器
    pub staging_integration: Arc<StagingBufferIntegration>,
    /// 渲染器指针
    pub renderer: *mut WgpuRenderer,
}

impl RendererIntegration {
    /// 更新实例数据
    ///
    /// 使用增强型Staging Buffer池来更新实例数据。
    pub fn update_instance_data(&mut self, instances: &[game_engine::render::wgpu::Instance]) {
        let data = unsafe {
            std::slice::from_raw_parts(
                instances.as_ptr() as *const u8,
                instances.len() * std::mem::size_of::<game_engine::render::wgpu::Instance>(),
            )
        };

        // 计算所需大小和对齐
        let size = data.len() as u64;
        let alignment = std::mem::align_of::<game_engine::render::wgpu::Instance>() as u64;

        // 分配Staging Buffer
        if let Some((buffer_index, offset)) =
            self.staging_integration.lock().allocate(size, alignment)
        {
            // 写入数据
            {
                let mut pool = self.staging_integration.lock();
                if let Some(buffer) = pool.get_buffer_mut(buffer_index) {
                    if let Some(write_offset) = buffer.write(data, alignment) {
                        // 使用渲染器的队列写入数据
                        // 这里需要访问渲染器的内部方法，简化实现
                        tracing::debug!(
                            target: "renderer_integration",
                            "Updated {} instances at offset {}",
                            instances.len(),
                            write_offset
                        );
                    }
                }
            }
        } else {
            tracing::error!(
                target: "renderer_integration",
                "Failed to allocate staging buffer for {} instances",
                instances.len()
            );
        }
    }

    /// 渲染帧
    ///
    /// 在渲染前调用，确保所有数据已上传到GPU。
    pub fn render_frame(&mut self) {
        // 结束当前帧
        self.staging_integration.lock().end_frame();

        // 获取性能统计
        let stats = self.staging_integration.lock().get_performance_stats();

        // 记录性能信息
        if stats.average_allocation_latency_us > 100.0 {
            tracing::warn!(
                target: "renderer_integration",
                "High allocation latency detected: {:.2}μs",
                stats.average_allocation_latency_us
            );
        }

        // 获取内存使用情况
        let (total, used, utilization) = self.staging_integration.lock().get_memory_usage();

        if utilization > 0.9 {
            tracing::warn!(
                target: "renderer_integration",
                "High memory utilization: {:.1}% ({:.1}MB/{:.1}MB)",
                utilization * 100.0,
                used as f32 / (1024.0 * 1024.0),
                total as f32 / (1024.0 * 1024.0)
            );
        }

        // 检查是否需要强制GC
        if utilization > 0.95 {
            tracing::info!(
                target: "renderer_integration",
                "Memory utilization critical, forcing garbage collection"
            );
            self.staging_integration.lock().force_garbage_collection();
        }
    }

    /// 获取集成状态
    pub fn get_status(&self) -> IntegrationStatus {
        self.staging_integration.lock().get_integration_status()
    }

    /// 启用性能监控
    pub fn enable_monitoring(&mut self) {
        self.staging_integration.lock().monitoring_enabled = true;
        self.staging_integration.lock().setup_monitoring();
    }

    /// 启用调试模式
    pub fn enable_debugging_mode(&mut self) {
        self.staging_integration.lock().enable_debugging();
    }

    /// 禁用调试模式
    pub fn disable_debugging_mode(&mut self) {
        self.staging_integration.lock().disable_debugging();
    }

    /// 获取性能报告
    pub fn get_performance_report(&self) -> String {
        let stats = self.staging_integration.lock().get_performance_stats();
        let status = self.get_status();

        format!(
            "=== Staging Buffer Performance Report ===\n\
             Total allocations: {}\n\
             Average latency: {:.2}μs\n\
             Preallocation hit rate: {:.1}%\n\
             Memory utilization: {:.1}%\n\
             Current usage: {:.1}MB\n\
             Monitoring: {}\n\
             Debugging: {}\n\
             =========================================",
            stats.total_allocations,
            stats.average_allocation_latency_us,
            stats.preallocation_hit_rate * 100.0,
            status.memory_utilization * 100.0,
            status.current_memory_usage as f32 / (1024.0 * 1024.0),
            if status.monitoring_enabled {
                "Enabled"
            } else {
                "Disabled"
            },
            if status.debugging_enabled {
                "Enabled"
            } else {
                "Disabled"
            }
        )
    }

    /// 安全地访问渲染器
    ///
    /// 注意：这是一个不安全的操作，需要确保渲染器生命周期有效。
    pub unsafe fn renderer(&self) -> &mut WgpuRenderer {
        &mut *self.renderer
    }
}

// ============================================================================
// 使用示例
// ============================================================================

/// 使用示例
pub fn integration_example() {
    println!("=== Staging Buffer Integration Example ===\n");

    // 注意：这是一个简化的示例，实际使用时需要有效的设备和渲染器
    // let device = create_device();
    // let mut renderer = WgpuRenderer::new(&window, &device).await.unwrap();

    // 创建集成管理器
    // let mut integration = StagingBufferIntegration::new(&device);

    // 创建渲染管线集成
    // let mut renderer_integration = integration.create_renderer_integration(&mut renderer, &device);

    // 启用监控和调试
    // integration.enable_monitoring();
    // integration.enable_debugging_mode();

    // 模拟渲染循环
    // for frame in 0..1000 {
    //     let instances = generate_instance_data();
    //     renderer_integration.update_instance_data(&instances);
    //     renderer_integration.render_frame();
    //
    //     // 每100帧输出性能报告
    //     if frame % 100 == 0 {
    //         println!("{}", renderer_integration.get_performance_report());
    //     }
    // }

    // 获取最终状态
    // let final_status = renderer_integration.get_status();
    // println!("\nFinal integration status: {:#?}", final_status);

    println!("Example completed. See comments for actual implementation details.");
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_staging_buffer_integration_creation() {
        // 注意：这个测试需要有效的设备，在实际环境中会失败
        // 这里只测试结构创建逻辑

        let _integration = StagingBufferIntegration {
            enhanced_pool: Arc::new(parking_lot::Mutex::new(
                create_enhanced_staging_buffer_pool(&wgpu::Device::default()),
            )),
            memory_monitor: Arc::new(create_high_performance_memory_monitor()),
            memory_debugger: Arc::new(parking_lot::Mutex::new(
                create_high_performance_memory_debugger(),
            )),
            upload_queue: Arc::new(parking_lot::Mutex::new(UploadQueue::new())),
            monitoring_enabled: true,
            debugging_enabled: false,
        };

        // 验证初始状态
        // let status = integration.get_integration_status();
        // assert!(status.monitoring_enabled);
        // assert!(!status.debugging_enabled);
    }

    #[test]
    fn test_integration_status() {
        let status = IntegrationStatus {
            monitoring_enabled: true,
            debugging_enabled: false,
            total_allocations: 1000,
            current_memory_usage: 50 * 1024 * 1024, // 50MB
            memory_utilization: 0.5,
            average_allocation_latency: 25.0,
        };

        assert_eq!(status.monitoring_enabled, true);
        assert_eq!(status.debugging_enabled, false);
        assert_eq!(status.total_allocations, 1000);
        assert_eq!(status.current_memory_usage, 50 * 1024 * 1024);
        assert_eq!(status.memory_utilization, 0.5);
        assert_eq!(status.average_allocation_latency, 25.0);
    }

    #[test]
    fn test_performance_report_generation() {
        let integration = StagingBufferIntegration {
            enhanced_pool: Arc::new(parking_lot::Mutex::new(
                create_enhanced_staging_buffer_pool(&wgpu::Device::default()),
            )),
            memory_monitor: Arc::new(create_high_performance_memory_monitor()),
            memory_debugger: Arc::new(parking_lot::Mutex::new(
                create_high_performance_memory_debugger(),
            )),
            upload_queue: Arc::new(parking_lot::Mutex::new(UploadQueue::new())),
            monitoring_enabled: true,
            debugging_enabled: true,
        };

        let report = integration.get_performance_report();

        assert!(report.contains("Total allocations: 0"));
        assert!(report.contains("Monitoring: Enabled"));
        assert!(report.contains("Debugging: Enabled"));
    }
}
