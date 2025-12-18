//! 网络状态插值系统
//!
//! 实现网络状态插值和平滑，减少抖动，提升游戏体验。

use crate::impl_default;
use bevy_ecs::prelude::*;
use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// 插值状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterpolatedState {
    /// 位置
    pub position: Vec3,
    /// 旋转
    pub rotation: Quat,
    /// 缩放
    pub scale: Vec3,
    /// 时间戳
    pub timestamp: u64,
}

/// 插值组件
#[derive(Component, Debug, Clone)]
pub struct InterpolationComponent {
    /// 状态缓冲区
    states: VecDeque<InterpolatedState>,
    /// 最大缓冲区大小
    max_buffer_size: usize,
    /// 插值延迟（毫秒）
    interpolation_delay_ms: u64,
    /// 当前插值时间
    current_time: u64,
}

impl_default!(InterpolationComponent {
    states: VecDeque::new(),
    max_buffer_size: 64,
    interpolation_delay_ms: 100,
    current_time: 0,
});

impl InterpolationComponent {
    /// 添加状态到缓冲区
    pub fn add_state(&mut self, state: InterpolatedState) {
        self.states.push_back(state);

        // 限制缓冲区大小
        while self.states.len() > self.max_buffer_size {
            self.states.pop_front();
        }
    }

    /// 获取插值后的状态
    pub fn get_interpolated(&self, target_time: u64) -> Option<InterpolatedState> {
        if self.states.len() < 2 {
            return self.states.back().cloned();
        }

        // 找到目标时间前后的两个状态
        let mut before: Option<&InterpolatedState> = None;
        let mut after: Option<&InterpolatedState> = None;

        for state in self.states.iter() {
            if state.timestamp <= target_time {
                before = Some(state);
            } else {
                after = Some(state);
                break;
            }
        }

        match (before, after) {
            (Some(b), Some(a)) => {
                // 线性插值
                let t = if a.timestamp > b.timestamp {
                    (target_time - b.timestamp) as f32 / (a.timestamp - b.timestamp) as f32
                } else {
                    0.0
                };

                Some(InterpolatedState {
                    position: b.position.lerp(a.position, t),
                    rotation: Quat::lerp(b.rotation, a.rotation, t),
                    scale: b.scale.lerp(a.scale, t),
                    timestamp: target_time,
                })
            }
            (Some(b), None) => Some(b.clone()),
            (None, Some(a)) => Some(a.clone()),
            (None, None) => None,
        }
    }
}

/// 插值系统 - 更新插值状态
pub fn interpolation_system(
    mut query: Query<(&mut InterpolationComponent, &mut crate::ecs::Transform)>,
) {
    for (mut interp, mut transform) in query.iter_mut() {
        // 计算目标时间（当前时间 - 延迟）
        let target_time = interp
            .current_time
            .saturating_sub(interp.interpolation_delay_ms);

        if let Some(state) = interp.get_interpolated(target_time) {
            transform.pos = state.position;
            transform.rot = state.rotation;
            transform.scale = state.scale;
        }

        interp.current_time += 1;
    }
}
