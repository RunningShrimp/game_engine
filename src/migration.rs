//! # API迁移指南
//!
//! 本模块提供从旧API到新API的迁移帮助。
//!
//! ## 迁移概览
//!
//! 从游戏引擎 v0.1.x 版本升级到 v0.2.x 时，API 发生了重大重构：
//! - **旧API**: 使用 `PhysicsWorld`/`AudioSystem` 这些包含业务逻辑的包装类型
//! - **新API**: 使用 `PhysicsState`/`AudioState` (纯数据) + `PhysicsService`/`AudioService` (业务逻辑) 的贫血模型
//!
//! ## 迁移步骤
//!
//! 1. 将 `PhysicsWorld` 替换为 `PhysicsState::default()`
//! 2. 将方法调用改为 `PhysicsService::` 静态方法
//! 3. 将 `AudioSystem` 替换为 `AudioState::new()`
//! 4. 将方法调用改为 `AudioService::` 静态方法
//!
//! ## 示例
//!
//! ```rust
//! // 旧代码 - 不推荐
//! let mut world = PhysicsWorld::default();
//! world.step();
//!
//! // 新代码 - 推荐
//! let mut state = PhysicsState::default();
//! PhysicsService::step(&mut state);
//! ```
//!
//! ## Feature Flags
//!
//! 废弃的 API 通过 `deprecated-apis` feature 控制。
//! 默认情况下，这些 API 被隐藏，仅在需要兼容性时启用：
//!
//! ```toml
//! [dependencies.game_engine]
//! version = "0.2.0"
//! features = ["deprecated-apis"]  # 仅在必要时启用
//! ```

pub mod physics_world_migration {
    //! 物理系统迁移助手
    //!
    //! 从 `PhysicsWorld` 迁移到 `PhysicsState` + `PhysicsService`

    use crate::physics::{PhysicsState, PhysicsService};

    /// 从 PhysicsWorld 创建新的 API 对象
    ///
    /// 这个函数创建一个全新的状态对象。
    /// 如果你有现有的 PhysicsWorld 数据需要迁移，
    /// 请手动从 PhysicsWorld 提取数据并重新在新的 API 中创建。
    pub fn migrate_to_new_api() -> (PhysicsState, PhysicsService) {
        (PhysicsState::default(), PhysicsService)
    }

    /// 旧API方法到新API的映射
    ///
    /// PhysicsWorld.step() -> PhysicsService::step(&mut physics_state)
    ///
    /// PhysicsWorld.set_gravity(g) -> PhysicsService::set_gravity(&mut physics_state, [g.x, g.y])
    ///
    /// PhysicsWorld.create_rigid_body(...) -> PhysicsService::create_rigid_body(&mut physics_state, ...)
    pub fn _note_api_mapping() {
        // 这个函数只是为了文档目的
    }
}

pub mod audio_system_migration {
    //! 音频系统迁移助手
    //!
    //! 从 `AudioSystem` 迁移到 `AudioState` + `AudioService`

    use crate::audio::{AudioState, AudioService};

    /// 从 AudioSystem 创建新的 API 对象
    ///
    /// 这个函数创建一个全新的状态对象。
    /// 如果你有现有的 AudioSystem 数据需要保留，
    /// 请考虑直接使用新的 API 重新初始化音频系统。
    pub fn migrate_to_new_api() -> (AudioState, AudioService) {
        (AudioState::new(), AudioService)
    }

    /// 旧API方法到新API的映射
    ///
    /// AudioSystem.is_available() -> AudioService::is_available(&audio_state)
    ///
    /// AudioSystem.play_file(entity, path, volume, looped) -> AudioService::play_file(&audio_state, entity, path, volume, looped)
    ///
    /// AudioSystem.stop(entity) -> AudioService::stop(&audio_state, entity)
    pub fn _note_api_mapping() {
        // 这个函数只是为了文档目的
    }
}

/// 废弃API汇总 - 将在 v0.3.0 中完全移除
#[cfg(feature = "deprecated-apis")]
pub mod deprecated_apis {
    //! 废弃的API汇总
    //!
    //! 这些API将被完全移除。请使用上面指定的迁移助手完成迁移。
    //!
    //! 要继续使用这些API，请在 Cargo.toml 中启用 `deprecated-apis` feature：
    //!
    //! ```toml
    //! [dependencies.game_engine]
    //! version = "0.2.0"
    //! features = ["deprecated-apis"]
    //! ```
    //!
    //! 注意：这个feature仅用于临时兼容。随着时间推移，我们会完全移除这些API。

    pub use crate::audio::AudioSystem;
    pub use crate::physics::PhysicsWorld;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physics_migration() {
        let (state, _service) = physics_world_migration::migrate_to_new_api();
        // 验证新API对象的创建
        let _gravity = crate::physics::PhysicsService::get_gravity(&state);
        // 验证重力默认值
    }

    #[test]
    fn test_audio_migration() {
        let (state, _service) = audio_system_migration::migrate_to_new_api();
        // 验证新API对象的创建
        // AudioState 创建后默认值为1.0
        assert_eq!(state.master_volume, 1.0);
    }

    #[cfg(feature = "deprecated-apis")]
    #[test]
    fn test_deprecated_apis_available() {
        // 测试废弃的API在启用feature时可用
        let _system = crate::audio::AudioSystem::new();
        let _world = crate::physics::PhysicsWorld::default();
    }
}