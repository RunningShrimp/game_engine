# 核心概念

了解游戏引擎的核心概念和架构。

## ECS (Entity Component System)

ECS 是引擎的核心架构模式。

### 实体 (Entity)

实体是游戏世界中的对象，由唯一 ID 标识。

```rust
let entity = world.spawn(()).id();
```

### 组件 (Component)

组件是实体的数据。

```rust
#[derive(Component)]
struct Health {
    current: f32,
    max: f32,
}

world.spawn(Health { current: 100.0, max: 100.0 });
```

### 系统 (System)

系统是操作组件的逻辑。

```rust
fn update_health_system(mut query: Query<&mut Health>) {
    for mut health in query.iter_mut() {
        health.current -= 1.0;
    }
}
```

## 渲染系统

### 渲染管线

引擎使用基于 wgpu 的现代渲染管线：
- 前向渲染
- 延迟渲染（可选）
- PBR 材质
- 后处理效果

### 材质系统

```rust
use game_engine::render::pbr::PbrMaterial;

let material = PbrMaterial {
    base_color: glam::Vec4::new(1.0, 0.0, 0.0, 1.0),
    metallic: 0.5,
    roughness: 0.3,
    ..Default::default()
};
```

## 物理系统

基于 Rapier 物理引擎，支持：
- 刚体物理
- 碰撞检测
- 关节约束
- 触发器

## 资源管理

### 异步加载

```rust
use game_engine::resources::coroutine_loader::CoroutineAssetLoader;

let mut loader = CoroutineAssetLoader::new(config);
loader.load_texture("texture.png", priority)?;
```

### 热重载

资源支持热重载，修改文件后自动更新。

## 网络同步

### 客户端预测

减少输入延迟，提升响应性。

### 服务器权威

防止作弊，确保游戏公平性。

## 性能优化

### SIMD 加速

自动使用 SIMD 指令加速数学运算。

### GPU 驱动剔除

使用 GPU 进行视锥体剔除，减少 CPU 负担。

### 批渲染

自动批处理绘制调用，减少状态切换。

