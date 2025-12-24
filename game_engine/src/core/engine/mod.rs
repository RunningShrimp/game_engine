//  引擎主入口
//
//  定义Engine结构和主运行循环

pub mod asset_processor;
pub mod demo_scene;
pub mod engine;
#[cfg(test)]
mod engine_tests;
pub mod game_loop;
pub mod game_loop_coroutine;
pub mod game_loop_fixed;
pub mod initialization;
pub mod input_handler;
pub mod renderer;

pub use crate::config::EngineConfig;
pub use crate::core::engine::engine::Engine;
pub use asset_processor::*;
pub use game_loop::*;
pub use renderer::*;
