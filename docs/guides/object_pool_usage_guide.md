# 对象池使用指南

## 概述

对象池系统用于减少高频内存分配的开销，特别是在渲染和物理计算等热点路径中。本指南说明如何在游戏引擎中使用对象池。

## 可用的对象池

### 基础类型池

- `Vec<u8>` - 用于临时缓冲区
- `Vec<f32>` - 用于浮点数组
- `Vec<Vec3>` - 用于3D向量数组
- `Vec<Mat4>` - 用于矩阵数组
- `Vec<u32>` - 用于索引数组
- `String` - 用于临时字符串
- `HashMap<String, String>` - 用于临时映射

### 扩展类型池

- `Vec<Vec3>` (扩展) - 用于位置数组等额外用途

## 使用方法

### 方法1：使用全局便捷函数

```rust
use crate::performance::memory::pool_manager::{acquire_vec_u8, release_vec_u8};

// 获取对象
let mut buffer = acquire_vec_u8();
buffer.push(42);

// 使用完毕后归还
release_vec_u8(buffer);
```

### 方法2：使用全局池管理器

```rust
use crate::performance::memory::pool_manager::global_pool_manager;

let pool = global_pool_manager().vec_f32_pool();
let mut data = pool.acquire();
data.push(1.0);
pool.release(data);
```

### 方法3：使用RAII包装器（推荐）

```rust
use crate::performance::memory::pool_manager::global_pool_manager;
use game_engine_performance::memory::object_pool::Pooled;

let pool = global_pool_manager().vec_u8_pool();
let pooled_vec = Pooled::new(pool);
// 使用 pooled_vec.get() 或 pooled_vec.get_mut()
// 当 pooled_vec 离开作用域时自动归还
```

## 最佳实践

### 1. 在热点路径中使用

对象池最适合在以下场景使用：
- 每帧调用的渲染函数
- 物理计算循环
- 批量数据处理
- 临时缓冲区分配

### 2. 避免在以下场景使用

- 长期持有的对象（应使用常规分配）
- 大小变化很大的对象（可能导致内存浪费）
- 单次使用的对象（对象池开销可能大于收益）

### 3. 性能考虑

- 对象池在缓存命中时性能最佳
- 如果缓存命中率低于50%，考虑调整池大小或使用常规分配
- 使用 `PoolManagerStats` 监控缓存命中率

## 示例：在渲染路径中使用

```rust
use crate::performance::memory::pool_manager::{acquire_vec_vec3, release_vec_vec3};

fn process_vertices(vertices: &[Vertex]) -> Vec<Vec3> {
    // 从池中获取临时向量数组
    let mut positions = acquire_vec_vec3();
    
    // 处理数据
    for vertex in vertices {
        positions.push(vertex.position);
    }
    
    // 使用数据...
    let result = positions.clone();
    
    // 归还到池中
    release_vec_vec3(positions);
    
    result
}
```

## 监控和调优

### 查看统计信息

```rust
use crate::performance::memory::pool_manager::global_pool_manager;

let stats = global_pool_manager().stats();
stats.print_stats(); // 打印所有池的统计信息

// 检查总体缓存命中率
let hit_rate = stats.overall_hit_rate();
if hit_rate < 0.5 {
    tracing::warn!("Object pool hit rate is low: {:.2}%", hit_rate * 100.0);
}
```

### 调整池大小

```rust
use crate::performance::memory::pool_manager::{PoolConfig, PoolManager};

let config = PoolConfig {
    vec_u8_initial: 64,  // 增加初始大小
    vec_u8_max: 512,      // 增加最大大小
    ..Default::default()
};

let manager = PoolManager::with_config(config);
```

## 注意事项

1. **线程安全**：所有对象池都是线程安全的，可以在多线程环境中使用
2. **内存管理**：对象池会自动管理内存，但要注意不要持有过多对象
3. **重置行为**：某些类型（如 `Vec`）在归还时会自动清空，无需手动重置
4. **性能权衡**：对象池减少了分配开销，但增加了代码复杂度，需要权衡

## 扩展对象池

如果需要为自定义类型创建对象池：

```rust
use game_engine_performance::memory::object_pool::{Resettable, SyncObjectPool};

// 1. 实现 Resettable trait
impl Resettable for MyType {
    fn reset(&mut self) {
        // 重置对象到初始状态
        *self = MyType::default();
    }
}

// 2. 创建对象池
let pool = Arc::new(SyncObjectPool::new(
    || MyType::default(),
    16,  // 初始大小
    128, // 最大大小
));

// 3. 使用
let obj = pool.acquire();
// ... 使用对象
pool.release(obj);
```

## 相关文档

- `game_engine_performance::memory::object_pool` - 对象池实现
- `game_engine::performance::memory::pool_manager` - 池管理器API

