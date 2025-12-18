use glam::{Vec2, Vec4};

/// 裁剪区域
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClipRect {
    /// 左上角位置
    pub min: Vec2,
    /// 右下角位置
    pub max: Vec2,
}

impl ClipRect {
    /// 创建新的裁剪区域
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    /// 从位置和大小创建裁剪区域
    pub fn from_pos_size(pos: Vec2, size: Vec2) -> Self {
        Self {
            min: pos,
            max: pos + size,
        }
    }

    /// 获取裁剪区域的宽度
    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    /// 获取裁剪区域的高度
    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }

    /// 获取裁剪区域的大小
    pub fn size(&self) -> Vec2 {
        self.max - self.min
    }

    /// 检查点是否在裁剪区域内
    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }

    /// 检查另一个裁剪区域是否与此区域相交
    pub fn intersects(&self, other: &ClipRect) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }

    /// 计算与另一个裁剪区域的交集
    pub fn intersection(&self, other: &ClipRect) -> Option<ClipRect> {
        if !self.intersects(other) {
            return None;
        }

        Some(ClipRect {
            min: Vec2::new(self.min.x.max(other.min.x), self.min.y.max(other.min.y)),
            max: Vec2::new(self.max.x.min(other.max.x), self.max.y.min(other.max.y)),
        })
    }

    /// 转换为Vec4 (x, y, width, height)
    pub fn to_vec4(&self) -> Vec4 {
        Vec4::new(self.min.x, self.min.y, self.width(), self.height())
    }
}

/// 裁剪栈,用于管理嵌套的裁剪区域
pub struct ClipStack {
    stack: Vec<ClipRect>,
}

impl ClipStack {
    /// 创建新的裁剪栈
    pub fn new() -> Self {
        Self::default()
    }

    /// 推入新的裁剪区域
    pub fn push(&mut self, clip: ClipRect) {
        if let Some(current) = self.current() {
            // 与当前裁剪区域求交集
            if let Some(intersection) = current.intersection(&clip) {
                self.stack.push(intersection);
            } else {
                // 如果没有交集,推入一个空区域
                self.stack.push(ClipRect::new(Vec2::ZERO, Vec2::ZERO));
            }
        } else {
            self.stack.push(clip);
        }
    }

    /// 弹出裁剪区域
    pub fn pop(&mut self) -> Option<ClipRect> {
        self.stack.pop()
    }

    /// 获取当前裁剪区域
    pub fn current(&self) -> Option<ClipRect> {
        self.stack.last().copied()
    }

    /// 清空裁剪栈
    pub fn clear(&mut self) {
        self.stack.clear();
    }
}

impl Default for ClipStack {
    fn default() -> Self {
        Self { stack: Vec::new() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clip_rect() {
        let rect = ClipRect::from_pos_size(Vec2::new(10.0, 10.0), Vec2::new(100.0, 100.0));

        assert_eq!(rect.width(), 100.0);
        assert_eq!(rect.height(), 100.0);
        assert!(rect.contains(Vec2::new(50.0, 50.0)));
        assert!(!rect.contains(Vec2::new(5.0, 5.0)));
    }

    #[test]
    fn test_clip_intersection() {
        let rect1 = ClipRect::from_pos_size(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));
        let rect2 = ClipRect::from_pos_size(Vec2::new(50.0, 50.0), Vec2::new(100.0, 100.0));

        let intersection = rect1.intersection(&rect2).unwrap();
        assert_eq!(intersection.min, Vec2::new(50.0, 50.0));
        assert_eq!(intersection.max, Vec2::new(100.0, 100.0));
    }

    #[test]
    fn test_clip_stack() {
        let mut stack = ClipStack::new();

        stack.push(ClipRect::from_pos_size(
            Vec2::new(0.0, 0.0),
            Vec2::new(200.0, 200.0),
        ));
        stack.push(ClipRect::from_pos_size(
            Vec2::new(50.0, 50.0),
            Vec2::new(100.0, 100.0),
        ));

        let current = stack.current().unwrap();
        assert_eq!(current.min, Vec2::new(50.0, 50.0));
        assert_eq!(current.max, Vec2::new(150.0, 150.0));

        stack.pop();
        let current = stack.current().unwrap();
        assert_eq!(current.min, Vec2::new(0.0, 0.0));
        assert_eq!(current.max, Vec2::new(200.0, 200.0));
    }
}
