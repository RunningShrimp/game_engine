//! 蒙皮绑定数据
//!
//! 定义蒙皮与骨骼的绑定关系。

use bevy_ecs::prelude::*;
use glam::Mat4;
use std::sync::Arc;

/// 蒙皮数据
#[derive(Debug, Clone)]
pub struct Skin {
    /// 骨骼名称列表
    pub joints: Vec<String>,
    /// 逆绑定矩阵
    pub inverse_bind_matrices: Vec<Mat4>,
}

impl Skin {
    pub fn new(joints: Vec<String>, inverse_bind_matrices: Vec<Mat4>) -> Self {
        Self {
            joints,
            inverse_bind_matrices,
        }
    }
}

/// 蒙皮组件
#[derive(Component, Clone)]
pub struct SkinComponent {
    pub skin: Arc<Skin>,
}
