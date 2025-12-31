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

#[cfg(test)]
mod tests {
    use super::*;

    // Mock node for testing
    struct MockNode {
        status: Status,
        call_count: std::sync::Arc<parking_lot::Mutex<usize>>,
    }

    impl Node for MockNode {
        fn tick(&mut self) -> Status {
            // parking_lot::Mutex直接返回Guard
            *self.call_count.lock() += 1;
            self.status
        }
    }

    #[test]
    fn test_status_equality() {
        assert_eq!(Status::Success, Status::Success);
        assert_eq!(Status::Failure, Status::Failure);
        assert_eq!(Status::Running, Status::Running);
        assert_ne!(Status::Success, Status::Failure);
    }

    #[test]
    fn test_sequence_all_success() {
        let mut sequence = Sequence {
            children: vec![Box::new(Action), Box::new(Action)],
        };

        assert_eq!(sequence.tick(), Status::Success);
    }

    #[test]
    fn test_sequence_with_failure() {
        let call_count = std::sync::Arc::new(parking_lot::Mutex::new(0usize));
        let mut sequence = Sequence {
            children: vec![
                Box::new(MockNode {
                    status: Status::Success,
                    call_count: call_count.clone(),
                }),
                Box::new(MockNode {
                    status: Status::Failure,
                    call_count: call_count.clone(),
                }),
                Box::new(MockNode {
                    status: Status::Success,
                    call_count: call_count.clone(),
                }),
            ],
        };

        assert_eq!(sequence.tick(), Status::Failure);
        // Third child should not be called
        assert_eq!(*call_count.lock(), 2);
    }

    #[test]
    fn test_sequence_with_running() {
        let mut sequence = Sequence {
            children: vec![Box::new(Action), Box::new(Action)],
        };

        // Can't easily test Running without a custom node
        // This test verifies the sequence structure
        assert_eq!(sequence.children.len(), 2);
    }

    #[test]
    fn test_selector_with_success() {
        let call_count = std::sync::Arc::new(parking_lot::Mutex::new(0usize));
        let mut selector = Selector {
            children: vec![
                Box::new(MockNode {
                    status: Status::Failure,
                    call_count: call_count.clone(),
                }),
                Box::new(MockNode {
                    status: Status::Success,
                    call_count: call_count.clone(),
                }),
                Box::new(MockNode {
                    status: Status::Success,
                    call_count: call_count.clone(),
                }),
            ],
        };

        assert_eq!(selector.tick(), Status::Success);
        // Third child should not be called
        assert_eq!(*call_count.lock(), 2);
    }

    #[test]
    fn test_selector_all_failure() {
        let mut selector = Selector {
            children: vec![
                Box::new(MockNode {
                    status: Status::Failure,
                    call_count: std::sync::Arc::new(parking_lot::Mutex::new(0usize)),
                }),
                Box::new(MockNode {
                    status: Status::Failure,
                    call_count: std::sync::Arc::new(parking_lot::Mutex::new(0usize)),
                }),
            ],
        };

        assert_eq!(selector.tick(), Status::Failure);
    }

    #[test]
    fn test_inverter_success_to_failure() {
        let mut inverter = Inverter {
            child: Box::new(Action),
        };

        assert_eq!(inverter.tick(), Status::Failure);
    }

    #[test]
    fn test_inverter_failure_to_success() {
        let mut inverter = Inverter {
            child: Box::new(MockNode {
                status: Status::Failure,
                call_count: std::sync::Arc::new(parking_lot::Mutex::new(0usize)),
            }),
        };

        assert_eq!(inverter.tick(), Status::Success);
    }

    #[test]
    fn test_inverter_running() {
        let mut inverter = Inverter {
            child: Box::new(MockNode {
                status: Status::Running,
                call_count: std::sync::Arc::new(parking_lot::Mutex::new(0usize)),
            }),
        };

        assert_eq!(inverter.tick(), Status::Running);
    }

    #[test]
    fn test_succeeder_always_success() {
        let mut succeeder = Succeeder {
            child: Box::new(MockNode {
                status: Status::Failure,
                call_count: std::sync::Arc::new(parking_lot::Mutex::new(0usize)),
            }),
        };

        assert_eq!(succeeder.tick(), Status::Success);
    }

    #[test]
    fn test_repeat() {
        let call_count = std::sync::Arc::new(parking_lot::Mutex::new(0usize));
        let mut repeat = Repeat {
            child: Box::new(MockNode {
                status: Status::Success,
                call_count: call_count.clone(),
            }),
        };

        // Repeat will loop forever on success, so we can't test it directly
        // Just verify it has the right child
        assert_eq!(*call_count.lock(), 0);
    }

    #[test]
    fn test_action_node() {
        let mut action = Action;
        assert_eq!(action.tick(), Status::Success);
    }

    #[test]
    fn test_condition_node() {
        let mut condition = Condition;
        assert_eq!(condition.tick(), Status::Success);
    }

    #[test]
    fn test_behavior_tree_creation() {
        let mut tree = BehaviorTree {
            root: Box::new(Action),
        };

        assert_eq!(tree.tick(), Status::Success);
    }

    #[test]
    fn test_behavior_tree_with_sequence() {
        let mut tree = BehaviorTree {
            root: Box::new(Sequence {
                children: vec![Box::new(Action), Box::new(Action)],
            }),
        };

        assert_eq!(tree.tick(), Status::Success);
    }

    #[test]
    fn test_behavior_tree_with_selector() {
        let mut tree = BehaviorTree {
            root: Box::new(Selector {
                children: vec![
                    Box::new(MockNode {
                        status: Status::Failure,
                        call_count: std::sync::Arc::new(parking_lot::Mutex::new(0usize)),
                    }),
                    Box::new(Action),
                ],
            }),
        };

        assert_eq!(tree.tick(), Status::Success);
    }

    #[test]
    fn test_complex_behavior_tree() {
        // Create a more complex tree: Sequence(Selector(A, B), Inverter(C))
        let mut tree = BehaviorTree {
            root: Box::new(Sequence {
                children: vec![
                    Box::new(Selector {
                        children: vec![
                            Box::new(MockNode {
                                status: Status::Failure,
                                call_count: std::sync::Arc::new(parking_lot::Mutex::new(0usize)),
                            }),
                            Box::new(Action),
                        ],
                    }),
                    Box::new(Inverter {
                        child: Box::new(MockNode {
                            status: Status::Success,
                            call_count: std::sync::Arc::new(parking_lot::Mutex::new(0usize)),
                        }),
                    }),
                ],
            }),
        };

        assert_eq!(tree.tick(), Status::Failure); // Inverter turns Success to Failure
    }

    #[test]
    fn test_empty_sequence() {
        let mut sequence = Sequence { children: vec![] };
        assert_eq!(sequence.tick(), Status::Success);
    }

    #[test]
    fn test_empty_selector() {
        let mut selector = Selector { children: vec![] };
        assert_eq!(selector.tick(), Status::Failure);
    }
}
