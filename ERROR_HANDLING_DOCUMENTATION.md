# 错误处理文档

本文档详细说明了游戏引擎的错误处理架构、错误类型、严重级别、恢复策略以及使用模式。

## 目录

1. [概述](#概述)
2. [错误架构](#错误架构)
3. [错误严重级别](#错误严重级别)
4. [错误分类](#错误分类)
5. [核心错误类型](#核心错误类型)
6. [子系统错误类型](#子系统错误类型)
7. [错误恢复机制](#错误恢复机制)
8. [错误处理最佳实践](#错误处理最佳实践)
9. [常见错误场景](#常见错误场景)

## 概述

游戏引擎采用统一的错误处理架构，通过类型安全的错误类型、错误链、上下文传播和恢复机制，确保错误能够被正确捕获、处理和恢复。

### 主要特性

- **类型安全**：使用强类型枚举定义所有错误类型
- **错误链**：支持错误嵌套和上下文传播
- **严重级别**：根据错误影响程度进行分级
- **错误分类**：根据错误来源进行分类
- **恢复机制**：提供多种错误恢复策略
- **统一接口**：所有子系统使用相同的错误处理模式

## 错误架构

### 模块结构

```
error/
├── mod.rs                    # 错误模块定义和架构
├── engine_error.rs           # 核心统一错误类型
├── render_error.rs           # 渲染系统错误
├── physics_error.rs          # 物理系统错误
├── audio_error.rs            # 音频系统错误
├── resource_error.rs         # 资源管理错误
├── input_error.rs            # 输入系统错误
├── system_error.rs           # 系统级错误
├── recovery.rs               # 错误恢复机制
├── retry.rs                  # 重试策略
├── monitoring.rs             # 错误监控
├── logging.rs                # 错误日志
└── error_handler.rs          # 错误处理器
```

### Result类型别名

每个子系统都有专门的Result类型：

```rust
pub type EngineResult<T> = Result<T, EngineError>;
pub type RenderResult<T> = Result<T, RenderError>;
pub type PhysicsResult<T> = Result<T, PhysicsError>;
pub type AudioResult<T> = Result<T, AudioError>;
pub type ResourceResult<T> = Result<T, ResourceError>;
pub type InputResult<T> = Result<T, InputError>;
pub type SystemResult<T> = Result<T, SystemError>;
```

## 错误严重级别

错误严重级别用于表示错误对系统运行的影响程度，从低到高分为五个级别：

### Info（信息级别）
- **定义**：不会影响系统运行的信息性错误
- **处理方式**：记录日志，不中断流程
- **示例**：非关键资源加载警告、性能提示

### Warning（警告级别）
- **定义**：可能影响性能或用户体验但不影响核心功能的错误
- **处理方式**：记录警告日志，考虑降级处理
- **示例**：资源加载延迟、非关键功能不可用

### Error（错误级别）
- **定义**：影响部分功能但系统可继续运行的错误
- **处理方式**：记录错误日志，尝试恢复或降级
- **示例**：单个资源加载失败、部分渲染错误

### Critical（严重错误）
- **定义**：影响核心功能，需要立即处理的错误
- **处理方式**：记录严重错误，尝试紧急恢复
- **示例**：GPU设备错误、关键资源缺失

### Fatal（致命错误）
- **定义**：系统无法继续运行的错误
- **处理方式**：记录致命错误，优雅关闭系统
- **示例**：内存不足、系统初始化失败

## 错误分类

错误根据来源分为以下类别：

### Render（渲染相关）
- GPU适配器错误
- 设备创建错误
- 着色器编译错误
- 管线创建错误
- 缓冲区/纹理创建错误
- 渲染通道错误
- 帧提交错误

### Physics（物理相关）
- 刚体创建错误
- 碰撞体错误
- 约束错误
- 物理世界初始化错误
- 查询错误

### Audio（音频相关）
- 设备初始化错误
- 文件加载错误
- 解码错误
- 播放控制错误
- 音频效果错误

### Resource（资源相关）
- 资源未找到
- 加载失败
- 格式错误
- 解析错误
- 缓存错误

### Input（输入相关）
- 设备初始化错误
- 设备未找到
- 映射错误
- 绑定冲突
- 输入处理错误

### System（系统相关）
- 初始化错误
- 配置错误
- 权限错误
- 文件系统错误
- 网络错误

## 核心错误类型

### EngineError

`EngineError`是所有引擎错误的统一入口点，支持错误链、上下文信息和错误聚合。

#### 主要变体

```rust
pub enum EngineError {
    /// 子系统错误（自动转换）
    Render(#[from] RenderError),
    Physics(#[from] PhysicsError),
    Audio(#[from] AudioError),
    Resource(#[from] ResourceError),
    Input(#[from] InputError),
    System(#[from] SystemError),

    /// 通用错误
    General {
        message: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        severity: ErrorSeverity,
        location: Option<String>,
        backtrace: Option<Backtrace>,
    },

    /// 多个错误聚合
    Multiple {
        count: usize,
        errors: Vec<EngineError>,
        primary: Option<Box<EngineError>>,
    },

    /// 错误链
    Chain {
        context: String,
        source: Box<EngineError>,
        metadata: HashMap<String, String>,
    },
}
```

#### 关键方法

```rust
// 获取错误的严重级别
pub fn severity(&self) -> ErrorSeverity

// 检查错误是否可恢复
pub fn is_recoverable(&self) -> bool

// 获取错误的根本原因
pub fn root_cause(&self) -> &EngineError

// 收集所有错误链中的错误
pub fn collect_chain(&self) -> Vec<&EngineError>

// 获取错误分类
pub fn category(&self) -> ErrorCategory

// 添加上下文信息
pub fn with_context(self, context: &str) -> Self
```

## 子系统错误类型

### RenderError（渲染错误）

渲染系统错误涵盖GPU初始化、着色器编译、资源创建等所有渲染相关操作。

#### 主要错误类型

| 错误类型 | 描述 | 可恢复性 |
|---------|------|---------|
| Adapter | GPU适配器错误 | 取决于严重级别 |
| DeviceCreation | 设备创建失败 | 取决于严重级别 |
| SurfaceCreation | 表面创建失败 | 取决于严重级别 |
| ShaderCompilation | 着色器编译失败 | 不可恢复 |
| PipelineCreation | 管线创建失败 | 可恢复 |
| BufferCreation | 缓冲区创建失败 | 可恢复 |
| TextureCreation | 纹理创建失败 | 可恢复 |
| BindGroupCreation | 绑定组创建失败 | 可恢复 |
| RenderPass | 渲染通道错误 | 可恢复 |
| FrameSubmission | 帧提交失败 | 可恢复 |
| OutOfMemory | GPU内存不足 | 取决于严重级别 |
| Timeout | 操作超时 | 可恢复 |

#### 使用示例

```rust
use game_engine::error::{RenderError, RenderResult};

fn create_pipeline(device: &wgpu::Device) -> RenderResult<wgpu::RenderPipeline> {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });

    // 着色器编译错误不可恢复
    if let Err(e) = shader {
        return Err(RenderError::shader_compilation(
            "shader.wgsl",
            e.to_string(),
        ));
    }

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("pipeline"),
        layout: None,
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    // 管线创建错误可恢复
    pipeline.map_err(|e| RenderError::pipeline_creation(e.to_string()))
}
```

### PhysicsError（物理错误）

物理系统错误涵盖刚体创建、碰撞体、约束、物理世界等所有物理相关操作。

#### 主要错误类型

| 错误类型 | 描述 | 可恢复性 |
|---------|------|---------|
| RigidBodyCreation | 刚体创建失败 | 可恢复 |
| RigidBodyNotFound | 刚体未找到 | 可恢复 |
| InvalidRigidBodyParameter | 无效刚体参数 | 可恢复 |
| ColliderCreation | 碰撞体创建失败 | 可恢复 |
| ColliderNotFound | 碰撞体未找到 | 可恢复 |
| InvalidColliderParameter | 无效碰撞体参数 | 可恢复 |
| JointCreation | 约束创建失败 | 可恢复 |
| JointNotFound | 约束未找到 | 可恢复 |
| WorldNotInitialized | 物理世界未初始化 | 取决于严重级别 |
| Query | 查询错误 | 可恢复 |
| Simulation | 模拟错误 | 可恢复 |

#### 使用示例

```rust
use game_engine::error::{PhysicsError, PhysicsResult};

fn create_rigid_body(
    world: &mut PhysicsWorld,
    position: Vec3,
    mass: f32,
) -> PhysicsResult<RigidBodyHandle> {
    // 检查物理世界是否已初始化
    if !world.is_initialized() {
        return Err(PhysicsError::world_not_initialized(
            "Physics world not initialized",
        ));
    }

    // 验证质量参数
    if mass <= 0.0 {
        return Err(PhysicsError::invalid_rigid_body_parameter(
            "mass",
            mass.to_string(),
        ));
    }

    // 创建刚体
    let body = world.create_rigid_body(RigidBodyDesc::new()
        .translation(position)
        .mass(mass)
    );

    body.map_err(|e| PhysicsError::rigid_body_creation(e.to_string()))
}
```

### AudioError（音频错误）

音频系统错误涵盖设备初始化、文件加载、解码、播放控制等所有音频相关操作。

#### 主要错误类型

| 错误类型 | 描述 | 可恢复性 |
|---------|------|---------|
| DeviceInitialization | 设备初始化失败 | 可恢复 |
| DeviceNotFound | 设备未找到 | 可恢复 |
| FileLoading | 文件加载失败 | 可恢复 |
| Decoding | 解码失败 | 可恢复 |
| UnsupportedFormat | 格式不支持 | 可恢复 |
| Playback | 播放控制错误 | 可恢复 |
| Volume | 音量控制错误 | 可恢复 |
| Mixer | 混音器错误 | 可恢复 |
| Effect | 音频效果错误 | 可恢复 |
| Streaming | 流式加载错误 | 可恢复 |

#### 使用示例

```rust
use game_engine::error::{AudioError, AudioResult};

fn load_audio_file(path: &str) -> AudioResult<AudioBuffer> {
    // 检查文件是否存在
    if !std::path::Path::new(path).exists() {
        return Err(AudioError::file_loading(
            path,
            "File not found".to_string(),
        ));
    }

    // 读取文件
    let data = std::fs::read(path)
        .map_err(|e| AudioError::file_loading(path, e.to_string()))?;

    // 解码音频
    let decoder = AudioDecoder::new(&data)
        .map_err(|e| AudioError::decoding(path, e.to_string()))?;

    // 检查格式支持
    if !decoder.is_format_supported() {
        return Err(AudioError::unsupported_format(
            path,
            decoder.format().to_string(),
        ));
    }

    // 解码音频数据
    let buffer = decoder.decode()
        .map_err(|e| AudioError::decoding(path, e.to_string()))?;

    Ok(buffer)
}
```

### ResourceError（资源错误）

资源管理错误涵盖资源发现、加载、解析、缓存、依赖等所有资源相关操作。

#### 主要错误类型

| 错误类型 | 描述 | 可恢复性 |
|---------|------|---------|
| NotFound | 资源未找到 | 可恢复 |
| LoadFailed | 资源加载失败 | 可恢复 |
| InvalidFormat | 无效格式 | 不可恢复 |
| Parsing | 解析错误 | 不可恢复 |
| Dependency | 依赖错误 | 可恢复 |
| Cache | 缓存错误 | 可恢复 |
| OutOfMemory | 内存不足 | 取决于严重级别 |
| Version | 版本不兼容 | 可恢复 |
| Permission | 权限错误 | 可恢复 |
| Download | 下载失败 | 可恢复 |

#### 使用示例

```rust
use game_engine::error::{ResourceError, ResourceResult};

fn load_texture(path: &str) -> ResourceResult<Texture> {
    // 检查资源是否存在
    if !resource_manager.exists(path) {
        return Err(ResourceError::not_found(path));
    }

    // 尝试从缓存加载
    if let Some(cached) = resource_manager.get_from_cache(path) {
        return Ok(cached);
    }

    // 加载资源
    let data = std::fs::read(path)
        .map_err(|e| ResourceError::load_failed(path, e.to_string()))?;

    // 解析纹理格式
    let format = detect_texture_format(&data)
        .ok_or_else(|| ResourceError::invalid_format(
            path,
            "Unknown texture format".to_string(),
        ))?;

    // 解析纹理数据
    let texture = parse_texture(&data, format)
        .map_err(|e| ResourceError::parsing(path, e.to_string()))?;

    // 缓存纹理
    resource_manager.cache(path, texture.clone());

    Ok(texture)
}
```

### InputError（输入错误）

输入系统错误涵盖设备初始化、输入映射、绑定冲突、输入处理等所有输入相关操作。

#### 主要错误类型

| 错误类型 | 描述 | 可恢复性 |
|---------|------|---------|
| DeviceInitialization | 设备初始化失败 | 可恢复 |
| DeviceNotFound | 设备未找到 | 可恢复 |
| DeviceDisconnected | 设备断开连接 | 可恢复 |
| Mapping | 输入映射错误 | 可恢复 |
| BindingConflict | 绑定冲突 | 可恢复 |
| InvalidBinding | 无效绑定 | 可恢复 |
| EventProcessing | 事件处理错误 | 可恢复 |
| Calibration | 校准错误 | 可恢复 |
| Driver | 驱动错误 | 可恢复 |
| Configuration | 配置错误 | 可恢复 |

#### 使用示例

```rust
use game_engine::error::{InputError, InputResult};

fn map_input_action(
    input_manager: &mut InputManager,
    action: &str,
    binding: InputBinding,
) -> InputResult<()> {
    // 检查设备是否存在
    if !input_manager.has_device(binding.device_id()) {
        return Err(InputError::device_not_found(
            binding.device_id().to_string(),
        ));
    }

    // 检查绑定是否有效
    if !binding.is_valid() {
        return Err(InputError::invalid_binding(
            binding.to_string(),
            "Invalid binding configuration".to_string(),
        ));
    }

    // 检查是否有绑定冲突
    if let Some(existing) = input_manager.get_action_for_binding(&binding) {
        return Err(InputError::binding_conflict(
            action.to_string(),
            existing.to_string(),
        ));
    }

    // 添加输入映射
    input_manager.add_mapping(action, binding);

    Ok(())
}
```

### SystemError（系统错误）

系统级错误涵盖系统初始化、配置、权限、文件系统、网络等所有系统相关操作。

#### 主要错误类型

| 错误类型 | 描述 | 可恢复性 |
|---------|------|---------|
| Initialization | 系统初始化失败 | 取决于严重级别 |
| Shutdown | 系统关闭失败 | 取决于严重级别 |
| Configuration | 配置错误 | 可恢复 |
| Permission | 权限错误 | 可恢复 |
| FileSystem | 文件系统错误 | 可恢复 |
| Network | 网络错误 | 可恢复 |
| OutOfMemory | 内存不足 | 取决于严重级别 |
| Security | 安全错误 | 取决于严重级别 |
| Thread | 线程错误 | 可恢复 |
| Process | 进程错误 | 可恢复 |

#### 使用示例

```rust
use game_engine::error::{SystemError, SystemResult};

fn initialize_system(config: &SystemConfig) -> SystemResult<System> {
    // 检查权限
    if !has_required_permissions() {
        return Err(SystemError::permission(
            "System initialization",
            "Insufficient permissions".to_string(),
        ));
    }

    // 检查内存
    if !has_sufficient_memory(config.required_memory()) {
        return Err(SystemError::out_of_memory(
            "System initialization",
            "Insufficient memory".to_string(),
        ));
    }

    // 初始化系统
    let system = System::new(config)
        .map_err(|e| SystemError::initialization(
            "System",
            e.to_string(),
        ))?;

    Ok(system)
}
```

## 错误恢复机制

### 恢复策略

引擎提供多种错误恢复策略，根据错误类型和严重级别自动选择合适的恢复方式。

#### Retry（重试）

适用于临时性错误，如网络超时、资源加载延迟等。

```rust
RecoveryStrategy::Retry {
    max_attempts: 3,
    base_delay_ms: 100,
    backoff_multiplier: 2.0,
    max_delay_ms: 5000,
}
```

#### UseDefault（使用默认值）

适用于非关键错误，可以使用默认值继续运行。

```rust
RecoveryStrategy::UseDefault {
    default_description: "Using default texture",
    log_warning: true,
}
```

#### Skip（跳过）

适用于可以跳过的操作，如非关键资源加载。

```rust
RecoveryStrategy::Skip {
    reason: "Non-critical resource",
    log_warning: true,
}
```

#### LogAndContinue（记录并继续）

适用于已知的非致命错误，记录日志后继续运行。

```rust
RecoveryStrategy::LogAndContinue {
    log_level: ErrorSeverity::Warning,
    context: "Texture loading failed, using fallback",
}
```

#### GracefulDegradation（优雅降级）

适用于可以降级处理的错误，如GPU内存不足时降低渲染质量。

```rust
RecoveryStrategy::GracefulDegradation {
    degradation_level: 1,
    description: "GPU memory low, reducing render quality",
    fallback: "Low quality rendering",
}
```

#### FailFast（快速失败）

适用于无法恢复的错误，立即终止操作。

```rust
RecoveryStrategy::FailFast {
    reason: "Critical system error",
    log_error: true,
}
```

### 错误恢复器

错误恢复器实现了`ErrorRecovery` trait，为不同类型的错误提供特定的恢复逻辑。

```rust
pub trait ErrorRecovery {
    /// 尝试恢复错误
    fn recover(&self, error: &EngineError, context: &RecoveryContext) -> RecoveryResult<()>;

    /// 检查是否可以处理该错误
    fn can_handle(&self, error: &EngineError) -> bool;

    /// 获取恢复器名称
    fn name(&self) -> &str;
}
```

### 使用示例

```rust
use game_engine::error::{EngineError, ErrorRecovery, RecoveryContext, RecoveryResult};

struct CustomRecovery;

impl ErrorRecovery for CustomRecovery {
    fn recover(&self, error: &EngineError, context: &RecoveryContext) -> RecoveryResult<()> {
        match error {
            EngineError::Render(render_err) => {
                // 处理渲染错误
                match render_err {
                    RenderError::OutOfMemory { .. } => {
                        // GPU内存不足，降级渲染质量
                        RecoveryResult::Degraded(
                            (),
                            RecoveryInfo {
                                strategy: RecoveryStrategy::GracefulDegradation {
                                    degradation_level: 1,
                                    description: "GPU memory low".to_string(),
                                    fallback: "Low quality rendering".to_string(),
                                },
                                description: "Render quality degradation".to_string(),
                                duration: context.start_time.elapsed(),
                                metadata: HashMap::new(),
                            },
                        )
                    }
                    _ => RecoveryResult::CannotRecover,
                }
            }
            _ => RecoveryResult::CannotRecover,
        }
    }

    fn can_handle(&self, error: &EngineError) -> bool {
        matches!(error, EngineError::Render(_))
    }

    fn name(&self) -> &str {
        "CustomRecovery"
    }
}
```

## 错误处理最佳实践

### 1. 使用类型安全的错误

始终使用定义好的错误类型，而不是使用字符串或通用错误。

```rust
// 好的做法
fn load_texture(path: &str) -> Result<Texture, ResourceError> {
    // ...
}

// 不好的做法
fn load_texture(path: &str) -> Result<Texture, String> {
    // ...
}
```

### 2. 提供有用的上下文信息

在创建错误时，提供足够的上下文信息以便调试。

```rust
// 好的做法
Err(RenderError::shader_compilation(
    "shader.wgsl",
    "Line 42: undefined variable 'position'",
))

// 不好的做法
Err(RenderError::shader_compilation(
    "shader.wgsl",
    "compilation failed",
))
```

### 3. 正确设置错误严重级别

根据错误的实际影响设置正确的严重级别。

```rust
// 好的做法
Err(RenderError::out_of_memory(
    "GPU",
    "Failed to allocate 512MB texture",
    ErrorSeverity::Critical,
))

// 不好的做法
Err(RenderError::out_of_memory(
    "GPU",
    "Failed to allocate 512MB texture",
    ErrorSeverity::Warning, // 严重级别设置错误
))
```

### 4. 使用错误链传播上下文

使用`?`操作符和`with_context`方法传播错误并添加上下文。

```rust
// 好的做法
fn load_scene(path: &str) -> Result<Scene, EngineError> {
    let data = std::fs::read(path)
        .map_err(|e| ResourceError::load_failed(path, e.to_string()))?;

    let scene = parse_scene(&data)
        .map_err(|e| ResourceError::parsing(path, e.to_string()))?;

    Ok(scene)
}

// 或者使用错误链
fn load_scene(path: &str) -> Result<Scene, EngineError> {
    let data = std::fs::read(path)
        .map_err(|e| EngineError::General {
            message: format!("Failed to read scene file: {}", path),
            source: Some(Box::new(e)),
            severity: ErrorSeverity::Error,
            location: Some("load_scene".to_string()),
            backtrace: None,
        })?;

    let scene = parse_scene(&data)
        .map_err(|e| EngineError::General {
            message: format!("Failed to parse scene: {}", path),
            source: Some(Box::new(e)),
            severity: ErrorSeverity::Error,
            location: Some("load_scene".to_string()),
            backtrace: None,
        })?;

    Ok(scene)
}
```

### 5. 实现适当的错误恢复

根据错误类型实现适当的恢复策略。

```rust
// 好的做法
fn render_frame(&mut self) -> RenderResult<()> {
    match self.render_pass.begin() {
        Ok(pass) => {
            // 正常渲染
            self.draw_scene(pass)?;
            Ok(())
        }
        Err(e) if e.is_recoverable() => {
            // 尝试恢复
            self.recover_from_error(&e)?;
            Ok(())
        }
        Err(e) => {
            // 不可恢复的错误
            Err(e)
        }
    }
}

// 不好的做法
fn render_frame(&mut self) -> RenderResult<()> {
    self.render_pass.begin()?;
    self.draw_scene(self.render_pass)?;
    Ok(())
}
```

### 6. 记录错误日志

始终记录错误日志，包括错误上下文和堆栈跟踪。

```rust
// 好的做法
fn load_resource(path: &str) -> ResourceResult<Resource> {
    match load_resource_internal(path) {
        Ok(resource) => Ok(resource),
        Err(e) => {
            error!(
                "Failed to load resource: {} - Error: {:?}",
                path, e
            );
            Err(e)
        }
    }
}

// 不好的做法
fn load_resource(path: &str) -> ResourceResult<Resource> {
    load_resource_internal(path)
}
```

### 7. 使用适当的Result类型

使用子系统特定的Result类型，而不是通用的Result。

```rust
// 好的做法
fn create_pipeline(device: &wgpu::Device) -> RenderResult<wgpu::RenderPipeline> {
    // ...
}

// 不好的做法
fn create_pipeline(device: &wgpu::Device) -> Result<wgpu::RenderPipeline, Box<dyn std::error::Error>> {
    // ...
}
```

## 常见错误场景

### 场景1：GPU内存不足

```rust
fn create_texture(device: &wgpu::Device, size: u32) -> RenderResult<wgpu::Texture> {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("texture"),
        size: wgpu::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    match texture {
        Ok(texture) => Ok(texture),
        Err(e) if e.to_string().contains("out of memory") => {
            Err(RenderError::out_of_memory(
                "GPU",
                e.to_string(),
                ErrorSeverity::Critical,
            ))
        }
        Err(e) => Err(RenderError::texture_creation(e.to_string())),
    }
}
```

### 场景2：资源加载失败

```rust
fn load_texture(path: &str) -> ResourceResult<Texture> {
    // 尝试从缓存加载
    if let Some(cached) = cache.get(path) {
        return Ok(cached);
    }

    // 尝试从磁盘加载
    let data = std::fs::read(path)
        .map_err(|e| ResourceError::load_failed(path, e.to_string()))?;

    // 解析纹理
    let texture = parse_texture(&data)
        .map_err(|e| ResourceError::parsing(path, e.to_string()))?;

    // 缓存纹理
    cache.insert(path, texture.clone());

    Ok(texture)
}
```

### 场景3：输入设备断开

```rust
fn process_input(&mut self) -> InputResult<()> {
    // 检查设备是否仍然连接
    if !self.input_manager.is_device_connected(self.active_device) {
        // 尝试重新连接
        if let Some(new_device) = self.input_manager.find_similar_device(self.active_device) {
            self.active_device = new_device;
            warn!("Input device reconnected: {:?}", new_device);
        } else {
            return Err(InputError::device_disconnected(
                self.active_device.to_string(),
            ));
        }
    }

    // 处理输入
    self.input_manager.process_events()
}
```

### 场景4：物理世界未初始化

```rust
fn step_physics(&mut self, dt: f32) -> PhysicsResult<()> {
    // 检查物理世界是否已初始化
    if !self.world.is_initialized() {
        // 尝试初始化物理世界
        self.world.initialize()
            .map_err(|e| PhysicsError::world_not_initialized(e.to_string()))?;
    }

    // 步进物理模拟
    self.world.step(dt)
        .map_err(|e| PhysicsError::simulation(e.to_string()))
}
```

### 场景5：音频设备初始化失败

```rust
fn initialize_audio() -> AudioResult<AudioContext> {
    // 尝试初始化默认设备
    match AudioContext::new() {
        Ok(context) => Ok(context),
        Err(e) => {
            // 尝试查找其他可用设备
            let devices = AudioDevice::enumerate();
            if devices.is_empty() {
                return Err(AudioError::device_initialization(
                    "No audio devices available".to_string(),
                    ErrorSeverity::Warning,
                ));
            }

            // 尝试使用第一个可用设备
            AudioContext::with_device(&devices[0])
                .map_err(|e| AudioError::device_initialization(
                    format!("Failed to initialize device {}: {}", devices[0].name(), e),
                    ErrorSeverity::Warning,
                ))
        }
    }
}
```

## 总结

游戏引擎的错误处理架构提供了：

1. **类型安全**：使用强类型枚举定义所有错误类型
2. **错误链**：支持错误嵌套和上下文传播
3. **严重级别**：根据错误影响程度进行分级
4. **错误分类**：根据错误来源进行分类
5. **恢复机制**：提供多种错误恢复策略
6. **统一接口**：所有子系统使用相同的错误处理模式

通过遵循本文档中的最佳实践，可以确保错误能够被正确捕获、处理和恢复，提高系统的稳定性和可靠性。
