# P2阶段实施框架：生态系统扩展

**阶段**: P2 - 中优先级任务（12-18个月）
**心智负担减少**: 60%
**状态**: 📋 框架设计完成

---

## P2-1: 3D格式支持扩展（1-2个月）

### 核心功能框架

#### 1. FBX加载器
```rust
// game_engine/src/assets/fbx_loader.rs
pub struct FbxLoader;
pub struct FbxScene {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
    pub animations: Vec<Animation>,
    pub skeleton: Option<Skeleton>,
}

impl FbxLoader {
    pub fn load(path: &Path) -> Result<FbxScene, FbxError>;
    pub fn load_with_options(path: &Path, opts: &LoadOptions) -> Result<FbxScene, FbxError>;
}
```

**依赖**: `fbx-rust` (开源FBX SDK替代方案)
**工期**: 4周

#### 2. OBJ加载器
```rust
// game_engine/src/assets/obj_loader.rs
pub struct ObjLoader;

impl ObjLoader {
    pub fn load(path: &Path) -> Result<ObjModel, ObjError>;
    pub fn load_mtl(path: &Path) -> Result<Vec<Material>, ObjError>;
}
```

**工期**: 1周

#### 3. 格式转换CLI工具
```bash
# 命令行工具
game-engine-convert
├── fbx-to-gltf <input> <output>
├── obj-to-gltf <input> <output>
└── batch-convert <dir> <format>
```

**工期**: 2周

#### 4. 法线/切线自动生成
- **目标**: 从几何体自动计算法线和切线
- **算法**: 面法线平均、MikkTSpace切线
- **工期**: 1周

#### 5. UV Atlas生成
- **目标**: 自动生成UV Atlas用于光照贴图
- **算法**: Chart化 + Atlas打包
- **工期**: 2周

### 里程碑
- 支持主流3D格式（FBX/OBJ）
- 艺术家工作流显著改善

---

## P2-2: 跨平台支持扩展（2-3个月）

### 核心组件框架

#### 1. 鸿蒙系统支持
```rust
// game_engine/src/platform/harmonyos.rs
#[cfg(target_os = "harmonyos")]
pub struct HarmonyOSPlatform;

impl Platform for HarmonyOSPlatform {
    fn init() -> Result<(), PlatformError>;
    fn window_create(&self) -> Result<Window, WindowError>;
    fn input_manager(&self) -> InputManager;
}
```

**关键技术**:
- 鸿蒙窗口系统集成
- 鸿蒙输入系统适配
- 鸿蒙文件系统API
- OpenHarmony图形API（兼容Vulkan）

**工期**: 4周

#### 2. 集成显卡优化
```rust
// game_engine/src/render/integrated_gpu.rs
pub struct IntegratedGPUOptimizer {
    pub tile_based_rendering: bool,
    pub bandwidth_optimization: bool,
    pub tiler_mode: TilerMode,
}

impl IntegratedGPUOptimizer {
    pub fn optimize_for_integrated(&self, pipeline: &mut RenderPipeline);
}
```

**优化项**:
- Tile-based渲染优化
- 带宽节省（Early-Z, TBDR优化）
- 片段着色器简化

**工期**: 2周

#### 3. 移动端Tile-based优化
```rust
pub struct MobileTilerConfig {
    pub tile_size: (u32, u32),
    pub early_z_test: bool,
    pub hi_z_buffer: bool,
    pub deferred_rendering: bool,
}
```

**工期**: 2周

#### 4. ARM NEON优化
```rust
#[cfg(target_arch = "aarch64")]
pub mod neon {
    pub fn transform_vertices_simd(vertices: &mut [Vertex], matrix: &Matrix4);
    pub fn skinning_simd(vertices: &mut [Vertex], bones: &[Bone], weights: &[f32]);
}
```

**工期**: 2周

#### 5. Web端ASTC纹理压缩
```rust
pub struct AstcCompressor {
    pub block_size: AstcBlockSize,
    pub quality: AstcQuality,
}

pub enum AstcBlockSize {
    B4x4, B6x6, B8x8,
}
```

**工期**: 1周

#### 6. 游戏机支持研究
- **目标**: PS5/Switch/Xbox Series X可行性分析
- **输出**: 技术报告 + SDK集成计划
- **工期**: 4周

### 里程碑
- 鸿蒙系统支持
- 移动端性能提升50%+

---

## P2-3: 资源依赖分析工具（1个月）

### 核心组件

#### 1. 资源依赖图生成
```rust
// game_engine/src/tools/dependency_analyzer.rs
pub struct DependencyGraph {
    pub nodes: HashMap<AssetId, AssetNode>,
    pub edges: HashMap<AssetId, Vec<AssetId>>,
}

pub struct AssetNode {
    pub id: AssetId,
    pub asset_type: AssetType,
    pub path: PathBuf,
    pub size: usize,
    pub dependencies: Vec<AssetId>,
    pub dependents: Vec<AssetId>,
}

impl DependencyGraph {
    pub fn build_from_project(project: &Project) -> Self;
    pub fn find_unused(&self) -> Vec<AssetId>;
    pub fn find_circular(&self) -> Vec<AssetId>;
    pub fn visualize(&self, format: GraphFormat) -> String;
}
```

**工期**: 2周

#### 2. 未使用资源检测
```rust
pub struct UnusedAssetDetector {
    pub entry_points: Vec<AssetId>,  // 场景、预制体等入口资源
}

impl UnusedAssetDetector {
    pub fn detect(&self, graph: &DependencyGraph) -> Vec<AssetId>;
    pub fn generate_report(&self, unused: &[AssetId]) -> CleanupReport;
}
```

**工期**: 1周

#### 3. 冗余资产自动清理
```rust
pub struct AssetCleanupTool {
    pub dry_run: bool,
    pub backup_dir: Option<PathBuf>,
}

impl AssetCleanupTool {
    pub fn cleanup_unused(&self, assets: Vec<AssetId>) -> Result<CleanupStats, CleanupError>;
    pub fn compress_textures(&self, assets: Vec<AssetId>) -> Result<CompressionStats, CleanupError>;
}
```

**工期**: 1周

### 里程碑
- 资源管理优化，减少冗余30%+

---

## P2-4: DDD架构完善（2周）

### 核心改进

#### 1. 完善Repository模式
```rust
// game_engine/src/domain/repositories/mod.rs
pub trait Repository<T, ID> {
    fn find_by_id(&self, id: ID) -> Result<Option<T>, RepoError>;
    fn find_all(&self) -> Result<Vec<T>, RepoError>;
    fn save(&mut self, entity: T) -> Result<(), RepoError>;
    fn delete(&mut self, id: ID) -> Result<(), RepoError>;
}

// 具体Repository实现
pub struct MeshRepository {
    db: SqlitePool,
    cache: LruCache<AssetId, Mesh>,
}

impl Repository<Mesh, AssetId> for MeshRepository {
    // ...
}
```

**工期**: 1周

#### 2. 定义具体聚合根
```rust
// 聚合根定义
pub struct SceneAggregate {
    pub scene: Scene,
    pub entities: Vec<Entity>,
    pub components: HashMap<EntityId, Vec<Component>>,
    pub relationships: Vec<EntityRelationship>,
}

pub struct MaterialAggregate {
    pub material: Material,
    pub textures: Vec<Texture>,
    pub shaders: Vec<Shader>,
}
```

**工期**: 1周

### 里程碑
- DDD架构完善，领域边界清晰

---

## P2-5: 插件系统增强（2周）

### 核心组件

#### 1. 插件版本管理
```rust
// game_engine/src/plugin/version.rs
pub struct PluginVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

pub struct PluginManifest {
    pub name: String,
    pub version: PluginVersion,
    pub engine_version_req: String,  // semver要求
    pub dependencies: Vec<PluginDependency>,
    pub api_level: u32,
}

impl PluginManifest {
    pub fn verify_compatibility(&self, engine_version: &Version) -> bool;
}
```

**工期**: 1周

#### 2. 插件沙箱（WASI）
```rust
// game_engine/src/plugin/sandbox.rs
pub struct WasiSandbox {
    instance: wasmtime::Instance,
    store: wasmtime::Store<WasiState>,
}

impl WasiSandbox {
    pub fn new(plugin_path: &Path) -> Result<Self, SandboxError>;
    pub fn call(&mut self, func: &str, args: &[WasmValue]) -> Result<Vec<WasmValue>, SandboxError>;
    pub fn set_resource_limits(&mut self, memory: usize, cpu: Duration);
}
```

**安全特性**:
- 内存隔离
- CPU时间限制
- 文件系统沙箱
- 网络访问控制

**工期**: 1周

### 里程碑
- 插件系统更安全，版本管理完善

---

## 依赖需求

### P2新增依赖
```toml
[dependencies]
# 3D格式
fbx-rust = "0.3"
gltf = "1.0"
obj-rs = "0.4"

# 跨平台
# 鸿蒙SDK（官方，需手动集成）

# ARM优化
stdsimd = "0.1"

# ASTC压缩
astc-encoder = "0.4"

# WASM沙箱
wasmtime = "19.0"

# 依赖图分析
petgraph = "0.6"
```

---

## 成功指标

- [ ] 支持FBX格式导入
- [ ] 支持鸿蒙系统
- [ ] 资源依赖分析工具可用
- [ ] DDD架构完善
- [ ] 插件系统沙箱化

---

## 关键里程碑

| 里程碑 | 时间 | 验收标准 |
|--------|------|---------|
| M10: 3D格式支持 | M10末 | FBX/OBJ加载器可用 |
| M11: 鸿蒙支持 | M11末 | 鸿蒙平台Demo运行 |
| M12: P2完成 | M12末 | 生态系统扩展完成 |

---

**状态**: 框架设计完成
**下一步**: 实施P2-1 FBX加载器

---

**技术亮点**:

1. **FBX支持**: 使用开源`fbx-rust`避免SDK授权问题
2. **鸿蒙适配**: 第一批支持鸿蒙的游戏引擎之一
3. **ARM优化**: NEON SIMD加速移动端
4. **依赖分析**: 自动检测未使用资源
5. **WASI沙箱**: 插件系统安全隔离

**与P1对比**:

| 维度 | P1 | P2 |
|------|----|----|
| 重点 | 编辑器和工具 | 生态和平台 |
| 心智负担减少 | 80% | 60% |
| 技术深度 | 深 | 中等 |
| 影响范围 | 开发体验 | 用户生态 |

---

**风险评估**:

- **中风险**: 鸿蒙文档不完善（缓解：与华为合作）
- **低风险**: FBX格式复杂性（缓解：使用成熟库）
- **低风险**: ARM优化复杂度（缓解：社区支持）
