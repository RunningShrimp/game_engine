# 软体物理指南

本指南详细介绍如何使用软体物理系统进行布料和流体模拟。

## 概述

软体物理系统提供了两种主要的模拟类型：

- **布料模拟**: 基于弹簧-质点系统的布料物理
- **流体模拟**: 基于SPH（Smoothed Particle Hydrodynamics）的流体物理

## 布料模拟

### 创建布料

```rust
use game_engine::physics::soft_body::{ClothSoftBody, SoftBodyComponent, SoftBodyType};
use bevy_ecs::prelude::*;

fn spawn_cloth(mut commands: Commands) {
    // 创建10x10的布料网格
    let cloth = ClothSoftBody::new_rectangular(
        10,    // 宽度（粒子数）
        10,    // 高度（粒子数）
        0.1,   // 粒子间距
        0.1,   // 粒子质量
    );

    // 创建实体并添加软体组件
    commands.spawn(SoftBodyComponent::new_cloth(cloth));
}
```

### 布料参数

```rust
let mut cloth = ClothSoftBody::new_rectangular(10, 10, 0.1, 0.1);

// 调整重力
cloth.gravity = Vec3A::new(0.0, -9.81, 0.0);

// 调整空气阻力
cloth.air_damping = 0.99; // 0.0-1.0，值越大阻力越大

// 固定特定粒子
cloth.particles[0].fixed = true;
```

### 布料弹簧类型

布料使用三种类型的弹簧：

1. **结构弹簧**: 连接相邻粒子，保持基本形状
2. **剪切弹簧**: 连接对角线粒子，防止剪切变形
3. **弯曲弹簧**: 连接间隔粒子，保持平滑度

```rust
// 调整弹簧参数
for spring in &mut cloth.structural_springs {
    spring.stiffness = 1000.0;  // 弹簧常数
    spring.damping = 0.1;       // 阻尼系数
}
```

## 流体模拟

### 创建流体

```rust
use game_engine::physics::soft_body::{
    FluidSoftBody, SphParameters, SoftBodyComponent,
};
use bevy_ecs::prelude::*;

fn spawn_fluid(mut commands: Commands) {
    // 配置SPH参数
    let params = SphParameters {
        particle_radius: 0.1,
        smoothing_radius: 0.2,
        rest_density: 1000.0,      // 水的密度
        gas_constant: 2000.0,      // 压力常数
        viscosity: 0.018,          // 水的粘性
        surface_tension: 0.0728,   // 表面张力
        gravity: Vec3A::new(0.0, -9.81, 0.0),
    };

    // 创建包含1000个粒子的流体
    let fluid = FluidSoftBody::new(1000, params);

    // 创建实体并添加软体组件
    commands.spawn(SoftBodyComponent::new_fluid(fluid));
}
```

### SPH参数说明

```rust
let params = SphParameters {
    // 粒子半径：单个粒子的半径
    particle_radius: 0.1,
    
    // 平滑半径：用于计算密度的平滑范围
    smoothing_radius: 0.2,
    
    // 静止密度：流体的目标密度
    rest_density: 1000.0,
    
    // 气体常数：用于压力计算
    gas_constant: 2000.0,
    
    // 粘性系数：流体的粘性
    viscosity: 0.018,
    
    // 表面张力系数：流体表面的张力
    surface_tension: 0.0728,
    
    // 重力：作用在流体上的重力
    gravity: Vec3A::new(0.0, -9.81, 0.0),
};
```

### 流体物理更新

流体系统会自动：
1. 更新空间分区（加速邻居查找）
2. 计算每个粒子的密度和压力
3. 计算压力力、粘性力和重力
4. 更新粒子位置和速度

## 系统集成

### 添加软体物理系统

```rust
use game_engine::physics::soft_body::{
    SoftBodyPhysicsWorld, soft_body_physics_system,
};
use bevy_ecs::prelude::*;

fn setup_soft_body_physics(mut commands: Commands) {
    // 添加软体物理世界资源
    commands.insert_resource(SoftBodyPhysicsWorld::new());
}

// 在App中添加系统
app.add_systems(Update, soft_body_physics_system);
```

### 配置时间步长

```rust
// 使用默认时间步长（60 FPS）
let world = SoftBodyPhysicsWorld::new();

// 自定义时间步长和子步数
let world = SoftBodyPhysicsWorld::with_substeps(
    1.0 / 60.0,  // 时间步长
    2,           // 子步数（用于稳定性）
);
```

### 启用/禁用软体

```rust
fn toggle_soft_body(mut query: Query<&mut SoftBodyComponent>) {
    for mut soft_body in query.iter_mut() {
        soft_body.enabled = !soft_body.enabled;
    }
}
```

## 渲染软体

### 渲染布料

```rust
use bevy_ecs::prelude::*;
use game_engine::physics::soft_body::SoftBodyComponent;

fn render_cloth(
    query: Query<&SoftBodyComponent>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for soft_body in query.iter() {
        if let Some(cloth) = &soft_body.cloth {
            // 从粒子位置生成网格
            let vertices: Vec<Vec3> = cloth
                .particles
                .iter()
                .map(|p| p.position.into())
                .collect();

            // 生成三角形索引
            let mut indices = Vec::new();
            for y in 0..cloth.height - 1 {
                for x in 0..cloth.width - 1 {
                    let idx = y * cloth.width + x;
                    // 第一个三角形
                    indices.extend_from_slice(&[
                        idx,
                        idx + 1,
                        idx + cloth.width,
                    ]);
                    // 第二个三角形
                    indices.extend_from_slice(&[
                        idx + 1,
                        idx + cloth.width + 1,
                        idx + cloth.width,
                    ]);
                }
            }

            // 创建网格并渲染
            // ...
        }
    }
}
```

### 渲染流体

```rust
use bevy_ecs::prelude::*;
use game_engine::physics::soft_body::SoftBodyComponent;

fn render_fluid(
    query: Query<&SoftBodyComponent>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for soft_body in query.iter() {
        if let Some(fluid) = &soft_body.fluid {
            // 为每个粒子创建球体网格
            for particle in &fluid.particles {
                // 创建球体网格，半径 = particle_radius
                // ...
            }
        }
    }
}
```

## 与Rapier物理集成

### 碰撞检测

可以将软体粒子与Rapier刚体进行碰撞检测：

```rust
use rapier3d::prelude::*;
use game_engine::physics::soft_body::SoftBodyComponent;

fn check_collisions(
    mut soft_bodies: Query<&mut SoftBodyComponent>,
    physics_world: Res<PhysicsWorld3D>,
) {
    for mut soft_body in soft_bodies.iter_mut() {
        if let Some(cloth) = &mut soft_body.cloth {
            for particle in &mut cloth.particles {
                // 检查与Rapier碰撞体的碰撞
                // 使用physics_world.collider_set进行查询
                // ...
            }
        }
    }
}
```

## 性能优化

### 1. 减少粒子数量

- 布料：使用较少的粒子（如8x8而不是20x20）
- 流体：根据性能需求调整粒子数量

### 2. 使用空间分区

流体系统自动使用空间分区加速邻居查找。可以调整单元格大小：

```rust
fluid.cell_size = fluid.parameters.smoothing_radius * 2.0;
```

### 3. 调整子步数

对于快速移动的软体，增加子步数可以提高稳定性：

```rust
let world = SoftBodyPhysicsWorld::with_substeps(1.0 / 60.0, 4);
```

### 4. 限制更新频率

对于不需要每帧更新的软体，可以降低更新频率：

```rust
fn update_soft_body_occasionally(
    mut query: Query<&mut SoftBodyComponent>,
    time: Res<Time>,
) {
    if time.elapsed_seconds() % 0.1 < 0.016 {
        // 每0.1秒更新一次
        // ...
    }
}
```

## 最佳实践

### 1. 布料模拟

- **固定点**: 固定布料的某些点以创建悬挂效果
- **弹簧参数**: 根据布料类型调整弹簧常数
- **阻尼**: 使用适当的阻尼防止振荡

### 2. 流体模拟

- **粒子数量**: 根据性能需求平衡粒子数量
- **SPH参数**: 根据流体类型调整参数（水、油等）
- **边界处理**: 实现边界条件防止粒子逃逸

### 3. 性能考虑

- **LOD系统**: 根据距离调整粒子数量
- **异步更新**: 对于非关键软体使用异步更新
- **GPU加速**: 考虑使用GPU进行大规模模拟

## 常见问题

### Q: 布料太软或太硬？

**A**: 调整弹簧常数：
- 增加`stiffness`使布料更硬
- 减少`stiffness`使布料更软

### Q: 流体不稳定？

**A**: 
1. 增加子步数
2. 减小时间步长
3. 调整SPH参数

### Q: 性能问题？

**A**: 
1. 减少粒子数量
2. 降低更新频率
3. 使用空间分区优化
4. 考虑GPU加速

## 相关文档

- [物理系统](../physics/mod.rs)
- [Rapier物理引擎](https://rapier.rs/)
- [SPH方法](https://en.wikipedia.org/wiki/Smoothed-particle_hydrodynamics)

