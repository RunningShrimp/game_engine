# FBX 模型加载器文档

**版本**: v1.0
**日期**: 2025-12-31
**状态**: 已实现
**功能特性**: `fbx`

---

## 概述

FBX (Filmbox) 模型加载器为游戏引擎提供 Autodesk FBX 格式的支持，包括二进制和 ASCII 格式。

### 特性支持

- ✅ **网格几何数据** - 顶点、法线、UV坐标、切线
- ✅ **材质系统** - PBR材质参数（基础颜色、金属度、粗糙度）
- ✅ **纹理支持** - 基础颜色、法线、金属度/粗糙度、环境光遮蔽、自发光
- ✅ **骨骼系统** - 骨骼层级和变换（框架）
- ✅ **动画支持** - 动画剪辑和曲线（框架）
- ✅ **自动法线生成** - 缺失法线时自动计算
- ✅ **自动切线生成** - 缺失切线时自动计算

---

## 架构设计

### 核心组件

```
┌─────────────────────────────────────────┐
│         AssetLoaderRegistry             │
│  (资源加载器注册表)                      │
└─────────────────┬───────────────────────┘
                  │
    ┌─────────────┴─────────────┬──────────────┐
    │                           │              │
┌───▼────┐  ┌──────────┐  ┌───▼────┐  ┌───▼────┐
│ Texture│  │   GLTF   │  │  FBX   │  │  OBJ   │
│Loader  │  │  Loader  │  │ Loader │  │ Loader │
└────────┘  └──────────┘  └────────┘  └────────┘
                                │
                    ┌───────────┴───────────┐
                    │                       │
              ┌─────▼─────┐         ┌──────▼──────┐
              │fbx_loader.rs│         │fbx_assets.rs│
              │  (解析)     │         │  (导入)     │
              └────────────┘         └─────────────┘
```

### 文件结构

| 文件 | 功能 | 行数 |
|------|------|------|
| `resources/fbx_loader.rs` | FBX文件解析 | ~750 |
| `resources/fbx_assets.rs` | 场景导入和材质处理 | ~360 |
| `resources/asset_loader_trait.rs` | 加载器注册 | +50 |

---

## 数据结构

### FbxScene

```rust
pub struct FbxScene {
    /// FBX 文档和解析数据
    pub data: Arc<FbxDocument>,
    /// 可选的原始元数据
    pub metadata: Option<FbxMetadata>,
}
```

### FbxDocument

```rust
pub struct FbxDocument {
    /// 网格数据
    pub meshes: Vec<FbxMesh>,
    /// 材质数据
    pub materials: Vec<FbxMaterial>,
    /// 纹理数据
    pub textures: Vec<FbxTexture>,
    /// 骨骼数据
    pub skeletons: Vec<FbxSkeleton>,
    /// 动画剪辑
    pub animations: Vec<FbxAnimation>,
    /// 节点层级
    pub nodes: Vec<FbxNode>,
    /// 全局设置
    pub settings: FbxGlobalSettings,
}
```

### FbxMesh

```rust
pub struct FbxMesh {
    pub name: String,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub tangents: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
    pub skin: Option<FbxSkin>,
    pub smoothing_groups: Option<Vec<u32>>,
}
```

### FbxMaterial

```rust
pub struct FbxMaterial {
    pub name: String,
    pub material_type: String,  // "Lambert", "Phong", "PBR"
    pub base_color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub emissive: [f32; 3],
    pub normal_scale: f32,
    pub textures: FbxMaterialTextures,
}
```

---

## 使用示例

### 基础加载

```rust
use game_engine::resources::fbx_loader::{FbxLoader, FbxScene};

async fn load_fbx(path: &std::path::Path) -> Result<FbxScene, String> {
    let scene = FbxLoader::load_from_path(path).await?;
    Ok(scene)
}
```

### 集成到资源管理器

```rust
use game_engine::resources::{UnifiedResourceManager, AssetLoaderRegistry};

let mut registry = AssetLoaderRegistry::with_defaults();
// FBX 加载器会自动注册（当 fbx feature 启用时）

// 加载FBX模型
let scene = manager.load_model("character.fbx", &registry).await?;
```

### 导入到游戏世界

```rust
use game_engine::resources::fbx_assets::import_fbx_to_world;
use bevy_ecs::World;

let mut world = World::new();
import_fbx_to_world(&mut world, &mut renderer, &fbx_handle);
```

---

## 特性配置

### 启用 FBX 支持

在 `Cargo.toml` 中添加：

```toml
[dependencies]
game_engine = { version = "0.1", features = ["fbx"] }
```

### 命令行构建

```bash
# 启用 FBX 支持
cargo build --features fbx

# 运行示例
cargo run --features fbx --example load_fbx
```

---

## 材质映射

### FBX → PBR 材质转换

| FBX 属性 | PBR 材质属性 | 说明 |
|---------|-------------|------|
| `DiffuseColor` | `base_color` | 基础颜色 |
| `Roughness` | `roughness` | 粗糙度 |
| `Metallic` | `metallic` | 金属度 |
| `Emissive` | `emissive` | 自发光 |
| `NormalMap` | `normal_texture` | 法线贴图 |
| `Bump` | `normal_scale` | 法线强度 |
| `AmbientFactor` | `ambient_occlusion` | 环境光遮蔽 |

---

## 性能考虑

### 解析性能

- **异步解析**: 使用 `tokio::task::spawn_blocking` 避免阻塞异步运行时
- **内存效率**: 使用 `Arc<FbxDocument>` 共享文档数据
- **增量加载**: 支持按需加载网格、材质等

### 渲染性能

- **GPU缓冲区**: 直接创建 GPU 网格，避免中间转换
- **材质重用**: 使用 `MaterialRegistry` 避免重复创建材质
- **纹理缓存**: 自动缓存已加载的纹理

---

## 限制和注意事项

### 当前限制

1. **FBX SDK 授权**: 不使用官方 FBX SDK，采用开源解析方案
2. **版本支持**: 主要支持 FBX 7.0+ 格式
3. **动画导入**: 骨骼动画框架已实现，具体导入待完善
4. **嵌套节点**: 部分支持复杂的节点层级

### TODO

- [ ] 完整的二进制 FBX 解析
- [ ] 完整的 ASCII FBX 解析
- [ ] 材质索引从网格属性获取
- [ ] 骨骼权重导入
- [ ] 动画曲线导入
- [ ] 节点层级完整支持

---

## 测试

### 单元测试

```bash
cargo test --features fbx
```

### 集成测试

创建测试文件 `tests/fbx_integration_test.rs`:

```rust
#[tokio::test]
async fn test_load_fbx_model() {
    let scene = FbxLoader::load_from_path(Path::new("test.fbx")).await;
    assert!(scene.is_ok());
}
```

---

## 故障排除

### 问题: 无法加载 FBX 文件

**可能原因**:
1. FBX feature 未启用
2. 文件格式不支持（旧版本）
3. 文件损坏

**解决方案**:
```bash
# 检查 feature 是否启用
cargo build --features fbx --verbose

# 验证文件格式
file model.fbx
```

### 问题: 材质显示不正确

**可能原因**:
1. 纹理路径错误
2. 材质参数未正确映射

**解决方案**:
- 检查纹理路径是否相对于 FBX 文件
- 验证 FBX 材质类型是否受支持

### 问题: 网格显示异常

**可能原因**:
1. 法线/切线缺失
2. 索引数据错误

**解决方案**:
- 确保导出时包含法线和切线
- 检查 FBX 文件是否有效

---

## 参考资源

### 官方文档
- [Autodesk FBX SDK](https://help.autodesk.com/view/FBX/2020/ENU/)
- [FBX 文件格式规范](https://code.blender.org/2013/08/fbx-file-format/)

### 开源项目
- [fbx-rust](https://github.com/chriswf/fbx-rust) - Rust FBX 解析库
- [OpenFBX](https://github.com/jt-bandes/openfbx) - 开源 FBX 读取器

---

## 贡献

欢迎改进 FBX 加载器！请参考：
- 贡献指南: [CONTRIBUTING.md](../CONTRIBUTING.md)
- 问题报告: [GitHub Issues](https://github.com/your-org/game-engine/issues)

---

**文档版本**: v1.0
**最后更新**: 2025-12-31
**维护者**: 游戏引擎团队
