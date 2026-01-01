//! AI组件系统
//!
//! 提供行为树节点、黑板等AI组件，用于构建NPC行为。

use glam::Vec3;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// 行为执行状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BehaviorStatus {
    /// 成功
    Success,
    /// 失败
    Failure,
    /// 运行中
    Running,
}

/// 行为树上下文
///
/// 包含执行行为树节点所需的所有上下文信息，包括黑板。
pub struct BehaviorContext {
    /// 黑板，用于存储和共享数据
    pub blackboard: Blackboard,
}

impl BehaviorContext {
    /// 创建新的行为树上下文
    pub fn new() -> Self {
        Self {
            blackboard: Blackboard::new(),
        }
    }
}

impl Default for BehaviorContext {
    fn default() -> Self {
        Self::new()
    }
}

/// 黑板
///
/// 用于在行为树节点之间共享数据的数据结构。
#[derive(Clone)]
pub struct Blackboard {
    /// 内部数据存储
    data: Arc<Mutex<HashMap<String, BlackboardValue>>>,
}

/// 黑板值类型
#[derive(Debug, Clone)]
pub enum BlackboardValue {
    Bool(bool),
    I32(i32),
    F32(f32),
    F64(f64),
    String(String),
    Vec3(Vec3),
    Vec(Vec<BlackboardValue>),
}

impl Blackboard {
    /// 创建新的黑板
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 获取布尔值
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.data.lock().ok()?.get(key).and_then(|v| {
            if let BlackboardValue::Bool(b) = v {
                Some(*b)
            } else {
                None
            }
        })
    }

    /// 获取i32值
    pub fn get_i32(&self, key: &str) -> Option<i32> {
        self.data.lock().ok()?.get(key).and_then(|v| {
            if let BlackboardValue::I32(i) = v {
                Some(*i)
            } else {
                None
            }
        })
    }

    /// 获取f32值
    pub fn get_f32(&self, key: &str) -> Option<f32> {
        self.data.lock().ok()?.get(key).and_then(|v| {
            if let BlackboardValue::F32(f) = v {
                Some(*f)
            } else {
                None
            }
        })
    }

    /// 获取f64值
    pub fn get_f64(&self, key: &str) -> Option<f64> {
        self.data.lock().ok()?.get(key).and_then(|v| {
            if let BlackboardValue::F64(f) = v {
                Some(*f)
            } else {
                None
            }
        })
    }

    /// 获取字符串值
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.data.lock().ok()?.get(key).and_then(|v| {
            if let BlackboardValue::String(s) = v {
                Some(s.clone())
            } else {
                None
            }
        })
    }

    /// 获取Vec3值
    pub fn get_vec3(&self, key: &str) -> Option<Vec3> {
        self.data.lock().ok()?.get(key).and_then(|v| {
            if let BlackboardValue::Vec3(v) = v {
                Some(*v)
            } else {
                None
            }
        })
    }

    /// 获取Vec值（简化实现，返回Vec<BlackboardValue>）
    pub fn get_vec(&self, key: &str) -> Option<Vec<BlackboardValue>> {
        self.data.lock().ok()?.get(key).and_then(|v| {
            if let BlackboardValue::Vec(v) = v {
                Some(v.clone())
            } else {
                None
            }
        })
    }

    /// 获取usize值（从i32转换）
    pub fn get_usize(&self, key: &str) -> Option<usize> {
        self.get_i32(key).map(|i| i as usize)
    }

    /// 设置值
    pub fn set<T: Into<BlackboardValue>>(&self, key: &str, value: T) {
        if let Ok(mut data) = self.data.lock() {
            data.insert(key.to_string(), value.into());
        }
    }
}

impl Default for Blackboard {
    fn default() -> Self {
        Self::new()
    }
}

impl From<bool> for BlackboardValue {
    fn from(b: bool) -> Self {
        BlackboardValue::Bool(b)
    }
}

impl From<i32> for BlackboardValue {
    fn from(i: i32) -> Self {
        BlackboardValue::I32(i)
    }
}

impl From<f32> for BlackboardValue {
    fn from(f: f32) -> Self {
        BlackboardValue::F32(f)
    }
}

impl From<f64> for BlackboardValue {
    fn from(f: f64) -> Self {
        BlackboardValue::F64(f)
    }
}

impl From<String> for BlackboardValue {
    fn from(s: String) -> Self {
        BlackboardValue::String(s)
    }
}

impl From<&str> for BlackboardValue {
    fn from(s: &str) -> Self {
        BlackboardValue::String(s.to_string())
    }
}

impl From<Vec3> for BlackboardValue {
    fn from(v: Vec3) -> Self {
        BlackboardValue::Vec3(v)
    }
}

impl From<Vec<BlackboardValue>> for BlackboardValue {
    fn from(v: Vec<BlackboardValue>) -> Self {
        BlackboardValue::Vec(v)
    }
}

/// 行为树
///
/// 用于组织和管理行为树节点。
pub struct BehaviorTree {
    /// 行为树名称
    pub name: String,
    /// 根节点
    root: Option<Box<dyn BehaviorNode>>,
}

impl BehaviorTree {
    /// 创建新的行为树
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            root: None,
        }
    }

    /// 设置根节点
    pub fn set_root(&mut self, root: Box<dyn BehaviorNode>) {
        self.root = Some(root);
    }

    /// 执行行为树
    pub fn tick(&mut self, ctx: &mut BehaviorContext) -> BehaviorStatus {
        if let Some(ref mut root) = self.root {
            root.tick(ctx)
        } else {
            BehaviorStatus::Failure
        }
    }
}

/// 行为树节点trait
pub trait BehaviorNode: Send + Sync {
    /// 执行节点
    fn tick(&mut self, ctx: &mut BehaviorContext) -> BehaviorStatus;

    /// 获取节点名称
    fn name(&self) -> &str;
}

/// 选择器节点
///
/// 按顺序尝试执行子节点，直到有一个成功。
pub struct SelectorNode {
    /// 节点名称
    name: String,
    /// 子节点列表
    children: Vec<Box<dyn BehaviorNode>>,
}

impl SelectorNode {
    /// 创建新的选择器节点
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            children: Vec::new(),
        }
    }

    /// 添加子节点
    pub fn add_child(&mut self, child: Box<dyn BehaviorNode>) {
        self.children.push(child);
    }
}

impl BehaviorNode for SelectorNode {
    fn tick(&mut self, ctx: &mut BehaviorContext) -> BehaviorStatus {
        for child in &mut self.children {
            match child.tick(ctx) {
                BehaviorStatus::Success => return BehaviorStatus::Success,
                BehaviorStatus::Running => return BehaviorStatus::Running,
                BehaviorStatus::Failure => continue,
            }
        }
        BehaviorStatus::Failure
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// 序列节点
///
/// 按顺序执行所有子节点，全部成功才算成功。
pub struct SequenceNode {
    /// 节点名称
    name: String,
    /// 子节点列表
    children: Vec<Box<dyn BehaviorNode>>,
}

impl SequenceNode {
    /// 创建新的序列节点
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            children: Vec::new(),
        }
    }

    /// 添加子节点
    pub fn add_child(&mut self, child: Box<dyn BehaviorNode>) {
        self.children.push(child);
    }
}

impl BehaviorNode for SequenceNode {
    fn tick(&mut self, ctx: &mut BehaviorContext) -> BehaviorStatus {
        for child in &mut self.children {
            match child.tick(ctx) {
                BehaviorStatus::Failure => return BehaviorStatus::Failure,
                BehaviorStatus::Running => return BehaviorStatus::Running,
                BehaviorStatus::Success => continue,
            }
        }
        BehaviorStatus::Success
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// 条件节点
///
/// 检查条件是否满足。
pub struct ConditionNode {
    /// 节点名称
    name: String,
    /// 条件检查函数
    condition: Box<dyn Fn(&BehaviorContext) -> bool + Send + Sync>,
}

impl ConditionNode {
    /// 创建新的条件节点
    pub fn new<F>(name: &str, condition: F) -> Self
    where
        F: Fn(&BehaviorContext) -> bool + Send + Sync + 'static,
    {
        Self {
            name: name.to_string(),
            condition: Box::new(condition),
        }
    }
}

impl BehaviorNode for ConditionNode {
    fn tick(&mut self, ctx: &mut BehaviorContext) -> BehaviorStatus {
        if (self.condition)(ctx) {
            BehaviorStatus::Success
        } else {
            BehaviorStatus::Failure
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// 动作节点
///
/// 执行具体动作。
pub struct ActionNode {
    /// 节点名称
    name: String,
    /// 动作执行函数
    action: Box<dyn FnMut(&mut BehaviorContext) -> BehaviorStatus + Send + Sync>,
}

impl ActionNode {
    /// 创建新的动作节点
    pub fn new<F>(name: &str, action: F) -> Self
    where
        F: FnMut(&mut BehaviorContext) -> BehaviorStatus + Send + Sync + 'static,
    {
        Self {
            name: name.to_string(),
            action: Box::new(action),
        }
    }
}

impl BehaviorNode for ActionNode {
    fn tick(&mut self, ctx: &mut BehaviorContext) -> BehaviorStatus {
        (self.action)(ctx)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blackboard_set_get() {
        let blackboard = Blackboard::new();
        blackboard.set("test_bool", true);
        blackboard.set("test_i32", 42);
        blackboard.set("test_f32", 3.14);
        blackboard.set("test_string", "hello");
        blackboard.set("test_vec3", Vec3::new(1.0, 2.0, 3.0));

        assert_eq!(blackboard.get_bool("test_bool"), Some(true));
        assert_eq!(blackboard.get_i32("test_i32"), Some(42));
        assert_eq!(blackboard.get_f32("test_f32"), Some(3.14));
        assert_eq!(
            blackboard.get_string("test_string"),
            Some("hello".to_string())
        );
        assert_eq!(
            blackboard.get_vec3("test_vec3"),
            Some(Vec3::new(1.0, 2.0, 3.0))
        );
    }

    #[test]
    fn test_selector_node() {
        let mut selector = SelectorNode::new("Test Selector");
        let mut ctx = BehaviorContext::new();

        // 添加一个总是失败的条件节点
        let fail_condition = ConditionNode::new("Fail", |_| false);
        selector.add_child(Box::new(fail_condition));

        // 添加一个总是成功的条件节点
        let success_condition = ConditionNode::new("Success", |_| true);
        selector.add_child(Box::new(success_condition));

        assert_eq!(selector.tick(&mut ctx), BehaviorStatus::Success);
    }

    #[test]
    fn test_sequence_node() {
        let mut sequence = SequenceNode::new("Test Sequence");
        let mut ctx = BehaviorContext::new();

        // 添加两个总是成功的条件节点
        let success1 = ConditionNode::new("Success1", |_| true);
        let success2 = ConditionNode::new("Success2", |_| true);
        sequence.add_child(Box::new(success1));
        sequence.add_child(Box::new(success2));

        assert_eq!(sequence.tick(&mut ctx), BehaviorStatus::Success);
    }
}
