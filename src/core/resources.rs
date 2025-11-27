//! 核心资源类型
//!
//! 定义引擎运行时使用的ECS资源

use bevy_ecs::prelude::*;
use std::collections::VecDeque;

/// 基准测试配置
#[derive(Resource, Default)]
pub struct Benchmark {
    /// 是否启用基准测试
    pub enabled: bool,
    /// 当前精灵数量
    pub sprite_count: usize,
}

/// 渲染统计信息
#[derive(Resource, Default)]
pub struct RenderStats {
    /// GPU渲染耗时 (毫秒)
    pub gpu_pass_ms: Option<f32>,
    /// Draw Call 数量
    pub draw_calls: u32,
    /// 实例数量
    pub instances: u32,
    /// 渲染通道数量
    pub passes: u32,
    /// 上传阶段耗时 (毫秒)
    pub upload_ms: Option<f32>,
    /// 主渲染阶段耗时 (毫秒)
    pub main_ms: Option<f32>,
    /// UI渲染阶段耗时 (毫秒)
    pub ui_ms: Option<f32>,
    /// 离屏渲染阶段耗时 (毫秒)
    pub offscreen_ms: Option<f32>,
    /// 上传耗时警告计数
    pub alerts_upload: u32,
    /// 主渲染耗时警告计数
    pub alerts_main: u32,
    /// UI渲染耗时警告计数
    pub alerts_ui: u32,
    /// 离屏渲染耗时警告计数
    pub alerts_offscreen: u32,
    /// 被视锥剔除的对象数量
    pub culled_objects: u32,
    /// 总对象数量
    pub total_objects: u32,
}

/// 资源加载指标
#[derive(Resource, Default)]
pub struct AssetMetrics {
    /// 最近一次资源加载延迟 (毫秒)
    pub last_latency_ms: Option<f32>,
    /// 已加载纹理数量
    pub textures_loaded: u32,
    /// 已加载图集数量
    pub atlases_loaded: u32,
}

/// 日志事件缓冲
#[derive(Resource, Default)]
pub struct LogEvents {
    /// 日志条目队列
    pub entries: VecDeque<String>,
    /// 日志过滤器
    pub filter: String,
    /// 最大容量
    pub capacity: usize,
}

impl LogEvents {
    /// 创建具有指定容量的日志事件缓冲
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(capacity),
            filter: String::new(),
            capacity,
        }
    }
    
    /// 添加日志条目
    pub fn push(&mut self, message: String) {
        if self.entries.len() >= self.capacity {
            self.entries.pop_front();
        }
        self.entries.push_back(message);
    }
    
    /// 清空日志
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_log_events_capacity() {
        let mut logs = LogEvents::with_capacity(3);
        logs.push("msg1".to_string());
        logs.push("msg2".to_string());
        logs.push("msg3".to_string());
        logs.push("msg4".to_string());
        
        assert_eq!(logs.entries.len(), 3);
        assert_eq!(logs.entries[0], "msg2");
    }
    
    #[test]
    fn test_render_stats_default() {
        let stats = RenderStats::default();
        assert_eq!(stats.draw_calls, 0);
        assert!(stats.gpu_pass_ms.is_none());
    }
}

