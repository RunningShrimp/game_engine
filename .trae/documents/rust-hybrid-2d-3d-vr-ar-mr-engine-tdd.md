# Rust混合2D/3D/VR/AR/MR游戏引擎技术设计文档（TDD）规划

## 架构总览
- 分层模块化：`platform`、`core`、`ecs`、`render`、`physics`、`resources`、`script`、`xr`、`tools`、`web`
- 平台抽象层：统一窗口/输入/文件系统接口，特别抽象Web（`HTMLCanvasElement`、事件、虚拟FS/`fetch`）
- 子系统管理器：引擎生命周期（init/start/update/shutdown）、调度、服务注册与依赖注入（DDD服务）
- 混合场景管理：声明式场景描述、层级合成、2D/3D/XR统一场景树（UI层/2D层/3D层/XR层）
- 设计模式：观察者（事件总线）、工厂（资源/渲染管线）、策略（后端选择/批处理策略）、命令（渲染队列）、适配器（平台与第三方）
- Rust安全：所有权/借用确保数据竞态避免；`Send`/`Sync`界限；无锁结构与分区并行；`Arc`与分代Arena

## ECS与DDD贫血模型
- ECS组件：纯数据容器（位置、渲染、物理、XR姿态、脚本绑定等）
- 系统：仅编排调用领域服务，不含业务逻辑
- 领域服务：独立模块实现规则与行为（移动、动画、AI、脚本交互等）
- 事件驱动：系统与服务通过事件总线通信；快照与回滚支持
- 存储：稀疏集+chunk分配；调度：阶段/标签/依赖；类型绑定：跨语言安全桥接

## 渲染系统（wgpu）
- 统一图形API抽象：命令缓冲/渲染队列/资源描述（管线/绑定/纹理/缓冲），支持Vulkan/Metal/DX12/WebGPU
- Shader工具链：WGSL为主，SPIR-V/HLSL/MSL交叉编译；离线/在线编译缓存；反射生成绑定布局
- 声明式场景 & 分层合成：参考Flutter，逻辑场景→渲染对象管道，差异计算最小重绘；合成层由GPU合成
- Web集成：`WebGPU + HTML/CSS`，UI复合到合成层；Canvas层次与CSS变换同步；输入事件桥接
- 2D管线：保留模式、批处理（Sprite/Shape/Text）、图层合成、九宫格/图块、Paper2D风格；字体与文本布局
- 3D管线：前向+延迟渲染、PBR材质、CSM阴影、后处理；实例化/LOD；天空盒/探针；可编程材质图
- 相机系统：透视/正交、叠层相机、立体渲染（左右眼、IPD、畸变校正）；XR相机同步

## OpenXR与XR模块
- OpenXR集成：会话/空间/动作系统、图层提交、帧时序；平台适配（SteamVR、Oculus等）
- VR/AR/MR支持：透传/混合渲染、锚点/平面检测（AR平台桥接）、手势/控制器输入映射
- XR渲染管线：双眼/多视图、时间扭曲、固定/可变注视点渲染（Foveated）；90FPS目标策略
- 追踪与校准：传感器融合、预测（Kalman/滑动平均）、重投影；安全边界与舒适度策略

## 物理与资源管理
- 物理后端：2D（Rapier2D等）、3D（Rapier3D/PhysX桥接）；状态插值、子步长、调试绘制
- 异步资源管线：`async/await`加载、引用计数、依赖图、热重载（原生动态库/脚本；Web用WebSocket替代）
- 文件系统监控：原生`notify`/`inotify`/`FSEvents`；Web用虚拟FS与`fetch`缓存；替换策略（原子切换）

## 脚本与工具链
- 多语言脚本层：
  - C#：.NET运行时/托管互操作（P/Invoke/hostfxr），调用领域服务
  - JavaScript：`wasm-bindgen` + QuickJS/Node；Web宿主优化边界调用
  - Python：PyO3/CPython嵌入；GIL与异步桥接
  - Go：Yaegi解释器；类型映射
- 统一绑定接口：IDL/宏生成、类型安全/生命周期管理、零拷贝传参（`&[u8]`/`WasmPtr`）
- 任务调度：脚本任务→引擎调度队列；沙箱与权限控制
- 工具：场景编辑器/性能分析器（egui），热重载、ECS可视化、RenderDoc/trace集成

## 性能优化
- CPU：SIMD（`std::simd`/`packed_simd`）、批处理、锁分离/无锁队列、工作窃取线程池
- GPU：实例化、合批、绑定复用、着色器特化、图层合成、管线缓存
- 内存：自定义分配器、Arena/Pool、SoA布局、跨帧复用；资源压缩
- 并发：任务分区、`Rayon`/自研调度、跨平台线程亲和
- WebAssembly：体积优化（`-Oz`/裁剪特性）、边界调用降频、共享内存、`wasm-opt`
- XR专注：时间预算（11ms）、异步时间扭曲、优先级调度、固定注视点渲染确保≥90FPS

## 接口与关键算法（交付内容将详细列出）
- 平台接口：`Window`, `Input`, `Filesystem`（Web适配）
- 渲染接口：`RenderDevice`, `RenderQueue`, `PipelineDesc`, `ShaderModule`, `LayerCompositor`
- ECS接口：`World`, `Scheduler`, `ComponentStore`, `EventBus`
- 资源接口：`AssetLoader`, `AssetCache`, `HotReloadService`
- 物理接口：`PhysicsWorld`, `DebugDraw`, `InterpolationService`
- XR接口：`XrSession`, `XrSpace`, `XrSwapchain`, `XrAction`
- 脚本绑定：`ScriptHost`, `TypeBridge`, `ServiceInvoker`
- 算法：场景差异计算（树哈希/增量布局）、批处理分桶、CSM级联划分、PBR BRDF、插值预测

## 阶段里程碑与交付
- 阶段1（2D基础 + Web）：2D管线、ECS/DDD、资源加载、WebAssembly演示；单测/基准/API文档/示例
- 阶段2（3D扩展）：PBR/CSM、3D相机/实例化、物理3D、编辑器原型；跨平台桌面/移动
- 阶段3（VR/AR/MR集成）：OpenXR、立体渲染、Foveated、AR平面/锚点、MR合成；90FPS基准与工具

## 输出格式与内容
- 最终TDD以Markdown提供：详细架构、接口签名、数据结构、流程图、伪代码/示例、表格
- 完整覆盖：Rust安全并发、wgpu抽象、ECS+DDD、OpenXR、2D/3D/XR管线、脚本绑定、WASM集成
- 每阶段含：测试策略、性能指标、API文档与示例项目

---
请确认本规划是否符合预期。确认后我将生成完整、可执行的TDD文档。