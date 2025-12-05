# 物理系统 API 参考

## Physics3D

3D 物理世界，基于 Rapier 物理引擎。

### 示例

```rust
use game_engine::physics::Physics3D;

let mut physics = Physics3D::new();
physics.add_rigid_body(rigid_body_desc, collider_desc)?;
physics.step(delta_time);
```

## RigidBodyDesc

刚体描述符，定义刚体的物理属性。

### 示例

```rust
use game_engine::physics::{RigidBodyDesc, RigidBodyType};

let desc = RigidBodyDesc {
    body_type: RigidBodyType::Dynamic,
    mass: 1.0,
    ..Default::default()
};
```

## ColliderDesc

碰撞体描述符，定义碰撞形状。

### 支持的形状

- Box（盒子）
- Sphere（球体）
- Capsule（胶囊）
- Cylinder（圆柱）
- ConvexMesh（凸网格）
- Trimesh（三角网格）

