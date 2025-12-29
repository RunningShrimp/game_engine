//! WebAssembly示例
//!
//! 展示如何在Web平台上使用游戏引擎。

#[cfg(target_arch = "wasm32")]
use bevy_ecs::prelude::*;
#[cfg(target_arch = "wasm32")]
use game_engine::ecs::{Sprite, Transform};
#[cfg(target_arch = "wasm32")]
use glam::Quat;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use web_sys::console;

#[cfg(target_arch = "wasm32")]
fn wasm_main() {
    console::log_1(&"=== Game Engine WASM Example ===".into());

    console::log_1(&"Engine initialized successfully!".into());

    let mut world = World::new();

    // 创建一个简单的实体
    world.spawn((
        Transform {
            pos: glam::Vec3::new(0.0, 0.0, 0.0),
            rot: Quat::IDENTITY,
            scale: glam::Vec3::ONE,
        },
        Sprite {
            color: [1.0, 0.0, 0.0, 1.0],
            tex_index: 0,
            normal_tex_index: 0,
            uv_off: [0.0, 0.0],
            uv_scale: [1.0, 1.0],
            layer: 0.0,
        },
    ));

    console::log_1(&"Created a sprite entity".into());

    console::log_1(&"Example completed!".into());
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
