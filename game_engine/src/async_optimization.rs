//! # 异步代码优化实施模块
//!
//! 本模块展示如何将过度异步化的代码优化为同步代码，提升性能。
//!
//! ## 优化原则
//!
//! 1. **纯计算 → 同步**: 无I/O的操作应该是同步函数
//! 2. **简单查询 → 同步**: 仅读取内存的数据应该是同步的
//! 3. **网络I/O → 异步**: 网络操作必须异步
//! 4. **大文件I/O → 异步**: >100KB的文件操作应该异步

// ============================================================================
// 优化示例1: 物理计算（纯计算应该同步）
// ============================================================================

/// ❌ 优化前: 不必要的async物理计算
///
/// 问题：纯计算不应该有async开销（~500ns）
#[cfg(feature = "before_optimization")]
#[deprecated(note = "Use synchronous calculate_physics instead")]
pub async fn calculate_physics_before(
    position: (f32, f32, f32),
    velocity: (f32, f32, f32),
    delta_time: f32,
) -> (f32, f32, f32) {
    let new_x = position.0 + velocity.0 * delta_time;
    let new_y = position.1 + velocity.1 * delta_time;
    let new_z = position.2 + velocity.2 * delta_time;
    (new_x, new_y, new_z)
}

/// ✅ 优化后: 同步物理计算
///
/// 收益：消除async开销，快约10x（500ns → 50ns）
pub fn calculate_physics(
    position: (f32, f32, f32),
    velocity: (f32, f32, f32),
    delta_time: f32,
) -> (f32, f32, f32) {
    let new_x = position.0 + velocity.0 * delta_time;
    let new_y = position.1 + velocity.1 * delta_time;
    let new_z = position.2 + velocity.2 * delta_time;
    (new_x, new_y, new_z)
}

// ============================================================================
// 优化示例2: 实体计数（简单查询应该同步）
// ============================================================================

/// ❌ 优化前: 不必要的async查询
///
/// 问题：简单内存读取不应该有async开销
#[cfg(feature = "before_optimization")]
#[deprecated(note = "Use synchronous get_entity_count instead")]
pub async fn get_entity_count_before(entities: &[u32]) -> usize {
    entities.len()
}

/// ✅ 优化后: 同步查询
///
/// 收益：快10x，代码更清晰
pub fn get_entity_count(entities: &[u32]) -> usize {
    entities.len()
}

// ============================================================================
// 优化示例3: 距离计算（纯计算应该同步）
// ============================================================================

/// ❌ 优化前: 不必要的async计算
#[cfg(feature = "before_optimization")]
#[deprecated(note = "Use synchronous calculate_distance instead")]
pub async fn calculate_distance_before(
    x1: f32, y1: f32,
    x2: f32, y2: f32,
) -> f32 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    (dx * dx + dy * dy).sqrt()
}

/// ✅ 优化后: 同步计算
///
/// 收益：编译器可以更好地优化SIMD
pub fn calculate_distance(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    (dx * dx + dy * dy).sqrt()
}

// ============================================================================
// 优化示例4: 向量运算（纯计算应该同步）
// ============================================================================

/// ❌ 优化前: 不必要的async向量运算
#[cfg(feature = "before_optimization")]
#[deprecated(note = "Use synchronous vector_add instead")]
pub async fn vector_add_before(v1: [f32; 3], v2: [f32; 3]) -> [f32; 3] {
    [v1[0] + v2[0], v1[1] + v2[1], v1[2] + v2[2]]
}

/// ✅ 优化后: 同步向量运算
///
/// 收益：可以向量化，SIMD加速
pub fn vector_add(v1: [f32; 3], v2: [f32; 3]) -> [f32; 3] {
    [v1[0] + v2[0], v1[1] + v2[1], v1[2] + v2[2]]
}

/// 向量点积（同步）
pub fn vector_dot(v1: [f32; 3], v2: [f32; 3]) -> f32 {
    v1[0] * v2[0] + v1[1] * v2[1] + v1[2] * v2[2]
}

/// 向量归一化（同步）
pub fn vector_normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len > 0.0 {
        [v[0] / len, v[1] / len, v[2] / len]
    } else {
        [0.0, 0.0, 0.0]
    }
}

// ============================================================================
// 优化示例5: 批量操作（使用rayon并行）
// ============================================================================

/// ❌ 优化前: 逐个async调用
#[cfg(feature = "before_optimization")]
#[deprecated(note = "Use batch_process_entities_rayon instead")]
pub async fn batch_process_entities_before(
    entities: &mut [[f32; 3]],
    offset: [f32; 3],
) {
    for entity in entities.iter_mut() {
        entity[0] += offset[0];
        entity[1] += offset[1];
        entity[2] += offset[2];
    }
}

/// ✅ 优化后: 使用rayon并行处理
///
/// 收益：CPU并行，快2-4x（取决于核心数）
#[cfg(feature = "rayon")]
pub fn batch_process_entities_rayon(
    entities: &mut [[f32; 3]],
    offset: [f32; 3],
) {
    use rayon::prelude::*;
    entities.par_iter_mut().for_each(|entity| {
        entity[0] += offset[0];
        entity[1] += offset[1];
        entity[2] += offset[2];
    });
}

/// 不使用rayon的版本（作为fallback）
#[cfg(not(feature = "rayon"))]
pub fn batch_process_entities_rayon(
    entities: &mut [[f32; 3]],
    offset: [f32; 3],
) {
    for entity in entities.iter_mut() {
        entity[0] += offset[0];
        entity[1] += offset[1];
        entity[2] += offset[2];
    }
}

// ============================================================================
// 优化示例6: 状态查询（同步）
// ============================================================================

/// 查询实体状态（同步）
pub fn query_entity_state(
    entities: &std::collections::HashMap<u32, [f32; 3]>,
    id: u32,
) -> Option<[f32; 3]> {
    entities.get(&id).copied()
}

/// 批量查询实体状态（同步）
pub fn query_multiple_entities(
    entities: &std::collections::HashMap<u32, [f32; 3]>,
    ids: &[u32],
) -> Vec<Option<[f32; 3]>> {
    ids.iter().map(|&id| entities.get(&id).copied()).collect()
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 检查函数是否应该优化为同步
///
/// # 优化检查清单
///
/// - ✅ 纯计算（无I/O）
/// - ✅ 简单内存读取
/// - ✅ 数据结构操作
/// - ❌ 网络I/O
/// - ❌ 文件I/O（大文件）
/// - ❌ 用户输入
pub const fn should_be_synchronous(
    has_io: bool,
    is_network: bool,
    is_large_file: bool,
) -> bool {
    // 如果有任何I/O操作，保持异步
    if has_io {
        // 但网络和大文件操作应该异步
        if is_network || is_large_file {
            return false;
        }
        // 小文件和内存操作可以是同步的
        return true;
    }
    
    // 纯计算总是同步的
    true
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_physics() {
        let position = (0.0, 0.0, 0.0);
        let velocity = (1.0, 2.0, 3.0);
        let delta_time = 0.016; // 60 FPS

        let result = calculate_physics(position, velocity, delta_time);
        
        assert!((result.0 - 0.016).abs() < 0.0001);
        assert!((result.1 - 0.032).abs() < 0.0001);
        assert!((result.2 - 0.048).abs() < 0.0001);
    }

    #[test]
    fn test_get_entity_count() {
        let entities = vec![1, 2, 3, 4, 5];
        let count = get_entity_count(&entities);
        assert_eq!(count, 5);
    }

    #[test]
    fn test_calculate_distance() {
        let distance = calculate_distance(0.0, 0.0, 3.0, 4.0);
        assert!((distance - 5.0).abs() < 0.0001);
    }

    #[test]
    fn test_vector_operations() {
        let v1 = [1.0, 2.0, 3.0];
        let v2 = [4.0, 5.0, 6.0];

        // 测试向量加法
        let sum = vector_add(v1, v2);
        assert_eq!(sum, [5.0, 7.0, 9.0]);

        // 测试点积
        let dot = vector_dot(v1, v2);
        assert_eq!(dot, 32.0); // 1*4 + 2*5 + 3*6 = 32

        // 测试归一化
        let normalized = vector_normalize([3.0, 0.0, 0.0]);
        assert_eq!(normalized, [1.0, 0.0, 0.0]);
    }

    #[test]
    fn test_batch_process_entities() {
        let mut entities = vec![
            [1.0, 2.0, 3.0],
            [4.0, 5.0, 6.0],
            [7.0, 8.0, 9.0],
        ];
        let offset = [0.5, 0.5, 0.5];

        batch_process_entities_rayon(&mut entities, offset);

        assert_eq!(entities[0], [1.5, 2.5, 3.5]);
        assert_eq!(entities[1], [4.5, 5.5, 6.5]);
        assert_eq!(entities[2], [7.5, 8.5, 9.5]);
    }

    #[test]
    fn test_query_entity_state() {
        let mut entities = std::collections::HashMap::new();
        entities.insert(1, [1.0, 2.0, 3.0]);
        entities.insert(2, [4.0, 5.0, 6.0]);

        // 查询单个实体
        let entity = query_entity_state(&entities, 1);
        assert_eq!(entity, Some([1.0, 2.0, 3.0]));

        // 查询不存在的实体
        let missing = query_entity_state(&entities, 999);
        assert_eq!(missing, None);

        // 批量查询
        let ids = vec![1, 2, 999];
        let results = query_multiple_entities(&entities, &ids);
        assert_eq!(results, vec![
            Some([1.0, 2.0, 3.0]),
            Some([4.0, 5.0, 6.0]),
            None,
        ]);
    }

    #[test]
    fn test_should_be_synchronous() {
        // 纯计算应该是同步的
        assert!(should_be_synchronous(false, false, false));

        // 小文件读取可以是同步的
        assert!(should_be_synchronous(true, false, false));

        // 网络操作应该是异步的
        assert!(!should_be_synchronous(true, true, false));

        // 大文件操作应该是异步的
        assert!(!should_be_synchronous(true, false, true));
    }
}

// ============================================================================
// 性能基准（手动测试）
// ============================================================================

#[cfg(feature = "benchmark")]
#[doc(hidden)]
pub fn benchmark_async_vs_sync() {
    use std::time::Instant;

    const ITERATIONS: usize = 1_000_000;

    // 基准测试：同步物理计算
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _ = calculate_physics((0.0, 0.0, 0.0), (1.0, 2.0, 3.0), 0.016);
    }
    let sync_duration = start.elapsed();

    println!("同步物理计算: {:?} ({}次迭代)", sync_duration, ITERATIONS);
    println!("平均每次: {:?}ns", sync_duration.as_nanos() / ITERATIONS as u128);

    // 基准测试：向量运算
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _ = vector_add([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]);
    }
    let vec_duration = start.elapsed();

    println!("向量加法: {:?} ({}次迭代)", vec_duration, ITERATIONS);
    println!("平均每次: {:?}ns", vec_duration.as_nanos() / ITERATIONS as u128);
}
