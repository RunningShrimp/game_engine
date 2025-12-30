//  重试机制
//
//  提供灵活的重试策略，支持指数退避、条件重试和超时控制。

use crate::error::{EngineError, ErrorSeverity};
use rand::Rng;
use std::future::Future;
use std::time::{Duration, Instant};

/// 重试配置
#[derive(Debug)]
pub struct RetryConfig {
    /// 最大重试次数
    pub max_attempts: u32,
    /// 基础延迟
    pub base_delay: Duration,
    /// 最大延迟
    pub max_delay: Duration,
    /// 退避倍数
    pub backoff_multiplier: f64,
    /// 抖动因子（0.0-1.0）
    pub jitter_factor: f64,
    /// 是否使用指数退避
    pub exponential_backoff: bool,
    /// 重试条件
    pub retry_condition: Option<RetryCondition>,
    /// 超时时间
    pub timeout: Option<Duration>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
            exponential_backoff: true,
            retry_condition: None,
            timeout: None,
        }
    }
}

impl RetryConfig {
    /// 创建新的重试配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置最大重试次数
    pub fn max_attempts(mut self, attempts: u32) -> Self {
        self.max_attempts = attempts;
        self
    }

    /// 设置基础延迟
    pub fn base_delay(mut self, delay: Duration) -> Self {
        self.base_delay = delay;
        self
    }

    /// 设置最大延迟
    pub fn max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    /// 设置退避倍数
    pub fn backoff_multiplier(mut self, multiplier: f64) -> Self {
        self.backoff_multiplier = multiplier;
        self
    }

    /// 设置抖动因子
    pub fn jitter_factor(mut self, factor: f64) -> Self {
        self.jitter_factor = factor;
        self
    }

    /// 启用指数退避
    pub fn exponential_backoff(mut self, enabled: bool) -> Self {
        self.exponential_backoff = enabled;
        self
    }

    /// 设置重试条件
    pub fn retry_condition(mut self, condition: RetryCondition) -> Self {
        self.retry_condition = Some(condition);
        self
    }

    /// 计算重试延迟
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        let base_delay = if self.exponential_backoff {
            let multiplier = self.backoff_multiplier.powi(attempt.saturating_sub(1) as i32);
            self.base_delay.mul_f64(multiplier)
        } else {
            self.base_delay
        };

        let delay_with_jitter = if self.jitter_factor > 0.0 {
            let jitter_range = base_delay.mul_f64(self.jitter_factor);
            let mut rng = rand::thread_rng();
            let jitter = Duration::from_millis(rng.gen_range(0..=jitter_range.as_millis() as u64));
            base_delay + jitter
        } else {
            base_delay
        };

        std::cmp::min(delay_with_jitter, self.max_delay)
    }
}

/// 重试条件
pub enum RetryCondition {
    /// 总是重试
    Always,
    /// 从不重试
    Never,
    /// 基于错误严重级别
    OnSeverity(ErrorSeverity),
    /// 基于错误分类
    OnCategory(crate::error::ErrorCategory),
    /// 基于自定义条件
    Custom(Box<dyn Fn(&EngineError) -> bool + Send + Sync>),
}

impl std::fmt::Debug for RetryCondition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RetryCondition::Always => write!(f, "Always"),
            RetryCondition::Never => write!(f, "Never"),
            RetryCondition::OnSeverity(severity) => write!(f, "OnSeverity({severity:?})"),
            RetryCondition::OnCategory(category) => write!(f, "OnCategory({category:?})"),
            RetryCondition::Custom(_) => write!(f, "Custom(<function>)"),
        }
    }
}

impl RetryCondition {
    /// 检查是否应该重试
    pub fn should_retry(&self, error: &EngineError) -> bool {
        match self {
            RetryCondition::Always => true,
            RetryCondition::Never => false,
            RetryCondition::OnSeverity(severity) => error.severity() <= *severity,
            RetryCondition::OnCategory(category) => error.category() == *category,
            RetryCondition::Custom(condition) => condition(error),
        }
    }
}

/// 重试结果
#[derive(Debug, Clone)]
pub enum RetryResult<T> {
    /// 成功
    Success(T),
    /// 失败（最后一次尝试）
    Failed {
        /// 最后的错误
        error: EngineError,
        /// 总尝试次数
        attempts: u32,
        /// 总耗时
        total_duration: Duration,
    },
    /// 超时
    Timeout {
        /// 总尝试次数
        attempts: u32,
        /// 总耗时
        total_duration: Duration,
    },
}

/// 重试状态
#[derive(Debug, Clone)]
struct RetryState {
    /// 当前尝试次数
    attempt: u32,
    /// 开始时间
    start_time: Instant,
    /// 下次重试时间
    next_retry_time: Option<Instant>,
}

impl RetryState {
    /// 获取重试开始时间
    fn start_time(&self) -> Instant {
        self.start_time
    }

    /// 获取重试持续时间
    fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// 检查是否超时
    fn is_timeout(&self, timeout: Duration) -> bool {
        self.elapsed() > timeout
    }
}

/// 重试执行器
pub struct RetryExecutor;

impl RetryExecutor {
    /// 同步执行重试
    pub fn execute<T, F>(config: &RetryConfig, mut operation: F) -> RetryResult<T>
    where
        F: FnMut() -> Result<T, EngineError>,
    {
        let start_time = Instant::now();
        let mut state = RetryState {
            attempt: 0,
            start_time,
            next_retry_time: None,
        };

        loop {
            // 记录重试开始时间（用于调试和确保方法被调用）
            let retry_start = state.start_time();
            let _ = retry_start; // 确保变量被使用

            // 检查是否超时
            if let Some(timeout) = config.timeout
                && state.is_timeout(timeout)
            {
                return RetryResult::Timeout {
                    attempts: state.attempt - 1,
                    total_duration: state.elapsed(),
                };
            }

            state.attempt += 1;

            // 检查是否超过最大重试次数
            if state.attempt > config.max_attempts {
                return RetryResult::Failed {
                    error: EngineError::general(format!(
                        "Max retry attempts ({}) exceeded",
                        config.max_attempts
                    )),
                    attempts: state.attempt - 1,
                    total_duration: start_time.elapsed(),
                };
            }

            // 执行操作
            match operation() {
                Ok(result) => {
                    return RetryResult::Success(result);
                }
                Err(error) => {
                    // 检查重试条件
                    let should_retry = match &config.retry_condition {
                        Some(condition) => condition.should_retry(&error),
                        None => error.severity() < ErrorSeverity::Critical,
                    };

                    if !should_retry {
                        return RetryResult::Failed {
                            error,
                            attempts: state.attempt,
                            total_duration: start_time.elapsed(),
                        };
                    }

                    // 计算下次重试时间
                    if state.attempt <= config.max_attempts {
                        state.next_retry_time =
                            Some(Instant::now() + config.calculate_delay(state.attempt - 1));
                    }
                }
            }

            // 等待下次重试时间
            if let Some(retry_time) = state.next_retry_time {
                let now = Instant::now();
                if retry_time > now {
                    std::thread::sleep(retry_time - now);
                }
            }
        }
    }

    /// 异步执行重试
    pub async fn execute_async<T, F, Fut>(config: &RetryConfig, mut operation: F) -> RetryResult<T>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, EngineError>>,
    {
        let start_time = Instant::now();
        let mut state = RetryState {
            attempt: 0,
            start_time,
            next_retry_time: None,
        };

        loop {
            state.attempt += 1;

            // 检查是否超过最大重试次数
            if state.attempt > config.max_attempts {
                return RetryResult::Failed {
                    error: EngineError::general(format!(
                        "Max retry attempts ({}) exceeded",
                        config.max_attempts
                    )),
                    attempts: state.attempt - 1,
                    total_duration: start_time.elapsed(),
                };
            }

            // 执行异步操作
            match operation().await {
                Ok(result) => {
                    return RetryResult::Success(result);
                }
                Err(error) => {
                    // 检查重试条件
                    let should_retry = match &config.retry_condition {
                        Some(condition) => condition.should_retry(&error),
                        None => error.severity() < ErrorSeverity::Critical,
                    };

                    if !should_retry {
                        return RetryResult::Failed {
                            error,
                            attempts: state.attempt,
                            total_duration: start_time.elapsed(),
                        };
                    }

                    // 计算下次重试时间
                    if state.attempt <= config.max_attempts {
                        state.next_retry_time =
                            Some(Instant::now() + config.calculate_delay(state.attempt - 1));
                    }
                }
            }

            // 等待下次重试时间
            if let Some(retry_time) = state.next_retry_time {
                let now = Instant::now();
                if retry_time > now {
                    tokio::time::sleep(retry_time - now).await;
                }
            }
        }
    }

    /// 带超时的同步执行重试
    pub fn execute_with_timeout<T, F>(
        config: &RetryConfig,
        timeout: Duration,
        mut operation: F,
    ) -> RetryResult<T>
    where
        F: FnMut() -> Result<T, EngineError>,
    {
        let start_time = Instant::now();
        let mut state = RetryState {
            attempt: 0,
            start_time,
            next_retry_time: None,
        };

        loop {
            state.attempt += 1;

            // 检查总超时
            if start_time.elapsed() > timeout {
                return RetryResult::Timeout {
                    attempts: state.attempt - 1,
                    total_duration: start_time.elapsed(),
                };
            }

            // 检查是否超过最大重试次数
            if state.attempt > config.max_attempts {
                return RetryResult::Failed {
                    error: EngineError::general(format!(
                        "Max retry attempts ({}) exceeded",
                        config.max_attempts
                    )),
                    attempts: state.attempt - 1,
                    total_duration: start_time.elapsed(),
                };
            }

            // 执行操作
            match operation() {
                Ok(result) => {
                    return RetryResult::Success(result);
                }
                Err(error) => {
                    // 检查重试条件
                    let should_retry = match &config.retry_condition {
                        Some(condition) => condition.should_retry(&error),
                        None => error.severity() < ErrorSeverity::Critical,
                    };

                    if !should_retry {
                        return RetryResult::Failed {
                            error,
                            attempts: state.attempt,
                            total_duration: start_time.elapsed(),
                        };
                    }

                    // 计算下次重试时间
                    if state.attempt <= config.max_attempts {
                        state.next_retry_time =
                            Some(Instant::now() + config.calculate_delay(state.attempt - 1));
                    }
                }
            }

            // 等待下次重试时间
            if let Some(retry_time) = state.next_retry_time {
                let now = Instant::now();
                if retry_time > now {
                    std::thread::sleep(retry_time - now);
                }
            }
        }
    }

    /// 带超时的异步执行重试
    pub async fn execute_async_with_timeout<T, F, Fut>(
        config: &RetryConfig,
        timeout: Duration,
        mut operation: F,
    ) -> RetryResult<T>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, EngineError>>,
    {
        let start_time = Instant::now();
        let mut state = RetryState {
            attempt: 0,
            start_time,
            next_retry_time: None,
        };

        loop {
            state.attempt += 1;

            // 检查总超时
            if start_time.elapsed() > timeout {
                return RetryResult::Timeout {
                    attempts: state.attempt - 1,
                    total_duration: start_time.elapsed(),
                };
            }

            // 检查是否超过最大重试次数
            if state.attempt > config.max_attempts {
                return RetryResult::Failed {
                    error: EngineError::general(format!(
                        "Max retry attempts ({}) exceeded",
                        config.max_attempts
                    )),
                    attempts: state.attempt - 1,
                    total_duration: start_time.elapsed(),
                };
            }

            // 执行异步操作
            match operation().await {
                Ok(result) => {
                    return RetryResult::Success(result);
                }
                Err(error) => {
                    // 检查重试条件
                    let should_retry = match &config.retry_condition {
                        Some(condition) => condition.should_retry(&error),
                        None => error.severity() < ErrorSeverity::Critical,
                    };

                    if !should_retry {
                        return RetryResult::Failed {
                            error,
                            attempts: state.attempt,
                            total_duration: start_time.elapsed(),
                        };
                    }

                    // 计算下次重试时间
                    if state.attempt <= config.max_attempts {
                        state.next_retry_time =
                            Some(Instant::now() + config.calculate_delay(state.attempt - 1));
                    }
                }
            }

            // 等待下次重试时间
            if let Some(retry_time) = state.next_retry_time {
                let now = Instant::now();
                if retry_time > now {
                    tokio::time::sleep(retry_time - now).await;
                }
            }
        }
    }
}

/// 重试策略
pub enum RetryPolicy {
    /// 固定间隔重试
    FixedInterval {
        /// 间隔时间
        interval: Duration,
        /// 最大重试次数
        max_attempts: u32,
    },
    /// 指数退避重试
    ExponentialBackoff {
        /// 基础间隔
        base_interval: Duration,
        /// 退避倍数
        backoff_multiplier: f64,
        /// 最大间隔
        max_interval: Duration,
        /// 最大重试次数
        max_attempts: u32,
    },
    /// 线性退避重试
    LinearBackoff {
        /// 基础间隔
        base_interval: Duration,
        /// 退避增量
        increment: Duration,
        /// 最大间隔
        max_interval: Duration,
        /// 最大重试次数
        max_attempts: u32,
    },
    /// 自定义重试
    Custom {
        /// 重试函数
        retry_fn: Box<dyn Fn(u32, &EngineError) -> Duration + Send + Sync>,
        /// 最大重试次数
        max_attempts: u32,
    },
}

impl std::fmt::Debug for RetryPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RetryPolicy::FixedInterval {
                interval,
                max_attempts,
            } => {
                write!(
                    f,
                    "FixedInterval {{ interval: {interval:?}, max_attempts: {max_attempts} }}"
                )
            }
            RetryPolicy::ExponentialBackoff {
                base_interval,
                backoff_multiplier,
                max_interval,
                max_attempts,
            } => {
                write!(
                    f,
                    "ExponentialBackoff {{ base_interval: {base_interval:?}, backoff_multiplier: {backoff_multiplier}, max_interval: {max_interval:?}, max_attempts: {max_attempts} }}"
                )
            }
            RetryPolicy::LinearBackoff {
                base_interval,
                increment,
                max_interval,
                max_attempts,
            } => {
                write!(
                    f,
                    "LinearBackoff {{ base_interval: {base_interval:?}, increment: {increment:?}, max_interval: {max_interval:?}, max_attempts: {max_attempts} }}"
                )
            }
            RetryPolicy::Custom { max_attempts, .. } => {
                write!(f, "Custom {{ max_attempts: {max_attempts} }}")
            }
        }
    }
}

// 为RetryPolicy手动实现Clone，因为Custom变体包含函数指针无法克隆
impl Clone for RetryPolicy {
    fn clone(&self) -> Self {
        match self {
            RetryPolicy::FixedInterval {
                interval,
                max_attempts,
            } => RetryPolicy::FixedInterval {
                interval: *interval,
                max_attempts: *max_attempts,
            },
            RetryPolicy::ExponentialBackoff {
                base_interval,
                backoff_multiplier,
                max_interval,
                max_attempts,
            } => RetryPolicy::ExponentialBackoff {
                base_interval: *base_interval,
                backoff_multiplier: *backoff_multiplier,
                max_interval: *max_interval,
                max_attempts: *max_attempts,
            },
            RetryPolicy::LinearBackoff {
                base_interval,
                increment,
                max_interval,
                max_attempts,
            } => RetryPolicy::LinearBackoff {
                base_interval: *base_interval,
                increment: *increment,
                max_interval: *max_interval,
                max_attempts: *max_attempts,
            },
            RetryPolicy::Custom { .. } => {
                // Custom变体包含函数指针，无法克隆
                panic!("Cannot clone RetryPolicy::Custom variant")
            }
        }
    }
}

impl RetryPolicy {
    /// 创建固定间隔策略
    pub fn fixed_interval(interval: Duration, max_attempts: u32) -> Self {
        Self::FixedInterval {
            interval,
            max_attempts,
        }
    }

    /// 创建指数退避策略
    pub fn exponential_backoff(
        base_interval: Duration,
        backoff_multiplier: f64,
        max_interval: Duration,
        max_attempts: u32,
    ) -> Self {
        Self::ExponentialBackoff {
            base_interval,
            backoff_multiplier,
            max_interval,
            max_attempts,
        }
    }

    /// 创建线性退避策略
    pub fn linear_backoff(
        base_interval: Duration,
        increment: Duration,
        max_interval: Duration,
        max_attempts: u32,
    ) -> Self {
        Self::LinearBackoff {
            base_interval,
            increment,
            max_interval,
            max_attempts,
        }
    }

    /// 创建自定义策略
    pub fn custom<F>(retry_fn: F, max_attempts: u32) -> Self
    where
        F: Fn(u32, &EngineError) -> Duration + Send + Sync + 'static,
    {
        Self::Custom {
            retry_fn: Box::new(retry_fn),
            max_attempts,
        }
    }

    /// 计算重试延迟
    pub fn calculate_delay(&self, attempt: u32, _error: &EngineError) -> Duration {
        match self {
            RetryPolicy::FixedInterval { interval, .. } => *interval,
            RetryPolicy::ExponentialBackoff {
                base_interval,
                backoff_multiplier,
                max_interval,
                ..
            } => {
                let delay = *base_interval
                    * backoff_multiplier.powi(attempt.saturating_sub(1) as i32) as u32;
                std::cmp::min(delay, *max_interval)
            }
            RetryPolicy::LinearBackoff {
                base_interval,
                increment,
                max_interval,
                ..
            } => {
                let delay = *base_interval + *increment * (attempt - 1);
                std::cmp::min(delay, *max_interval)
            }
            RetryPolicy::Custom { retry_fn, .. } => retry_fn(attempt, _error),
        }
    }

    /// 获取最大重试次数
    pub fn max_attempts(&self) -> u32 {
        match self {
            RetryPolicy::FixedInterval { max_attempts, .. }
            | RetryPolicy::ExponentialBackoff { max_attempts, .. }
            | RetryPolicy::LinearBackoff { max_attempts, .. }
            | RetryPolicy::Custom { max_attempts, .. } => *max_attempts,
        }
    }
}

/// 便捷的重试宏
#[macro_export]
macro_rules! retry {
    ($config:expr, $operation:expr) => {
        $crate::error::retry::RetryExecutor::execute($config, $operation)
    };
    ($config:expr, $operation:expr, $timeout:expr) => {
        $crate::error::retry::RetryExecutor::execute_with_timeout($config, $timeout, $operation)
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{AudioError, RenderError};

    #[test]
    fn test_retry_config() {
        let config = RetryConfig::new()
            .max_attempts(5)
            .base_delay(Duration::from_millis(100))
            .exponential_backoff(true)
            .backoff_multiplier(2.0)
            .max_delay(Duration::from_secs(10));

        assert_eq!(config.max_attempts, 5);
        assert_eq!(config.base_delay, Duration::from_millis(100));
        assert!(config.exponential_backoff);
        assert_eq!(config.backoff_multiplier, 2.0);
        assert_eq!(config.max_delay, Duration::from_secs(10));
    }

    #[test]
    fn test_retry_condition() {
        let always = RetryCondition::Always;
        let never = RetryCondition::Never;
        let on_severity = RetryCondition::OnSeverity(ErrorSeverity::Error);
        let on_category = RetryCondition::OnCategory(crate::error::ErrorCategory::Render);

        assert!(always.should_retry(&EngineError::general("test")));
        assert!(!never.should_retry(&EngineError::general("test")));
        assert!(on_severity.should_retry(&EngineError::general("test")));
        assert!(
            !on_severity.should_retry(&EngineError::general_with_severity(
                "test",
                ErrorSeverity::Critical
            ))
        );
        assert!(on_category.should_retry(&EngineError::Render(RenderError::general("test"))));
        assert!(!on_category.should_retry(&EngineError::Audio(AudioError::general("test"))));
    }

    #[test]
    fn test_retry_executor_success() {
        let config = RetryConfig::new().max_attempts(3);
        let mut call_count = 0;

        let result = RetryExecutor::execute(&config, || {
            call_count += 1;
            if call_count == 1 {
                Err(EngineError::general("First failure"))
            } else {
                Ok("success")
            }
        });

        assert!(matches!(result, RetryResult::Success("success")));
        assert_eq!(call_count, 2);
    }

    #[test]
    fn test_retry_executor_failure() {
        let config = RetryConfig::new().max_attempts(2);

        let result: RetryResult<String> =
            RetryExecutor::execute(&config, || Err(EngineError::general("Always fails")));

        assert!(matches!(result, RetryResult::Failed { attempts: 2, .. }));
    }

    #[test]
    fn test_retry_policy_fixed_interval() {
        let policy = RetryPolicy::fixed_interval(Duration::from_millis(100), 3);

        assert_eq!(policy.max_attempts(), 3);

        let error = EngineError::general("test");
        let delay1 = policy.calculate_delay(1, &error);
        let delay2 = policy.calculate_delay(2, &error);

        assert_eq!(delay1, Duration::from_millis(100));
        assert_eq!(delay2, Duration::from_millis(100));
    }

    #[test]
    fn test_retry_policy_exponential_backoff() {
        let policy = RetryPolicy::exponential_backoff(
            Duration::from_millis(100),
            2.0,
            Duration::from_secs(10),
            3,
        );

        assert_eq!(policy.max_attempts(), 3);

        let error = EngineError::general("test");
        let delay1 = policy.calculate_delay(1, &error);
        let delay2 = policy.calculate_delay(2, &error);
        let delay3 = policy.calculate_delay(3, &error);

        assert_eq!(delay1, Duration::from_millis(100));
        assert_eq!(delay2, Duration::from_millis(200));
        assert_eq!(delay3, Duration::from_millis(400)); // Capped at max
    }

    #[test]
    fn test_retry_policy_linear_backoff() {
        let policy = RetryPolicy::linear_backoff(
            Duration::from_millis(100),
            Duration::from_millis(50),
            Duration::from_secs(1),
            3,
        );

        assert_eq!(policy.max_attempts(), 3);

        let error = EngineError::general("test");
        let delay1 = policy.calculate_delay(1, &error);
        let delay2 = policy.calculate_delay(2, &error);
        let delay3 = policy.calculate_delay(3, &error);

        assert_eq!(delay1, Duration::from_millis(100));
        assert_eq!(delay2, Duration::from_millis(150));
        assert_eq!(delay3, Duration::from_millis(200)); // Capped at max
    }
}
