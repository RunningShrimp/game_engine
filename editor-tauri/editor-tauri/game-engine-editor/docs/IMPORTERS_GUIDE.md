# 3D资源导入器使用指南

本项目实现了完整的3D模型导入系统，支持glTF 2.0、FBX和OBJ三种主流格式。

## 支持的格式

### 1. glTF 2.0 (.gltf, .glb) - 推荐

**优先级：最高**

glTF是现代3D传输的标准格式，具有以下优势：
- JSON描述 + 二进制数据（或纯二进制GLB）
- 完整的PBR材质支持
- 支持骨骼动画和蒙皮
- 支持多个场景和相机
- 文件体积小，加载速度快

**支持的功能：**
- 网格数据（顶点、索引、法线、UV）
- PBR材质（基础颜色、金属度、粗糙度）
- 法线贴图、发光贴图
- 节点层级和变换
- 骨骼和蒙皮
- 动画（平移、旋转、缩放）

**使用示例：**
```rust
use importers::GltfImporter;

let importer = GltfImporter::new();
let model_data = importer.import("path/to/model.gltf")?;
```

### 2. FBX (.fbx)

**优先级：高**

FBX是Autodesk开发的格式，广泛应用于3D软件。

**支持的功能：**
- 网格和材质导入
- 节点层级
- 坐标系转换（FBX -> OpenGL）
- 动画支持（基础）

**使用示例：**
```rust
use importers::FbxImporter;

let importer = FbxImporter::new()
    .with_external_loading(true)
    .ignore_geometry(false);

let model_data = importer.import("path/to/model.fbx")?;
```

### 3. OBJ (.obj)

**优先级：中**

OBJ是最简单的3D模型格式之一，适合静态网格。

**支持的功能：**
- 网格数据（顶点、法线、UV）
- MTL材质文件
- 多个对象组

**使用示例：**
```rust
use importers::ObjImporter;

let importer = ObjImporter::new()
    .with_materials(true)
    .with_uv_flip(true);

let model_data = importer.import("path/to/model.obj")?;
```

## 数据结构

### ModelData

导入的模型数据包含以下内容：

```rust
pub struct ModelData {
    pub meshes: Vec<Mesh>,           // 网格列表
    pub materials: Vec<Material>,     // 材质列表
    pub nodes: Vec<Node>,             // 节点层级
    pub animations: Vec<Animation>,   // 动画列表
    pub skins: Vec<Skin>,            // 皮肤/蒙皮数据
}
```

### Mesh - 网格

```rust
pub struct Mesh {
    pub name: Option<String>,         // 网格名称
    pub primitives: Vec<Primitive>,   // 图元列表
}
```

### Primitive - 图元

```rust
pub struct Primitive {
    pub vertices: Vec<Vertex>,        // 顶点数据
    pub indices: Vec<u32>,            // 索引数据
    pub material_index: Option<usize>, // 材质索引
    pub mode: PrimitiveMode,          // 拓扑类型
}
```

### Vertex - 顶点

```rust
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],    // 位置 (x, y, z)
    pub normal: [f32; 3],      // 法线 (nx, ny, nz)
    pub uv: [f32; 2],          // UV坐标 (u, v)
    pub tangent: [f32; 4],     // 切线 (tx, ty, tz, tw)
    pub color: [f32; 4],       // 颜色 (r, g, b, a)
    pub joints: [u16; 4],      // 关节索引（用于蒙皮）
    pub weights: [f32; 4],     // 关节权重
}
```

顶点结构兼容`bytemuck`，可以直接用于GPU缓冲区。

### Material - 材质

```rust
pub struct Material {
    pub name: Option<String>,
    pub pbr: PbrMaterial,              // PBR材质参数
    pub normal_texture: Option<TextureInfo>,
    pub emissive_texture: Option<TextureInfo>,
    pub emissive_factor: [f32; 3],
    pub alpha_mode: AlphaMode,
    pub alpha_cutoff: f32,
    pub double_sided: bool,
}
```

## Tauri集成

导入器已集成到Tauri后端，可以通过前端调用：

### 前端调用示例

```typescript
import { invoke } from '@tauri-apps/api/tauri';

async function importModel(filePath: string) {
  const result = await invoke('import_3d_model', {
    filePath: filePath
  });

  if (result.success) {
    console.log(`Imported ${result.mesh_count} meshes`);
    console.log(`Loaded ${result.material_count} materials`);
    console.log(`Found ${result.node_count} nodes`);
    console.log(`Detected ${result.animation_count} animations`);
  } else {
    console.error(`Import failed: ${result.message}`);
  }

  return result;
}
```

### 返回数据结构

```typescript
interface ModelInfo {
  file_path: string;
  mesh_count: number;
  material_count: number;
  node_count: number;
  animation_count: number;
  success: boolean;
  message: string;
}
```

## 通用导入函数

使用文件扩展名自动选择导入器：

```rust
use importers::import_model;

// 自动检测格式并导入
let model_data = import_model("model.gltf")?;  // 使用glTF导入器
let model_data = import_model("model.fbx")?;   // 使用FBX导入器
let model_data = import_model("model.obj")?;   // 使用OBJ导入器
```

## 错误处理

所有导入操作返回`ImportResult<T>`：

```rust
pub type ImportResult<T> = Result<T, ImportError>;

pub enum ImportError {
    UnsupportedFormat(String),
    Io(io::Error),
    ParseError(String),
    InvalidData(String),
    MissingData(String),
    InvalidVersion(String),
    AssetNotFound(String),
    NotSupported(String),
}
```

## 性能优化建议

1. **大文件处理**
   - 对于大型模型，考虑使用GLB格式（二进制glTF）
   - 启用文件流式加载（导入器支持）

2. **内存管理**
   - 导入后及时清理不需要的数据
   - 复用相同的材质和纹理

3. **缓存**
   - 对常用模型建立缓存
   - 考虑使用内存映射文件

## 最佳实践

### 1. 格式选择

- **首选glTF 2.0**：现代、高效、功能完整
- **FBX**：从Maya/3ds Max导出时使用
- **OBJ**：简单静态模型或与其他工具兼容

### 2. 资源准备

**导出glTF时的建议：**
- 使用Draco压缩（减小文件体积）
- 合并材质槽（减少draw call）
- 优化网格（删除重复顶点）
- 生成法线和切线

**导出FBX时的建议：**
- 设置正确的单位（厘米或米）
- 向上轴选择Y-up（glTF标准）或Z-up（FBX标准）
- 烘焙变换

**导出OBJ时的建议：**
- 包含法线（-n参数）
- 包含UV坐标
- 确保MTL文件在同一目录

### 3. 坐标系

不同格式使用不同的坐标系：

| 格式 | 向上轴 | 坐标系 | 转换 |
|------|--------|--------|------|
| glTF | Y+ | 右手 | 标准 |
| FBX | Y+ | 右手 | 自动转换 |
| OBJ | Y+ | 右手 | 标准 |

导入器会自动处理坐标系转换。

## 测试

导入器包含单元测试：

```bash
cd src-tauri
cargo test importers
```

## 限制和已知问题

### glTF导入器
- 外部buffer和image需要正确设置路径
- GLB的blob数据提取需要额外处理

### FBX导入器
- FBX格式复杂，某些高级特性可能不支持
- 坐标转换可能需要调整

### OBJ导入器
- 不支持平滑组
- 材质支持有限（仅基本PBR参数）

## 扩展导入器

要添加新的格式支持，实现`Importer` trait：

```rust
pub trait Importer {
    fn import<P: AsRef<Path>>(&self, path: P) -> ImportResult<ModelData>;
}

// 示例：添加STL导入器
pub struct StlImporter;

impl Importer for StlImporter {
    fn import<P: AsRef<Path>>(&self, path: P) -> ImportResult<ModelData> {
        // 实现STL解析
    }
}
```

然后在`get_importer_for_path`函数中添加新的扩展名支持。

## 参考资源

- [glTF 2.0规范](https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html)
- [FBX SDK文档](https://help.autodesk.com/view/FBX/2020/ENU/)
- [OBJ格式规范](https://en.wikipedia.org/wiki/Wavefront_.obj_file)
- [glTF最佳实践](https://www.khronos.org/gltf/)

## 示例项目

完整的导入示例请参见：
- `examples/model_viewer/` - 模型查看器
- `examples/asset_pipeline/` - 资源处理管道

## 许可证

导入器模块遵循项目整体许可证。
