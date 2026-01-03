# 3D资源导入器快速入门

本指南帮助您快速开始使用3D资源导入器。

## 5分钟快速开始

### 1. 在Rust代码中使用

```rust
use importers::{import_model, GltfImporter};

// 方法1：自动检测格式
let model = import_model("assets/models/character.gltf")?;
println!("Loaded {} meshes", model.meshes.len());

// 方法2：使用特定导入器
let importer = GltfImporter::new();
let model = importer.import("assets/models/scene.glb")?;
```

### 2. 在Tauri前端调用

```typescript
import { invoke } from '@tauri-apps/api/tauri';

// 导入模型
async function onFileSelected(path: string) {
  const result = await invoke('import_3d_model', {
    filePath: path
  });

  if (result.success) {
    console.log('导入成功！');
    console.log(`- 网格数: ${result.mesh_count}`);
    console.log(`- 材质数: ${result.material_count}`);
    console.log(`- 节点数: ${result.node_count}`);
    console.log(`- 动画数: ${result.animation_count}`);
  }
}
```

## 支持的文件格式

| 格式 | 扩展名 | 推荐度 | 功能支持 |
|------|--------|--------|----------|
| glTF | .gltf | ★★★★★ | 完整PBR、动画、蒙皮 |
| glTF Binary | .glb | ★★★★★ | 完整PBR、动画、蒙皮 |
| FBX | .fbx | ★★★★☆ | 网格、材质、动画 |
| OBJ | .obj | ★★★☆☆ | 网格、材质（静态） |

## 导入器选项

### glTF导入器

```rust
use importers::GltfImporter;

// 基本使用
let importer = GltfImporter::new();

// 禁用外部资源加载（GLB内嵌）
let importer = GltfImporter::new()
    .with_external_loading(false);

let model = importer.import("model.glb")?;
```

### FBX导入器

```rust
use importers::FbxImporter;

let importer = FbxImporter::new()
    .with_external_loading(true)    // 加载外部资源
    .ignore_geometry(false)          // 导入几何体
    .ignore_animation(false);        // 导入动画

let model = importer.import("model.fbx")?;
```

### OBJ导入器

```rust
use importers::ObjImporter;

let importer = ObjImporter::new()
    .with_materials(true)      // 加载MTL材质
    .with_uv_flip(true)        // 翻转UV坐标
    .with_normals_flip(false); // 不翻转法线

let model = importer.import("model.obj")?;
```

## 导入的数据结构

```rust
pub struct ModelData {
    pub meshes: Vec<Mesh>,         // 网格数据
    pub materials: Vec<Material>,   // 材质数据
    pub nodes: Vec<Node>,          // 节点层级
    pub animations: Vec<Animation>, // 动画数据
    pub skins: Vec<Skin>,         // 蒙皮数据
}
```

## 常见问题

### Q: 如何处理导入错误？

```rust
use importers::{ImportError, import_model};

match import_model("model.gltf") {
    Ok(model) => {
        // 处理成功导入的数据
    },
    Err(ImportError::UnsupportedFormat(ext)) => {
        eprintln!("不支持的格式: {}", ext);
    },
    Err(ImportError::ParseError(msg)) => {
        eprintln!("解析错误: {}", msg);
    },
    Err(e) => {
        eprintln!("导入失败: {}", e);
    }
}
```

### Q: 如何遍历网格和顶点？

```rust
let model = import_model("model.gltf")?;

for mesh in &model.meshes {
    println!("Mesh: {:?}", mesh.name);

    for primitive in &mesh.primitives {
        println!("  Vertices: {}", primitive.vertices.len());
        println!("  Indices: {}", primitive.indices.len());

        // 访问顶点数据
        for vertex in &primitive.vertices {
            println!("    Position: {:?}", vertex.position);
            println!("    Normal: {:?}", vertex.normal);
            println!("    UV: {:?}", vertex.uv);
        }
    }
}
```

### Q: 如何获取材质信息？

```rust
let model = import_model("model.gltf")?;

for (i, material) in model.materials.iter().enumerate() {
    println!("Material {}: {:?}", i, material.name);
    println!("  Base Color: {:?}", material.pbr.base_color_factor);
    println!("  Metallic: {}", material.pbr.metallic_factor);
    println!("  Roughness: {}", material.pbr.roughness_factor);
}
```

### Q: OBJ模型的UV坐标不正确怎么办？

```rust
// OBJ文件通常需要翻转UV坐标的Y轴
let importer = ObjImporter::new()
    .with_uv_flip(true);  // 默认就是true

let model = importer.import("model.obj")?;
```

### Q: 如何处理大型模型？

```rust
use importers::GltfImporter;

// 对于大型模型，考虑：
// 1. 使用GLB格式（更紧凑）
// 2. 禁用外部资源加载（更快）
let importer = GltfImporter::new()
    .with_external_loading(false);

// 3. 分块处理
let model = importer.import("large_model.glb")?;

// 4. 按需加载纹理（延迟加载）
```

## 最佳实践

### 1. 资源导出设置

**Blender导出glTF：**
```
- Format: glTF Binary (.glb)
- Include: Selected Objects
- Mesh: Apply Modifiers, Tangents, Normals
- Apply: Modifiers
- Geometry: Compression - Draco
- Mesh: + Tangents, + Normals
```

### 2. 文件组织

```
assets/
├── models/
│   ├── characters/
│   │   ├── hero.gltf
│   │   └── hero.bin
│   ├── environments/
│   │   ├── forest.glb
│   │   └── city.glb
│   └── props/
│       ├── crate.fbx
│       └── barrel.obj
```

### 3. 错误处理模板

```rust
use importers::{import_model, ImportError};

pub fn load_asset(path: &str) -> Result<ModelData, String> {
    import_model(path)
        .map_err(|e| format!("Failed to load {}: {}", path, e))
        .and_then(|model| {
            if model.meshes.is_empty() {
                Err("Model contains no meshes".to_string())
            } else {
                Ok(model)
            }
        })
}
```

## 完整示例

### 示例1：简单的模型查看器

```rust
use importers::import_model;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <model-path>", args[0]);
        std::process::exit(1);
    }

    let model_path = &args[1];
    let model = import_model(model_path)?;

    println!("Model loaded successfully!");
    println!("Path: {}", model_path);
    println!("Meshes: {}", model.meshes.len());
    println!("Materials: {}", model.materials.len());
    println!("Nodes: {}", model.nodes.len());
    println!("Animations: {}", model.animations.len());

    Ok(())
}
```

### 示例2：转换工具

```rust
use importers::{GltfImporter, ObjImporter};

fn convert_obj_to_gltf(obj_path: &str, gltf_path: &str) -> Result<(), String> {
    // 导入OBJ
    let obj_importer = ObjImporter::new();
    let model = obj_importer.import(obj_path)
        .map_err(|e| format!("OBJ import failed: {}", e))?;

    // 导出为glTF（需要导出器实现）
    // save_gltf(&model, gltf_path)?;

    println!("Converted {} to {}", obj_path, gltf_path);
    Ok(())
}
```

### 示例3：Tauri前端集成

```typescript
// Vue组件示例
<script setup lang="ts">
import { ref } from 'vue';
import { open } from '@tauri-apps/api/dialog';
import { invoke } from '@tauri-apps/api/tauri';

const importStatus = ref({
  success: false,
  message: '',
  meshCount: 0,
  materialCount: 0
});

async function selectAndImport() {
  const selected = await open({
    multiple: false,
    filters: [{
      name: '3D Models',
      extensions: ['gltf', 'glb', 'fbx', 'obj']
    }]
  });

  if (selected && typeof selected === 'string') {
    const result = await invoke('import_3d_model', {
      filePath: selected
    });

    importStatus.value = result;
  }
}
</script>
```

## 下一步

- 阅读完整的[导入器指南](./IMPORTERS_GUIDE.md)
- 查看[API文档](./api_reference.md)
- 浏览[示例项目](../examples/)

## 获取帮助

如果遇到问题：
1. 检查文件格式是否受支持
2. 确认文件没有损坏
3. 查看错误消息的详细信息
4. 参考官方文档
