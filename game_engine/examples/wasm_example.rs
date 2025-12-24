//! WebAssembly示例
//!
//! 展示如何在Web平台上使用游戏引擎。
//! 这个示例演示了：
//! - WASM初始化
//! - 引擎在浏览器中运行
//! - WebGL渲染
//! - 输入处理
//! - 性能优化

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use web_sys::console;

use bevy_ecs::prelude::*;
use game_engine::core::Engine;
use game_engine::ecs::{Sprite, Transform};
use game_engine::platform::wasm_performance::{WasmLinearMemoryOptimizer, WasmMemoryPoolConfig};

#[cfg(target_arch = "wasm32")]
fn wasm_main() {
    // 设置panic hook以便在浏览器控制台看到错误
    console_error_panic_hook::set_once();

    // 初始化日志（在浏览器控制台输出）
    console::log_1(&"=== Game Engine WASM Example ===".into());

    // 初始化WASM性能优化器
    let memory_config = WasmMemoryPoolConfig::default();
    let mut optimizer = WasmLinearMemoryOptimizer::new(memory_config);

    // 获取优化建议
    let suggestions = optimizer.get_optimization_suggestions();
    for suggestion in suggestions {
        console::log_1(&format!("Optimization: {}", suggestion).into());
    }

    // 创建引擎实例
    let mut engine = Engine::new();

    // 初始化引擎
    if let Err(e) = engine.initialize() {
        console::error_1(&format!("Failed to initialize engine: {}", e).into());
        return;
    }

    console::log_1(&"Engine initialized successfully!".into());

    // 创建一些实体
    let world = engine.world_mut();

    // 创建一个简单的实体
    world.spawn((
        Transform::default(),
        Sprite {
            color: [1.0, 0.0, 0.0, 1.0],
            size: [100.0, 100.0],
        },
    ));

    console::log_1(&"Created a sprite entity".into());

    // 运行游戏循环
    console::log_1(&"Starting game loop...".into());

    // 在WASM中，游戏循环通常通过requestAnimationFrame处理
    // 这里只是示例，实际实现需要与浏览器事件循环集成
    // 注意：实际游戏循环应该由引擎内部管理
    console::log_1(&"Game loop started (simplified example)".into());
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start() {
    wasm_main();
}

/// 非WASM平台的main函数
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!("This example is designed for WASM target.");
    println!("Build with: cargo build --target wasm32-unknown-unknown --example wasm_example");
}
