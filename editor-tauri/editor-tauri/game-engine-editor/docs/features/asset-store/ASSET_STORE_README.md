# 资源商店系统

完整的游戏引擎资源商店实现，提供资源浏览、搜索、下载、导入和管理功能。

## 功能特性

### 核心功能

✅ **资源搜索**
- 全文搜索
- 多维度过滤（类型、类别、标签、评分）
- 高级排序
- 分页浏览

✅ **资源管理**
- 一键下载
- 自动导入项目
- 本地缓存
- 版本追踪

✅ **用户功能**
- 收藏系统
- 下载历史
- 个性化推荐（可扩展）

✅ **预览系统**
- 图片预览
- 资源详情
- 文件列表
- 自动预览生成

✅ **多平台支持**
- Web界面（编辑器内嵌）
- 命令行工具
- REST API

## 支持的资源类型

| 类型 | 格式 | 说明 |
|------|------|------|
| 3D模型 | glTF, FBX, OBJ, USD | 角色、环境、道具等 |
| 材质 | JSON, 自定义格式 | PBR材质、着色器材质 |
| 纹理 | PNG, JPEG, EXR, HDR | 漫反射、法线、粗糙度等 |
| 音频 | MP3, WAV, OGG | 音效、背景音乐 |
| 脚本 | TypeScript, Rust, Lua | 游戏逻辑脚本 |
| 着色器 | WGSL, GLSL, HLSL | 自定义着色器 |
| 场景 | JSON, 自定义格式 | 完整游戏场景 |
| 模板 | ZIP, JSON | 项目模板 |
| 插件 | WASM, 动态库 | 扩展插件 |

## 系统架构

```
┌─────────────────────────────────────────────────────────┐
│                   资源商店系统                           │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌──────────────┐         ┌──────────────┐            │
│  │   前端UI     │         │   CLI工具    │            │
│  │  TypeScript  │         │    Rust      │            │
│  └──────┬───────┘         └──────┬───────┘            │
│         │                        │                      │
│         └──────────┬─────────────┘                      │
│                    ▼                                    │
│         ┌────────────────────┐                         │
│         │   Tauri Commands   │                         │
│         └─────────┬──────────┘                         │
│                   ▼                                    │
│         ┌────────────────────┐                         │
│         │  Rust Backend API  │                         │
│         └─────────┬──────────┘                         │
│                   ▼                                    │
│         ┌────────────────────┐                         │
│         │   Storage Layer    │                         │
│         │  - Local Cache     │                         │
│         │  - CDN Storage     │                         │
│         └────────────────────┘                         │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

## 项目结构

```
.
├── src-tauri/src/
│   └── asset_store.rs              # Rust后端实现
├── src/
│   ├── types/
│   │   └── assetStore.ts           # TypeScript类型定义
│   ├── api/
│   │   └── assetStore.ts           # API客户端
│   └── components/
│       └── AssetStorePanel/        # UI组件
│           ├── AssetStorePanel.tsx
│           ├── AssetCard.tsx
│           ├── AssetDetails.tsx
│           ├── FilterPanel.tsx
│           └── index.ts
├── asset-store-cli/                # CLI工具
│   ├── src/main.rs
│   └── Cargo.toml
├── asset-store-preview-generator/  # 预览生成器
│   ├── src/main.rs
│   └── Cargo.toml
├── ASSET_STORE_IMPLEMENTATION.md   # 完整文档
├── ASSET_STORE_QUICKSTART.md       # 快速入门
└── ASSET_STORE_README.md           # 本文件
```

## 快速开始

### 安装依赖

```bash
# Rust依赖（已在src-tauri/Cargo.toml中）
cargo build

# CLI工具
cd asset-store-cli
cargo build --release

# 预览生成器
cd asset-store-preview-generator
cargo build --release
```

### 在编辑器中使用

```tsx
import { AssetStorePanel } from '@/components/AssetStorePanel';

function MyEditor() {
  return (
    <AssetStorePanel
      projectPath="/path/to/project"
      userId="user-123"
    />
  );
}
```

### CLI使用

```bash
# 搜索资源
asset-store search "character" --asset-type model_3d

# 下载资源
asset-store download asset-id-123 --output ./Assets

# 查看详情
asset-store info asset-id-123

# 查看收藏
asset-store favorites --user-id user@example.com

# 查看历史
asset-store history
```

### 生成预览

```bash
# 为模型生成预览
preview-generator model.fbx --output preview.png \
  --width 512 --height 512 --asset-type model

# 为纹理生成缩略图
preview-generator texture.png --output thumb.png \
  --width 256 --height 256 --asset-type texture

# 为音频生成波形图
preview-generator sound.wav --output wave.png \
  --width 512 --height 256 --asset-type audio
```

## API端点

### 搜索
```typescript
POST /api/v1/assets/search
{
  "query": "character",
  "asset_type": "model_3d",
  "category": "characters",
  "tags": ["low-poly", "animated"],
  "page": 1,
  "per_page": 20,
  "sort_by": "rating",
  "sort_order": "desc"
}
```

### 下载
```typescript
GET /api/v1/assets/:id/download
```

### 获取详情
```typescript
GET /api/v1/assets/:id
```

### 收藏
```typescript
POST /api/v1/favorites
{
  "user_id": "user-123",
  "asset_id": "asset-456"
}

DELETE /api/v1/favorites/:id
GET /api/v1/favorites?user_id=user-123
```

## 核心API

### TypeScript

```typescript
// 搜索资源
const results = await AssetStoreAPI.searchAssets({
  query: 'character',
  assetType: AssetType.Model3D,
  category: AssetCategory.Characters,
  tags: ['low-poly'],
  page: 1,
  perPage: 20
});

// 下载资源
const asset = await AssetStoreAPI.downloadAsset('asset-id-123');

// 导入到项目
const path = await AssetStoreAPI.importAsset(
  'asset-id-123',
  '/path/to/project'
);

// 添加收藏
await AssetStoreAPI.addFavorite('user-123', 'asset-456');

// 获取收藏列表
const favorites = await AssetStoreAPI.getFavorites('user-123');
```

### Rust

```rust
// 搜索
let result = client.search_assets(SearchQuery {
    query: Some("character".to_string()),
    asset_type: Some(AssetType::Model3D),
    ..Default::default()
}).await?;

// 下载
let asset = client.download_asset("asset-id-123").await?;

// 导入
let path = client.import_asset("asset-id-123", project_path).await?;
```

## 配置

### 环境变量

```bash
# API配置
export ASSET_STORE_URL="https://api.example.com"
export ASSET_STORE_API_KEY="your-api-key"

# CDN配置
export ASSET_STORE_CDN_URL="https://cdn.example.com"

# 本地存储
export ASSET_STORE_CACHE_DIR="$HOME/.game-engine/asset-store"
```

### Tauri配置

```json
// src-tauri/tauri.conf.json
{
  "tauri": {
    "allowlist": {
      "all": false,
      "shell": {
        "all": false,
        "open": true
      },
      "fs": {
        "all": false,
        "readFile": true,
        "writeFile": true,
        "scope": ["$HOME/.game-engine/*"]
      }
    }
  }
}
```

## 测试

### Rust测试

```bash
cd src-tauri
cargo test asset_store
```

### TypeScript测试

```bash
npm test -- AssetStore
```

### 集成测试

```bash
# 启动测试服务器
cargo run --bin test-server

# 运行集成测试
npm run test:integration
```

## 性能优化

### 缓存策略

- **内存缓存**：频繁访问的资源
- **本地缓存**：已下载资源
- **预览缓存**：预览图片

### 分页加载

- 默认每页20个资源
- 按需加载更多
- 预加载相邻页

### CDN加速

- 全球CDN节点
- 智能路由
- 边缘缓存

## 安全

### 认证

- API密钥认证
- 用户登录
- 令牌刷新

### 权限

- 资源访问控制
- 下载权限验证
- 许可证检查

### 数据验证

- 文件哈希校验
- 格式验证
- 大小限制

## 扩展

### 添加新的资源类型

1. 定义新的枚举值：
```rust
pub enum AssetType {
    // 现有类型...
    NewType,  // 新类型
}
```

2. 添加类型处理逻辑
3. 更新UI组件
4. 更新文档

### 自定义预览生成器

```rust
impl PreviewGenerator {
    fn generate_custom_preview(&self, path: &PathBuf) -> Result<RgbImage> {
        // 自定义生成逻辑
    }
}
```

### 添加新的过滤器

```typescript
interface SearchQuery {
    // 现有过滤条件...
    customFilter?: string;  // 新过滤条件
}
```

## 文档

- **[完整实现文档](ASSET_STORE_IMPLEMENTATION.md)** - 详细的系统设计和实现
- **[快速入门指南](ASSET_STORE_QUICKSTART.md)** - 用户使用指南
- **[API参考](docs/API_REFERENCE.md)** - API文档（待创建）
- **[创作者指南](docs/CREATOR_GUIDE.md)** - 上传资源指南（待创建）

## 贡献

欢迎贡献！请查看：

1. [CONTRIBUTING.md](CONTRIBUTING.md)
2. [开发指南](docs/DEVELOPMENT.md)
3. [代码规范](docs/CODE_STYLE.md)

## 许可证

MIT License - 详见 [LICENSE](LICENSE) 文件

## 作者

游戏引擎开发团队

## 致谢

感谢所有贡献者和资源创作者！

## 更新日志

### v0.1.0 (2024-01-15)
- ✨ 初始版本
- ✅ 基础搜索功能
- ✅ 下载和导入
- ✅ 收藏系统
- ✅ CLI工具
- ✅ 预览生成器

### 计划功能

- [ ] 评价和评论系统
- [ ] 创作者工具
- [ ] 推荐系统
- [ ] 批量操作
- [ ] 离线模式
- [ ] 云同步
- [ ] 社区功能

## 联系方式

- **主页**: https://game-engine.example.com
- **文档**: https://docs.game-engine.example.com
- **论坛**: https://forum.game-engine.example.com
- **GitHub**: https://github.com/example/game-engine
- **Email**: support@example.com

## 常见问题

**Q: 资源商店是免费的吗？**

A: 资源商店本身免费，但有些资源可能需要付费购买。

**Q: 可以离线使用吗？**

A: 搜索和下载需要网络，但已下载的资源可离线使用。

**Q: 如何上传自己的资源？**

A: 请使用CLI工具的 `upload` 命令，参考创作者指南。

**Q: 支持退款吗？**

A: 请查看具体的退款政策。

---

**祝您使用愉快！** 🎮✨
