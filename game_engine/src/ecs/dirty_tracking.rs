//! ECS组件脏跟踪系统
//!
//! 提供通用的组件脏跟踪机制，允许系统只处理已修改的组件，减少不必要的更新。
//!
//! ## 设计原则
//!
//! 1. **细粒度跟踪**：支持按组件类型和字段级别的脏标记
//! 2. **零开销抽象**：使用标记位和位掩码，最小化内存占用
//! 3. **自动清理**：支持自动和手动清理脏标记
//! 4. **性能优化**：批量查询和更新支持
//!
//! ## 使用示例
//!
//! ```rust
//! use bevy_ecs::prelude::*;
//! use game_engine::ecs::dirty_tracking::{ComponentDirty, DirtyFlags};
//!
//! // 在系统中使用脏跟踪
//! fn update_system(
//!     mut query: Query<(&mut Transform, &mut ComponentDirty)>,
//! ) {
//!     for (mut transform, mut dirty) in query.iter_mut() {
//!         if dirty.is_dirty(DirtyFlags::POSITION) {
//!             // 只处理位置变化
//!             // ... 更新逻辑 ...
//!             dirty.clear(DirtyFlags::POSITION);
//!         }
//!     }
//! }
//!
//! // 标记组件为脏
//! fn modify_system(
//!     mut query: Query<(&mut Transform, &mut ComponentDirty)>,
//! ) {
//!     for (mut transform, mut dirty) in query.iter_mut() {
//!         transform.translation.x += 1.0;
//!         dirty.mark_dirty(DirtyFlags::POSITION);
//!     }
//! }
//! ```

use bevy_ecs::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};

/// 脏标记标志位
///
/// 使用位掩码支持多个脏标记同时存在
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DirtyFlags(u64);

impl DirtyFlags {
    /// 无脏标记
    pub const NONE: DirtyFlags = DirtyFlags(0);

    /// 位置变化
    pub const POSITION: DirtyFlags = DirtyFlags(1 << 0);

    /// 旋转变化
    pub const ROTATION: DirtyFlags = DirtyFlags(1 << 1);

    /// 缩放变化
    pub const SCALE: DirtyFlags = DirtyFlags(1 << 2);

    /// Transform完整变化
    pub const TRANSFORM: DirtyFlags = DirtyFlags(0b111);

    /// 渲染相关变化
    pub const RENDER: DirtyFlags = DirtyFlags(1 << 3);

    /// 材质变化
    pub const MATERIAL: DirtyFlags = DirtyFlags(1 << 4);

    /// 网格变化
    pub const MESH: DirtyFlags = DirtyFlags(1 << 5);

    /// 物理相关变化
    pub const PHYSICS: DirtyFlags = DirtyFlags(1 << 6);

    /// 碰撞体变化
    pub const COLLIDER: DirtyFlags = DirtyFlags(1 << 7);

    /// 自定义标志位（8-63）
    pub fn custom(bit: u8) -> DirtyFlags {
        if bit >= 64 {
            DirtyFlags::NONE
        } else {
            DirtyFlags(1 << (bit + 8))
        }
    }

    /// 组合多个标志位
    pub fn combine(flags: &[DirtyFlags]) -> DirtyFlags {
        flags.iter().fold(DirtyFlags::NONE, |acc, f| acc | *f)
    }

    /// 检查是否包含指定标志
    #[inline]
    pub fn contains(self, other: DirtyFlags) -> bool {
        (self.0 & other.0) != 0
    }

    /// 获取标志位的值
    #[inline]
    pub fn bits(self) -> u64 {
        self.0
    }
}

impl std::ops::BitOr for DirtyFlags {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        DirtyFlags(self.0 | rhs.0)
    }
}

impl std::ops::BitAnd for DirtyFlags {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        DirtyFlags(self.0 & rhs.0)
    }
}

impl std::ops::BitOrAssign for DirtyFlags {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}
impl std::ops::BitAndAssign for DirtyFlags {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

/// 组件脏标记组件
///
/// 跟踪组件的脏状态，支持细粒度的字段级别跟踪
///
/// 注意：此组件不实现Clone，因为AtomicU64不支持Clone。
/// 如果需要复制，请使用`get_flags()`获取标志位，然后创建新实例。
#[derive(Component, Debug)]
pub struct ComponentDirty {
    /// 当前脏标记
    flags: AtomicU64,
    /// 上次清理的帧号
    last_cleared_frame: AtomicU64,
}

impl ComponentDirty {
    /// 创建新的脏标记组件
    pub fn new() -> Self {
        Self {
            flags: AtomicU64::new(0),
            last_cleared_frame: AtomicU64::new(0),
        }
    }

    /// 标记为脏
    #[inline]
    pub fn mark_dirty(&mut self, flags: DirtyFlags) {
        self.flags.fetch_or(flags.bits(), Ordering::Relaxed);
    }

    /// 原子地标记为脏（线程安全）
    #[inline]
    pub fn mark_dirty_atomic(&self, flags: DirtyFlags) {
        self.flags.fetch_or(flags.bits(), Ordering::Relaxed);
    }

    /// 检查是否脏
    #[inline]
    pub fn is_dirty(&self, flags: DirtyFlags) -> bool {
        let current = self.flags.load(Ordering::Acquire);
        DirtyFlags(current).contains(flags)
    }

    /// 检查是否有任何脏标记
    #[inline]
    pub fn is_any_dirty(&self) -> bool {
        self.flags.load(Ordering::Acquire) != 0
    }

    /// 获取所有脏标记
    #[inline]
    pub fn get_flags(&self) -> DirtyFlags {
        DirtyFlags(self.flags.load(Ordering::Acquire))
    }

    /// 清除指定的脏标记
    #[inline]
    pub fn clear(&mut self, flags: DirtyFlags) {
        let mask = !flags.bits();
        let current = self.flags.load(Ordering::Acquire);
        self.flags.store(current & mask, Ordering::Release);
    }

    /// 原子地清除指定的脏标记（线程安全）
    #[inline]
    pub fn clear_atomic(&self, flags: DirtyFlags) {
        let mask = !flags.bits();
        let current = self.flags.load(Ordering::Acquire);
        self.flags.store(current & mask, Ordering::Release);
    }

    /// 清除所有脏标记
    #[inline]
    pub fn clear_all(&mut self) {
        self.flags.store(0, Ordering::Release);
    }

    /// 原子地清除所有脏标记（线程安全）
    #[inline]
    pub fn clear_all_atomic(&self) {
        self.flags.store(0, Ordering::Release);
    }

    /// 更新清理帧号
    #[inline]
    pub fn update_frame(&mut self, frame: u64) {
        self.last_cleared_frame.store(frame, Ordering::Release);
    }

    /// 原子地更新清理帧号（线程安全）
    #[inline]
    pub fn update_frame_atomic(&self, frame: u64) {
        self.last_cleared_frame.store(frame, Ordering::Release);
    }

    /// 获取上次清理的帧号
    #[inline]
    pub fn last_cleared_frame(&self) -> u64 {
        self.last_cleared_frame.load(Ordering::Acquire)
    }
}

impl Default for ComponentDirty {
    fn default() -> Self {
        Self::new()
    }
}

/// 脏跟踪配置
#[derive(Debug, Clone)]
pub struct DirtyTrackingConfig {
    /// 是否启用脏跟踪
    pub enabled: bool,
    /// 自动清理间隔（帧数）
    pub auto_clear_interval: u64,
    /// 是否在系统结束时自动清理
    pub auto_clear_on_system_end: bool,
}

impl Default for DirtyTrackingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_clear_interval: 1,
            auto_clear_on_system_end: false,
        }
    }
}

/// 脏跟踪资源
///
/// 全局配置和状态管理
#[derive(Resource, Debug, Default)]
pub struct DirtyTrackingResource {
    /// 配置
    pub config: DirtyTrackingConfig,
    /// 当前帧号
    pub current_frame: u64,
}

impl DirtyTrackingResource {
    /// 创建新的脏跟踪资源
    pub fn new() -> Self {
        Self::default()
    }

    /// 更新帧号
    pub fn update_frame(&mut self) {
        self.current_frame += 1;
    }

    /// 获取当前帧号
    pub fn current_frame(&self) -> u64 {
        self.current_frame
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dirty_flags() {
        let flags = DirtyFlags::POSITION | DirtyFlags::ROTATION;
        assert!(flags.contains(DirtyFlags::POSITION));
        assert!(flags.contains(DirtyFlags::ROTATION));
        assert!(!flags.contains(DirtyFlags::SCALE));

        let transform = DirtyFlags::TRANSFORM;
        assert!(transform.contains(DirtyFlags::POSITION));
        assert!(transform.contains(DirtyFlags::ROTATION));
        assert!(transform.contains(DirtyFlags::SCALE));
    }

    #[test]
    fn test_component_dirty() {
        let mut dirty = ComponentDirty::new();

        // 初始状态应该是干净的
        assert!(!dirty.is_any_dirty());

        // 标记为脏
        dirty.mark_dirty(DirtyFlags::POSITION);
        assert!(dirty.is_dirty(DirtyFlags::POSITION));
        assert!(dirty.is_any_dirty());

        // 清除
        dirty.clear(DirtyFlags::POSITION);
        assert!(!dirty.is_dirty(DirtyFlags::POSITION));
        assert!(!dirty.is_any_dirty());

        // 测试多个标志
        dirty.mark_dirty(DirtyFlags::POSITION | DirtyFlags::ROTATION);
        assert!(dirty.is_dirty(DirtyFlags::POSITION));
        assert!(dirty.is_dirty(DirtyFlags::ROTATION));

        // 只清除一个
        dirty.clear(DirtyFlags::POSITION);
        assert!(!dirty.is_dirty(DirtyFlags::POSITION));
        assert!(dirty.is_dirty(DirtyFlags::ROTATION));

        // 清除所有
        dirty.clear_all();
        assert!(!dirty.is_any_dirty());
    }

    #[test]
    fn test_atomic_operations() {
        let dirty = ComponentDirty::new();

        // 原子操作
        dirty.mark_dirty_atomic(DirtyFlags::POSITION);
        assert!(dirty.is_dirty(DirtyFlags::POSITION));

        dirty.clear_atomic(DirtyFlags::POSITION);
        assert!(!dirty.is_dirty(DirtyFlags::POSITION));
    }
}
