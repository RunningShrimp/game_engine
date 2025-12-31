//! UI布局系统
//!
//! 实现UI组件的布局管理和RectTransform。

use crate::ui::ComponentId;
use bevy_ecs::prelude::Component;
use glam::{FloatExt, Vec2};
use serde::{Deserialize, Serialize};

/// RectTransform
///
/// 定义UI组件的位置、大小和锚点。
#[derive(Debug, Clone, Serialize, Deserialize, Component)]
pub struct RectTransform {
    /// 锚点（相对于父组件）
    pub anchor_min: Vec2,
    pub anchor_max: Vec2,

    /// 位置偏移（像素）
    pub anchored_position: Vec2,

    /// 大小（像素）
    pub size_delta: Vec2,

    /// Pivot点（0-1）
    pub pivot: Vec2,

    /// 旋转角度（度）
    pub rotation: f32,

    /// 缩放
    pub scale: Vec2,
}

impl Default for RectTransform {
    fn default() -> Self {
        Self {
            anchor_min: Vec2::new(0.5, 0.5),
            anchor_max: Vec2::new(0.5, 0.5),
            anchored_position: Vec2::ZERO,
            size_delta: Vec2::new(100.0, 100.0),
            pivot: Vec2::new(0.5, 0.5),
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }
}

impl RectTransform {
    /// 创建新的RectTransform
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置位置
    pub fn with_position(mut self, x: f32, y: f32) -> Self {
        self.anchored_position = Vec2::new(x, y);
        self
    }

    /// 设置大小
    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.size_delta = Vec2::new(width, height);
        self
    }

    /// 设置锚点
    pub fn with_anchors(mut self, min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Self {
        self.anchor_min = Vec2::new(min_x, min_y);
        self.anchor_max = Vec2::new(max_x, max_y);
        self
    }

    /// 设置为左上角
    pub fn set_top_left(&mut self) {
        self.anchor_min = Vec2::new(0.0, 1.0);
        self.anchor_max = Vec2::new(0.0, 1.0);
        self.pivot = Vec2::new(0.0, 1.0);
    }

    /// 设置为中心
    pub fn set_center(&mut self) {
        self.anchor_min = Vec2::new(0.5, 0.5);
        self.anchor_max = Vec2::new(0.5, 0.5);
        self.pivot = Vec2::new(0.5, 0.5);
    }

    /// 设置为右下角
    pub fn set_bottom_right(&mut self) {
        self.anchor_min = Vec2::new(1.0, 0.0);
        self.anchor_max = Vec2::new(1.0, 0.0);
        self.pivot = Vec2::new(1.0, 0.0);
    }

    /// 设置为拉伸填充
    pub fn set_stretch(&mut self) {
        self.anchor_min = Vec2::ZERO;
        self.anchor_max = Vec2::ONE;
        self.pivot = Vec2::new(0.5, 0.5);
    }

    /// 计算世界位置
    pub fn world_position(&self, parent_size: Vec2) -> Vec2 {
        let anchor_pos = Vec2::new(
            self.anchor_min.x.lerp(self.anchor_max.x, self.pivot.x),
            self.anchor_min.y.lerp(self.anchor_max.y, self.pivot.y),
        );
        let parent_anchor = parent_size * anchor_pos;
        parent_anchor + self.anchored_position
    }

    /// 计算世界大小
    pub fn world_size(&self, parent_size: Vec2) -> Vec2 {
        let anchor_size = (self.anchor_max - self.anchor_min) * parent_size;
        anchor_size + self.size_delta
    }

    /// 获取点击检测框
    pub fn get_bounds(&self, parent_size: Vec2) -> (Vec2, Vec2) {
        let world_pos = self.world_position(parent_size);
        let world_size = self.world_size(parent_size);
        let min = world_pos - (world_size * self.pivot);
        let max = min + world_size;
        (min, max)
    }
}

/// 布局类型（兼容现有枚举）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutType {
    Absolute,
    Relative,
    Horizontal,
    Vertical,
    Grid { rows: u32, cols: u32 },
}

/// 布局算法
pub trait LayoutAlgorithm: Send + Sync {
    /// 计算布局
    fn calculate(&self, children: &mut [(ComponentId, RectTransform)], parent_size: Vec2);
}

/// 绝对布局算法
pub struct AbsoluteLayout;

impl LayoutAlgorithm for AbsoluteLayout {
    fn calculate(&self, _children: &mut [(ComponentId, RectTransform)], _parent_size: Vec2) {
        // 绝对布局不调整子组件位置
    }
}

/// 水平布局算法
pub struct HorizontalLayout {
    pub spacing: f32,
    pub padding: f32,
}

impl Default for HorizontalLayout {
    fn default() -> Self {
        Self {
            spacing: 10.0,
            padding: 10.0,
        }
    }
}

impl LayoutAlgorithm for HorizontalLayout {
    fn calculate(&self, children: &mut [(ComponentId, RectTransform)], parent_size: Vec2) {
        let mut current_x = self.padding - (parent_size.x * 0.5) + (self.spacing / 2.0);

        for (_, rect) in children.iter_mut() {
            rect.anchored_position.x = current_x;
            current_x += rect.size_delta.x + self.spacing;
        }
    }
}

/// 垂直布局算法
pub struct VerticalLayout {
    pub spacing: f32,
    pub padding: f32,
}

impl Default for VerticalLayout {
    fn default() -> Self {
        Self {
            spacing: 10.0,
            padding: 10.0,
        }
    }
}

impl LayoutAlgorithm for VerticalLayout {
    fn calculate(&self, children: &mut [(ComponentId, RectTransform)], parent_size: Vec2) {
        let mut current_y = (parent_size.y * 0.5) - self.padding - (self.spacing / 2.0);

        for (_, rect) in children.iter_mut() {
            rect.anchored_position.y = current_y;
            current_y -= rect.size_delta.y + self.spacing;
        }
    }
}

/// 网格布局算法
pub struct GridLayout {
    pub columns: usize,
    pub row_spacing: f32,
    pub column_spacing: f32,
    pub cell_size: Vec2,
}

impl Default for GridLayout {
    fn default() -> Self {
        Self {
            columns: 3,
            row_spacing: 10.0,
            column_spacing: 10.0,
            cell_size: Vec2::new(100.0, 100.0),
        }
    }
}

impl LayoutAlgorithm for GridLayout {
    fn calculate(&self, children: &mut [(ComponentId, RectTransform)], parent_size: Vec2) {
        let start_x = -(parent_size.x * 0.5) + (self.cell_size.x * 0.5);
        let start_y = (parent_size.y * 0.5) - (self.cell_size.y * 0.5);

        for (index, (_, rect)) in children.iter_mut().enumerate() {
            let row = index / self.columns;
            let col = index % self.columns;

            rect.anchored_position.x =
                start_x + (col as f32 * (self.cell_size.x + self.column_spacing));
            rect.anchored_position.y =
                start_y - (row as f32 * (self.cell_size.y + self.row_spacing));
            rect.size_delta = self.cell_size;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_transform_default() {
        let rect = RectTransform::default();
        assert_eq!(rect.anchor_min, Vec2::new(0.5, 0.5));
        assert_eq!(rect.size_delta, Vec2::new(100.0, 100.0));
    }

    #[test]
    fn test_rect_transform_builder() {
        let rect = RectTransform::new().with_position(50.0, 100.0).with_size(200.0, 150.0);

        assert_eq!(rect.anchored_position, Vec2::new(50.0, 100.0));
        assert_eq!(rect.size_delta, Vec2::new(200.0, 150.0));
    }

    #[test]
    fn test_rect_transform_anchors() {
        let mut rect = RectTransform::default();
        rect.set_top_left();
        assert_eq!(rect.anchor_min, Vec2::new(0.0, 1.0));

        rect.set_center();
        assert_eq!(rect.anchor_min, Vec2::new(0.5, 0.5));

        rect.set_stretch();
        assert_eq!(rect.anchor_min, Vec2::ZERO);
        assert_eq!(rect.anchor_max, Vec2::ONE);
    }

    #[test]
    fn test_world_calculation() {
        let rect = RectTransform::new().with_position(10.0, 20.0).with_size(100.0, 50.0);

        let parent_size = Vec2::new(800.0, 600.0);
        let world_pos = rect.world_position(parent_size);
        let world_size = rect.world_size(parent_size);

        assert_eq!(world_pos, Vec2::new(410.0, 320.0));
        assert_eq!(world_size, Vec2::new(100.0, 50.0));
    }

    #[test]
    fn test_horizontal_layout() {
        let layout = HorizontalLayout::default();
        let mut children = vec![
            (
                ComponentId::new(),
                RectTransform::new().with_size(100.0, 50.0),
            ),
            (
                ComponentId::new(),
                RectTransform::new().with_size(100.0, 50.0),
            ),
        ];

        let parent_size = Vec2::new(800.0, 600.0);
        layout.calculate(&mut children, parent_size);

        assert_eq!(children[0].1.anchored_position.x, -345.0);
        assert_eq!(children[1].1.anchored_position.x, -235.0);
    }
}
