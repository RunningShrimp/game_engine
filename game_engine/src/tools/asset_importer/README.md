# 资源导入工具（Asset Importer）- P2-2

## 概述

图形化资源导入工具提供基于egui的友好的资源导入向导，支持拖拽导入和格式自动检测。

## 功能特性

### 1. 格式自动检测
- **支持的格式**:
  - 3D模型: GLTF, GLB, FBX, OBJ
  - 纹理: PNG, JPEG, JPG, TGA, BMP, GIF
  - 音频: WAV, MP3, OGG, FLAC
  - 字体: TTF, OTF
  - 着色器: HLSL, VERT, FRAG, WGSL, SPV

- **检测方法**:
  - 文件扩展名快速检测
  - Magic Number深度检测（PNG, JPEG等）

### 2. 资源验证
- 文件完整性检查
- 格式特定验证
- 常见问题检测:
  - 缺少法线
  - 缺少材质
  - 非幂次纹理尺寸
  - 几何体损坏
  - JSON格式错误

### 3. 自动修复
- 自动生成顶点法线
- 添加默认材质
- 调整纹理尺寸到2的幂
- 修复GLTF/OBJ常见问题

### 4. 批量导入
- 串行导入模式
- 并行导入模式（可配置并发数）
- 实时进度跟踪
- 错误处理和继续

### 5. 图形化向导
- 多步骤向导界面
- 拖拽文件导入
- 实时预览
- 导入设置配置
- 进度显示

## 模块结构

```
src/tools/asset_importer/
├── mod.rs          # 模块定义，公共API导出
├── detector.rs     # 格式检测器
├── validator.rs    # 资源验证器
├── fixer.rs        # 错误修复工具
├── importer.rs     # 资源导入器
├── batch.rs        # 批量导入
├── wizard.rs       # 导入向导UI
└── tests.rs        # 集成测试
```

## 使用示例

### 基础使用

```rust
use game_engine::tools::asset_importer::{AssetImportWizard, ImportSettings};

// 创建并显示向导
let wizard = AssetImportWizard::new();
wizard.show(&egui_ctx);
```

### 在调试UI中使用

```rust
use game_engine::debug::DebugUI;

let mut debug_ui = DebugUI::new();

// 通过菜单打开: Debug -> Tools -> Import Assets
debug_ui.show_import_wizard();
```

### 编程式导入

```rust
use game_engine::tools::asset_importer::{AssetImporter, ImportOptions};

let importer = AssetImporter::new(output_dir)
    .with_options(ImportOptions {
        generate_mipmaps: true,
        normalize_normals: true,
        compression: CompressionFormat::BC3,
        quality: 0.9,
        ..Default::default()
    });

let result = importer.import(&source_path)?;
```

### 批量导入

```rust
use game_engine::tools::asset_importer::{BatchImporter, BatchImportSettings};

let mut batch = BatchImporter::new(files, output_dir)
    .with_settings(BatchImportSettings {
        continue_on_error: true,
        parallel: true,
        max_parallel: 4,
        output_directory: output_dir,
    });

let report = batch.import_all().await?;
println!("Imported {}/{} files", report.successful_imports, report.total_files);
```

## API文档

### 主要类型

#### `AssetImportWizard`
图形化导入向导，提供完整的导入流程UI。

**方法:**
- `new()` - 创建新向导
- `show(&mut self, ctx: &egui::Context) -> WizardResult` - 显示向导UI

#### `AssetDetector`
格式检测器，自动识别文件格式。

**方法:**
- `detect_format(path: &Path) -> Result<AssetFormat, DetectorError>` - 检测格式
- `analyze_file(path: &Path) -> Result<FileAnalysis, DetectorError>` - 分析文件

#### `AssetValidator`
资源验证器，检测文件问题。

**方法:**
- `validate(path: &Path) -> ValidationResult` - 验证资源

#### `AssetFixer`
错误修复工具，自动修复常见问题。

**方法:**
- `auto_fix(path: &Path, issues: &[ValidationIssue]) -> Result<(), FixerError>` - 自动修复

#### `AssetImporter`
资源导入器，执行实际的导入操作。

**方法:**
- `new(output_directory: PathBuf) -> Self` - 创建导入器
- `with_options(self, options: ImportOptions) -> Self` - 设置选项
- `import(&self, source_path: &Path) -> Result<ImportResult, ImportError>` - 导入单个文件

#### `BatchImporter`
批量导入器，支持多文件并行导入。

**方法:**
- `new(files: Vec<PathBuf>, output_directory: PathBuf) -> Self` - 创建批量导入器
- `with_settings(self, settings: BatchImportSettings) -> Self` - 设置批量导入选项
- `import_all(&mut self) -> Result<BatchReport, ImportError>` - 导入所有文件
- `import_all_parallel(&mut self) -> Result<BatchReport, ImportError>` - 并行导入

### 数据类型

#### `AssetFormat`
资源格式枚举：
```rust
pub enum AssetFormat {
    GLTF,
    FBX,
    OBJ,
    Texture,
    Audio,
    Font,
    Shader,
    Unknown,
}
```

#### `CompressionFormat`
压缩格式：
```rust
pub enum CompressionFormat {
    None,
    BC1,  // DXT1
    BC2,  // DXT3
    BC3,  // DXT5
    BC4,
    BC5,
}
```

#### `ImportOptions`
导入选项：
```rust
pub struct ImportOptions {
    pub skip_validation: bool,
    pub generate_mipmaps: bool,
    pub normalize_normals: bool,
    pub compression: CompressionFormat,
    pub quality: f32,
}
```

## 测试

运行测试：
```bash
cargo test -p game_engine --lib asset_importer
```

测试覆盖：
- 格式检测测试
- 验证功能测试
- 修复功能测试
- 导入工作流测试
- 批量导入测试
- UI组件测试

## 集成状态

✅ **已完成:**
- 模块结构创建
- 格式检测器实现
- 资源验证器实现
- 错误修复工具实现
- 资源导入器实现
- 批量导入实现
- 图形化向导UI实现
- 调试UI集成
- 综合测试用例

✅ **编译状态:** 通过编译检查，无错误

## 使用场景

1. **游戏开发**
   - 快速导入美术资源
   - 批量处理资产
   - 自动修复常见问题

2. **工具链集成**
   - 作为资源管线的一部分
   - 自动化资源处理
   - CI/CD集成

3. **编辑器工具**
   - 内嵌到游戏编辑器
   - 提供可视化资源管理
   - 实时预览和验证

## 未来改进方向

1. **更多格式支持**
   - USD (Universal Scene Description)
   - Alembic (.abc)
   - 更多纹理格式 (WebP, AVIF)

2. **高级功能**
   - 资源压缩优化
   - LOD自动生成
   - 纹理图集生成
   - 法线贴图生成

3. **性能优化**
   - 多线程导入
   - 异步处理
   - 缓存机制

4. **UI增强**
   - 预览窗口优化
   - 撤销/重做功能
   - 导入历史记录
   - 批量操作界面

## 依赖项

- `egui` - 图形界面框架
- `image` - 图像处理
- `serde` - JSON序列化
- `tokio` - 异步运行时
- `tempfile` - 测试临时文件

## 作者

P2-2任务实现团队

## 许可证

MIT OR Apache-2.0
