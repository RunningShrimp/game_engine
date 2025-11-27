use std::collections::VecDeque;

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warning,
    Error,
}

/// 日志条目
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// 时间戳
    pub timestamp: f64,
    /// 日志级别
    pub level: LogLevel,
    /// 日志消息
    pub message: String,
    /// 来源
    pub source: Option<String>,
}

/// 编辑器控制台
pub struct EditorConsole {
    /// 日志历史
    logs: VecDeque<LogEntry>,
    /// 最大日志数量
    max_logs: usize,
    /// 命令历史
    command_history: VecDeque<String>,
    /// 最大命令历史数量
    max_commands: usize,
    /// 当前输入
    current_input: String,
    /// 日志过滤器
    filter: LogFilter,
}

/// 日志过滤器
#[derive(Debug, Clone)]
pub struct LogFilter {
    /// 最小日志级别
    pub min_level: LogLevel,
    /// 搜索关键词
    pub search_term: Option<String>,
    /// 来源过滤
    pub source_filter: Option<String>,
}

impl EditorConsole {
    pub fn new() -> Self {
        Self {
            logs: VecDeque::new(),
            max_logs: 1000,
            command_history: VecDeque::new(),
            max_commands: 100,
            current_input: String::new(),
            filter: LogFilter {
                min_level: LogLevel::Trace,
                search_term: None,
                source_filter: None,
            },
        }
    }
    
    /// 添加日志
    pub fn log(&mut self, level: LogLevel, message: impl Into<String>, source: Option<String>) {
        let entry = LogEntry {
            timestamp: 0.0, // 实际实现需要获取真实时间
            level,
            message: message.into(),
            source,
        };
        
        self.logs.push_back(entry);
        
        // 限制日志数量
        while self.logs.len() > self.max_logs {
            self.logs.pop_front();
        }
    }
    
    /// 获取过滤后的日志
    pub fn get_filtered_logs(&self) -> Vec<&LogEntry> {
        self.logs
            .iter()
            .filter(|entry| self.filter.matches(entry))
            .collect()
    }
    
    /// 清空日志
    pub fn clear_logs(&mut self) {
        self.logs.clear();
    }
    
    /// 执行命令
    pub fn execute_command(&mut self, command: &str) -> Result<String, String> {
        // 添加到命令历史
        self.command_history.push_back(command.to_string());
        while self.command_history.len() > self.max_commands {
            self.command_history.pop_front();
        }
        
        // 解析并执行命令
        self.parse_and_execute(command)
    }
    
    /// 解析并执行命令
    fn parse_and_execute(&self, command: &str) -> Result<String, String> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Err("Empty command".to_string());
        }
        
        match parts[0] {
            "help" => Ok(self.get_help()),
            "clear" => Ok("Logs cleared".to_string()),
            "echo" => Ok(parts[1..].join(" ")),
            "version" => Ok("Game Engine v0.1.0".to_string()),
            _ => Err(format!("Unknown command: {}", parts[0])),
        }
    }
    
    /// 获取帮助信息
    fn get_help(&self) -> String {
        "Available commands:\n\
         - help: Show this help message\n\
         - clear: Clear console logs\n\
         - echo <message>: Echo a message\n\
         - version: Show engine version".to_string()
    }
    
    /// 设置过滤器
    pub fn set_filter(&mut self, filter: LogFilter) {
        self.filter = filter;
    }
    
    /// 获取命令历史
    pub fn get_command_history(&self) -> &VecDeque<String> {
        &self.command_history
    }
}

impl LogFilter {
    /// 检查日志是否匹配过滤器
    pub fn matches(&self, entry: &LogEntry) -> bool {
        // 检查日志级别
        if !self.level_matches(entry.level) {
            return false;
        }
        
        // 检查搜索关键词
        if let Some(ref term) = self.search_term {
            if !entry.message.contains(term) {
                return false;
            }
        }
        
        // 检查来源过滤
        if let Some(ref source_filter) = self.source_filter {
            if let Some(ref source) = entry.source {
                if source != source_filter {
                    return false;
                }
            } else {
                return false;
            }
        }
        
        true
    }
    
    /// 检查日志级别是否匹配
    fn level_matches(&self, level: LogLevel) -> bool {
        match self.min_level {
            LogLevel::Trace => true,
            LogLevel::Debug => matches!(level, LogLevel::Debug | LogLevel::Info | LogLevel::Warning | LogLevel::Error),
            LogLevel::Info => matches!(level, LogLevel::Info | LogLevel::Warning | LogLevel::Error),
            LogLevel::Warning => matches!(level, LogLevel::Warning | LogLevel::Error),
            LogLevel::Error => matches!(level, LogLevel::Error),
        }
    }
}

impl Default for EditorConsole {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_editor_console() {
        let mut console = EditorConsole::new();
        
        // 添加日志
        console.log(LogLevel::Info, "Test message 1", None);
        console.log(LogLevel::Warning, "Test message 2", Some("renderer".to_string()));
        console.log(LogLevel::Error, "Test message 3", None);
        
        // 获取所有日志
        let logs = console.get_filtered_logs();
        assert_eq!(logs.len(), 3);
        
        // 设置过滤器
        console.set_filter(LogFilter {
            min_level: LogLevel::Warning,
            search_term: None,
            source_filter: None,
        });
        
        // 获取过滤后的日志
        let logs = console.get_filtered_logs();
        assert_eq!(logs.len(), 2);
    }
    
    #[test]
    fn test_execute_command() {
        let mut console = EditorConsole::new();
        
        // 执行命令
        let result = console.execute_command("help");
        assert!(result.is_ok());
        
        let result = console.execute_command("echo Hello World");
        assert_eq!(result.unwrap(), "Hello World");
        
        let result = console.execute_command("unknown");
        assert!(result.is_err());
    }
}
