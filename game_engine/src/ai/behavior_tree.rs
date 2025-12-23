//  行为树系统
// 
//  实现行为树的创建、执行和管理。

// 移除未使用的通配符导入，如果需要特定类型可以单独导入
use std::boxed::Box;

/// 节点执行状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// 正在执行中
    Running,
    /// 执行成功
    Success,
    /// 执行失败
    Failure,
}

/// 行为树节点 trait
pub trait Node: Send + Sync {
    /// 执行节点并返回状态
    fn tick(&mut self) -> Status;
}

/// 顺序节点，按顺序执行子节点
pub struct Sequence {
    /// 子节点列表
    pub children: Vec<Box<dyn Node>>,
}

impl Node for Sequence {
    fn tick(&mut self) -> Status {
        for child in &mut self.children {
            match child.tick() {
                Status::Failure => return Status::Failure,
                Status::Running => return Status::Running,
                Status::Success => continue,
            }
        }
        Status::Success
    }
}

/// 选择器节点，按顺序尝试子节点，直到有一个成功
pub struct Selector {
    /// 子节点列表
    pub children: Vec<Box<dyn Node>>,
}

impl Node for Selector {
    fn tick(&mut self) -> Status {
        for child in &mut self.children {
            match child.tick() {
                Status::Success => return Status::Success,
                Status::Running => return Status::Running,
                Status::Failure => continue,
            }
        }
        Status::Failure
    }
}

// 装饰器节点类型
/// 反转器节点，反转子节点的执行结果
pub struct Inverter {
    /// 子节点
    pub child: Box<dyn Node>,
}

/// 成功器节点，总是返回成功状态
pub struct Succeeder {
    /// 子节点
    pub child: Box<dyn Node>,
}

/// 重复器节点，重复执行子节点
pub struct Repeat {
    /// 子节点
    pub child: Box<dyn Node>,
}

impl Node for Inverter {
    fn tick(&mut self) -> Status {
        match self.child.tick() {
            Status::Success => Status::Failure,
            Status::Failure => Status::Success,
            Status::Running => Status::Running,
        }
    }
}

impl Node for Succeeder {
    fn tick(&mut self) -> Status {
        self.child.tick();
        Status::Success
    }
}

impl Node for Repeat {
    fn tick(&mut self) -> Status {
        loop {
            match self.child.tick() {
                Status::Failure => return Status::Failure,
                Status::Running => return Status::Running,
                Status::Success => continue,
            }
        }
    }
}

// 叶子节点类型
/// 动作节点，执行具体动作
pub struct Action;

/// 条件节点，检查条件是否满足
pub struct Condition;
impl Node for Action {
    fn tick(&mut self) -> Status {
        Status::Success
    }
}

impl Node for Condition {
    fn tick(&mut self) -> Status {
        Status::Success
    }
}

// 行为树结构体
/// 行为树，用于AI决策
pub struct BehaviorTree {
    /// 根节点
    pub root: Box<dyn Node>,
}

impl BehaviorTree {
    /// 创建新的行为树
    pub fn new(root: Box<dyn Node>) -> Self {
        Self { root }
    }

    /// 执行行为树
    pub fn tick(&mut self) -> Status {
        self.root.tick()
    }
}
