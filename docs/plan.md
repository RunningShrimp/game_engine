# Rust 高性能跨平台混合游戏引擎技术设计文档 (v2.0)

**版本**: 2.0  
**日期**: 2025-11-26  
**状态**: [🚧 进行中]

## 1. 架构总览

本引擎采用分层模块化架构，核心设计原则为 **ECS 数据驱动** 与 **DDD 贫血模型** 相结合，利用 Rust 的所有权系统确保内存安全，通过 `wgpu` 实现跨平台高性能渲染。

### 1.1 模块划分

| 模块 | 职责 | 状态 |
| :--- | :--- | :--- |
| `platform` | 平台抽象（窗口、输入、文件、Web桥接） | [🚧 部分实现] |
| `core` | 引擎内核（主循环、事件总线、时间步） | [✅ 已实现] |
| `ecs` | 实体组件系统（Bevy ECS集成） | [✅ 已实现] |
| `render` | 渲染管线（wgpu, 2D/3D/XR, 场景图） | [🚧 部分实现] |
| `physics` | 物理模拟（Rapier 2D/3D） | [🚧 2D已实现] |
| `resources` | 资源管理（异步加载、热重载、依赖图） | [✅ 已实现] |
| `scripting` | 多语言脚本绑定（JS/C#/Python/Go） | [🚧 JS已实现] |
| `xr` | VR/AR/MR 支持（OpenXR） | [📋 待实现] |
| `tools` | 编辑器与调试工具（egui） | [🚧 原型] |

### 1.2 核心设计模式

- **ECS + DDD 贫血模型**:
  - **Component (组件)**: 纯数据结构 (struct)，无方法逻辑。
  - **System (系统)**: 负责调度和编排，从 ECS 查询数据。
  - **Service (领域服务)**: 封装核心业务逻辑，由 System 调用。
  - *优势*: 逻辑复用（跨脚本/Native），测试便利，关注点分离。

- **声明式渲染 (Flutter-like)**:
  - 逻辑层生成轻量级 `RenderObject` 树。
  - 渲染层计算 Diff，生成 `LayerTree`。
  - GPU 后端进行图层合成与光栅化。

---

## 2. 平台抽象层 (Platform Abstraction)

### 2.1 平台支持矩阵

| Host Platform | Guest Arch | 状态 | 关键技术 |
| :--- | :--- | :--- | :--- |
| **Windows** | x86_64, AArch64 | [✅] | `winit`, DX12/Vulkan |
| **macOS** | x86_64, AArch64 (M1/M2/M3) | [✅] | `winit`, Metal |
| **Linux** | x86_64, RISC-V64 | [✅] | `winit`, Vulkan/Wayland |
| **Android** | AArch64 | [📋] | `android-activity`, JNI, Vulkan |
| **iOS/iPadOS** | AArch64 | [📋] | `winit`, Metal |
| **HarmonyOS** | AArch64 | [📋] | ArkUI NAPI, Vulkan/OpenGLES |
| **Web** | wasm32 | [✅] | `web-sys`, WebGPU/WebGL2 |

### 2.2 核心接口定义

```rust
// src/platform/mod.rs

/// 平台窗口抽象
pub trait Window: Send + Sync {
    fn size(&self) -> (u32, u32);
    fn scale_factor(&self) -> f32;
    fn request_redraw(&self);
    fn raw_handle(&self) -> raw_window_handle::RawWindowHandle;
}

/// 平台输入抽象
pub trait Input: Send + Sync {
    fn poll_events(&mut self) -> Vec<InputEvent>;
    fn set_cursor_grab(&mut self, grab: bool);
    // XR 特有输入
    fn xr_actions(&self) -> Option<&XrActionSet>;
}

/// 文件系统抽象 (支持 Native IO 和 Web fetch)
#[async_trait]
pub trait Filesystem: Send + Sync {
    async fn read(&self, path: &Path) -> Result<Vec<u8>, FsError>;
    async fn write(&self, path: &Path, data: &[u8]) -> Result<(), FsError>;
    fn watch(&self, path: &Path, tx: Sender<FsEvent>) -> Result<WatchHandle, FsError>;
}
```

### 2.3 HarmonyOS ArkUI 集成 (伪代码)

```rust
// src/platform/harmony.rs [📋 待实现]

#[cfg(target_os = "harmony")]
pub mod harmony {
    use node_api_sys::*;
    
    #[no_mangle]
    pub extern "C" fn NAPI_Init(env: napi_env, exports: napi_value) -> napi_value {
        // 初始化引擎实例，绑定到 ArkUI 的 XComponent
        // 注册渲染回调
        exports
    }
    
    pub fn get_native_window() -> *mut c_void {
        // 获取 OH_NativeXComponent
    }
}
```

---

## 3. 渲染系统 (Rendering System)

基于 `wgpu` 的统一渲染后端，支持 2D/3D/XR 混合渲染。

### 3.1 声明式场景图与 Diff 算法

参考 Flutter 的 Layer 体系，实现高效的增量更新。

```rust
// src/render/graph.rs [✅ 部分实现]

#[derive(Clone, PartialEq)]
pub enum Layer {
    Container { transform: Mat4, children: Vec<Layer> },
    Picture { mesh: Handle<GpuMesh>, material: Handle<Material> },
    Opacity { alpha: f32, child: Box<Layer> },
    ClipRect { rect: Rect, child: Box<Layer> },
}

pub struct LayerTree {
    pub root: Layer,
    pub dirty: bool,
}

impl LayerTree {
    /// 计算 Diff 并生成渲染命令列表
    pub fn diff(&self, old: &LayerTree) -> Vec<RenderCommand> {
        let mut commands = Vec::new();
        self.diff_recursive(&self.root, &old.root, &mut commands, Mat4::IDENTITY);
        commands
    }

    fn diff_recursive(&self, new: &Layer, old: &Layer, cmds: &mut Vec<RenderCommand>, parent_tf: Mat4) {
        // 1. 如果节点类型不同，完全重绘
        if std::mem::discriminant(new) != std::mem::discriminant(old) {
            self.emit_draw(new, cmds, parent_tf);
            return;
        }

        match (new, old) {
            (Layer::Container { transform: nt, children: nc }, 
             Layer::Container { transform: ot, children: oc }) => {
                let global_tf = parent_tf * *nt;
                // 简单优化：如果变换矩阵变了，子节点可能都需要更新
                // 更深度的优化需要对比每个子节点的 Hash
                for (n_child, o_child) in nc.iter().zip(oc.iter()) {
                    self.diff_recursive(n_child, o_child, cmds, global_tf);
                }
                // 处理新增/删除的子节点...
            },
            // ... 其他类型处理
            _ => {}
        }
    }
}
```

### 3.2 PBR 材质与光照 (WGSL)

实现 Cook-Torrance BRDF 模型。

```wgsl
// assets/shaders/pbr.wgsl [📋 待实现]

struct PbrInput {
    albedo: vec3<f32>,
    roughness: f32,
    metallic: f32,
    normal: vec3<f32>,
    view_dir: vec3<f32>,
    f0: vec3<f32>,
};

fn distribution_ggx(N: vec3<f32>, H: vec3<f32>, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let NdotH = max(dot(N, H), 0.0);
    let NdotH2 = NdotH * NdotH;
    let num = a2;
    let denom = (NdotH2 * (a2 - 1.0) + 1.0);
    return num / (3.14159 * denom * denom);
}

fn geometry_schlick_ggx(NdotV: f32, roughness: f32) -> f32 {
    let r = (roughness + 1.0);
    let k = (r * r) / 8.0;
    return NdotV / (NdotV * (1.0 - k) + k);
}

fn fresnel_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
    return f0 + (1.0 - f0) * pow(1.0 - cos_theta, 5.0);
}

fn pbr_lighting(in: PbrInput, light_dir: vec3<f32>, light_color: vec3<f32>) -> vec3<f32> {
    let L = normalize(light_dir);
    let H = normalize(in.view_dir + L);
    let N = normalize(in.normal);
    
    let NdotL = max(dot(N, L), 0.0);
    let NdotV = max(dot(N, in.view_dir), 0.0);

    let D = distribution_ggx(N, H, in.roughness);
    let G = geometry_schlick_ggx(NdotV, in.roughness) * geometry_schlick_ggx(NdotL, in.roughness);
    let F = fresnel_schlick(max(dot(H, in.view_dir), 0.0), in.f0);

    let numerator = D * G * F;
    let denominator = 4.0 * NdotV * NdotL + 0.0001;
    let specular = numerator / denominator;

    let kS = F;
    let kD = (vec3<f32>(1.0) - kS) * (1.0 - in.metallic);

    return (kD * in.albedo / 3.14159 + specular) * light_color * NdotL;
}
```

### 3.3 CSM 级联阴影 (Cascaded Shadow Maps)

```wgsl
// assets/shaders/csm.wgsl [📋 待实现]

struct Cascade {
    view_proj: mat4x4<f32>,
    split_depth: f32,
};

@group(1) @binding(0) var shadow_map: texture_depth_2d_array;
@group(1) @binding(1) var shadow_sampler: sampler_comparison;
@group(1) @binding(2) var<uniform> cascades: array<Cascade, 4>;

fn calculate_shadow(world_pos: vec3<f32>, view_depth: f32) -> f32 {
    // 1. 选择级联层级
    var cascade_idx = 3u;
    for (var i = 0u; i < 3u; i++) {
        if (view_depth < cascades[i].split_depth) {
            cascade_idx = i;
            break;
        }
    }

    // 2. 投影到光照空间
    let light_space_pos = cascades[cascade_idx].view_proj * vec4<f32>(world_pos, 1.0);
    let proj_coords = light_space_pos.xyz / light_space_pos.w;
    let uv = proj_coords.xy * 0.5 + 0.5;
    let current_depth = proj_coords.z;

    // 3. PCF 采样
    var shadow = 0.0;
    // ... PCF loop ...
    return shadow;
}
```

---

## 4. OpenXR 与 VR/AR/MR 集成

### 4.1 核心接口 (Traits)

```rust
// src/xr/mod.rs [📋 待实现]

pub trait XrSession: Send + Sync {
    fn begin_frame(&mut self) -> Result<(), XrError>;
    fn end_frame(&mut self) -> Result<(), XrError>;
    fn locate_views(&self, time: XrTime) -> Vec<XrView>;
    fn poll_events(&mut self) -> Vec<XrEvent>;
}

pub struct XrView {
    pub pose: Pose,
    pub fov: Fov,
    pub view_idx: u32,
}

pub trait XrSwapchain {
    fn acquire_image(&mut self) -> Result<u32, XrError>;
    fn release_image(&mut self) -> Result<(), XrError>;
    fn get_render_target(&self, index: u32) -> &wgpu::TextureView;
}
```

### 4.2 异步时间扭曲 (ATW) 伪代码

ATW 用于在 GPU 渲染帧率不足时，通过重投影当前帧来减少延迟晕动症。

```rust
// src/xr/atw.rs [📋 待实现]

fn atw_reprojection(
    rendered_frame: &Texture, 
    depth_buffer: &Texture,
    rendered_pose: &Pose, 
    current_pose: &Pose
) {
    // 1. 计算姿态差 (Delta Pose)
    let delta_rot = current_pose.rotation * rendered_pose.rotation.inverse();
    
    // 2. 在 Compute Shader 中对渲染帧进行扭曲
    // 对于每个像素：
    //   a. 重建世界空间位置 (利用深度缓冲)
    //   b. 应用 delta_rot
    //   c. 重新投影到屏幕空间
    //   d. 采样原纹理颜色
    
    dispatch_compute_shader(atw_pipeline, rendered_frame, output_frame, delta_rot);
}
```

### 4.3 Foveated Rendering (注视点渲染)

- **固定注视点 (FFR)**: 降低周边分辨率，中心高分辨率。
- **动态注视点 (DFR)**: 结合眼动追踪 (OpenXR `XR_EXT_eye_gaze_interaction`)。

实现策略：
1. 使用 `wgpu` 的多视口 (Multi-viewport) 或 
2. 使用 Variable Rate Shading (VRS) (如果硬件支持 Tier 2)。
3. 软件回退：渲染到三个同心圆纹理，最后合成。

---

## 5. 多语言脚本绑定 (Scripting)

### 5.1 统一绑定协议

基于 `bindings/protocol.rs`，扩展支持更多语言。

### 5.2 C# (.NET) 绑定

使用 `hostfxr` 加载 .NET 运行时。

```rust
// src/scripting/csharp.rs [📋 待实现]

use netcorehost::{nethost, pdcstr};

pub struct CSharpHost {
    host_context: netcorehost::hostfxr::HostfxrContext,
    fn_update: extern "C" fn(f32),
}

impl CSharpHost {
    pub fn init() -> Self {
        let hostfxr = nethost::load_hostfxr().unwrap();
        let context = hostfxr.initialize_for_runtime_config(pdcstr!("GameAssembly.runtimeconfig.json")).unwrap();
        
        // 获取托管函数指针
        let fn_loader = context.get_delegate_loader_for_assembly(pdcstr!("GameAssembly.dll")).unwrap();
        let update_ptr = fn_loader.get_function_pointer(
            pdcstr!("Game.Engine.Core.Bridge, GameAssembly"),
            pdcstr!("Update"),
            pdcstr!("UnmanagedCallersOnly")
        ).unwrap();
        
        Self { host_context: context, fn_update: unsafe { std::mem::transmute(update_ptr) } }
    }
}
```

### 5.3 Python 绑定 (PyO3)

```rust
// src/scripting/python.rs [📋 待实现]

use pyo3::prelude::*;

#[pyclass]
struct PyEntity {
    id: u64,
}

#[pymethods]
impl PyEntity {
    #[new]
    fn new(id: u64) -> Self { PyEntity { id } }

    fn set_position(&self, x: f32, y: f32, z: f32) {
        // 发送命令到 ECS
        crate::bindings::send_command(BindingCommand::SetPosition { entity_id: self.id, x, y, z });
    }
}

pub fn init_python_module(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyEntity>()?;
    Ok(())
}
```

---

## 6. 性能优化 (Optimization)

### 6.1 CPU SIMD 优化

针对不同架构的矩阵运算优化。

```rust
// src/math/simd.rs [📋 待实现]

#[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
mod avx2 {
    use std::arch::x86_64::*;
    
    pub unsafe fn mat4_mul_avx2(a: &[f32; 16], b: &[f32; 16], out: &mut [f32; 16]) {
        // AVX2 4x4 矩阵乘法实现
        let row0 = _mm256_load_ps(a.as_ptr());
        // ...
    }
}

#[cfg(target_arch = "aarch64")]
mod neon {
    use std::arch::aarch64::*;
    
    pub unsafe fn mat4_mul_neon(a: &[f32; 16], b: &[f32; 16], out: &mut [f32; 16]) {
        // NEON 4x4 矩阵乘法实现
    }
}
```

### 6.2 内存管理

- **Arena Allocation**: 用于每帧重置的渲染命令分配。
- **SoA (Structure of Arrays)**: ECS 组件存储默认布局，提高缓存命中率。

### 6.3 WebAssembly 优化

- **体积优化**: 启用 `lto = true`, `opt-level = "z"`, `panic = "abort"`.
- **JS 边界**: 减少 `JsValue` 转换，使用 `SharedArrayBuffer` 直接传递大块数据（如纹理、网格）。

---

## 7. 里程碑与交付计划

### 阶段 1: 2D 基础与 Web 平台 [进行中]
- [✅] 核心 ECS 架构与主循环
- [✅] 基础 2D 渲染 (Sprite, Batching)
- [✅] 资源异步加载
- [✅] WebAssembly 编译与运行
- [🚧] 完善 WebGPU 后端稳定性
- [📋] 文本渲染 (MSDF)

### 阶段 2: 3D 扩展与编辑器 [待开始]
- [📋] PBR 材质与光照管线
- [📋] CSM 阴影
- [📋] 3D 物理 (Rapier3D)
- [📋] 场景编辑器 (Gizmos, Hierarchy)
- [📋] C#/Python 脚本绑定

### 阶段 3: XR 集成与性能达标 [待开始]
- [📋] OpenXR 会话集成
- [📋] 立体渲染管线
- [📋] ATW 与 Foveated Rendering
- [📋] AR 平面检测
- [📋] 性能优化 (VR ≥ 90FPS)

---
