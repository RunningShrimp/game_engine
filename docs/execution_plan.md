# 游戏引擎开发执行计划

**版本**: 1.0  
**日期**: 2025-11-27  
**状态**: 🚀 即将启动

---

## 📊 项目现状分析

### 已完成的功能模块

| 模块 | 状态 | 说明 |
|------|------|------|
| ECS 核心 | ✅ 完成 | Bevy ECS 集成，贫血模型架构 |
| 2D 渲染 | ✅ 完成 | SpriteBatch, 纹理图集 |
| 异步资源管理 | ⚡ 部分完成 | 纹理异步加载已实现，模型解析待优化 |
| 2D 物理 | ✅ 完成 | Rapier2D 集成 |
| GPU Driven 剔除 | ✅ 完成 | Compute Shader 视锥剔除 |
| 骨骼蒙皮计算 | ⚡ 基础完成 | SIMD 优化的 LBS/DQS 蒙皮 |

### 待实现的关键功能

| 功能 | 优先级 | 当前进度 |
|------|--------|----------|
| 3D 网格实例化渲染 | P0 | 基础 `Instance3D` 结构已有 |
| 骨骼动画系统 | P0 | 蒙皮计算已有，缺少完整管线 |
| GPU 粒子系统 | P1 | 编辑器原型已有，运行时待实现 |
| 物理脏标记优化 | P1 | 待实现 |

---

## 🗓️ 第一阶段：核心性能与关键功能 (Weeks 1-4)

### Week 1-2: 3D 网格实例化渲染 (P0)

**目标**: 减少 70-90% Draw Call，支持大规模场景渲染

#### 1.1 创建 InstanceBatch 系统

```
📁 src/render/
   ├── instance_batch.rs    [新建] 实例批处理核心
   ├── batch_builder.rs     [新建] 批次构建器
   └── mod.rs               [修改] 导出新模块
```

**核心数据结构**:

```rust
// src/render/instance_batch.rs

/// 实例批次 - 相同 Mesh + Material 的实例集合
pub struct InstanceBatch {
    /// 网格句柄
    pub mesh: Handle<GpuMesh>,
    /// 材质句柄
    pub material: Handle<PbrMaterial>,
    /// 实例数据
    pub instances: Vec<Instance3D>,
    /// GPU 实例缓冲区
    pub instance_buffer: Option<wgpu::Buffer>,
    /// 批次脏标记
    pub dirty: bool,
}

/// 批次管理器
pub struct BatchManager {
    /// 批次映射: (mesh_id, material_id) -> batch_index
    batch_map: HashMap<(u64, u64), usize>,
    /// 所有批次
    batches: Vec<InstanceBatch>,
    /// 静态批次（不常更新）
    static_batches: Vec<InstanceBatch>,
}
```

**任务清单**:

- [ ] **Task 1.1.1**: 创建 `InstanceBatch` 和 `BatchManager` 结构
- [ ] **Task 1.1.2**: 实现批次收集系统 (`batch_collection_system`)
  - 遍历所有 `Mesh3D` + `Transform` 实体
  - 按 (mesh_id, material_id) 分组
  - 生成 `Instance3D` 数据
- [ ] **Task 1.1.3**: 实现批次上传系统 (`batch_upload_system`)
  - 脏批次检测
  - 双缓冲实例数据上传
- [ ] **Task 1.1.4**: 修改 `PbrRenderer` 支持实例化绘制
  - 替换逐对象 `draw_indexed` 为 `draw_indexed_instanced`
- [ ] **Task 1.1.5**: 集成 GPU Driven 剔除
  - 已有 `GpuDrivenRenderer` 的剔除结果作为可见实例

**性能指标**:
- 10,000 相同网格对象: Draw Call 从 10,000 → 1
- 帧时间预期提升: 30-50%

#### 1.2 重构渲染管线

```rust
// 新的渲染流程
pub fn render_pbr_instanced(
    renderer: &PbrRenderer,
    batch_manager: &BatchManager,
    view_proj: Mat4,
    encoder: &mut CommandEncoder,
) {
    for batch in batch_manager.visible_batches() {
        // 1. 绑定 Mesh
        render_pass.set_vertex_buffer(0, batch.mesh.vertex_buffer.slice(..));
        render_pass.set_index_buffer(batch.mesh.index_buffer.slice(..), IndexFormat::Uint32);
        
        // 2. 绑定实例缓冲区
        render_pass.set_vertex_buffer(1, batch.instance_buffer.slice(..));
        
        // 3. 绑定材质
        render_pass.set_bind_group(1, &batch.material_bind_group, &[]);
        
        // 4. 实例化绘制
        render_pass.draw_indexed(0..batch.mesh.index_count, 0, 0..batch.instance_count);
    }
}
```

---

### Week 2-3: 异步资源加载优化 (P0)

**目标**: 消除主线程卡顿，纹理/模型加载完全异步化

#### 2.1 现状分析

当前 `AssetServer` 已实现:
- ✅ 纹理异步读取 (`tokio::fs::read`)
- ✅ 纹理异步解码 (`spawn_blocking`)
- ⚠️ 模型解析未异步化
- ⚠️ GPU 上传仍在主线程

#### 2.2 优化任务

```
📁 src/resources/
   ├── manager.rs           [修改] 添加模型异步加载
   ├── staging_buffer.rs    [新建] GPU Staging Buffer 管理
   └── upload_queue.rs      [新建] 异步上传队列
```

**任务清单**:

- [ ] **Task 2.2.1**: 实现 GLTF 异步解析
  ```rust
  pub async fn load_gltf(&self, path: &Path) -> Handle<GltfScene> {
      let bytes = tokio::fs::read(path).await?;
      let scene = tokio::task::spawn_blocking(move || {
          gltf::import_slice(&bytes)
      }).await??;
      // 返回 Handle，实际 GPU 资源稍后上传
      Handle::new_loading()
  }
  ```

- [ ] **Task 2.2.2**: 实现 Staging Buffer 上传队列
  ```rust
  pub struct UploadQueue {
      pending: Vec<PendingUpload>,
      staging_buffer: wgpu::Buffer,
  }
  
  impl UploadQueue {
      pub fn queue_texture(&mut self, data: &[u8], target: &wgpu::Texture);
      pub fn queue_buffer(&mut self, data: &[u8], target: &wgpu::Buffer);
      pub fn flush(&mut self, encoder: &mut CommandEncoder);
  }
  ```

- [ ] **Task 2.2.3**: 实现加载优先级队列
  - 近距离资源高优先级
  - LOD 0 优先于 LOD 1+
  - 可见物体优先于不可见

- [ ] **Task 2.2.4**: 添加资源加载进度回调
  ```rust
  pub enum AssetEvent {
      Progress { path: PathBuf, loaded: usize, total: usize },
      Completed { path: PathBuf, handle: HandleId },
      Failed { path: PathBuf, error: String },
  }
  ```

**性能指标**:
- 大纹理 (4K) 加载: 主线程阻塞 0ms
- 场景加载时 FPS 波动: < 5%

---

### Week 3-4: 骨骼动画系统 (P0)

**目标**: 支持复杂角色动画，GPU Skinning

#### 3.1 数据结构设计

```
📁 src/animation/
   ├── skeleton.rs          [新建] 骨骼层级结构
   ├── skin.rs              [新建] 蒙皮绑定数据
   ├── skinned_mesh.rs      [新建] 蒙皮网格组件
   └── gpu_skinning.wgsl    [新建] GPU 蒙皮着色器
```

**核心组件**:

```rust
// src/animation/skeleton.rs

/// 骨骼节点
#[derive(Clone)]
pub struct Bone {
    pub name: String,
    pub parent_index: Option<usize>,
    pub local_transform: Transform,
    pub inverse_bind_matrix: Mat4,
}

/// 骨骼层级
#[derive(Component)]
pub struct Skeleton {
    pub bones: Vec<Bone>,
    /// 当前姿态的骨骼矩阵 (世界空间)
    pub bone_matrices: Vec<Mat4>,
    /// GPU 骨骼矩阵缓冲区
    pub matrix_buffer: Option<wgpu::Buffer>,
}

// src/animation/skinned_mesh.rs

/// 蒙皮网格组件
#[derive(Component)]
pub struct SkinnedMesh {
    /// 基础网格
    pub mesh: Handle<GpuMesh>,
    /// 关联的骨骼
    pub skeleton: Entity,
    /// 顶点骨骼权重 (已烘焙到顶点属性)
    pub skin_weights_buffer: wgpu::Buffer,
}
```

#### 3.2 任务清单

- [ ] **Task 3.2.1**: 实现 `Skeleton` 和 `Bone` 数据结构
- [ ] **Task 3.2.2**: 扩展 GLTF 加载器解析骨骼数据
  ```rust
  fn parse_gltf_skin(gltf_skin: &gltf::Skin) -> Skeleton {
      // 解析骨骼层级
      // 提取 inverse_bind_matrices
  }
  ```

- [ ] **Task 3.2.3**: 实现骨骼姿态更新系统
  ```rust
  pub fn skeleton_update_system(
      time: Res<Time>,
      mut query: Query<(&mut Skeleton, &AnimationPlayer)>,
  ) {
      for (mut skeleton, player) in query.iter_mut() {
          if let Some(clip) = &player.current_clip {
              // 1. 采样当前时间的骨骼变换
              // 2. 计算世界空间骨骼矩阵
              // 3. 更新 GPU 缓冲区
          }
      }
  }
  ```

- [ ] **Task 3.2.4**: 实现 GPU Skinning 着色器
  ```wgsl
  // src/animation/gpu_skinning.wgsl
  
  @group(2) @binding(0) var<storage, read> bone_matrices: array<mat4x4<f32>>;
  
  struct VertexInput {
      @location(0) position: vec3<f32>,
      @location(1) normal: vec3<f32>,
      @location(2) uv: vec2<f32>,
      @location(3) bone_indices: vec4<u32>,
      @location(4) bone_weights: vec4<f32>,
  };
  
  fn skin_vertex(input: VertexInput) -> vec3<f32> {
      var skinned_pos = vec3<f32>(0.0);
      for (var i = 0u; i < 4u; i++) {
          let bone_idx = input.bone_indices[i];
          let weight = input.bone_weights[i];
          skinned_pos += (bone_matrices[bone_idx] * vec4(input.position, 1.0)).xyz * weight;
      }
      return skinned_pos;
  }
  ```

- [ ] **Task 3.2.5**: 扩展 `Vertex3D` 支持蒙皮属性
  ```rust
  #[repr(C)]
  pub struct SkinnedVertex3D {
      pub pos: [f32; 3],
      pub normal: [f32; 3],
      pub uv: [f32; 2],
      pub bone_indices: [u32; 4],  // 新增
      pub bone_weights: [f32; 4],  // 新增
  }
  ```

- [ ] **Task 3.2.6**: 集成到 PBR 渲染管线
  - 创建支持骨骼的 PBR Pipeline 变体
  - 骨骼矩阵作为 Storage Buffer 绑定

**性能指标**:
- 100 骨骼角色: < 0.5ms/帧 (GPU Skinning)
- 支持最大骨骼数: 256

---

## 🗓️ 第二阶段：物理与粒子系统 (Weeks 5-8)

### Week 5-6: 物理同步优化 (P0/P1)

**目标**: 减少物理→Transform 同步开销

#### 4.1 脏标记机制

```rust
// src/physics/dirty_tracker.rs [新建]

/// 物理脏标记组件
#[derive(Component, Default)]
pub struct PhysicsDirty {
    pub transform_changed: bool,
    pub velocity_changed: bool,
}

/// 优化的同步系统
pub fn sync_physics_to_transform_system(
    physics_state: Res<PhysicsState>,
    mut query: Query<(&RigidBodyComp, &mut Transform), Changed<RigidBodyComp>>,
) {
    for (rb, mut transform) in query.iter_mut() {
        if let Some(body) = physics_state.rigid_body_set.get(rb.handle) {
            // 跳过休眠的刚体
            if body.is_sleeping() {
                continue;
            }
            
            let pos = body.translation();
            let rot = body.rotation();
            transform.position = Vec3::new(pos.x, pos.y, 0.0);
            transform.rotation = rot.angle();
        }
    }
}
```

**任务清单**:

- [ ] **Task 4.1.1**: 实现 `PhysicsDirty` 组件
- [ ] **Task 4.1.2**: 利用 Bevy ECS `Changed<T>` 过滤器优化同步
- [ ] **Task 4.1.3**: 添加休眠体跳过逻辑
- [ ] **Task 4.1.4**: 实现批量同步优化

---

### Week 6-8: GPU 粒子系统 (P1)

**目标**: 百万级粒子实时模拟

```
📁 src/render/particles/
   ├── mod.rs               [新建] 模块入口
   ├── emitter.rs           [新建] 发射器组件
   ├── simulation.wgsl      [新建] 粒子模拟 Compute Shader
   └── render.wgsl          [新建] 粒子渲染着色器
```

#### 5.1 粒子系统架构

```rust
// src/render/particles/emitter.rs

#[derive(Component)]
pub struct ParticleEmitter {
    pub max_particles: u32,
    pub emission_rate: f32,
    pub lifetime: Range<f32>,
    pub initial_velocity: Range<Vec3>,
    pub gravity: Vec3,
    pub color_over_lifetime: ColorGradient,
    pub size_over_lifetime: Curve,
}

pub struct GpuParticleSystem {
    // GPU Buffers
    particle_buffer: wgpu::Buffer,      // 粒子状态
    alive_list: wgpu::Buffer,           // 存活粒子索引
    dead_list: wgpu::Buffer,            // 死亡粒子索引
    counter_buffer: wgpu::Buffer,       // 原子计数器
    
    // Pipelines
    emit_pipeline: wgpu::ComputePipeline,
    update_pipeline: wgpu::ComputePipeline,
    render_pipeline: wgpu::RenderPipeline,
}
```

#### 5.2 Compute Shader 模拟

```wgsl
// src/render/particles/simulation.wgsl

struct Particle {
    position: vec3<f32>,
    velocity: vec3<f32>,
    lifetime: f32,
    age: f32,
    color: vec4<f32>,
    size: f32,
};

@group(0) @binding(0) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(1) var<storage, read_write> alive_count: atomic<u32>;

@compute @workgroup_size(64)
fn update_particles(@builtin(global_invocation_id) id: vec3<u32>) {
    let idx = id.x;
    if (idx >= arrayLength(&particles)) { return; }
    
    var p = particles[idx];
    if (p.age >= p.lifetime) { return; } // Dead
    
    // Physics update
    p.velocity += uniforms.gravity * uniforms.delta_time;
    p.position += p.velocity * uniforms.delta_time;
    p.age += uniforms.delta_time;
    
    // Color/size over lifetime
    let t = p.age / p.lifetime;
    p.color = sample_gradient(t);
    p.size = sample_curve(t);
    
    particles[idx] = p;
}
```

**任务清单**:

- [ ] **Task 5.2.1**: 创建 `ParticleEmitter` 组件
- [ ] **Task 5.2.2**: 实现 GPU 粒子缓冲区管理
- [ ] **Task 5.2.3**: 实现粒子发射 Compute Shader
- [ ] **Task 5.2.4**: 实现粒子更新 Compute Shader
- [ ] **Task 5.2.5**: 实现粒子渲染 (Billboard/Point Sprite)
- [ ] **Task 5.2.6**: 集成到编辑器粒子预览

**性能指标**:
- 100 万粒子: < 2ms/帧 (Compute + Render)
- 粒子排序: GPU Radix Sort

---

## 🗓️ 第三阶段：工程质量与扩展 (Weeks 9-12)

### Week 9-10: 自动化测试框架 (P1)

```
📁 tests/
   ├── render/
   │   ├── instance_batch_tests.rs
   │   ├── pbr_visual_tests.rs
   │   └── golden_images/
   └── physics/
       ├── collision_tests.rs
       └── sync_tests.rs
```

#### 6.1 渲染回归测试

```rust
// tests/render/pbr_visual_tests.rs

#[test]
fn test_pbr_sphere_lighting() {
    let ctx = create_headless_render_context();
    
    // 设置场景
    let sphere = ctx.create_mesh(Mesh::sphere(1.0));
    let material = PbrMaterial { roughness: 0.5, metallic: 0.0, ..default() };
    ctx.add_render_object(sphere, material, Transform::IDENTITY);
    ctx.add_light(DirectionalLight::new(Vec3::new(1.0, -1.0, -1.0)));
    
    // 渲染
    let frame = ctx.render_frame();
    
    // 对比 Golden Image
    assert_image_matches!(frame, "golden_images/pbr_sphere.png", tolerance: 0.01);
}
```

**任务清单**:

- [ ] **Task 6.1.1**: 创建无头渲染测试框架
- [ ] **Task 6.1.2**: 实现图像对比工具
- [ ] **Task 6.1.3**: 添加 PBR 渲染测试用例
- [ ] **Task 6.1.4**: 添加实例化渲染测试用例

#### 6.2 物理确定性测试

```rust
#[test]
fn test_physics_determinism() {
    let mut state1 = PhysicsState::default();
    let mut state2 = PhysicsState::default();
    
    // 相同初始条件
    setup_test_scene(&mut state1);
    setup_test_scene(&mut state2);
    
    // 模拟 100 帧
    for _ in 0..100 {
        PhysicsService::step(&mut state1);
        PhysicsService::step(&mut state2);
    }
    
    // 验证结果一致
    assert_eq!(
        get_body_position(&state1, "ball"),
        get_body_position(&state2, "ball")
    );
}
```

---

### Week 11-12: 插件系统 (P1)

```
📁 src/plugins/
   ├── mod.rs               [新建] 插件系统核心
   ├── registry.rs          [新建] 插件注册表
   └── builtin/             [新建] 内置插件
       ├── physics.rs
       └── audio.rs
```

#### 7.1 插件 Trait 定义

```rust
// src/plugins/mod.rs

pub trait EnginePlugin: Send + Sync {
    /// 插件名称
    fn name(&self) -> &'static str;
    
    /// 构建阶段 - 注册资源和系统
    fn build(&self, app: &mut App);
    
    /// 启动阶段 - 初始化运行时状态
    fn startup(&self, world: &mut World) {}
    
    /// 更新阶段 - 每帧调用
    fn update(&self, world: &mut World) {}
    
    /// 关闭阶段 - 清理资源
    fn shutdown(&self, world: &mut World) {}
}

pub struct PluginRegistry {
    plugins: Vec<Box<dyn EnginePlugin>>,
}

impl PluginRegistry {
    pub fn add<P: EnginePlugin + 'static>(&mut self, plugin: P) {
        self.plugins.push(Box::new(plugin));
    }
    
    pub fn build_all(&self, app: &mut App) {
        for plugin in &self.plugins {
            plugin.build(app);
        }
    }
}
```

#### 7.2 重构现有模块为插件

```rust
// src/plugins/builtin/physics.rs

pub struct PhysicsPlugin {
    pub gravity: Vec2,
    pub timestep: f32,
}

impl EnginePlugin for PhysicsPlugin {
    fn name(&self) -> &'static str { "Physics2D" }
    
    fn build(&self, app: &mut App) {
        app.insert_resource(PhysicsState::default());
        app.add_system(physics_step_system);
        app.add_system(sync_physics_to_transform_system);
    }
}
```

**任务清单**:

- [ ] **Task 7.2.1**: 定义 `EnginePlugin` trait
- [ ] **Task 7.2.2**: 实现 `PluginRegistry`
- [ ] **Task 7.2.3**: 重构物理模块为 `PhysicsPlugin`
- [ ] **Task 7.2.4**: 重构音频模块为 `AudioPlugin`
- [ ] **Task 7.2.5**: 文档化插件开发指南

---

## 🗓️ 第四阶段：未来规划

### 网络系统
- [ ] 评估 `quinn` (QUIC) vs `tokio-tungstenite` (WebSocket)
- [ ] 设计网络同步架构 (状态同步 vs 命令同步)

### AI 导航
- [ ] NavMesh 生成
- [ ] A* 寻路实现
- [ ] 集成 `recast` 或自研

### UI 运行时
- [ ] 评估 `kayak_ui` / `bevy_ui` / 自研
- [ ] 设计数据绑定系统

---

## 📋 附录：文件变更清单

### 新建文件

| 文件路径 | 功能描述 |
|----------|----------|
| `src/render/instance_batch.rs` | 实例批处理系统 |
| `src/render/batch_builder.rs` | 批次构建器 |
| `src/resources/staging_buffer.rs` | GPU Staging Buffer |
| `src/resources/upload_queue.rs` | 异步上传队列 |
| `src/animation/skeleton.rs` | 骨骼数据结构 |
| `src/animation/skin.rs` | 蒙皮绑定 |
| `src/animation/skinned_mesh.rs` | 蒙皮网格组件 |
| `src/animation/gpu_skinning.wgsl` | GPU 蒙皮着色器 |
| `src/physics/dirty_tracker.rs` | 物理脏标记 |
| `src/render/particles/mod.rs` | GPU 粒子系统 |
| `src/plugins/mod.rs` | 插件系统 |

### 修改文件

| 文件路径 | 修改内容 |
|----------|----------|
| `src/render/mod.rs` | 导出新模块 |
| `src/render/pbr_renderer.rs` | 支持实例化绘制 |
| `src/render/mesh.rs` | 添加 `SkinnedVertex3D` |
| `src/resources/manager.rs` | 添加模型异步加载 |
| `src/physics/mod.rs` | 脏标记优化 |
| `src/animation/mod.rs` | 导出骨骼动画模块 |

---

## 🎯 成功标准

| 阶段 | 验收标准 |
|------|----------|
| 阶段 1 | 10K 实例渲染 60FPS，骨骼动画流畅播放 |
| 阶段 2 | 100 万粒子 60FPS，物理同步开销 < 1ms |
| 阶段 3 | 测试覆盖率 > 60%，插件系统可用 |

---

*文档版本: 1.0 | 最后更新: 2025-11-27*
