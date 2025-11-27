//! 撤销/重做系统
//!
//! 提供编辑器操作的撤销和重做功能，基于命令模式实现。
//!
//! # 架构设计
//!
//! - `Command` trait: 定义可撤销操作的接口
//! - `CommandManager`: 管理命令历史和执行
//! - `CompositeCommand`: 组合多个命令为一个原子操作
//! - `PropertyChange`: 通用属性变更命令
//!
//! # 示例
//!
//! ```ignore
//! // 创建命令管理器
//! let mut manager = CommandManager::new(100);
//!
//! // 执行命令
//! manager.execute(Box::new(SetPositionCommand::new(entity, old_pos, new_pos)));
//!
//! // 撤销
//! manager.undo();
//!
//! // 重做
//! manager.redo();
//! ```

use std::any::Any;
use std::collections::VecDeque;
use std::fmt;

/// 命令 trait - 定义可撤销操作的接口
pub trait Command: fmt::Debug + Send {
    /// 执行命令
    fn execute(&mut self, context: &mut dyn Any) -> Result<(), CommandError>;
    
    /// 撤销命令
    fn undo(&mut self, context: &mut dyn Any) -> Result<(), CommandError>;
    
    /// 获取命令描述 (用于显示)
    fn description(&self) -> &str;
    
    /// 检查是否可以与其他命令合并
    fn can_merge(&self, _other: &dyn Command) -> bool {
        false
    }
    
    /// 合并另一个命令到当前命令
    fn merge(&mut self, _other: &dyn Command) -> Result<(), CommandError> {
        Err(CommandError::CannotMerge)
    }
    
    /// 获取命令 ID (用于合并检查)
    fn command_id(&self) -> Option<&str> {
        None
    }
}

/// 命令错误
#[derive(Debug, Clone)]
pub enum CommandError {
    /// 执行失败
    ExecutionFailed(String),
    /// 撤销失败
    UndoFailed(String),
    /// 无法合并
    CannotMerge,
    /// 无效状态
    InvalidState(String),
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandError::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
            CommandError::UndoFailed(msg) => write!(f, "Undo failed: {}", msg),
            CommandError::CannotMerge => write!(f, "Commands cannot be merged"),
            CommandError::InvalidState(msg) => write!(f, "Invalid state: {}", msg),
        }
    }
}

impl std::error::Error for CommandError {}

/// 命令管理器
/// 
/// 管理命令历史，支持撤销/重做操作
pub struct CommandManager {
    /// 撤销栈
    undo_stack: VecDeque<Box<dyn Command>>,
    /// 重做栈
    redo_stack: VecDeque<Box<dyn Command>>,
    /// 最大历史长度
    max_history: usize,
    /// 是否正在执行撤销/重做 (防止嵌套)
    is_undoing: bool,
    /// 合并窗口时间 (毫秒)
    merge_window_ms: u64,
    /// 上次命令时间戳
    last_command_time: std::time::Instant,
    /// 变更监听器
    change_listeners: Vec<Box<dyn Fn(&dyn Command, bool) + Send>>,
}

impl CommandManager {
    /// 创建命令管理器
    pub fn new(max_history: usize) -> Self {
        Self {
            undo_stack: VecDeque::with_capacity(max_history),
            redo_stack: VecDeque::with_capacity(max_history / 2),
            max_history,
            is_undoing: false,
            merge_window_ms: 500,
            last_command_time: std::time::Instant::now(),
            change_listeners: Vec::new(),
        }
    }
    
    /// 设置合并窗口时间
    pub fn set_merge_window(&mut self, ms: u64) {
        self.merge_window_ms = ms;
    }
    
    /// 添加变更监听器
    pub fn add_listener<F>(&mut self, listener: F)
    where
        F: Fn(&dyn Command, bool) + Send + 'static,
    {
        self.change_listeners.push(Box::new(listener));
    }
    
    /// 执行命令
    pub fn execute(&mut self, mut command: Box<dyn Command>, context: &mut dyn Any) -> Result<(), CommandError> {
        if self.is_undoing {
            return Err(CommandError::InvalidState("Cannot execute while undoing".into()));
        }
        
        // 执行命令
        command.execute(context)?;
        
        // 检查是否可以与上一个命令合并
        let now = std::time::Instant::now();
        let time_since_last = now.duration_since(self.last_command_time).as_millis() as u64;
        
        let merged = if time_since_last < self.merge_window_ms {
            if let Some(last) = self.undo_stack.back_mut() {
                if last.can_merge(&*command) {
                    last.merge(&*command).is_ok()
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };
        
        if !merged {
            // 添加到撤销栈
            self.undo_stack.push_back(command);
            
            // 限制历史长度
            while self.undo_stack.len() > self.max_history {
                self.undo_stack.pop_front();
            }
        }
        
        // 清空重做栈
        self.redo_stack.clear();
        
        // 更新时间戳
        self.last_command_time = now;
        
        // 通知监听器
        if let Some(cmd) = self.undo_stack.back() {
            for listener in &self.change_listeners {
                listener(&**cmd, true);
            }
        }
        
        Ok(())
    }
    
    /// 撤销
    pub fn undo(&mut self, context: &mut dyn Any) -> Result<bool, CommandError> {
        if self.is_undoing {
            return Err(CommandError::InvalidState("Already undoing".into()));
        }
        
        if let Some(mut command) = self.undo_stack.pop_back() {
            self.is_undoing = true;
            let result = command.undo(context);
            self.is_undoing = false;
            
            result?;
            
            // 通知监听器
            for listener in &self.change_listeners {
                listener(&*command, false);
            }
            
            // 移到重做栈
            self.redo_stack.push_back(command);
            
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    /// 重做
    pub fn redo(&mut self, context: &mut dyn Any) -> Result<bool, CommandError> {
        if self.is_undoing {
            return Err(CommandError::InvalidState("Cannot redo while undoing".into()));
        }
        
        if let Some(mut command) = self.redo_stack.pop_back() {
            self.is_undoing = true;
            let result = command.execute(context);
            self.is_undoing = false;
            
            result?;
            
            // 通知监听器
            for listener in &self.change_listeners {
                listener(&*command, true);
            }
            
            // 移回撤销栈
            self.undo_stack.push_back(command);
            
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    /// 检查是否可以撤销
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }
    
    /// 检查是否可以重做
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
    
    /// 获取下一个撤销命令的描述
    pub fn undo_description(&self) -> Option<&str> {
        self.undo_stack.back().map(|c| c.description())
    }
    
    /// 获取下一个重做命令的描述
    pub fn redo_description(&self) -> Option<&str> {
        self.redo_stack.back().map(|c| c.description())
    }
    
    /// 获取撤销历史长度
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }
    
    /// 获取重做历史长度
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }
    
    /// 清空历史
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
    
    /// 获取所有撤销命令的描述
    pub fn get_undo_history(&self) -> Vec<&str> {
        self.undo_stack.iter().rev().map(|c| c.description()).collect()
    }
    
    /// 获取所有重做命令的描述
    pub fn get_redo_history(&self) -> Vec<&str> {
        self.redo_stack.iter().rev().map(|c| c.description()).collect()
    }
    
    /// 撤销多步
    pub fn undo_multiple(&mut self, count: usize, context: &mut dyn Any) -> Result<usize, CommandError> {
        let mut undone = 0;
        for _ in 0..count {
            if self.undo(context)? {
                undone += 1;
            } else {
                break;
            }
        }
        Ok(undone)
    }
    
    /// 重做多步
    pub fn redo_multiple(&mut self, count: usize, context: &mut dyn Any) -> Result<usize, CommandError> {
        let mut redone = 0;
        for _ in 0..count {
            if self.redo(context)? {
                redone += 1;
            } else {
                break;
            }
        }
        Ok(redone)
    }
}

impl Default for CommandManager {
    fn default() -> Self {
        Self::new(100)
    }
}

// ============================================================================
// 组合命令
// ============================================================================

/// 组合命令 - 将多个命令作为一个原子操作
#[derive(Debug)]
pub struct CompositeCommand {
    commands: Vec<Box<dyn Command>>,
    description: String,
}

impl CompositeCommand {
    /// 创建组合命令
    pub fn new(description: &str) -> Self {
        Self {
            commands: Vec::new(),
            description: description.to_string(),
        }
    }
    
    /// 添加子命令
    pub fn add(&mut self, command: Box<dyn Command>) {
        self.commands.push(command);
    }
    
    /// 从命令列表创建
    pub fn from_commands(description: &str, commands: Vec<Box<dyn Command>>) -> Self {
        Self {
            commands,
            description: description.to_string(),
        }
    }
    
    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

impl Command for CompositeCommand {
    fn execute(&mut self, context: &mut dyn Any) -> Result<(), CommandError> {
        for cmd in &mut self.commands {
            cmd.execute(context)?;
        }
        Ok(())
    }
    
    fn undo(&mut self, context: &mut dyn Any) -> Result<(), CommandError> {
        // 逆序撤销
        for cmd in self.commands.iter_mut().rev() {
            cmd.undo(context)?;
        }
        Ok(())
    }
    
    fn description(&self) -> &str {
        &self.description
    }
}

// ============================================================================
// 通用属性变更命令
// ============================================================================

/// 属性变更命令
/// 
/// 通用的属性变更命令，使用闭包实现 getter/setter
pub struct PropertyChangeCommand<T: Clone + Send + fmt::Debug + 'static> {
    /// 实体/对象标识
    target_id: u64,
    /// 旧值
    old_value: T,
    /// 新值
    new_value: T,
    /// 描述
    description: String,
    /// 应用函数
    apply_fn: Box<dyn Fn(&mut dyn Any, u64, &T) -> Result<(), String> + Send>,
}

impl<T: Clone + Send + fmt::Debug + 'static> PropertyChangeCommand<T> {
    /// 创建属性变更命令
    pub fn new<F>(
        target_id: u64,
        old_value: T,
        new_value: T,
        description: &str,
        apply_fn: F,
    ) -> Self
    where
        F: Fn(&mut dyn Any, u64, &T) -> Result<(), String> + Send + 'static,
    {
        Self {
            target_id,
            old_value,
            new_value,
            description: description.to_string(),
            apply_fn: Box::new(apply_fn),
        }
    }
}

impl<T: Clone + Send + fmt::Debug + 'static> fmt::Debug for PropertyChangeCommand<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PropertyChangeCommand")
            .field("target_id", &self.target_id)
            .field("old_value", &self.old_value)
            .field("new_value", &self.new_value)
            .field("description", &self.description)
            .finish()
    }
}

impl<T: Clone + Send + fmt::Debug + 'static> Command for PropertyChangeCommand<T> {
    fn execute(&mut self, context: &mut dyn Any) -> Result<(), CommandError> {
        (self.apply_fn)(context, self.target_id, &self.new_value)
            .map_err(|e| CommandError::ExecutionFailed(e))
    }
    
    fn undo(&mut self, context: &mut dyn Any) -> Result<(), CommandError> {
        (self.apply_fn)(context, self.target_id, &self.old_value)
            .map_err(|e| CommandError::UndoFailed(e))
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn can_merge(&self, other: &dyn Command) -> bool {
        // 尝试将 other 转换为相同类型的命令
        if let Some(cmd_id) = other.command_id() {
            if let Some(my_id) = self.command_id() {
                return cmd_id == my_id;
            }
        }
        false
    }
    
    fn command_id(&self) -> Option<&str> {
        Some(&self.description)
    }
}

// ============================================================================
// 常用命令类型
// ============================================================================

/// 实体创建命令
#[derive(Debug)]
pub struct CreateEntityCommand {
    /// 创建的实体 ID
    entity_id: Option<u64>,
    /// 实体数据
    entity_data: Vec<u8>,
    /// 描述
    description: String,
}

impl CreateEntityCommand {
    pub fn new(entity_data: Vec<u8>, description: &str) -> Self {
        Self {
            entity_id: None,
            entity_data,
            description: description.to_string(),
        }
    }
}

impl Command for CreateEntityCommand {
    fn execute(&mut self, _context: &mut dyn Any) -> Result<(), CommandError> {
        // 实际实现中，应该使用 context 创建实体并保存 ID
        // self.entity_id = Some(context.create_entity(&self.entity_data)?);
        Ok(())
    }
    
    fn undo(&mut self, _context: &mut dyn Any) -> Result<(), CommandError> {
        // 实际实现中，应该使用保存的 ID 删除实体
        // if let Some(id) = self.entity_id {
        //     context.delete_entity(id)?;
        // }
        Ok(())
    }
    
    fn description(&self) -> &str {
        &self.description
    }
}

/// 实体删除命令
#[derive(Debug)]
pub struct DeleteEntityCommand {
    /// 删除的实体 ID
    entity_id: u64,
    /// 备份的实体数据 (用于撤销时恢复)
    backup_data: Option<Vec<u8>>,
    /// 描述
    description: String,
}

impl DeleteEntityCommand {
    pub fn new(entity_id: u64, description: &str) -> Self {
        Self {
            entity_id,
            backup_data: None,
            description: description.to_string(),
        }
    }
}

impl Command for DeleteEntityCommand {
    fn execute(&mut self, _context: &mut dyn Any) -> Result<(), CommandError> {
        // 实际实现中，应该先备份实体数据，然后删除
        // self.backup_data = Some(context.serialize_entity(self.entity_id)?);
        // context.delete_entity(self.entity_id)?;
        Ok(())
    }
    
    fn undo(&mut self, _context: &mut dyn Any) -> Result<(), CommandError> {
        // 实际实现中，应该使用备份数据重新创建实体
        // if let Some(data) = &self.backup_data {
        //     context.create_entity_with_id(self.entity_id, data)?;
        // }
        Ok(())
    }
    
    fn description(&self) -> &str {
        &self.description
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[derive(Debug)]
    struct TestCommand {
        value: i32,
        executed: bool,
    }
    
    impl Command for TestCommand {
        fn execute(&mut self, _context: &mut dyn Any) -> Result<(), CommandError> {
            self.executed = true;
            Ok(())
        }
        
        fn undo(&mut self, _context: &mut dyn Any) -> Result<(), CommandError> {
            self.executed = false;
            Ok(())
        }
        
        fn description(&self) -> &str {
            "Test Command"
        }
    }
    
    #[test]
    fn test_command_manager() {
        let mut manager = CommandManager::new(10);
        let mut context: i32 = 0;
        
        assert!(!manager.can_undo());
        assert!(!manager.can_redo());
        
        // 执行命令
        manager.execute(
            Box::new(TestCommand { value: 1, executed: false }),
            &mut context,
        ).unwrap();
        
        assert!(manager.can_undo());
        assert!(!manager.can_redo());
        assert_eq!(manager.undo_count(), 1);
        
        // 撤销
        manager.undo(&mut context).unwrap();
        
        assert!(!manager.can_undo());
        assert!(manager.can_redo());
        assert_eq!(manager.redo_count(), 1);
        
        // 重做
        manager.redo(&mut context).unwrap();
        
        assert!(manager.can_undo());
        assert!(!manager.can_redo());
    }
    
    #[test]
    fn test_composite_command() {
        let mut composite = CompositeCommand::new("Multiple Changes");
        composite.add(Box::new(TestCommand { value: 1, executed: false }));
        composite.add(Box::new(TestCommand { value: 2, executed: false }));
        
        let mut context: i32 = 0;
        
        // 执行组合命令
        composite.execute(&mut context).unwrap();
        
        // 撤销组合命令
        composite.undo(&mut context).unwrap();
    }
}
