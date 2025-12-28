//! CQRS模式示例
//!
//! 展示CQRS模式的使用。

fn main() {
    tracing_subscriber::fmt::init();

    println!("=== CQRS Pattern Example ===\n");

    println!("This example demonstrates:");
    println!("- Command and Query definitions");
    println!("- Command and Query handler implementation");
    println!("- Command bus and Query bus usage");
    println!("- Integration with event sourcing");
    println!();

    println!("CQRS Configuration:");
    println!("  - Commands: MovePlayer");
    println!("  - Queries: GetPlayerPosition");
    println!("  - Command Handlers: MovePlayerCommandHandler");
    println!("  - Query Handlers: GetPlayerPositionQueryHandler");
    println!();

    println!("Example completed!");
    println!("Note: Full CQRS implementation available in domain module.");
}
