//! Tracy Profiler使用示例
//!
//! 演示如何使用Tracy进行性能分析和火焰图生成

use game_engine::profiling::tracy::{TracyMessage, TracyProfiler, TracyScope};

fn main() {
    println!("🚀 Tracy Profiler示例");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // 创建分析器
    let mut profiler = TracyProfiler::new();
    profiler.set_enabled(true);

    if !profiler.is_enabled() {
        println!("⚠️  Tracy未启用，请使用 --features tracy 编译");
        println!("   示例: cargo run --example tracy_profiling --features tracy");
        return;
    }

    println!("✓ Tracy Profiler已启用");
    println!();

    // 示例1: 基本作用域测量
    println!("示例1: 基本作用域测量");
    {
        let _scope = TracyScope::new("example_function");
        // 模拟一些工作
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    println!("✓ 作用域测量完成");
    println!();

    // 示例2: 带颜色的作用域
    println!("示例2: 带颜色的作用域");
    {
        let _scope = TracyScope::with_color("colored_scope", 0xFF0000);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    println!("✓ 带颜色作用域完成");
    println!();

    // 示例3: 发送消息
    println!("示例3: 发送消息");
    TracyMessage::text("这是一个重要的性能事件");
    TracyMessage::colored("带颜色的消息", 0x00FF00);
    println!("✓ 消息已发送");
    println!();

    // 示例4: 帧标记
    println!("示例4: 帧标记");
    for i in 0..5 {
        TracyMessage::frame_mark_named(&format!("frame_{}", i));
        std::thread::sleep(std::time::Duration::from_millis(16)); // 模拟60fps
    }
    println!("✓ 帧标记完成");
    println!();

    // 示例5: 嵌套作用域
    println!("示例5: 嵌套作用域");
    {
        let _outer = TracyScope::new("outer_scope");
        std::thread::sleep(std::time::Duration::from_millis(5));

        {
            let _inner = TracyScope::new("inner_scope");
            std::thread::sleep(std::time::Duration::from_millis(3));
        }
    }
    println!("✓ 嵌套作用域完成");
    println!();

    println!("✅ 所有示例完成！");
    println!();
    println!("📊 查看性能数据:");
    println!("   1. 启动Tracy Profiler应用程序");
    println!("   2. 连接到正在运行的应用程序");
    println!("   3. 查看火焰图和性能分析数据");
}

