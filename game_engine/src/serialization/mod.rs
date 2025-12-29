// 序列化系统
//
// 提供完整的游戏状态序列化、场景保存/加载、资源元数据序列化和版本兼容性管理。
//
// ## 功能特性
//
// - 游戏状态序列化 (GameState)
// - 场景保存/加载 (Scene serialization)
// - 资源元数据序列化 (ResourceMetadata)
// - 版本管理和迁移 (Versioning)
// - 多种序列化格式支持 (RON, Bincode, JSON)
//
// ## 使用示例
//
// ```rust
// use game_engine::serialization::{GameState, SerializationFormat};
//
// // 保存游戏状态
// let state = GameState::from_world(&mut world, "save_slot_1");
// state.save_to_file("save_1.ron", SerializationFormat::Ron)?;
//
// // 加载游戏状态
// let loaded = GameState::load_from_file("save_1.ron", SerializationFormat::Ron)?;
// loaded.apply_to_world(&mut world)?;
// ```

pub mod game_state;
pub mod resource_metadata;
pub mod versioning;

pub use game_state::{GameTime, GameState, GameStateMetadata, PlayerProgress, SerializationFormat};
pub use resource_metadata::{
    CachePolicy, ResourceIndex, ResourceLoadState, ResourceMetadata, ResourcePackMetadata,
    ResourceType,
};
pub use versioning::{
    CompatibilityChecker, CompatibilityInfo, MigrationRule, SemanticVersion, VersionManager,
    VersionedData,
};
