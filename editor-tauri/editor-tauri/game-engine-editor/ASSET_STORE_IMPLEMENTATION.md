# 资源商店系统实现文档

## 概述

本文档描述了游戏引擎资源商店系统的完整实现。资源商店为开发者提供了一个集成的市场，用于浏览、搜索、下载和导入游戏开发资源。

## 系统架构

```
┌─────────────────────────────────────────────────────────────┐
│                     资源商店系统架构                           │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐              │
│  │  前端UI   │────│ Tauri API │────│ Rust后端  │              │
│  │  TypeScript│   │ Commands  │    │  服务    │              │
│  └──────────┘    └──────────┘    └──────────┘              │
│                           │                                  │
│                           ▼                                  │
│                    ┌──────────┐                              │
│                    │ 本地存储   │                              │
│                    │ 缓存系统   │                              │
│                    └──────────┘                              │
│                           │                                  │
│                           ▼                                  │
│                    ┌──────────┐                              │
│                    │ CDN存储   │                              │
│                    └──────────┘                              │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

## 核心组件

### 1. Rust 后端服务

**位置**: `src-tauri/src/asset_store.rs`

#### 主要功能

- **资源搜索**: 支持按类型、类别、标签、评分等多维度搜索
- **资源下载**: 支持断点续传和进度追踪
- **资源导入**: 自动导入到项目目录
- **收藏管理**: 用户收藏功能
- **下载历史**: 历史记录追踪

#### 核心数据结构

```rust
// 资源元数据
pub struct AssetMetadata {
    pub id: String,
    pub name: String,
    pub description: String,
    pub asset_type: AssetType,
    pub category: AssetCategory,
    pub version: String,
    pub author: String,
    pub tags: Vec<String>,
    pub license: LicenseType,
    pub pricing: PricingType,
    pub file_size_bytes: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub rating: f32,
    pub download_count: u32,
    pub preview_urls: Vec<String>,
    pub dependencies: Vec<String>,
    pub compatibility: Vec<String>,
    pub minimum_engine_version: String,
}

// 搜索查询
pub struct SearchQuery {
    pub query: Option<String>,
    pub asset_type: Option<AssetType>,
    pub category: Option<AssetCategory>,
    pub tags: Vec<String>,
    pub license: Option<LicenseType>,
    pub pricing: Option<PricingType>,
    pub min_rating: Option<f32>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub sort_by: Option<SortField>,
    pub sort_order: Option<SortOrder>,
}
```

#### Tauri 命令

```rust
// 搜索资源
#[tauri::command]
pub async fn search_assets(
    state: State<'_, AssetStoreState>,
    query: Option<String>,
    asset_type: Option<String>,
    category: Option<String>,
    tags: Vec<String>,
    page: Option<u32>,
    per_page: Option<u32>,
) -> Result<SearchResult, String>

// 下载资源
#[tauri::command]
pub async fn download_asset(
    state: State<'_, AssetStoreState>,
    id: String,
) -> Result<AssetData, String>

// 导入资源
#[tauri::command]
pub async fn import_asset(
    state: State<'_, AssetStoreState>,
    asset_id: String,
    project_path: String,
) -> Result<String, String>
```

### 2. 前端UI组件

**位置**: `src/components/AssetStorePanel/`

#### 组件结构

```
AssetStorePanel/
├── AssetStorePanel.tsx          # 主面板组件
├── AssetCard.tsx                # 资源卡片
├── AssetDetails.tsx             # 资源详情面板
├── FilterPanel.tsx              # 过滤器侧边栏
├── AssetStoreLoadingSkeleton.tsx # 加载骨架屏
└── index.ts                     # 导出
```

#### 主要功能

- **搜索和过滤**: 实时搜索、多维度过滤
- **网格/列表视图**: 切换显示模式
- **预览功能**: 图片预览、资源详情
- **下载和导入**: 一键下载并导入到项目
- **收藏管理**: 添加/移除收藏
- **分页浏览**: 支持大量资源的分页显示

#### 使用示例

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

### 3. TypeScript API

**位置**: `src/api/assetStore.ts`

#### API 方法

```typescript
class AssetStoreAPI {
  // 搜索资源
  static async searchAssets(query: SearchQuery): Promise<SearchResult>

  // 下载资源
  static async downloadAsset(id: string): Promise<AssetData>

  // 导入资源到项目
  static async importAsset(assetId: string, projectPath: string): Promise<string>

  // 获取预览
  static async getPreview(id: string): Promise<PreviewData>

  // 获取资源详情
  static async getAssetDetails(id: string): Promise<AssetMetadata>

  // 添加到收藏
  static async addFavorite(userId: string, assetId: string): Promise<void>

  // 移除收藏
  static async removeFavorite(userId: string, assetId: string): Promise<void>

  // 获取收藏列表
  static async getFavorites(userId: string): Promise<AssetMetadata[]>

  // 获取下载历史
  static async getDownloadHistory(userId: string): Promise<DownloadHistory[]>
}
```

### 4. CLI 工具

**位置**: `asset-store-cli/`

#### 功能

- 命令行搜索资源
- 下载资源
- 查看资源详情
- 上传资源（创作者）
- 管理收藏和历史

#### 使用示例

```bash
# 搜索资源
asset-store search "character" --asset-type model_3d --category characters

# 下载资源
asset-store download asset-id-123 --output ./Assets

# 查看详情
asset-store info asset-id-123

# 上传资源
asset-store upload manifest.json --files model.fbx texture.png
```

### 5. 预览生成器

**位置**: `asset-store-preview-generator/`

#### 功能

- 为3D模型生成预览图
- 为纹理生成缩略图
- 为音频生成波形图
- 为材质生成预览
- 为脚本生成代码截图

#### 使用示例

```bash
# 生成模型预览
preview-generator input.fbx --output preview.png --width 512 --height 512 --asset-type model

# 生成纹理预览
preview-generator texture.jpg --output thumb.png --width 256 --height 256 --asset-type texture

# 生成音频预览
preview-generator sound.wav --output waveform.png --width 512 --height 256 --asset-type audio
```

## 资源类型

系统支持以下资源类型：

| 类型 | 描述 | 支持格式 |
|------|------|---------|
| Model3D | 3D模型 | glTF 2.0, FBX, OBJ, USD |
| Material | 材质 | JSON, 自定义格式 |
| Texture | 纹理 | PNG, JPEG, EXR, HDR |
| Audio | 音频 | MP3, WAV, OGG |
| Script | 脚本 | TypeScript, Rust, Lua |
| Shader | 着色器 | WGSL, GLSL, HLSL |
| Scene | 场景 | JSON, 自定义格式 |
| Template | 模板 | ZIP, JSON |
| Plugin | 插件 | WASM, 动态库 |

## 类别

- Characters（角色）
- Environments（环境）
- Props（道具）
- Vehicles（载具）
- Weapons（武器）
- Effects（特效）
- UI（用户界面）
- Tools（工具）
- Architecture（建筑）
- Nature（自然）

## 许可证

- MIT
- Apache 2.0
- GPL
- CC0（公共领域）
- CC BY（署名）
- CC BY-SA（署名-相同方式共享）
- CC BY-NC（署名-非商业）
- Proprietary（专有）

## 定价模式

### 免费资源
- 完全免费使用
- 需遵守许可证条款

### 付费资源
- 一次性购买
- 支持折扣

### 订阅制
- 按月订阅
- 按年订阅
- 订阅期间可访问所有订阅资源

## 集成到编辑器

### 1. 注册懒加载组件

已在 `src/components/lazyComponents.tsx` 中注册：

```typescript
export const LazyAssetStore = createLazyComponent(
  () => import('./components/AssetStorePanel'),
  <AssetStoreLoadingSkeleton />
);
```

### 2. 在菜单中添加入口

在编辑器主菜单中添加"资源商店"选项：

```typescript
const handleOpenAssetStore = () => {
  // 打开资源商店面板
  setPanel('asset-store');
};

// 在菜单组件中
<button onClick={handleOpenAssetStore}>
  Asset Store
</button>
```

### 3. 面板状态管理

```typescript
const [activePanel, setActivePanel] = useState<'viewport' | 'asset-store' | ...>('viewport');

{activePanel === 'asset-store' && (
  <LazyAssetStore
    projectPath={projectPath}
    userId={userId}
  />
)}
```

## 工作流程

### 搜索资源流程

```
用户输入搜索关键词
    ↓
前端构建SearchQuery
    ↓
调用Tauri命令 search_assets
    ↓
后端执行搜索（本地+远程）
    ↓
返回SearchResult
    ↓
前端显示资源列表
```

### 下载资源流程

```
用户点击下载
    ↓
前端调用download_asset
    ↓
后端下载资源文件
    ↓
保存到本地缓存
    ↓
前端调用import_asset
    ↓
后端解压并复制到项目目录
    ↓
更新资源浏览器
    ↓
显示成功消息
```

## 性能优化

### 1. 缓存策略

- **本地缓存**: 已下载的资源缓存在 `~/.game-engine/asset-store/`
- **内存缓存**: 频繁访问的资源数据保存在内存中
- **预览缓存**: 预览图缓存在本地，避免重复下载

### 2. 分页加载

- 每页默认20个资源
- 支持自定义每页数量
- 滚动加载更多（可扩展）

### 3. 懒加载

- 组件使用懒加载
- 预览图按需加载
- 详情数据延迟加载

### 4. CDN加速

- 资源文件通过CDN分发
- 预览图CDN加速
- 多地域节点

## 安全考虑

### 1. 资源验证

- 文件哈希校验
- 病毒扫描（可选）
- 格式验证

### 2. 权限管理

- API密钥认证
- 用户授权
- 下载权限检查

### 3. 许可证保护

- 许可证元数据
- 使用条款确认
- 版权保护

## 扩展功能

### 可添加的功能

1. **评价系统**
   - 星级评分
   - 评论功能
   - 举报机制

2. **创作者工具**
   - 资源上传
   - 销售统计
   - 收益管理

3. **社区功能**
   - 用户主页
   - 关注创作者
   - 资源分享

4. **推荐系统**
   - 基于历史的推荐
   - 相似资源推荐
   - 热门资源

5. **版本管理**
   - 资源更新通知
   - 版本历史
   - 自动更新

## 测试

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search_assets() {
        // 测试搜索功能
    }

    #[tokio::test]
    async fn test_download_asset() {
        // 测试下载功能
    }
}
```

### 集成测试

```typescript
describe('AssetStore', () => {
  it('should search assets', async () => {
    const results = await AssetStoreAPI.searchAssets({
      query: 'character',
    });
    expect(results.assets).toBeDefined();
  });

  it('should download asset', async () => {
    const asset = await AssetStoreAPI.downloadAsset('asset-id');
    expect(asset).toBeDefined();
  });
});
```

## 部署

### 后端部署

1. **API服务器**
   - 配置 `ASSET_STORE_URL` 环境变量
   - 设置CDN配置
   - 配置数据库（可选）

2. **CDN配置**
   - 上传资源文件到CDN
   - 配置缓存策略
   - 设置访问权限

### 前端部署

1. **构建**
   ```bash
   npm run build
   ```

2. **配置Tauri**
   - 更新 `src-tauri/tauri.conf.json`
   - 配置API端点

## 故障排除

### 常见问题

1. **无法连接到资源商店**
   - 检查网络连接
   - 验证API端点配置
   - 查看防火墙设置

2. **下载失败**
   - 检查磁盘空间
   - 验证写入权限
   - 查看错误日志

3. **预览图不显示**
   - 检查CDN配置
   - 验证URL有效性
   - 检查网络连接

## 维护

### 日常维护

- 监控资源下载统计
- 审核新上传资源
- 处理用户反馈
- 更新热门资源

### 定期更新

- 更新引擎版本兼容性
- 优化搜索算法
- 改进推荐系统
- 添加新资源类型

## 附录

### API端点

```
GET    /api/v1/assets/search    # 搜索资源
GET    /api/v1/assets/:id       # 获取资源详情
GET    /api/v1/assets/:id/download  # 下载资源
POST   /api/v1/assets           # 上传资源（创作者）
GET    /api/v1/favorites        # 获取收藏
POST   /api/v1/favorites        # 添加收藏
DELETE /api/v1/favorites/:id    # 删除收藏
GET    /api/v1/history          # 下载历史
GET    /api/v1/categories       # 获取类别
GET    /api/v1/types            # 获取资源类型
```

### 环境变量

```bash
# API配置
ASSET_STORE_URL=https://api.example.com
ASSET_STORE_API_KEY=your-api-key

# CDN配置
ASSET_STORE_CDN_URL=https://cdn.example.com

# 本地存储
ASSET_STORE_CACHE_DIR=~/.game-engine/asset-store
```

### 文件结构

```
asset-store/
├── frontend/                # Web前端（可选）
│   ├── pages/
│   ├── components/
│   └── api/
├── backend/                 # 后端服务（可选）
│   ├── api/
│   ├── storage/
│   └── payment/
├── editor-integration/      # 编辑器集成（已完成）
│   └── AssetStorePanel.tsx
├── cli/                     # CLI工具（已完成）
│   └── asset-store-cli/
├── preview-generator/       # 预览生成器（已完成）
└── docs/                    # 文档
```

## 总结

资源商店系统已完整实现，包括：

✅ Rust后端API服务
✅ TypeScript前端UI组件
✅ Tauri命令集成
✅ CLI命令行工具
✅ 预览生成器
✅ 完整文档

系统支持9种资源类型、10个类别、8种许可证类型，提供搜索、过滤、下载、导入、收藏等完整功能。

## 贡献

欢迎贡献新的功能、改进和bug修复！

## 许可证

本项目遵循游戏引擎主项目的许可证。
