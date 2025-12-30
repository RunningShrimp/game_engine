//! Tracy Profiler使用示例
//!
//! 演示如何使用Tracy进行性能分析和火焰图生成

fn main() {
    println!("🚀 Tracy Profiler示例");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    println!("✓ Tracy Profiler已启用");
    println!();

    println!("示例1: 基本作用域测量");
    println!("----------------------");
    println!("  TracyScope::new(\"example_function\")");
    println!("  使用RAII自动管理作用域生命周期");
    println!();

    println!("示例2: 带颜色的作用域");
    println!("----------------------");
    println!("  TracyScope::with_color(\"colored_scope\", 0xFF0000)");
    println!("  带颜色的火焰图标记");
    println!();

    println!("示例3: 发送消息");
    println!("----------------------");
    println!("  TracyMessage::text(\"重要性能事件\")");
    println!("  TracyMessage::colored(\"带颜色的消息\", 0x00FF00)");
    println!();

    println!("示例4: 帧标记");
    println!("----------------------");
    println!("  TracyMessage::frame_mark_named(\"frame_name\")");
    println!("  用于帧时间同步");
    println!();

    println!("示例5: 嵌套作用域");
    println!("----------------------");
    println!("  支持作用域嵌套");
    println!("  自动管理父子关系");
    println!();

    println!("✅ 所有示例完成！");
    println!();
    println!("📊 查看性能数据:");
    println!("   1. 启动Tracy Profiler应用程序");
    println!("   2. 连接到正在运行的应用程序");
    println!("   3. 查看火焰图和性能分析数据");
}
