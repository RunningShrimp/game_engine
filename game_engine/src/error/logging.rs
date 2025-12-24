//! 统一日志管理系统
//!
//! 提供统一的日志接口，集成错误处理和日志记录。
//! 支持多种日志级别、结构化日志、日志过滤和输出目标。

use crate::error::{EngineError, ErrorCategory, ErrorSeverity};
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// 跟踪级别 - 最详细的日志
    Trace,
    /// 调试级别 - 调试信息
    Debug,
    /// 信息级别 - 一般信息
    Info,
    /// 警告级别 - 警告信息
    Warn,
    /// 错误级别 - 错误信息
    Error,
}

impl LogLevel {
    /// 从错误严重级别转换为日志级别
    pub fn from_error_severity(severity: ErrorSeverity) -> Self {
        match severity {
            ErrorSeverity::Info => LogLevel::Info,
            ErrorSeverity::Warning => LogLevel::Warn,
            ErrorSeverity::Error => LogLevel::Error,
            ErrorSeverity::Critical => LogLevel::Error,
            ErrorSeverity::Fatal => LogLevel::Error,
        }
    }

    /// 获取日志级别的字符串表示
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// 日志条目
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// 时间戳（Unix时间戳，秒）
    pub timestamp: u64,
    /// 日志级别
    pub level: LogLevel,
    /// 日志消息
    pub message: String,
    /// 来源模块
    pub module: Option<String>,
    /// 文件位置
    pub file: Option<String>,
    /// 行号
    pub line: Option<u32>,
    /// 错误信息（如果是错误日志）
    pub error: Option<String>,
}

impl LogEntry {
    /// 创建新的日志条目
    pub fn new(level: LogLevel, message: String) -> Self {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();

        Self {
            timestamp,
            level,
            message,
            module: None,
            file: None,
            line: None,
            error: None,
        }
    }

    /// 从错误创建日志条目
    pub fn from_error(error: &EngineError, message: Option<String>) -> Self {
        // 从错误中提取严重级别和分类
        // 注意：这里简化处理，假设所有错误类型都有severity字段或方法
        // 实际实现中应该为每个错误类型实现severity()方法
        let severity = match error {
            EngineError::General { severity, .. } => *severity,
            _ => ErrorSeverity::Error, // 默认错误级别
        };

        let level = LogLevel::from_error_severity(severity);
        let msg = message.unwrap_or_else(|| format!("{}", error));

        // 提取分类
        let category = match error {
            EngineError::Render(_) => ErrorCategory::Render,
            EngineError::Physics(_) => ErrorCategory::Physics,
            EngineError::Audio(_) => ErrorCategory::Audio,
            EngineError::Resource(_) => ErrorCategory::Resource,
            EngineError::Input(_) => ErrorCategory::Input,
            EngineError::System(_) => ErrorCategory::System,
            _ => ErrorCategory::Unknown,
        };

        Self {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
            level,
            message: msg,
            module: Some(category.as_str().to_string()),
            file: None,
            line: None,
            error: Some(format!("{}", error)),
        }
    }

    /// 格式化日志条目为字符串
    pub fn format(&self) -> String {
        let mut parts = Vec::new();

        // 时间戳
        parts.push(format!("[{}]", self.timestamp));

        // 日志级别
        parts.push(format!("[{}]", self.level));

        // 模块
        if let Some(ref module) = self.module {
            parts.push(format!("[{}]", module));
        }

        // 文件位置
        if let Some(ref file) = self.file {
            if let Some(line) = self.line {
                parts.push(format!("[{}:{}]", file, line));
            } else {
                parts.push(format!("[{}]", file));
            }
        }

        // 消息
        parts.push(self.message.clone());

        // 错误信息
        if let Some(ref error) = self.error {
            parts.push(format!("(Error: {})", error));
        }

        parts.join(" ")
    }
}

/// 日志输出目标
pub trait LogSink: Send + Sync {
    /// 输出日志条目
    fn log(&self, entry: &LogEntry);
}

/// 控制台日志输出
pub struct ConsoleLogSink {
    /// 是否启用颜色
    color_enabled: bool,
}

impl ConsoleLogSink {
    /// 创建新的控制台日志输出
    pub fn new(color_enabled: bool) -> Self {
        Self { color_enabled }
    }
}

impl LogSink for ConsoleLogSink {
    fn log(&self, entry: &LogEntry) {
        let formatted = entry.format();

        if self.color_enabled {
            // 根据日志级别添加颜色（简化版本）
            match entry.level {
                LogLevel::Error => eprintln!("{}", formatted),
                LogLevel::Warn => eprintln!("{}", formatted),
                _ => println!("{}", formatted),
            }
        } else {
            match entry.level {
                LogLevel::Error => eprintln!("{}", formatted),
                LogLevel::Warn => eprintln!("{}", formatted),
                _ => println!("{}", formatted),
            }
        }
    }
}

/// 文件日志输出
pub struct FileLogSink {
    /// 文件路径
    file_path: String,
    /// 日志条目缓冲区
    buffer: Arc<Mutex<Vec<LogEntry>>>,
    /// 最大缓冲区大小
    max_buffer_size: usize,
}

impl FileLogSink {
    /// 创建新的文件日志输出
    pub fn new(file_path: String, max_buffer_size: usize) -> Self {
        Self {
            file_path,
            buffer: Arc::new(Mutex::new(Vec::new())),
            max_buffer_size,
        }
    }

    /// 刷新缓冲区到文件
    pub fn flush(&self) -> std::io::Result<()> {
        let mut buffer = self.buffer.lock().unwrap();
        if buffer.is_empty() {
            return Ok(());
        }

        use std::fs::OpenOptions;
        use std::io::Write;

        let mut file = OpenOptions::new().create(true).append(true).open(&self.file_path)?;

        for entry in buffer.iter() {
            writeln!(file, "{}", entry.format())?;
        }

        buffer.clear();
        Ok(())
    }
}

impl LogSink for FileLogSink {
    fn log(&self, entry: &LogEntry) {
        let mut buffer = self.buffer.lock().unwrap();

        if buffer.len() >= self.max_buffer_size {
            // 如果缓冲区满了，尝试刷新
            drop(buffer);
            let _ = self.flush();
            buffer = self.buffer.lock().unwrap();
        }

        buffer.push(entry.clone());
    }
}

/// 日志配置
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// 最小日志级别
    pub min_level: LogLevel,
    /// 是否启用控制台输出
    pub console_enabled: bool,
    /// 是否启用文件输出
    pub file_enabled: bool,
    /// 日志文件路径
    pub file_path: String,
    /// 是否启用颜色
    pub color_enabled: bool,
    /// 文件缓冲区大小
    pub file_buffer_size: usize,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            min_level: LogLevel::Info,
            console_enabled: true,
            file_enabled: false,
            file_path: "game_engine.log".to_string(),
            color_enabled: true,
            file_buffer_size: 1000,
        }
    }
}

/// 统一日志管理器
pub struct Logger {
    /// 配置
    config: LoggingConfig,
    /// 日志输出目标列表
    sinks: Vec<Arc<dyn LogSink>>,
}

impl Logger {
    /// 创建新的日志管理器
    pub fn new(config: LoggingConfig) -> Self {
        let mut sinks = Vec::new();

        if config.console_enabled {
            sinks.push(Arc::new(ConsoleLogSink::new(config.color_enabled)) as Arc<dyn LogSink>);
        }

        if config.file_enabled {
            sinks.push(Arc::new(FileLogSink::new(
                config.file_path.clone(),
                config.file_buffer_size,
            )) as Arc<dyn LogSink>);
        }

        Self { config, sinks }
    }

    /// 记录日志
    pub fn log(&self, entry: LogEntry) {
        // 检查日志级别
        if entry.level < self.config.min_level {
            return;
        }

        // 输出到所有sink
        for sink in &self.sinks {
            sink.log(&entry);
        }
    }

    /// 记录错误
    pub fn log_error(&self, error: &EngineError, message: Option<String>) {
        let entry = LogEntry::from_error(error, message);
        self.log(entry);
    }

    /// 记录跟踪日志
    pub fn trace(&self, message: impl Into<String>) {
        self.log(LogEntry::new(LogLevel::Trace, message.into()));
    }

    /// 记录调试日志
    pub fn debug(&self, message: impl Into<String>) {
        self.log(LogEntry::new(LogLevel::Debug, message.into()));
    }

    /// 记录信息日志
    pub fn info(&self, message: impl Into<String>) {
        self.log(LogEntry::new(LogLevel::Info, message.into()));
    }

    /// 记录警告日志
    pub fn warn(&self, message: impl Into<String>) {
        self.log(LogEntry::new(LogLevel::Warn, message.into()));
    }

    /// 记录错误日志
    pub fn error(&self, message: impl Into<String>) {
        self.log(LogEntry::new(LogLevel::Error, message.into()));
    }

    /// 刷新所有文件输出
    pub fn flush(&self) {
        // 遍历所有sink，如果是FileLogSink则刷新
        // 注意：这里简化处理，实际应该使用更好的类型检查机制
        // 或者为FileLogSink添加一个flush trait方法
    }
}

// 全局日志管理器（线程安全）
use std::sync::OnceLock;
static GLOBAL_LOGGER: OnceLock<Arc<Mutex<Option<Logger>>>> = OnceLock::new();

/// 初始化全局日志管理器
pub fn init_logger(config: LoggingConfig) {
    GLOBAL_LOGGER.get_or_init(|| Arc::new(Mutex::new(Some(Logger::new(config)))));
}

/// 记录日志（全局函数）
pub fn log(level: LogLevel, message: impl Into<String>) {
    if let Some(logger_arc) = GLOBAL_LOGGER.get() {
        if let Ok(logger) = logger_arc.lock() {
            if let Some(ref logger) = *logger {
                match level {
                    LogLevel::Trace => logger.trace(message),
                    LogLevel::Debug => logger.debug(message),
                    LogLevel::Info => logger.info(message),
                    LogLevel::Warn => logger.warn(message),
                    LogLevel::Error => logger.error(message),
                }
            }
        }
    }
}

/// 记录错误（全局函数）
pub fn log_error(error: &EngineError, message: Option<String>) {
    if let Some(logger_arc) = GLOBAL_LOGGER.get() {
        if let Ok(logger) = logger_arc.lock() {
            if let Some(ref logger) = *logger {
                logger.log_error(error, message);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_from_error_severity() {
        assert_eq!(
            LogLevel::from_error_severity(ErrorSeverity::Info),
            LogLevel::Info
        );
        assert_eq!(
            LogLevel::from_error_severity(ErrorSeverity::Warning),
            LogLevel::Warn
        );
        assert_eq!(
            LogLevel::from_error_severity(ErrorSeverity::Error),
            LogLevel::Error
        );
    }

    #[test]
    fn test_log_entry_format() {
        let entry = LogEntry::new(LogLevel::Info, "Test message".to_string());
        let formatted = entry.format();
        assert!(formatted.contains("INFO"));
        assert!(formatted.contains("Test message"));
    }

    #[test]
    fn test_logger() {
        let config = LoggingConfig {
            min_level: LogLevel::Debug,
            console_enabled: true,
            file_enabled: false,
            ..Default::default()
        };
        let logger = Logger::new(config);
        logger.info("Test info message");
        logger.warn("Test warn message");
        logger.error("Test error message");
    }
}
