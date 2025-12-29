pub mod arena;
pub mod arena_allocator;
pub mod entity_pool;
pub mod memory_optimization;
pub mod pool_manager;
pub mod advanced_pool;
pub use pool_manager::{PoolConfig, PoolManager, PoolManagerStats, global_pool_manager};
pub use advanced_pool::{
    AdvancedMemoryPool, AdvancedMemoryPoolResource, GlobalPoolStats, MemoryPoolConfig, PoolType,
    memory_pool_auto_tune_system, memory_pool_report_system,
};

pub use arena::{Arena, ArenaError, MemoryPool, TypedArena, TypedArenaWithDrop};
pub use arena_allocator::{ArenaAllocator, ArenaManager, ArenaManagerStats, ArenaError as AllocArenaError};
pub use entity_pool::{ComponentPool, ComponentPoolConfig, ComponentPoolStats, EcsObjectPoolManager,
                      EntityPool, EntityPoolConfig, EntityPoolStats, entity_pool_system};
pub use memory_optimization::*;
// 重新导出game_engine_performance中的object_pool类型
pub use game_engine_performance::memory::object_pool::{
    ObjectPool, PoolStats, Pooled, Resettable, ResettablePool, SizedPool, SyncObjectPool,
};
