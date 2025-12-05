//! 行为树系统
//!
//! 实现行为树的创建、执行和管理。

use super::*;
use std::boxed::Box;

// 节点执行状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Running,
    Success,
    Failure,
}

// 行为树节点 trait
pub trait Node: Send + Sync {
    fn tick(&mut self) -> Status;
}

// 复合节点类型
pub struct Sequence {
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

pub struct Selector {
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
pub struct Inverter {
    pub child: Box<dyn Node>,
}

pub struct Succeeder {
    pub child: Box<dyn Node>,
}

pub struct Repeat {
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
pub struct Action;

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
pub struct BehaviorTree {
    pub root: Box<dyn Node>,
}

impl BehaviorTree {
    pub fn new(root: Box<dyn Node>) -> Self {
        Self { root }
    }

    pub fn tick(&mut self) -> Status {
        self.root.tick()
    }
}
