// Behavior Tree Benchmarks
//
// Measures behavior tree creation and execution performance

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq)]
enum NodeStatus {
    Success,
    Failure,
    Running,
}

type NodeExecutor = Arc<dyn Fn(&BehaviorContext) -> NodeStatus + Send + Sync>;

#[derive(Clone)]
enum BehaviorNode {
    Sequence {
        children: Vec<BehaviorNode>,
    },
    Selector {
        children: Vec<BehaviorNode>,
    },
    Parallel {
        children: Vec<BehaviorNode>,
        success_threshold: usize,
    },
    Action {
        name: String,
        executor: NodeExecutor,
    },
    Condition {
        executor: NodeExecutor,
    },
    Decorator {
        child: Box<BehaviorNode>,
        decorator: NodeExecutor,
    },
}

struct BehaviorContext {
    delta_time: f32,
    blackboard: std::collections::HashMap<String, f32>,
}

impl BehaviorContext {
    fn new(delta_time: f32) -> Self {
        Self {
            delta_time,
            blackboard: std::collections::HashMap::new(),
        }
    }
}

impl BehaviorNode {
    fn execute(&self, ctx: &BehaviorContext) -> NodeStatus {
        match self {
            BehaviorNode::Sequence { children } => {
                for child in children {
                    match child.execute(ctx) {
                        NodeStatus::Failure => return NodeStatus::Failure,
                        NodeStatus::Running => return NodeStatus::Running,
                        NodeStatus::Success => continue,
                    }
                }
                NodeStatus::Success
            }
            BehaviorNode::Selector { children } => {
                for child in children {
                    match child.execute(ctx) {
                        NodeStatus::Success => return NodeStatus::Success,
                        NodeStatus::Running => return NodeStatus::Running,
                        NodeStatus::Failure => continue,
                    }
                }
                NodeStatus::Failure
            }
            BehaviorNode::Parallel {
                children,
                success_threshold,
            } => {
                let mut success_count = 0;
                let mut failure_count = 0;

                for child in children {
                    match child.execute(ctx) {
                        NodeStatus::Success => success_count += 1,
                        NodeStatus::Failure => failure_count += 1,
                        NodeStatus::Running => return NodeStatus::Running,
                    }
                }

                if success_count >= *success_threshold {
                    NodeStatus::Success
                } else {
                    NodeStatus::Failure
                }
            }
            BehaviorNode::Action { executor, .. } => executor(ctx),
            BehaviorNode::Condition { executor } => executor(ctx),
            BehaviorNode::Decorator { child, decorator } => {
                let decorator_result = decorator(ctx);
                if decorator_result == NodeStatus::Success {
                    child.execute(ctx)
                } else {
                    decorator_result
                }
            }
        }
    }
}

// Test fixtures
fn create_test_tree(depth: usize, branching_factor: usize) -> BehaviorNode {
    if depth == 0 {
        BehaviorNode::Action {
            name: "leaf".to_string(),
            executor: Arc::new(|_| NodeStatus::Success),
        }
    } else {
        let children = (0..branching_factor)
            .map(|_| create_test_tree(depth - 1, branching_factor))
            .collect();

        BehaviorNode::Sequence { children }
    }
}

fn create_complex_tree() -> BehaviorNode {
    BehaviorNode::Selector {
        children: vec![
            BehaviorNode::Sequence {
                children: vec![
                    BehaviorNode::Condition {
                        executor: Arc::new(|_| NodeStatus::Success),
                    },
                    BehaviorNode::Action {
                        name: "action1".to_string(),
                        executor: Arc::new(|_| NodeStatus::Success),
                    },
                ],
            },
            BehaviorNode::Parallel {
                children: vec![
                    BehaviorNode::Action {
                        name: "action2".to_string(),
                        executor: Arc::new(|_| NodeStatus::Success),
                    },
                    BehaviorNode::Action {
                        name: "action3".to_string(),
                        executor: Arc::new(|_| NodeStatus::Success),
                    },
                ],
                success_threshold: 1,
            },
        ],
    }
}

fn bench_tree_execution_depth(c: &mut Criterion) {
    let mut group = c.benchmark_group("tree_execution_depth");
    group.measurement_time(Duration::from_secs(10));

    for depth in [5, 10, 15, 20].iter() {
        let tree = create_test_tree(*depth, 2);
        let ctx = BehaviorContext::new(0.016);

        group.bench_with_input(BenchmarkId::from_parameter(depth), depth, |b, _| {
            b.iter(|| black_box(tree.execute(black_box(&ctx))));
        });
    }

    group.finish();
}

fn bench_tree_execution_branching(c: &mut Criterion) {
    let mut group = c.benchmark_group("tree_execution_branching");
    group.measurement_time(Duration::from_secs(10));

    for branching in [2, 3, 4, 5].iter() {
        let tree = create_test_tree(10, *branching);
        let ctx = BehaviorContext::new(0.016);

        group.bench_with_input(
            BenchmarkId::from_parameter(branching),
            branching,
            |b, _| {
                b.iter(|| black_box(tree.execute(black_box(&ctx))));
            },
        );
    }

    group.finish();
}

fn bench_tree_execution_repeated(c: &mut Criterion) {
    let mut group = c.benchmark_group("tree_execution_repeated");
    group.measurement_time(Duration::from_secs(10));

    let tree = create_complex_tree();
    let ctx = BehaviorContext::new(0.016);

    for iterations in [100, 1_000, 10_000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(iterations),
            iterations,
            |b, &n| {
                b.iter(|| {
                    for _ in 0..n {
                        black_box(tree.execute(black_box(&ctx)));
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_multiple_trees(c: &mut Criterion) {
    let mut group = c.benchmark_group("multiple_trees");
    group.measurement_time(Duration::from_secs(10));

    for tree_count in [10, 50, 100, 500].iter() {
        let trees: Vec<_> = (0..*tree_count)
            .map(|_| create_test_tree(5, 2))
            .collect();
        let ctx = BehaviorContext::new(0.016);

        group.bench_with_input(
            BenchmarkId::from_parameter(tree_count),
            tree_count,
            |b, _| {
                b.iter(|| {
                    for tree in &trees {
                        black_box(tree.execute(black_box(&ctx)));
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_condition_nodes(c: &mut Criterion) {
    let mut group = c.benchmark_group("condition_nodes");
    group.measurement_time(Duration::from_secs(10));

    let tree = BehaviorNode::Sequence {
        children: (0..100)
            .map(|_| BehaviorNode::Condition {
                executor: Arc::new(|ctx| {
                    if ctx.blackboard.get("test").unwrap_or(&0.0) > &0.5 {
                        NodeStatus::Success
                    } else {
                        NodeStatus::Failure
                    }
                }),
            })
            .collect(),
    };

    let mut ctx = BehaviorContext::new(0.016);
    ctx.blackboard.insert("test".to_string(), 0.7);

    group.bench_function("100_conditions", |b| {
        b.iter(|| black_box(tree.execute(black_box(&ctx))));
    });

    group.finish();
}

criterion_group!(
    name = behavior_benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(15))
        .sample_size(100);
    targets =
        bench_tree_execution_depth,
        bench_tree_execution_branching,
        bench_tree_execution_repeated,
        bench_multiple_trees,
        bench_condition_nodes
);

criterion_main!(behavior_benches);
