pub mod arena;
pub mod memory_optimization;
pub mod pool_manager;
pub use pool_manager::{PoolConfig, PoolManager, PoolManagerStats, global_pool_manager};

pub use arena::{Arena, ArenaError, MemoryPool, TypedArena, TypedArenaWithDrop};
pub use memory_optimization::*;
// 重新导出game_engine_performance中的object_pool类型
pub use game_engine_performance::memory::object_pool::{
    ObjectPool, PoolStats, Pooled, Resettable, ResettablePool, SizedPool, SyncObjectPool,
};
