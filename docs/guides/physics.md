# 物理系统

引擎集成Rapier物理引擎，提供2D和3D物理模拟，具有高性能和精确的碰撞检测。

## 物理世界架构

### 核心组件

```rust
// 2D物理世界
PhysicsWorld {
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    gravity: Vector2<f32>,
    integration_parameters: IntegrationParameters,
}

// 3D物理世界
PhysicsWorld3D {
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    gravity: Vector3<f32>,
    integration_parameters: IntegrationParameters,
}
```

### 物理实体类型

```rust
#[derive(Component)]
struct RigidBodyDesc {
    body_type: RigidBodyType,  // Dynamic, Fixed, Kinematic
    position: [f32; 2],        // 2D: [x, y], 3D: [x, y, z]
}

#[derive(Component)]
struct ColliderDesc {
    shape_type: ShapeType,     // Cuboid, Ball, Capsule, etc.
    half_extents: [f32; 2],    // 尺寸参数
    radius: f32,
}
```

## 基本使用

### 创建物理实体

```rust
fn setup_physics(
    mut commands: Commands,
) {
    // 创建动态刚体（受物理影响）
    commands.spawn((
        Transform::from_xyz(0.0, 10.0, 0.0),
        RigidBodyDesc {
            body_type: RigidBodyType::Dynamic,
            position: [0.0, 10.0],
        },
        ColliderDesc {
            shape_type: ShapeType::Cuboid,
            half_extents: [25.0, 25.0],
            radius: 0.0,
        },
    ));

    // 创建固定刚体（静态几何体）
    commands.spawn((
        Transform::from_xyz(0.0, -50.0, 0.0),
        RigidBodyDesc {
            body_type: RigidBodyType::Fixed,
            position: [0.0, -50.0],
        },
        ColliderDesc {
            shape_type: ShapeType::Cuboid,
            half_extents: [400.0, 25.0],
            radius: 0.0,
        },
    ));
}
```

### 自动系统

引擎包含内置的物理同步系统：

```rust
// 初始化物理刚体
init_physics_bodies_system(&mut physics_world, &transforms, &rigid_bodies, &colliders);

// 执行物理步进
physics_step_system(&mut physics_world, time.delta_seconds as f64);

// 同步物理结果到变换
sync_physics_to_transform_system(&mut physics_world, &mut transforms);
```

## 碰撞形状

### 支持的形状类型

```rust
enum ShapeType {
    /// 立方体/矩形
    Cuboid,
    /// 球体
    Ball,
    /// 胶囊体
    Capsule,
    /// 多边形
    ConvexPolygon,
    /// 三角形网格
    TriangleMesh,
    /// 高度场
    HeightField,
}
```

### 形状配置

```rust
// 立方体碰撞器
ColliderDesc {
    shape_type: ShapeType::Cuboid,
    half_extents: [width/2.0, height/2.0],
    radius: 0.0, // 不使用
}

// 圆形碰撞器
ColliderDesc {
    shape_type: ShapeType::Ball,
    half_extents: [0.0, 0.0], // 不使用
    radius: radius,
}

// 胶囊碰撞器（圆柱体两端有半球）
ColliderDesc {
    shape_type: ShapeType::Capsule,
    half_extents: [half_height, 0.0], // [半高, 未使用]
    radius: radius,
}
```

## 刚体类型

### 动态刚体 (Dynamic)

完全受物理力影响的物体：

```rust
RigidBodyDesc {
    body_type: RigidBodyType::Dynamic,
    position: [0.0, 10.0],
}
```

- 受重力影响
- 响应碰撞
- 可应用力、冲量和扭矩

### 固定刚体 (Fixed/Static)

不可移动的静态几何体：

```rust
RigidBodyDesc {
    body_type: RigidBodyType::Fixed,
    position: [0.0, -50.0],
}
```

- 不受物理力影响
- 其他物体可以与其碰撞
- 常用于地面、墙壁等

### 运动学刚体 (Kinematic)

以编程方式控制的动态物体：

```rust
RigidBodyDesc {
    body_type: RigidBodyType::Kinematic,
    position: [0.0, 0.0],
}
```

- 可编程控制位置/速度
- 影响其他动态物体
- 不受物理力直接影响

## 力、冲量和约束

### 施加力

```rust
// 在刚体中心施加力
rigid_body.add_force(force, true);

// 在特定点施加力（产生扭矩）
rigid_body.add_force_at_point(force, point, true);
```

### 施加冲量

```rust
// 瞬时速度变化
rigid_body.add_linear_impulse(impulse, true);

// 角冲量（改变角速度）
rigid_body.add_angular_impulse(impulse, true);
```

## 关节和约束

### 关节类型

引擎支持多种关节类型：

```rust
// 固定关节（焊接）
let joint = FixedJoint::new(rigid_body1, rigid_body2);

// 旋转关节（铰链）
let joint = RevoluteJoint::new(rigid_body1, rigid_body2, anchor);

// 棱柱关节（滑动）
let joint = PrismaticJoint::new(rigid_body1, rigid_body2, axis);
```

### 关节配置

```rust
// 创建旋转关节
let joint = RevoluteJointBuilder::new()
    .local_anchor1(point1)
    .local_anchor2(point2)
    .motor_position(target_angle, stiffness, damping)
    .build();
```

## 射线投射和碰撞查询

### 射线投射

```rust
// 执行射线投射
if let Some((handle, toi, normal)) = physics_world.cast_ray(
    &ray, // Ray { origin, dir }
    max_time_of_impact,
    true,  // 是否与传感器碰撞
) {
    // 处理碰撞结果
    let hit_point = ray.origin + ray.dir * toi;
    handle_entity_collision(handle, hit_point, normal);
}
```

### 几何体碰撞查询

```rust
// 检查点是否在几何体内
let point = Point::new(10.0, 20.0);
let colliding = physics_world.intersection_with_point(&point, groups, filter);

// 几何体相交测试
let aabb = AABB::new(min, max);
for intersecting_handle in physics_world.intersections_with_aabb(&aabb, groups, filter) {
    // 处理相交的刚体
}
```

## 传感器和触发区域

### 创建传感器

```rust
// 传感器碰撞器 - 检测重叠但不产生物理响应
let collider = ColliderBuilder::ball(radius)
    .sensor(true)
    .active_events(ActiveEvents::COLLISION_EVENTS)
    .build();
```

### 碰撞事件处理

```rust
fn handle_collision_events(
    mut collision_events: EventReader<CollisionEvent>,
) {
    for collision_event in collision_events.iter() {
        match collision_event {
            CollisionEvent::Started(entity1, entity2, _flags) => {
                // 碰撞开始
            }
            CollisionEvent::Stopped(entity1, entity2, _flags) => {
                // 碰撞结束
            }
        }
    }
}
```

## 性能优化

### 空间分割

引擎使用动态边界体积层次结构（DBVT）进行空间分割：

- 自动构建和维护包围盒层次结构
- 支持快速的碰撞查询和光束投射
- 动态更新以适应移动对象

### 睡眠和唤醒

静态对象自动进入睡眠状态：

```rust
// 唤醒对象
rigid_body.wake_up();

// 检查是否在睡眠
if rigid_body.is_sleeping() {
    // 对象处于睡眠状态
}
```

### 岛屿分割

大型场景中的孤立对象组被分别模拟：
- 减少不必要的计算
- 支持大世界分页加载
- 自动隔离独立物理群岛

## 调试可视化

```rust
// 启用物理调试渲染
#[cfg(feature = "debug_physics")]
fn physics_debug_render_system(
    physics_world: Res<PhysicsWorld>,
    mut gizmos: Gizmos,
) {
    // 渲染碰撞形状
    for (handle, collider) in physics_world.collider_set.iter() {
        let shape = collider.shape();
        match shape {
            // 渲染不同类型的形状
        }
    }

    // 渲染接合处和约束
    for (handle, joint) in physics_world.joint_set.iter() {
        // 渲染关节约束
    }
}
```

## 最佳实践

### 碰撞形状优化

- **优先使用简单形状**: Cuboid > Capsule > TriangleMesh
- **避免过多的三角形网格**: 对于复杂几何体考虑简化网格
- **使用合适的尺寸**: 避免极小或极大的碰撞器

### 性能建议

- **减少动态对象数量**: 静态对象开销更小
- **合并小对象**: 使用复合碰撞器减少单独碰撞器数量
- **合理使用传感器**: 只在需要触发逻辑时使用

### 稳定性建议

- **使用适当的固定时间步长**: 通常 60 FPS (1/60 = 0.0167秒)
- **正确设置质量和惯性**: 避免极端值导致不稳定性
- **处理高速对象**: 使用连续碰撞检测 (CCD)

## 故障排除

### 常见物理问题

**对象穿透墙壁**
- 增加固定时间步长的子步数
- 启用连续碰撞检测
- 调整位置修正参数

**抖动或不稳定**
- 增加重力
- 降低线性/角度阻尼
- 调整关节约束参数

**性能问题**
- 降低求解器迭代次数
- 使用更简单的碰撞形状
- 启用对象睡眠
- 实施视锥剔除

### 调试技巧

1. 可视化碰撞形状和关节约束
2. 检查对象质量和尺寸是否合理
3. 验证重力和阻尼设置
4. 使用物理调试器工具
5. 检查碰撞组和过滤器