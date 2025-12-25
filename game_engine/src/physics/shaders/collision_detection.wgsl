// GPU碰撞检测计算着色器

struct RigidBody {
    position: vec3<f32>,
    rotation: vec4<f32>,
    velocity: vec3<f32>,
    inv_mass: f32,
    aabb_min: vec3<f32>,
    aabb_max: vec3<f32>,
    _padding: f32,
}

struct CollisionInfo {
    body_a: u32,
    body_b: u32,
    normal: vec3<f32>,
    depth: f32,
    contact_point: vec3<f32>,
    _padding: vec3<f32>,
}

struct CollisionParams {
    body_count: u32,
    collision_margin: f32,
    _padding: vec2<f32>,
}

@group(0) @binding(0)
var<storage, read> rigid_bodies: array<RigidBody>;

@group(0) @binding(1)
var<storage, read_write> collisions: array<CollisionInfo>;

@group(0) @binding(2)
var<uniform> params: CollisionParams;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if (idx >= params.body_count) {
        return;
    }

    let body_a = rigid_bodies[idx];
    
    // 检查与其他刚体的碰撞
    for (var i = idx + 1u; i < params.body_count; i++) {
        let body_b = rigid_bodies[i];
        
        // AABB快速剔除
        if (body_a.aabb_max.x < body_b.aabb_min.x || body_a.aabb_min.x > body_b.aabb_max.x ||
            body_a.aabb_max.y < body_b.aabb_min.y || body_a.aabb_min.y > body_b.aabb_max.y ||
            body_a.aabb_max.z < body_b.aabb_min.z || body_a.aabb_min.z > body_b.aabb_max.z) {
            continue;
        }
        
        // 简化的球-球碰撞检测（使用AABB中心作为球心）
        let center_a = (body_a.aabb_min + body_a.aabb_max) * 0.5;
        let center_b = (body_b.aabb_min + body_b.aabb_max) * 0.5;
        let extents_a = body_a.aabb_max - body_a.aabb_min;
        let extents_b = body_b.aabb_max - body_b.aabb_min;
        let radius_a = length(extents_a) * 0.5;
        let radius_b = length(extents_b) * 0.5;
        
        let delta = center_b - center_a;
        let dist = length(delta);
        let min_dist = radius_a + radius_b + params.collision_margin;
        
        if (dist < min_dist && dist > 0.0) {
            let normal = normalize(delta);
            let depth = min_dist - dist;
            let contact_point = center_a + normal * radius_a;
            
            let collision_idx = idx * params.body_count + i;
            if (collision_idx < arrayLength(&collisions)) {
                collisions[collision_idx] = CollisionInfo(
                    idx,
                    i,
                    normal,
                    depth,
                    contact_point,
                    vec3<f32>(0.0)
                );
            }
        }
    }
}

