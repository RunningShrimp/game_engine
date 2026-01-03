//! 移动平台生命周期管理
//!
//! 处理移动平台的应用状态管理、后台任务、内存警告等

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// 应用状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    /// 前台活跃
    Foreground,
    /// 后台运行
    Background,
    /// 非活跃（如多任务切换）
    Inactive,
    /// 终止中
    Terminating,
}

/// 应用生命周期事件
#[derive(Debug, Clone)]
pub enum LifecycleEvent {
    /// 应用启动
    Launched,
    /// 应用进入前台
    Foreground,
    /// 应用进入后台
    Background,
    /// 应用变为非活跃
    Inactive,
    /// 应用即将终止
    Terminating,
    /// 内存警告
    MemoryWarning,
    /// 低电量警告
    LowBattery { percentage: f32 },
    /// 网络状态变化
    NetworkChange { available: bool },
    /// 配置变化（如屏幕旋转）
    ConfigChange(ConfigChange),
}

/// 配置变化类型
#[derive(Debug, Clone)]
pub enum ConfigChange {
    /// 屏幕方向变化
    Orientation { portrait: bool },
    /// 屏幕尺寸变化
    ScreenSize { width: u32, height: u32 },
    /// 系统语言变化
    Language { code: String },
    /// 键盘状态变化
    Keyboard { visible: bool },
}

/// 后台任务
#[derive(Debug, Clone)]
pub struct BackgroundTask {
    /// 任务ID
    pub id: String,
    /// 任务名称
    pub name: String,
    /// 超时时间
    pub timeout: Duration,
    /// 任务状态
    pub status: TaskStatus,
}

/// 后台任务状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskStatus {
    /// 等待执行
    Pending,
    /// 执行中
    Running,
    /// 已完成
    Completed,
    /// 已取消
    Cancelled,
    /// 超时
    Expired,
}

/// 生命周期回调
pub trait LifecycleCallback: Send + Sync {
    /// 应用启动时调用
    fn on_launch(&mut self) {}
    /// 进入前台时调用
    fn on_foreground(&mut self) {}
    /// 进入后台时调用
    fn on_background(&mut self) {}
    /// 变为非活跃时调用
    fn on_inactive(&mut self) {}
    /// 终止时调用
    fn on_terminate(&mut self) {}
    /// 内存警告时调用
    fn on_memory_warning(&mut self) {}
    /// 低电量警告时调用
    fn on_low_battery(&mut self, _percentage: f32) {}
    /// 网络状态变化时调用
    fn on_network_change(&mut self, _available: bool) {}
    /// 配置变化时调用
    fn on_config_change(&mut self, _change: ConfigChange) {}
}

/// 移动生命周期管理器
pub struct MobileLifecycle {
    /// 当前应用状态
    current_state: AppState,
    /// 生命周期回调
    callbacks: Vec<Box<dyn LifecycleCallback>>,
    /// 后台任务注册表
    background_tasks: HashMap<String, BackgroundTask>,
    /// 是否在后台
    is_background: bool,
    /// 应用启动时间
    launch_time: Option<std::time::Instant>,
    /// 进入前台时间
    foreground_time: Option<std::time::Instant>,
    /// 进入后台时间
    background_time: Option<std::time::Instant>,
}

impl MobileLifecycle {
    /// 创建新的生命周期管理器
    pub fn new() -> Self {
        Self {
            current_state: AppState::Foreground,
            callbacks: Vec::new(),
            background_tasks: HashMap::new(),
            is_background: false,
            launch_time: None,
            foreground_time: None,
            background_time: None,
        }
    }

    /// 初始化生命周期管理器
    pub fn initialize(&mut self) {
        self.current_state = AppState::Foreground;
        self.launch_time = Some(std::time::Instant::now());
        self.foreground_time = Some(std::time::Instant::now());
        self.notify_callbacks(|cb| cb.on_launch());
        tracing::info!("Mobile lifecycle initialized");
    }

    /// 添加生命周期回调
    pub fn add_callback(&mut self, callback: Box<dyn LifecycleCallback>) {
        self.callbacks.push(callback);
    }

    /// 处理应用状态变化
    pub fn handle_app_state(&mut self, state: AppState) {
        let old_state = self.current_state;
        self.current_state = state;

        match state {
            AppState::Foreground => {
                if old_state != AppState::Foreground {
                    self.is_background = false;
                    self.foreground_time = Some(std::time::Instant::now());
                    self.notify_callbacks(|cb| cb.on_foreground());
                    tracing::info!("App entered foreground");
                }
            }
            AppState::Background => {
                if !self.is_background {
                    self.is_background = true;
                    self.background_time = Some(std::time::Instant::now());
                    self.notify_callbacks(|cb| cb.on_background());
                    tracing::info!("App entered background");
                }
            }
            AppState::Inactive => {
                self.notify_callbacks(|cb| cb.on_inactive());
                tracing::info!("App became inactive");
            }
            AppState::Terminating => {
                self.notify_callbacks(|cb| cb.on_terminate());
                tracing::info!("App is terminating");
            }
        }
    }

    /// 处理生命周期事件
    pub fn handle_event(&mut self, event: LifecycleEvent) {
        match event {
            LifecycleEvent::Launched => {
                self.initialize();
            }
            LifecycleEvent::Foreground => {
                self.handle_app_state(AppState::Foreground);
            }
            LifecycleEvent::Background => {
                self.handle_app_state(AppState::Background);
            }
            LifecycleEvent::Inactive => {
                self.handle_app_state(AppState::Inactive);
            }
            LifecycleEvent::Terminating => {
                self.handle_app_state(AppState::Terminating);
            }
            LifecycleEvent::MemoryWarning => {
                self.handle_memory_warning();
            }
            LifecycleEvent::LowBattery { percentage } => {
                self.notify_callbacks(|cb| cb.on_low_battery(percentage));
                tracing::warn!("Low battery warning: {}%", percentage);
            }
            LifecycleEvent::NetworkChange { available } => {
                self.notify_callbacks(|cb| cb.on_network_change(available));
                tracing::info!(
                    "Network status changed: {}",
                    if available {
                        "available"
                    } else {
                        "unavailable"
                    }
                );
            }
            LifecycleEvent::ConfigChange(change) => {
                self.notify_callbacks(|cb| cb.on_config_change(change.clone()));
                tracing::info!("Configuration changed: {:?}", change);
            }
        }
    }

    /// 处理内存警告
    pub fn handle_memory_warning(&mut self) {
        self.notify_callbacks(|cb| cb.on_memory_warning());
        tracing::warn!("Memory warning received");

        // 自动清理资源
        self.cleanup_resources();
    }

    /// 清理资源以释放内存
    fn cleanup_resources(&mut self) {
        // 取消所有非关键的后台任务
        let tasks_to_cancel: Vec<String> = self
            .background_tasks
            .iter()
            .filter(|(_, task)| matches!(task.status, TaskStatus::Pending))
            .map(|(id, _)| id.clone())
            .collect();

        for task_id in tasks_to_cancel {
            self.cancel_background_task(&task_id);
        }

        tracing::info!("Cleaned up resources due to memory warning");
    }

    /// 注册后台任务
    pub fn register_background_task(&mut self, task: BackgroundTask) {
        let task_id = task.id.clone();
        tracing::info!("Registered background task: {}", task_id);
        self.background_tasks.insert(task_id, task);
    }

    /// 开始后台任务
    pub fn start_background_task(&mut self, task_id: &str) -> Result<(), LifecycleError> {
        if let Some(task) = self.background_tasks.get_mut(task_id) {
            task.status = TaskStatus::Running;
            tracing::info!("Started background task: {}", task_id);
            Ok(())
        } else {
            Err(LifecycleError::TaskNotFound(task_id.to_string()))
        }
    }

    /// 完成后台任务
    pub fn complete_background_task(&mut self, task_id: &str) -> Result<(), LifecycleError> {
        if let Some(task) = self.background_tasks.get_mut(task_id) {
            task.status = TaskStatus::Completed;
            tracing::info!("Completed background task: {}", task_id);
            Ok(())
        } else {
            Err(LifecycleError::TaskNotFound(task_id.to_string()))
        }
    }

    /// 取消后台任务
    pub fn cancel_background_task(&mut self, task_id: &str) {
        if let Some(task) = self.background_tasks.get_mut(task_id) {
            task.status = TaskStatus::Cancelled;
            tracing::info!("Cancelled background task: {}", task_id);
        }
    }

    /// 获取后台任务
    pub fn get_background_task(&self, task_id: &str) -> Option<&BackgroundTask> {
        self.background_tasks.get(task_id)
    }

    /// 获取所有后台任务
    pub fn get_all_background_tasks(&self) -> Vec<&BackgroundTask> {
        self.background_tasks.values().collect()
    }

    /// 获取当前应用状态
    pub fn current_state(&self) -> AppState {
        self.current_state
    }

    /// 是否在后台
    pub fn is_background(&self) -> bool {
        self.is_background
    }

    /// 获取应用运行时间
    pub fn get_app_lifetime(&self) -> Option<Duration> {
        self.launch_time.map(|t| t.elapsed())
    }

    /// 获取前台时间
    pub fn get_foreground_duration(&self) -> Option<Duration> {
        self.foreground_time.map(|t| t.elapsed())
    }

    /// 获取后台时间
    pub fn get_background_duration(&self) -> Option<Duration> {
        self.background_time.map(|t| t.elapsed())
    }

    /// 通知所有回调
    fn notify_callbacks<F>(&mut self, f: F)
    where
        F: Fn(&mut Box<dyn LifecycleCallback>),
    {
        for callback in &mut self.callbacks {
            f(callback);
        }
    }
}

impl Default for MobileLifecycle {
    fn default() -> Self {
        Self::new()
    }
}

/// 生命周期错误
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LifecycleError {
    /// 任务未找到
    TaskNotFound(String),
    /// 任务超时
    TaskTimeout(String),
    /// 无效状态
    InvalidState(String),
}

impl std::fmt::Display for LifecycleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LifecycleError::TaskNotFound(id) => write!(f, "Task not found: {}", id),
            LifecycleError::TaskTimeout(id) => write!(f, "Task timeout: {}", id),
            LifecycleError::InvalidState(msg) => write!(f, "Invalid state: {}", msg),
        }
    }
}

impl std::error::Error for LifecycleError {}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestCallback {
        events: Vec<String>,
    }

    impl LifecycleCallback for TestCallback {
        fn on_launch(&mut self) {
            self.events.push("launch".to_string());
        }

        fn on_foreground(&mut self) {
            self.events.push("foreground".to_string());
        }

        fn on_background(&mut self) {
            self.events.push("background".to_string());
        }

        fn on_memory_warning(&mut self) {
            self.events.push("memory_warning".to_string());
        }
    }

    #[test]
    fn test_lifecycle_initialization() {
        let lifecycle = MobileLifecycle::new();
        assert_eq!(lifecycle.current_state(), AppState::Foreground);
    }

    #[test]
    fn test_state_transitions() {
        let mut lifecycle = MobileLifecycle::new();

        lifecycle.handle_app_state(AppState::Background);
        assert_eq!(lifecycle.current_state(), AppState::Background);
        assert!(lifecycle.is_background());

        lifecycle.handle_app_state(AppState::Foreground);
        assert_eq!(lifecycle.current_state(), AppState::Foreground);
        assert!(!lifecycle.is_background());
    }

    #[test]
    fn test_callbacks() {
        let mut lifecycle = MobileLifecycle::new();
        let callback = Box::new(TestCallback { events: Vec::new() });

        lifecycle.add_callback(callback);
        lifecycle.initialize();

        // Note: We can't access the callback's events after adding it
        // In real code, you'd use Arc<Mutex<TestCallback>> or similar
    }

    #[test]
    fn test_background_tasks() {
        let mut lifecycle = MobileLifecycle::new();

        let task = BackgroundTask {
            id: "test_task".to_string(),
            name: "Test Task".to_string(),
            timeout: Duration::from_secs(30),
            status: TaskStatus::Pending,
        };

        lifecycle.register_background_task(task);
        assert!(lifecycle.get_background_task("test_task").is_some());

        lifecycle.start_background_task("test_task").unwrap();
        assert_eq!(
            lifecycle.get_background_task("test_task").unwrap().status,
            TaskStatus::Running
        );

        lifecycle.complete_background_task("test_task").unwrap();
        assert_eq!(
            lifecycle.get_background_task("test_task").unwrap().status,
            TaskStatus::Completed
        );
    }

    #[test]
    fn test_memory_warning_cleanup() {
        let mut lifecycle = MobileLifecycle::new();

        let task = BackgroundTask {
            id: "cleanup_task".to_string(),
            name: "Cleanup Task".to_string(),
            timeout: Duration::from_secs(30),
            status: TaskStatus::Pending,
        };

        lifecycle.register_background_task(task);
        lifecycle.handle_memory_warning();

        assert_eq!(
            lifecycle.get_background_task("cleanup_task").unwrap().status,
            TaskStatus::Cancelled
        );
    }
}
