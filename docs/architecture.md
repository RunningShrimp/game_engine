# 游戏引擎架构文档

## 概述

本游戏引擎是一个高性能、跨平台的2D/3D游戏引擎，使用Rust构建。引擎采用ECS（实体组件系统）架构，遵循领域驱动设计（DDD）原则，提供丰富的功能和可扩展性。

## 架构设计原则

### 1. 领域驱动设计（DDD）

引擎采用富领域模型（Rich Domain Model）设计：

- **聚合根（Aggregate Roots）**: `GameEntity`、`Scene`等
- **领域对象**: 包含业务逻辑的领域对象（如`AnimationPlayer`）
- **领域服务**: 协调复杂业务场景的服务层
- **值对象**: 不可变的领域值（如`EntityId`、`SceneId`）

### 2. 实体组件系统（ECS）

使用Bevy ECS进行游戏对象管理：

- **实体（Entity）**: 游戏对象的唯一标识符
- **组件（Component）**: 实体的数据和状态
- **系统（System）**: 处理组件的逻辑
- **资源（Resource）**: 全局状态和配置

### 3. 分层架构

```
┌─────────────────────────────────────┐
│        应用层（Application）          │
│  - 游戏逻辑                          │
│  - 场景管理                          │
└─────────────────────────────────────┘
           │
┌─────────────────────────────────────┐
│        领域层（Domain）               │
│  - 聚合根（GameEntity, Scene）       │
│  - 领域服务（AnimationService等）     │
│  - 领域事件                           │
└─────────────────────────────────────┘
           │
┌─────────────────────────────────────┐
│        服务层（Services）             │
│  - RenderService                    │
│  - AudioService                     │
│  - NetworkService                   │
└─────────────────────────────────────┘
           │
┌─────────────────────────────────────┐
│        基础设施层（Infrastructure）   │
│  - 渲染（wgpu）                      │
│  - 物理（Rapier）                    │
│  - 音频（rodio）                     │
│  - 网络（TCP/UDP）                    │
└─────────────────────────────────────┘
```

## 核心模块

### 引擎核心（core）

- **engine**: 主引擎循环和初始化
- **systems**: ECS系统定义
- **resources**: ECS资源定义
- **scheduler**: 任务调度系统
- **error_aggregator**: 错误聚合和统计

### 领域层（domain）

- **entity**: 游戏实体聚合根
- **scene**: 场景聚合根
- **events**: 领域事件系统
- **services**: 领域服务

### 渲染系统（render）

- **wgpu_utils**: wgpu渲染器封装
- **pbr**: 基于物理的渲染
- **gpu_driven**: GPU驱动渲染
- **postprocess**: 后处理效果
  - **PostProcessPipeline**: 固定效果链管线
  - **PostProcessEffectManager**: 动态效果链管理器，支持自适应质量调整
  - **效果类型**: Bloom、SSAO、Motion Blur、Depth of Field、Color Correction、Tonemap
- **webgl_adapter**: WebGL适配器和WGSL到GLSL转换器
- **instance_batch**: 实例批处理系统
- **lod**: 多级细节（LOD）系统
- **csm**: 级联阴影贴图
- **frustum**: 视锥剔除系统
- **occlusion_culling**: 遮挡剔除系统

### 物理系统（physics）

- **physics2d**: 2D物理模拟
- **physics3d**: 3D物理模拟
- **parallel**: 并行物理计算

### 资源管理（resources）

- **manager**: 资源加载和管理
- **coroutine_loader**: 协程优化的加载器
- **atlas**: 纹理图集管理
- **resource_trait**: 统一资源接口（Resource、ResourceLoader）
- **unified_manager**: 统一资源管理器
- **dependency_manager**: 资源依赖管理（DependencyGraph）
- **hot_reload**: 资源热重载系统
- **streaming_loader**: 流式资源加载器
- **compressed_cache**: 压缩资源缓存
- **gltf_loader**: GLTF/GLB文件加载器（可选特性）

### 网络系统（network）

- **server**: 服务器实现
- **synchronization**: 状态同步
- **prediction**: 客户端预测
- **parallel**: 并行消息处理
- **webrtc**: WebRTC网络协议支持

## 性能优化

### 1. 并行处理

- **动画系统**: 使用`rayon`并行更新动画
- **网络处理**: 线程池并行处理消息
- **AI路径寻找**: 并行路径计算

### 2. 内存优化

- **对象池**: 预定义的对象池减少分配
- **碎片整理**: 内存碎片监控和整理
- **Arena分配器**: 用于临时分配
- **WASM内存池**: WebAssembly平台的内存池管理
- **SIMD优化**: 使用SIMD指令加速计算
- **线性内存管理**: WASM线性内存优化策略

### 3. 渲染优化

- **GPU驱动渲染**: 使用计算着色器进行剔除
- **批处理**: 合并draw call
- **状态缓存**: LRU缓存减少状态切换
- **后处理优化**: 效果链自动优化和合并
- **自适应质量**: 根据性能自动调整后处理质量
- **WebGL优化**: WebGL能力检测和性能优化建议

## 扩展性

### 插件系统

引擎提供插件系统支持功能扩展：

- **EnginePlugin**: 插件trait定义
- **生命周期管理**: 初始化、更新、清理
- **依赖管理**: 插件依赖解析
- **热重载**: 支持运行时插件重载

### 脚本系统

支持多种脚本语言：

- **Lua**: 通过`rquickjs`支持
- **Python**: 通过`pyo3`支持（可选）
- **WebAssembly**: 通过`wasmtime`支持（可选）

## 跨平台支持

### 桌面平台

- Windows、macOS、Linux
- 使用`winit`进行窗口管理
- 使用`wgpu`进行渲染

### Web平台

- WebAssembly支持
- 使用`wasm-bindgen`进行绑定
- WebGL/WebGPU渲染
- **WASM性能优化**: 内存池、SIMD、线性内存管理
- **WebGL适配器**: WGSL到GLSL转换、能力检测、性能优化

### 移动平台

- iOS、Android
- 触摸输入支持
- 移动设备优化

### XR平台

- OpenXR支持
- VR/AR/MR应用
- 手部追踪和空间锚点

## 数据流

### 渲染流程

```
ECS World
  ↓
提取组件（Transform, Sprite, Camera等）
  ↓
视锥剔除
  ↓
批处理构建
  ↓
GPU驱动渲染
  ↓
后处理
  ↓
显示
```

### 资源加载流程

```
资源请求
  ↓
异步加载队列
  ↓
后台线程解码
  ↓
GPU上传
  ↓
完成回调
```

### 网络同步流程

```
客户端输入
  ↓
客户端预测
  ↓
发送到服务器
  ↓
服务器权威验证
  ↓
状态同步
  ↓
延迟补偿
```

## 错误处理

引擎使用统一的错误处理系统：

- **EngineError**: 统一错误类型
- **错误恢复**: 自动重试和降级
- **错误监控**: 错误统计和报告
- **错误链**: 上下文传播

详见[错误处理指南](guides/error_handling_guide.md)。

## 测试策略

### 单元测试

- 核心模块单元测试
- 领域对象测试
- 服务层测试

### 集成测试

- 场景加载测试
- 事件系统测试
- 资源加载测试

### 性能测试

- 基准测试（benchmarks）
- 性能回归测试
- 负载测试

## 新增架构特性

### 后处理效果管理系统

**PostProcessEffectManager** 提供动态后处理效果链管理：

- **动态效果链**: 运行时添加/移除效果
- **自动优化**: 效果链自动排序和合并兼容效果
- **自适应质量**: 根据性能自动调整质量模式
- **预设管理**: 保存和加载效果配置
- **性能监控**: 跟踪每个效果的GPU时间

详见[后处理效果API指南](guides/postprocess_api_guide.md)。

### 统一资源管理系统

**UnifiedResourceManager** 提供统一的资源管理接口：

- **统一接口**: Resource和ResourceLoader trait
- **依赖管理**: DependencyGraph自动管理资源依赖
- **热重载**: HotReloadManager支持运行时资源更新
- **流式加载**: StreamingLoader支持大资源流式加载
- **压缩缓存**: CompressedResourceCache自动压缩和缓存

### WebAssembly优化

针对Web平台的性能优化：

- **内存池**: WasmMemoryPool减少内存分配开销
- **SIMD支持**: WasmSimdSupport检测和利用SIMD指令
- **线性内存优化**: WasmLinearMemoryOptimizer管理内存增长策略

详见[WASM构建指南](guides/wasm_build_guide.md)。

### 异步寻路服务

**AsyncPathfindingService** 提供协程版本的寻路服务：

- **异步接口**: 使用Tokio协程实现异步寻路
- **性能提升**: 相比同步版本减少阻塞
- **易于集成**: 与异步资源加载系统集成

详见[异步寻路指南](guides/async_pathfinding_guide.md)。

### 性能监控系统

**PerformanceDashboard** 提供实时性能监控：

- **Web仪表盘**: 基于Web的实时性能监控界面
- **性能热力图**: 帧时间分布可视化
- **GPU监控**: GPU性能指标和着色器阶段分析
- **回归检测**: 自动检测性能回归并生成报告

## 相关文档

- [API参考](api_reference.md)
- [特性标志使用指南](guides/feature_flags_guide.md)
- [错误处理指南](guides/error_handling_guide.md)
- [服务层指南](guides/service_layer_guide.md)
- [插件系统指南](guides/plugin_system_guide.md)
- [对象池使用指南](guides/object_pool_usage_guide.md)
- [后处理效果API指南](guides/postprocess_api_guide.md)
- [WASM构建指南](guides/wasm_build_guide.md)
- [异步寻路指南](guides/async_pathfinding_guide.md)
- [ADR记录](adr/README.md)

