## 关键修订点
- 2D 参考 UE4 Paper2D：引入精灵组件、Flipbook 动画、纹理图集、TileMap/TileSet、2D 碰撞与灯光。
- 3D 渲染强化：明确场景图、材质系统（PBR）、光照模型（方向/点/聚光）、后处理（Bloom/SSAO/ToneMapping）。
- 深化 CPU/GPU 优化：在 ECS/资源/渲染/物理层面制定可量化策略与监控。

## 技术选型（保持不变并补充）
- 渲染抽象：`wgpu`（统一后端 Vulkan/Metal/DX12/OpenGL）。
- 窗口/输入：`winit`；ECS：`bevy_ecs`；资源：`serde`/`notify`；音频：`rodio`/`cpal`；脚本：Lua（`mlua`）+ Python（`pyo3` 可选）。
- 性能与调试：`tracing`/`tracing-subscriber`、`criterion`、`wgpu` timestamp queries。

## 2D（Paper2D 思路落地）
- 精灵组件（`SpriteComponent`）：
  - 属性：Pivot/Origin、Scale/Rotation、垂直/水平翻转、排序层/深度、混合模式。
  - 纹理图集：Atlas + 索引；批次渲染（实例化）以减少 draw calls。
- Flipbook 动画：
  - 定义帧序列（帧→AtlasRect→时长）；播放控制（速度/循环/方向/事件回调）。
  - 编辑器支持：时间线、拖拽帧、预览。
- TileMap/TileSet：
  - TileSet：包含多 tile 与元数据（碰撞形状/材质/标签）。
  - TileMap：多个层（地表/装饰/碰撞），支持自动铺设与邻接规则（rule tiles）。
  - 渲染：按块（chunk）实例化；摄像机裁剪；Z 分层。
- 2D 碰撞与物理：
  - 与 Rapier2D/Box2D 绑定；Tile 层可生成静态碰撞体；精灵支持触发器与刚体。
- 2D 光照：
  - 法线贴图支持（2D normal maps）；点光源与简单阴影投射（soft-mask）；光照混合到 sprite pass。
- UI：
  - `egui` 集成；九宫格、字体渲染；事件路由到输入系统。

## 3D 渲染（场景图/材质/光照/后处理）
- 场景图：
  - 统一 Node/Transform（层级更新、脏标记、双向绑定 ECS）。
  - 摄像机（透视/正交）组件；剔除（视锥/遮挡，后续）。
- 材质系统（PBR）：
  - 金属度/粗糙度/法线/AO/发光/透明；支持 IBL（环境贴图）与预过滤。
  - BRDF：GGX + Smith + Fresnel（Schlick）。
- 光照：
  - 方向光：CSM 阴影（级联数量可调）。
  - 点光：立方体阴影；聚光：聚光贴图；PCF/PCSS。
- 后处理：
  - Bloom、Tone Mapping（ACES）、色彩校正（LUT）、SSAO（后续）、FXAA/TAA（后续）。

## CPU 优化策略
- ECS 存储与调度：
  - SoA/Archetype 布局（`bevy_ecs` 默认支持），减少缓存未命中。
  - 系统调度并行化：基于数据访问分析（读/写）自动并行；避免不必要锁。
- 任务系统：
  - `rayon` 线程池；渲染前的组件收集/可见性计算/粒子更新并行化。
- 内存管理：
  - Arena/Pool 分配器；帧内临时分配使用复用缓冲；`smallvec` 减少堆分配。
- 资源加载：
  - 异步 IO + 解码流水线；零拷贝或最小拷贝；压缩纹理按需解压。
- 物理同步：
  - 固定步率（如 60Hz）；插值到渲染帧；批量 Transform 同步以减少锁与 cache miss。

## GPU 优化策略
- 绘制层面：
  - 实例化/批次渲染（2D/3D）；Texture Atlas；减少 pipeline/state 切换。
  - Render Graph 管理 barrier 与资源生命周期；多队列（后续）。
- 几何处理：
  - 剔除（视锥/包围体层级）；LOD；MDI（Multi-Draw Indirect）可选。
- 资源：
  - 压缩纹理（BC/DXT/ASTC/ETC2）；Mipmap；延迟上传与分块传输（staging + 双缓冲）。
- 阴影优化：
  - CSM 分层分辨率；稳定级联；PCF 采样数自适应；点光阴影选择性更新。
- 粒子：
  - GPU Compute 更新（位置/速度），仅渲染可见粒子；排序在半透明时可选近似。
- 时间查询：
  - `wgpu` timestamp queries 为各 pass 打点；驱动帧内热点分析。

## 可观测性与基准
- `tracing` 埋点：系统级与 pass 级事件（开始/结束/耗时/资源峰值）。
- GPU 计时器：各渲染 pass 与上传阶段打点；编辑器内展示帧时间分解。
- 基准测试：
  - 2D：1K/10K 精灵场景；TileMap 大图；Flipbook 帧推进。
  - 3D：100K 三角形、3 灯光阴影；材质切换；模型导入。
  - 资源：纹理/模型批量加载与热重载耗时统计。

## 工作空间与目录（补 Paper2D/优化相关）
- `render/2d/`：Sprite/Flipbook/TileMap/2D Lighting；`render/3d/`：PBR/Shadow/Post。
- `ecs/components/`：`Sprite/Tile/Light/Camera/RigidBody/Collider/AudioSource/Script`。
- `tools/editor/`：Paper2D 风格编辑器（Tile/Flipbook）、调试与性能分析器。
- `resource/`：Asset 管道与 Atlas 构建；压缩纹理工具链集成。

## 里程碑（调整含优化）
- M1：2D 精灵/Flipbook/TileMap + Rapier2D；Atlas 构建；2D 光照；2D 基准 >60FPS@1K 精灵。
- M2：3D PBR/灯光/CSM 阴影 + glTF；后处理（Bloom/ACES）；3D 基准 >60FPS@100K 三角。
- M3：脚本（Lua）绑定与事件；ECS 组件完善；输入动作映射。
- M4：场景/粒子/动画编辑器；Paper2D 工具；性能分析器（CPU/GPU）。
- M5：资源热重载与流式加载；压缩纹理；MDI/剔除/LOD；多线程渲染。
- M6：跨平台细节与多窗口；稳定化与文档；可选 PhysX/Bullet 适配。

## 验收与 KPI（含优化）
- 2D：Atlas+实例化下 1K 精灵 >60FPS，10K 精灵 >30FPS；热重载 <200ms。
- 3D：100K 三角 >60FPS（1080p），3 灯光阴影稳定；PBR/IBL 有效；后处理可切换。
- 资源：批量载入吞吐显示；压缩纹理显存占用下降 ≥40%。
- 工具：Flipbook/TileMap 编辑可用；分析器显示 CPU/GPU pass 耗时。

## 下一步（确认后执行）
- 搭建 2D 子系统（Sprite/Flipbook/TileMap）与 Atlas 管线；2D 光照原型。
- 引入 GPU 计时器与 `tracing` 埋点；建立 2D/3D 基准场景与度量仪表板。
- 完成 glTF 导入与 PBR 材质；CSM 阴影与 Bloom/ACES 后处理。