//! 固定时间步长游戏循环
//!
//! 实现确定性固定时间步长循环，支持：
//! - 时间累加器模式
//! - 固定更新和可变渲染分离
//! - 最大时间步长限制（防止螺旋死亡）
//! - 插值因子计算（用于平滑渲染）

use std::time::{Duration, Instant};

/// 固定时间步长循环管理器
///
/// 管理固定时间步长更新循环，确保物理模拟的确定性。
/// 使用时间累加器模式，将可变帧时间转换为固定时间步长更新。
///
/// # 设计要点
///
/// - **固定时间步长**：默认1/60秒（可配置）
/// - **时间累加器**：跟踪未处理的帧时间
/// - **固定更新循环**：独立于渲染帧率运行
/// - **插值因子**：用于平滑渲染的位置插值
///
/// # 示例
///
/// ```rust,no_run
/// use game_engine::core::engine::game_loop_fixed::FixedTimestepLoop;
/// use std::time::Duration;
///
/// let mut loop_manager = FixedTimestepLoop::new(Duration::from_secs_f64(1.0 / 60.0));
///
/// loop {
///     let frame_time = Duration::from_millis(16); // 假设60 FPS
///     let alpha = loop_manager.update(frame_time, |dt| {
///         // 固定时间步长更新
///         println!("Fixed update: {:?}", dt);
///     });
///
///     // 使用alpha进行插值渲染
///     render(alpha);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct FixedTimestepLoop {
    /// 固定时间步长
    fixed_time_step: Duration,
    /// 最大允许的帧时间（防止螺旋死亡）
    max_frame_time: Duration,
    /// 时间累加器
    accumulator: Duration,
    /// 上一次更新时间
    last_update: Instant,
}

impl FixedTimestepLoop {
    /// 创建新的固定时间步长循环管理器
    ///
    /// # 参数
    ///
    /// * `fixed_time_step` - 固定时间步长（例如：1/60秒）
    ///
    /// # 返回
    ///
    /// 返回新创建的`FixedTimestepLoop`实例
    pub fn new(fixed_time_step: Duration) -> Self {
        Self {
            fixed_time_step,
            max_frame_time: Duration::from_millis(250), // 默认最大250ms
            accumulator: Duration::ZERO,
            last_update: Instant::now(),
        }
    }

    /// 使用自定义最大帧时间创建
    ///
    /// # 参数
    ///
    /// * `fixed_time_step` - 固定时间步长
    /// * `max_frame_time` - 最大允许的帧时间（防止螺旋死亡）
    pub fn with_max_frame_time(fixed_time_step: Duration, max_frame_time: Duration) -> Self {
        Self {
            fixed_time_step,
            max_frame_time,
            accumulator: Duration::ZERO,
            last_update: Instant::now(),
        }
    }

    /// 更新固定时间步长循环
    ///
    /// 处理帧时间，运行固定时间步长更新，并返回插值因子。
    ///
    /// # 参数
    ///
    /// * `frame_time` - 当前帧的时间（从上一帧到现在的时间）
    /// * `update_fn` - 固定时间步长更新函数，接收固定时间步长作为参数
    ///
    /// # 返回
    ///
    /// 返回插值因子（0.0到1.0之间），用于平滑渲染
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use game_engine::core::engine::game_loop_fixed::FixedTimestepLoop;
    /// use std::time::Duration;
    ///
    /// let mut loop_manager = FixedTimestepLoop::new(Duration::from_secs_f64(1.0 / 60.0));
    ///
    /// let frame_time = Duration::from_millis(16);
    /// let alpha = loop_manager.update(frame_time, |dt| {
    ///     // 运行固定时间步长更新
    ///     physics_update(dt);
    /// });
    ///
    /// // 使用alpha进行插值
    /// render_with_interpolation(alpha);
    /// ```
    pub fn update<F>(&mut self, frame_time: Duration, mut update_fn: F) -> f64
    where
        F: FnMut(Duration),
    {
        // 限制最大帧时间（防止螺旋死亡）
        let frame_time = frame_time.min(self.max_frame_time);

        // 累加时间到累加器
        self.accumulator += frame_time;

        // 运行固定时间步长更新
        let mut update_count = 0;
        while self.accumulator >= self.fixed_time_step {
            update_fn(self.fixed_time_step);
            self.accumulator -= self.fixed_time_step;
            update_count += 1;

            // 防止无限循环（如果帧时间过大）
            if update_count > 10 {
                tracing::warn!(
                    "Fixed timestep loop: too many updates in one frame ({}), clamping accumulator",
                    update_count
                );
                self.accumulator = Duration::ZERO;
                break;
            }
        }

        // 计算插值因子（用于平滑渲染）
        
        self.accumulator.as_secs_f64() / self.fixed_time_step.as_secs_f64()
    }

    /// 更新并返回固定时间步长信息
    ///
    /// 与`update`类似，但返回更详细的信息。
    ///
    /// # 参数
    ///
    /// * `frame_time` - 当前帧的时间
    /// * `update_fn` - 固定时间步长更新函数
    ///
    /// # 返回
    ///
    /// 返回`FixedTimestepInfo`，包含插值因子和更新次数
    pub fn update_with_info<F>(
        &mut self,
        frame_time: Duration,
        mut update_fn: F,
    ) -> FixedTimestepInfo
    where
        F: FnMut(Duration),
    {
        // 限制最大帧时间
        let frame_time = frame_time.min(self.max_frame_time);

        // 累加时间到累加器
        self.accumulator += frame_time;

        // 运行固定时间步长更新
        let mut update_count = 0;
        while self.accumulator >= self.fixed_time_step {
            update_fn(self.fixed_time_step);
            self.accumulator -= self.fixed_time_step;
            update_count += 1;

            // 防止无限循环
            if update_count > 10 {
                tracing::warn!(
                    "Fixed timestep loop: too many updates in one frame ({}), clamping accumulator",
                    update_count
                );
                self.accumulator = Duration::ZERO;
                break;
            }
        }

        // 计算插值因子
        let alpha = self.accumulator.as_secs_f64() / self.fixed_time_step.as_secs_f64();

        FixedTimestepInfo {
            interpolation_alpha: alpha,
            update_count,
            fixed_time_step: self.fixed_time_step,
        }
    }

    /// 获取固定时间步长
    pub fn fixed_time_step(&self) -> Duration {
        self.fixed_time_step
    }

    /// 设置固定时间步长
    pub fn set_fixed_time_step(&mut self, fixed_time_step: Duration) {
        self.fixed_time_step = fixed_time_step;
    }

    /// 获取最大帧时间
    pub fn max_frame_time(&self) -> Duration {
        self.max_frame_time
    }

    /// 设置最大帧时间
    pub fn set_max_frame_time(&mut self, max_frame_time: Duration) {
        self.max_frame_time = max_frame_time;
    }

    /// 重置累加器
    pub fn reset(&mut self) {
        self.accumulator = Duration::ZERO;
        self.last_update = Instant::now();
    }
}

/// 固定时间步长信息
#[derive(Debug, Clone, Copy)]
pub struct FixedTimestepInfo {
    /// 插值因子（0.0到1.0之间）
    pub interpolation_alpha: f64,
    /// 本次更新运行的固定时间步长更新次数
    pub update_count: u32,
    /// 固定时间步长
    pub fixed_time_step: Duration,
}

// run_fixed_steps 已删除 - 请使用 FixedTimestepLoop 代替

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_timestep_loop_creation() {
        let loop_manager = FixedTimestepLoop::new(Duration::from_secs_f64(1.0 / 60.0));
        assert_eq!(
            loop_manager.fixed_time_step(),
            Duration::from_secs_f64(1.0 / 60.0)
        );
    }

    #[test]
    fn test_fixed_timestep_update() {
        let mut loop_manager = FixedTimestepLoop::new(Duration::from_secs_f64(1.0 / 60.0));
        let mut update_count = 0;

        // 模拟一个帧时间（16.67ms，约60 FPS）
        let frame_time = Duration::from_millis(16);
        let alpha = loop_manager.update(frame_time, |_| {
            update_count += 1;
        });

        // 应该运行一次固定更新
        assert_eq!(update_count, 1);
        // alpha应该在0.0到1.0之间
        assert!(alpha >= 0.0 && alpha <= 1.0);
    }

    #[test]
    fn test_fixed_timestep_multiple_updates() {
        let mut loop_manager = FixedTimestepLoop::new(Duration::from_secs_f64(1.0 / 60.0));
        let mut update_count = 0;

        // 模拟一个较大的帧时间（50ms，应该触发多次更新）
        let frame_time = Duration::from_millis(50);
        let alpha = loop_manager.update(frame_time, |_| {
            update_count += 1;
        });

        // 应该运行多次固定更新（50ms / 16.67ms ≈ 3次）
        assert!(update_count >= 2);
        // alpha应该在0.0到1.0之间
        assert!(alpha >= 0.0 && alpha <= 1.0);
    }

    #[test]
    fn test_fixed_timestep_max_frame_time() {
        let mut loop_manager = FixedTimestepLoop::with_max_frame_time(
            Duration::from_secs_f64(1.0 / 60.0),
            Duration::from_millis(100),
        );

        // 模拟一个非常大的帧时间（500ms）
        let frame_time = Duration::from_millis(500);
        let mut update_count = 0;
        let alpha = loop_manager.update(frame_time, |_| {
            update_count += 1;
        });

        // 应该被限制到100ms，所以更新次数应该基于100ms计算
        assert!(update_count <= 10); // 防止无限循环的保护
        assert!(alpha >= 0.0 && alpha <= 1.0);
    }

    #[test]
    fn test_fixed_timestep_reset() {
        let mut loop_manager = FixedTimestepLoop::new(Duration::from_secs_f64(1.0 / 60.0));

        // 运行一次更新
        let frame_time = Duration::from_millis(16);
        loop_manager.update(frame_time, |_| {});

        // 重置
        loop_manager.reset();

        // 验证重置后可以正常使用
        let mut update_count = 0;
        loop_manager.update(frame_time, |_| {
            update_count += 1;
        });
        assert_eq!(update_count, 1);
    }
}
