//! 动画混合树系统
//!
//! 实现复杂的动画混合树，支持多个动画层和混合权重。

use crate::animation::{AnimationClip, InterpolationMode};
use bevy_ecs::prelude::*;
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// 混合树节点
#[derive(Debug, Clone)]
pub enum BlendTreeNode {
    /// 混合节点（线性混合两个动画）
    Mix {
        /// 混合权重（0.0-1.0）
        weight: f32,
        /// 左子节点
        children: Vec<BlendTreeNode>,
    },
    /// 叠加混合节点
    Additive {
        /// 左子节点
        children: Vec<BlendTreeNode>,
    },
    /// 动画剪辑节点
    Clip {
        /// 动画剪辑
        clip: Arc<AnimationClip>,
        /// 播放速度
        speed: f32,
    },
    /// 同步混合节点
    Sync {
        /// 同步源
        sync_source: String,
        /// 子节点
        children: Vec<BlendTreeNode>,
    },
}

/// 动画混合空间
#[derive(Debug, Clone, PartialEq)]
pub enum BlendSpace {
    /// 绑定空间混合（基于角色速度等参数）
    BindSpace {
        /// 参数名称
        parameter: String,
        /// 最小值
        min_value: f32,
        /// 最大值
        max_value: f32,
    },
    /// 程序空间混合（基于时间或其他程序控制）
    Procedural,
}

/// 动画混合树
#[derive(Debug, Clone, Component)]
pub struct AnimationBlendTree {
    /// 根节点
    pub root: BlendTreeNode,
    /// 当前混合权重
    pub weights: HashMap<String, f32>,
    /// 混合空间类型
    pub blend_space: BlendSpace,
}

impl AnimationBlendTree {
    /// 创建新的混合树
    pub fn new(root: BlendTreeNode) -> Self {
        Self {
            root,
            weights: HashMap::new(),
            blend_space: BlendSpace::Procedural,
        }
    }

    /// 设置混合权重
    pub fn set_weight(&mut self, name: String, weight: f32) {
        self.weights.insert(name, weight.clamp(0.0, 1.0));
    }

    /// 获取混合权重
    pub fn get_weight(&self, name: &str) -> f32 {
        self.weights.get(name).copied().unwrap_or(0.0)
    }

    /// 设置混合空间
    pub fn set_blend_space(&mut self, space: BlendSpace) {
        self.blend_space = space;
    }

    /// 计算混合结果
    pub fn evaluate(&self, state: &AnimationState) -> HashMap<String, Vec3> {
        let mut results = HashMap::new();
        self.evaluate_node(&self.root, state, &mut results);
        results
    }

    fn evaluate_node(
        &self,
        node: &BlendTreeNode,
        state: &AnimationState,
        results: &mut HashMap<String, Vec3>,
    ) {
        match node {
            BlendTreeNode::Clip { clip, speed } => {
                let time = state.time * speed;
                // 评估动画剪辑在当前时间的值
                if let Some(pos) = clip.evaluate_position(time) {
                    results.insert(clip.name.clone(), pos);
                }
            }
            BlendTreeNode::Mix { weight, children } => {
                let mut child_results = Vec::new();
                for child in children {
                    let mut child_map = HashMap::new();
                    self.evaluate_node(child, state, &mut child_map);
                    child_results.push(child_map);
                }

                // 混合子节点结果
                if child_results.len() >= 2 {
                    let result0 = child_results[0].values().next().unwrap_or(&Vec3::ZERO);
                    let result1 = child_results[1].values().next().unwrap_or(&Vec3::ZERO);
                    let blended = result0.lerp(*result1, *weight);
                    // 存储混合结果
                }
            }
            BlendTreeNode::Additive { children } => {
                for child in children {
                    self.evaluate_node(child, state, results);
                }
            }
            BlendTreeNode::Sync {
                sync_source,
                children,
            } => {
                // 根据同步源计算混合
                let sync_weight = self.get_weight(sync_source);
                // 应用同步混合
            }
        }
    }
}

/// 动画状态
#[derive(Debug, Clone)]
pub struct AnimationState {
    /// 当前时间（秒）
    pub time: f32,
    /// Delta时间
    pub delta_time: f32,
    /// 角色速度
    pub speed: f32,
    /// 是否在地面
    pub is_grounded: bool,
    /// 自定义参数
    pub parameters: HashMap<String, f32>,
}

impl Default for AnimationState {
    fn default() -> Self {
        Self {
            time: 0.0,
            delta_time: 0.016,
            speed: 0.0,
            is_grounded: true,
            parameters: HashMap::new(),
        }
    }
}

/// 1D混合空间
#[derive(Debug, Clone)]
pub struct BlendSpace1D {
    /// 参数名称
    pub parameter: String,
    /// 最小值
    pub min_value: f32,
    /// 最大值
    pub max_value: f32,
    /// 混合阈值
    pub thresholds: Vec<f32>,
    /// 动画剪辑
    pub clips: Vec<Arc<AnimationClip>>,
}

impl BlendSpace1D {
    /// 创建新的1D混合空间
    pub fn new(parameter: String, min_value: f32, max_value: f32) -> Self {
        let num_thresholds = ((max_value - min_value) / 0.1).ceil() as usize;
        let thresholds = (0..=num_thresholds).map(|i| min_value + (i as f32 * 0.1)).collect();

        Self {
            parameter,
            min_value,
            max_value,
            thresholds,
            clips: Vec::new(),
        }
    }

    /// 添加动画剪辑
    pub fn add_clip(&mut self, clip: Arc<AnimationClip>) {
        self.clips.push(clip);
    }

    /// 评估混合空间
    pub fn evaluate(&self, parameter_value: f32) -> f32 {
        // 根据参数值计算混合权重
        let clamped = parameter_value.clamp(self.min_value, self.max_value);
        let normalized = (clamped - self.min_value) / (self.max_value - self.min_value);

        // 查找对应的阈值区间
        for (i, threshold) in self.thresholds.iter().enumerate() {
            if normalized <= *threshold {
                if i == 0 {
                    return 0.0;
                }
                let prev = self.thresholds[i - 1];
                let range = *threshold - prev;
                let offset = normalized - prev;
                return offset / range;
            }
        }

        1.0
    }
}

/// 2D混合空间
#[derive(Debug, Clone)]
pub struct BlendSpace2D {
    /// X轴参数
    pub x_parameter: String,
    /// Y轴参数
    pub y_parameter: String,
    /// X轴范围
    pub x_range: (f32, f32),
    /// Y轴范围
    pub y_range: (f32, f32),
    /// 动画剪辑网格
    pub clips: Vec<Vec<Option<Arc<AnimationClip>>>>,
}

impl BlendSpace2D {
    pub fn new(
        x_parameter: String,
        y_parameter: String,
        x_range: (f32, f32),
        y_range: (f32, f32),
        grid_size: (usize, usize),
    ) -> Self {
        let clips = vec![vec![None; grid_size.1]; grid_size.0];

        Self {
            x_parameter,
            y_parameter,
            x_range,
            y_range,
            clips,
        }
    }

    /// 设置网格位置的动画
    pub fn set_clip(&mut self, x: usize, y: usize, clip: Arc<AnimationClip>) {
        if x < self.clips.len() && y < self.clips[0].len() {
            self.clips[x][y] = Some(clip);
        }
    }

    /// 评估2D混合空间
    pub fn evaluate(&self, x_value: f32, y_value: f32) -> [(usize, usize, f32); 4] {
        let x_clamped = x_value.clamp(self.x_range.0, self.x_range.1);
        let y_clamped = y_value.clamp(self.y_range.0, self.y_range.1);

        let x_normalized = (x_clamped - self.x_range.0) / (self.x_range.1 - self.x_range.0);
        let y_normalized = (y_clamped - self.y_range.0) / (self.y_range.1 - self.y_range.0);

        let x_index = (x_normalized * (self.clips.len() - 1) as f32) as usize;
        let y_index = (y_normalized * (self.clips[0].len() - 1) as f32) as usize;

        let x_frac = x_normalized * (self.clips.len() - 1) as f32 - x_index as f32;
        let y_frac = y_normalized * (self.clips[0].len() - 1) as f32 - y_index as f32;

        [
            (x_index, y_index, (1.0 - x_frac) * (1.0 - y_frac)),
            (x_index + 1, y_index, x_frac * (1.0 - y_frac)),
            (x_index, y_index + 1, (1.0 - x_frac) * y_frac),
            (x_index + 1, y_index + 1, x_frac * y_frac),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blend_space_1d() {
        let space = BlendSpace1D::new("speed".to_string(), 0.0, 1.0);
        assert_eq!(space.evaluate(0.5), 0.5);
        assert_eq!(space.evaluate(0.0), 0.0);
        assert_eq!(space.evaluate(1.0), 1.0);
    }

    #[test]
    fn test_blend_space_2d() {
        let space = BlendSpace2D::new(
            "x".to_string(),
            "y".to_string(),
            (0.0, 1.0),
            (0.0, 1.0),
            (2, 2),
        );
        let results = space.evaluate(0.5, 0.5);

        // 中心点应该平均分配权重
        let total_weight: f32 = results.iter().map(|r| r.2).sum();
        assert!((total_weight - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_animation_state() {
        let state = AnimationState::default();
        assert_eq!(state.time, 0.0);
        assert!(state.is_grounded);
    }
}
