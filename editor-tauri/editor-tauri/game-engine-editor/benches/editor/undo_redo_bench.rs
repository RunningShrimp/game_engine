// Undo/Redo Benchmarks
//
// Measures the performance of undo/redo system operations

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;
use uuid::Uuid;

#[derive(Clone, Debug)]
struct Command {
    id: Uuid,
    name: String,
    execute_data: CommandData,
    undo_data: CommandData,
}

#[derive(Clone, Debug)]
enum CommandData {
    CreateEntity {
        entity_id: Uuid,
        name: String,
        position: (f32, f32, f32),
    },
    DeleteEntity {
        entity_id: Uuid,
        name: String,
        position: (f32, f32, f32),
    },
    MoveEntity {
        entity_id: Uuid,
        from: (f32, f32, f32),
        to: (f32, f32, f32),
    },
    UpdateProperty {
        entity_id: Uuid,
        property: String,
        old_value: String,
        new_value: String,
    },
}

struct HistoryManager {
    undo_stack: Vec<Command>,
    redo_stack: Vec<Command>,
    max_history: usize,
}

impl HistoryManager {
    fn new(max_history: usize) -> Self {
        Self {
            undo_stack: Vec::with_capacity(max_history),
            redo_stack: Vec::with_capacity(max_history),
            max_history,
        }
    }

    fn execute_command(&mut self, command: Command) {
        // Clear redo stack on new command
        self.redo_stack.clear();

        // Add to undo stack
        if self.undo_stack.len() >= self.max_history {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(command);
    }

    fn undo(&mut self) -> Option<Command> {
        let command = self.undo_stack.pop()?;
        self.redo_stack.push(command.clone());
        Some(command)
    }

    fn redo(&mut self) -> Option<Command> {
        let command = self.redo_stack.pop()?;
        self.undo_stack.push(command.clone());
        Some(command)
    }

    fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }
}

// Test fixtures
fn create_test_commands(count: usize) -> Vec<Command> {
    (0..count)
        .map(|i| Command {
            id: Uuid::new_v4(),
            name: format!("Command_{}", i),
            execute_data: CommandData::CreateEntity {
                entity_id: Uuid::new_v4(),
                name: format!("Entity_{}", i),
                position: (i as f32, 0.0, 0.0),
            },
            undo_data: CommandData::DeleteEntity {
                entity_id: Uuid::new_v4(),
                name: format!("Entity_{}", i),
                position: (i as f32, 0.0, 0.0),
            },
        })
        .collect()
}

fn bench_execute_commands(c: &mut Criterion) {
    let mut group = c.benchmark_group("execute_commands");
    group.measurement_time(Duration::from_secs(10));

    for count in [10, 50, 100, 500, 1_000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &n| {
            let commands = create_test_commands(n);
            let mut history = HistoryManager::new(1000);

            b.iter(|| {
                for cmd in &commands {
                    black_box(history.execute_command(black_box(cmd.clone())));
                }
            });
        });
    }

    group.finish();
}

fn bench_undo_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("undo_operations");
    group.measurement_time(Duration::from_secs(10));

    for count in [10, 50, 100, 500].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &n| {
            let commands = create_test_commands(n);
            let mut history = HistoryManager::new(1000);

            // Execute commands
            for cmd in &commands {
                history.execute_command(cmd.clone());
            }

            b.iter(|| {
                for _ in 0..n {
                    black_box(history.undo());
                }
            });
        });
    }

    group.finish();
}

fn bench_redo_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("redo_operations");
    group.measurement_time(Duration::from_secs(10));

    for count in [10, 50, 100, 500].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &n| {
            let commands = create_test_commands(n);
            let mut history = HistoryManager::new(1000);

            // Execute and undo commands
            for cmd in &commands {
                history.execute_command(cmd.clone());
            }
            for _ in 0..n {
                history.undo();
            }

            b.iter(|| {
                for _ in 0..n {
                    black_box(history.redo());
                }
            });
        });
    }

    group.finish();
}

fn bench_undo_redo_cycle(c: &mut Criterion) {
    let mut group = c.benchmark_group("undo_redo_cycle");
    group.measurement_time(Duration::from_secs(10));

    for count in [10, 50, 100, 500].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &n| {
            let commands = create_test_commands(n);

            b.iter(|| {
                let mut history = HistoryManager::new(1000);

                // Execute
                for cmd in &commands {
                    history.execute_command(cmd.clone());
                }

                // Undo all
                while history.can_undo() {
                    history.undo();
                }

                // Redo all
                while history.can_redo() {
                    history.redo();
                }
            });
        });
    }

    group.finish();
}

fn bench_large_history(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_history");
    group.measurement_time(Duration::from_secs(10));

    for history_size in [100, 1_000, 10_000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(history_size),
            history_size,
            |b, &size| {
                let mut history = HistoryManager::new(size);

                b.iter(|| {
                    // Fill history
                    for i in 0..size {
                        let cmd = Command {
                            id: Uuid::new_v4(),
                            name: format!("Command_{}", i),
                            execute_data: CommandData::MoveEntity {
                                entity_id: Uuid::new_v4(),
                                from: (0.0, 0.0, 0.0),
                                to: (1.0, 0.0, 0.0),
                            },
                            undo_data: CommandData::MoveEntity {
                                entity_id: Uuid::new_v4(),
                                from: (1.0, 0.0, 0.0),
                                to: (0.0, 0.0, 0.0),
                            },
                        };
                        history.execute_command(cmd);
                    }

                    // Undo all
                    while history.can_undo() {
                        history.undo();
                    }

                    black_box(history.undo_count());
                });
            },
        );
    }

    group.finish();
}

fn bench_command_cloning(c: &mut Criterion) {
    let mut group = c.benchmark_group("command_cloning");
    group.measurement_time(Duration::from_secs(10));

    let commands = create_test_commands(100);

    for batch_size in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            batch_size,
            |b, &size| {
                b.iter(|| {
                    for cmd in commands.iter().take(size) {
                        black_box(cmd.clone());
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_memory_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_overhead");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("1000_commands", |b| {
        b.iter(|| {
            let history = HistoryManager::new(1000);
            let commands = create_test_commands(1000);

            let size = std::mem::size_of_val(&commands) + std::mem::size_of_val(&history);

            black_box(size);
        });
    });

    group.finish();
}

criterion_group!(
    name = undo_redo_benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(15))
        .sample_size(100);
    targets =
        bench_execute_commands,
        bench_undo_operations,
        bench_redo_operations,
        bench_undo_redo_cycle,
        bench_large_history,
        bench_command_cloning,
        bench_memory_overhead
);

criterion_main!(undo_redo_benches);
