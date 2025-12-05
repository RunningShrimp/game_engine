//! 错误报告系统
//!
//! 提供ECS系统来定期报告错误统计信息

use crate::core::error_aggregator::ErrorAggregator;
use crate::ecs::Time;
use bevy_ecs::prelude::*;

/// 错误报告系统
///
/// 定期输出错误统计摘要，帮助开发者了解引擎的健康状态。
pub fn error_reporting_system(time: Res<Time>, error_aggregator: Option<Res<ErrorAggregator>>) {
    // 每5秒报告一次错误统计
    const REPORT_INTERVAL: f64 = 5.0;
    static mut LAST_REPORT_TIME: f64 = 0.0;

    unsafe {
        if time.elapsed_seconds - LAST_REPORT_TIME < REPORT_INTERVAL {
            return;
        }
        LAST_REPORT_TIME = time.elapsed_seconds;
    }

    if let Some(aggregator) = error_aggregator {
        let summary = aggregator.get_summary();

        // 只在有错误时报告
        if summary.total_errors > 0 {
            tracing::warn!(
                target: "error_reporting",
                "错误统计摘要:\n{}",
                summary.format()
            );
        }
    }
}

/// 错误可视化系统
///
/// 将错误统计信息写入ECS资源，供UI系统显示。
pub fn error_visualization_system(
    error_aggregator: Option<Res<ErrorAggregator>>,
    mut log_events: Option<ResMut<LogEvents>>,
) {
    if let Some(aggregator) = error_aggregator {
        let summary = aggregator.get_summary();

        // 如果有错误，添加到日志事件中
        if summary.total_errors > 0 {
            if let Some(mut logs) = log_events {
                logs.push(format!(
                    "[错误统计] 总数: {}, 错误率: {:.2}/秒",
                    summary.total_errors, summary.error_rate
                ));
            }
        }
    }
}

use crate::core::resources::LogEvents;

/// 注册错误报告系统到调度器
pub fn register_error_reporting_systems(schedule: &mut Schedule) {
    schedule.add_systems((error_reporting_system, error_visualization_system));
}
