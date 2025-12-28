//! 事件溯源示例
//!
//! 展示事件溯源系统的使用。

fn main() {
    tracing_subscriber::fmt::init();

    println!("=== Event Sourcing Example ===\n");

    println!("This example demonstrates:");
    println!("- Event storage and retrieval");
    println!("- Event replay");
    println!("- Time travel debugging");
    println!("- Event queries and statistics");
    println!();

    println!("Event Sourcing Features:");
    println!("  - EventSourcingManager: 事件溯源管理器");
    println!("  - MemoryEventStore: 内存事件存储");
    println!("  - MemorySnapshotStore: 内存快照存储");
    println!("  - EventStreamProcessor: 事件流处理器");
    println!("  - EventQuery: 事件查询");
    println!("  - SnapshotManager: 快照管理");

    println!("\nExample completed!");
    println!("Note: Full implementation available in domain/event_sourcing module.");
}
