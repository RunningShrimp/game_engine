//! 控制台面板
//!
//! 显示脚本日志、错误消息和调试输出。

use super::{LogLevel, Panel};
use bevy_ecs::prelude::*;
use chrono::Local;

/// 控制台面板
///
/// 显示日志消息和错误信息。
pub struct ConsolePanel {
    /// 是否显示面板
    visible: bool,
    /// 日志消息
    logs: Vec<LogMessage>,
    /// 最大日志行数
    max_lines: usize,
    /// 搜索过滤文本
    filter_text: String,
    /// 选中的日志级别过滤
    log_level_filter: Option<LogLevel>,
    /// 自动滚动到底部
    auto_scroll: bool,
}

/// 日志消息
#[derive(Debug, Clone)]
struct LogMessage {
    /// 消息内容
    message: String,
    /// 日志级别
    level: LogLevel,
    /// 时间戳
    timestamp: String,
    /// 来源
    source: String,
}

impl ConsolePanel {
    /// 创建新的控制台面板
    pub fn new() -> Self {
        Self::with_max_lines(1000)
    }

    /// 使用自定义最大行数创建控制台面板
    pub fn with_max_lines(max_lines: usize) -> Self {
        Self {
            visible: true,
            logs: Vec::with_capacity(max_lines),
            max_lines,
            filter_text: String::new(),
            log_level_filter: None,
            auto_scroll: true,
        }
    }

    /// 添加日志消息
    pub fn add_log(&mut self, message: String) {
        self.add_log_with_level(message, LogLevel::Info);
    }

    /// 添加错误消息
    pub fn add_error(&mut self, error: String) {
        self.add_log_with_level(error, LogLevel::Error);
    }

    /// 添加带级别的日志消息
    pub fn add_log_with_level(&mut self, message: String, level: LogLevel) {
        let timestamp = Local::now().format("%H:%M:%S%.3f").to_string();

        let log_msg = LogMessage {
            message,
            level,
            timestamp,
            source: "Engine".to_string(),
        };

        self.logs.push(log_msg);

        // 限制日志行数
        if self.logs.len() > self.max_lines {
            self.logs.remove(0);
        }
    }

    /// 添加调试消息
    pub fn add_debug(&mut self, message: String) {
        self.add_log_with_level(message, LogLevel::Debug);
    }

    /// 添加警告消息
    pub fn add_warning(&mut self, message: String) {
        self.add_log_with_level(message, LogLevel::Warning);
    }

    /// 清空日志
    pub fn clear(&mut self) {
        self.logs.clear();
    }

    /// 显示面板
    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.visible {
            return;
        }

        egui::Window::new("Console").default_size([600.0, 400.0]).show(ctx, |ui| {
            // 工具栏
            ui.horizontal(|ui| {
                ui.label("Filter:");
                let response = ui.text_edit_singleline(&mut self.filter_text);

                // 显示日志级别过滤
                ui.separator();
                if ui.checkbox(&mut (self.log_level_filter.is_none()), "All").clicked() {
                    if self.log_level_filter.is_some() {
                        self.log_level_filter = None;
                    }
                }

                if ui
                    .checkbox(
                        &mut (self.log_level_filter == Some(LogLevel::Error)),
                        "Errors",
                    )
                    .clicked()
                {
                    self.log_level_filter = Some(LogLevel::Error);
                }

                if ui
                    .checkbox(
                        &mut (self.log_level_filter == Some(LogLevel::Warning)),
                        "Warnings",
                    )
                    .clicked()
                {
                    self.log_level_filter = Some(LogLevel::Warning);
                }

                ui.separator();

                // 自动滚动
                ui.checkbox(&mut self.auto_scroll, "Auto-scroll");

                // 清空按钮
                if ui.button("Clear").clicked() {
                    self.clear();
                }
            });

            ui.separator();

            // 显示日志计数
            let filtered_count = self.filtered_logs().len();
            ui.label(format!(
                "Messages: {} / {}",
                filtered_count,
                self.logs.len()
            ));

            ui.separator();

            // 日志内容区域
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .stick_to_bottom(self.auto_scroll)
                .show(ui, |ui| {
                    ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                        for log in self.filtered_logs() {
                            self.show_log_message(ui, log);
                        }
                    });
                });
        });
    }

    /// 显示单条日志消息
    fn show_log_message(&mut self, ui: &mut egui::Ui, log: &LogMessage) {
        // 应用过滤器
        if !self.filter_text.is_empty() && !log.message.contains(&self.filter_text) {
            return;
        }

        if let Some(filter_level) = self.log_level_filter {
            if log.level != filter_level {
                return;
            }
        }

        // 显示日志
        ui.horizontal(|ui| {
            // 时间戳
            ui.colored_label(egui::Color32::GRAY, &log.timestamp);

            // 日志级别
            let level_label = match log.level {
                LogLevel::Info => "[INFO]",
                LogLevel::Warning => "[WARN]",
                LogLevel::Error => "[ERROR]",
                LogLevel::Debug => "[DEBUG]",
            };
            ui.colored_label(log.level.color(), level_label);

            // 来源
            ui.colored_label(egui::Color32::LIGHT_GRAY, &log.source);

            // 消息
            ui.label(&log.message);
        });
    }

    /// 获取过滤后的日志
    fn filtered_logs(&self) -> Vec<&LogMessage> {
        self.logs
            .iter()
            .filter(|log| {
                // 文本过滤
                if !self.filter_text.is_empty() && !log.message.contains(&self.filter_text) {
                    return false;
                }

                // 级别过滤
                if let Some(filter_level) = self.log_level_filter {
                    if log.level != filter_level {
                        return false;
                    }
                }

                true
            })
            .collect()
    }

    /// 导出日志到文件
    pub fn export_to_file(&self, path: &str) -> std::io::Result<()> {
        use std::io::Write;

        let mut file = std::fs::File::create(path)?;
        for log in &self.logs {
            writeln!(
                file,
                "{} [{}] {}: {}",
                log.timestamp,
                format!("{:?}", log.level),
                log.source,
                log.message
            )?;
        }
        Ok(())
    }

    /// 获取日志数量
    pub fn log_count(&self) -> usize {
        self.logs.len()
    }

    /// 获取错误数量
    pub fn error_count(&self) -> usize {
        self.logs.iter().filter(|log| log.level == LogLevel::Error).count()
    }

    /// 获取警告数量
    pub fn warning_count(&self) -> usize {
        self.logs.iter().filter(|log| log.level == LogLevel::Warning).count()
    }
}

impl Default for ConsolePanel {
    fn default() -> Self {
        Self::new()
    }
}

/// 日志收集器 - 可以与tracing或log库集成
pub struct LogCollector {
    panel: *mut ConsolePanel,
}

unsafe impl Send for LogCollector {}
unsafe impl Sync for LogCollector {}

impl LogCollector {
    /// 创建新的日志收集器
    ///
    /// # Safety
    ///
    /// 调用者必须确保panel的生命周期长于LogCollector
    pub unsafe fn new(panel: *mut ConsolePanel) -> Self {
        Self { panel }
    }

    /// 添加日志消息
    pub fn log(&self, message: String, level: LogLevel) {
        unsafe {
            if let Some(panel) = self.panel.as_mut() {
                panel.add_log_with_level(message, level);
            }
        }
    }
}

/// 实现log库的Log trait（可选）
#[cfg(feature = "logging-integration")]
impl log::Log for LogCollector {
    fn log(&self, record: &log::Record) {
        let message = format!("{}", record.args());
        let level = match record.level() {
            log::Level::Error => LogLevel::Error,
            log::Level::Warn => LogLevel::Warning,
            log::Level::Info => LogLevel::Info,
            log::Level::Debug | log::Level::Trace => LogLevel::Debug,
        };

        unsafe {
            if let Some(panel) = self.panel.as_mut() {
                panel.add_log_with_level(message, level);
            }
        }
    }

    fn flush(&self) {
        // No-op
    }

    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }
}
