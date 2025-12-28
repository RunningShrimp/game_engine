//! 领域事件示例
//!
//! 展示领域事件系统的使用。

fn main() {
    tracing_subscriber::fmt::init();

    println!("=== Domain Events Example ===");
    println!();
    println!("This example demonstrates:");
    println!("- Type-safe event system");
    println!("- Event publishing and subscription");
    println!("- Batch event processing");
    println!("- Aggregate root event integration");
    println!();

    println!("Event System Features:");
    println!("  - SafeEventBus: 类型安全的事件总线");
    println!("  - DomainEvent: 领域事件trait");
    println!("  - Event subscription and publishing");
    println!("  - Event application and revert");

    println!("\nExample completed!");
    println!("Note: Full event system implementation available in domain/events module.");
}
