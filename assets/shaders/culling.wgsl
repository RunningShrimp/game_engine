//! GPU 视锥剔除计算着色器
//!
//! 实现高性能的GPU端视锥剔除，支持批量AABB和球体测试。
//!
//! ## 性能特性
//!
//! - 并行处理：每个实例在独立的GPU线程中处理
//! - 原子操作：使用原子计数器收集可见实例
//! - 内存优化：紧凑的数据布局，减少内存带宽
//! - 早期退出：使用分支优化，快速剔除不可见对象

// 剔除 Uniform 数据
struct CullingUniforms {
    view_proj: mat4x4<f32>,
    frustum_planes: array<vec4<f32>, 6>,
    instance_count: u32,
    index_count: u32,  // 每个实例的索引数（用于间接绘制命令生成，0表示不生成）
    _pad: vec2<u32>,
}

// GPU 实例数据
struct GpuInstance {
    model: mat4x4<f32>,
    aabb_min: vec3<f32>,
    instance_id: u32,
    aabb_max: vec3<f32>,
    flags: u32,
}

// 绑定组
@group(0) @binding(0) var<uniform> uniforms: CullingUniforms;
@group(0) @binding(1) var<storage, read> input_instances: array<GpuInstance>;
@group(0) @binding(2) var<storage, read_write> output_instances: array<GpuInstance>;
@group(0) @binding(3) var<storage, read_write> counter: atomic<u32>;
@group(0) @binding(4) var<storage, read_write> indirect_commands: array<DrawIndexedIndirectArgs>;  // 可选的间接绘制命令缓冲区

// 间接绘制参数结构
struct DrawIndexedIndirectArgs {
    index_count: u32,
    instance_count: u32,
    first_index: u32,
    base_vertex: i32,
    first_instance: u32,
};

/// 检测 AABB 是否与视锥平面相交
/// 
/// 使用正顶点测试（P-vertex test）优化：
/// - 计算AABB在平面法向量方向上的正顶点
/// - 如果正顶点在平面外侧，则AABB完全在视锥外
fn aabb_vs_plane(aabb_min: vec3<f32>, aabb_max: vec3<f32>, plane: vec4<f32>) -> bool {
    // 计算 AABB 在平面法向量方向上的正顶点
    var p_vertex: vec3<f32>;
    p_vertex.x = select(aabb_min.x, aabb_max.x, plane.x >= 0.0);
    p_vertex.y = select(aabb_min.y, aabb_max.y, plane.y >= 0.0);
    p_vertex.z = select(aabb_min.z, aabb_max.z, plane.z >= 0.0);
    
    // 检测正顶点是否在平面外侧
    // plane.xyz 是法向量，plane.w 是距离
    return dot(plane.xyz, p_vertex) + plane.w >= 0.0;
}

/// 检测 AABB 是否在视锥内
/// 
/// 对6个视锥平面进行测试，如果AABB与所有平面相交，则可见
fn is_aabb_visible(aabb_min: vec3<f32>, aabb_max: vec3<f32>) -> bool {
    // 早期退出优化：一旦发现不可见，立即返回
    for (var i = 0u; i < 6u; i++) {
        if (!aabb_vs_plane(aabb_min, aabb_max, uniforms.frustum_planes[i])) {
            return false;
        }
    }
    return true;
}

/// 检测球体是否在视锥内
/// 
/// 使用球心到平面的距离测试，比AABB测试更快
fn is_sphere_visible(center: vec3<f32>, radius: f32) -> bool {
    for (var i = 0u; i < 6u; i++) {
        let plane = uniforms.frustum_planes[i];
        let distance = dot(plane.xyz, center) + plane.w;
        // 如果球心到平面的距离小于半径，则球体在平面外侧
        if (distance < -radius) {
            return false;
        }
    }
    return true;
}

/// 主剔除函数
/// 
/// 对每个实例执行视锥剔除，将可见实例写入输出缓冲区
@compute @workgroup_size(64)
fn cull_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    
    // 边界检查
    if (idx >= uniforms.instance_count) {
        return;
    }
    
    let instance = input_instances[idx];
    
    // 将 AABB 变换到世界空间
    // 注意：这里假设model矩阵已经包含了世界变换
    let world_min = (instance.model * vec4<f32>(instance.aabb_min, 1.0)).xyz;
    let world_max = (instance.model * vec4<f32>(instance.aabb_max, 1.0)).xyz;
    
    // 确保 min < max（处理负缩放的情况）
    let actual_min = min(world_min, world_max);
    let actual_max = max(world_min, world_max);
    
    // 视锥剔除
    var visible = false;
    
    // 根据flags选择测试方法
    // flags & 0x1: 使用球体测试（更快但可能不够精确）
    // flags & 0x2: 使用AABB测试（更精确但稍慢）
    if ((instance.flags & 0x1u) != 0u) {
        // 球体测试：使用AABB的中心和半径
        let center = (actual_min + actual_max) * 0.5;
        let radius = length(actual_max - actual_min) * 0.5;
        visible = is_sphere_visible(center, radius);
    } else {
        // AABB测试：更精确但稍慢
        visible = is_aabb_visible(actual_min, actual_max);
    }
    
    if (visible) {
        // 原子增加计数器并获取输出索引
        let output_idx = atomicAdd(&counter, 1u);
        output_instances[output_idx] = instance;
        
        // 如果提供了间接绘制命令缓冲区且index_count > 0，同时生成间接绘制命令
        // 这样可以完全避免CPU读取结果，实现完全GPU端剔除流程
        if (uniforms.index_count > 0u) {
            indirect_commands[output_idx] = DrawIndexedIndirectArgs {
                index_count: uniforms.index_count,
                instance_count: 1u,              // 每个命令的实例数（单个实例）
                first_index: 0u,                 // 第一个索引
                base_vertex: 0i,                 // 基础顶点
                first_instance: output_idx,      // 第一个实例（使用输出索引）
            };
        }
    }
}


