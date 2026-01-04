# 资源商店快速入门指南

## 简介

资源商店是一个集成的市场系统，让您可以轻松浏览、搜索和下载游戏开发资源。本文档将帮助您快速上手使用资源商店的各项功能。

## 目录

1. [基本使用](#基本使用)
2. [搜索资源](#搜索资源)
3. [下载和导入](#下载和导入)
4. [收藏管理](#收藏管理)
5. [命令行工具](#命令行工具)
6. [常见问题](#常见问题)

## 基本使用

### 打开资源商店

在编辑器中：

1. 点击菜单栏的"窗口" → "资源商店"
2. 或者使用快捷键 `Ctrl+Shift+A`（Windows/Linux）或 `Cmd+Shift+A`（Mac）

### 界面概览

```
┌──────────────────────────────────────────────────┐
│  🔍 [搜索框...]    [过滤器] [视图] [排序]       │
├──────────────────────────────────────────────────┤
│  Active filters: [×] [×] [清除所有]              │
│  Showing 20 of 1,234 assets                      │
├──────────────────────────────────────────────────┤
│                                                  │
│  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐          │
│  │ 预览 │ │ 预览 │ │ 预览 │ │ 预览 │          │
│  │ 图片 │ │ 图片 │ │ 图片 │ │ 图片 │          │
│  └──────┘ └──────┘ └──────┘ └──────┘          │
│  资源名称   资源名称   资源名称   资源名称        │
│  ★4.5      ★4.8      ★4.2      ★4.7            │
│  Free      $29       $15       Free            │
│                                                  │
│  [上一页] 第1页/共62页 [下一页]                  │
└──────────────────────────────────────────────────┘
```

## 搜索资源

### 基本搜索

1. 在搜索框中输入关键词
2. 按回车或点击搜索按钮
3. 浏览搜索结果

**示例**：
- `character` - 搜索角色相关资源
- `low poly` - 搜索低多边形风格
- `sci-fi weapon` - 搜索科幻武器

### 高级过滤

点击"过滤器"按钮打开过滤面板：

#### 按类型过滤
- 3D模型
- 材质
- 纹理
- 音频
- 脚本
- 着色器
- 场景
- 模板
- 插件

#### 按类别过滤
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

#### 按标签过滤
点击热门标签或输入自定义标签：

```
low-poly, pbr, animated, sci-fi, medieval,
modern, fantasy, realistic, stylized, hand-painted
```

#### 排序选项
- 日期创建
- 日期更新
- 名称
- 评分
- 下载次数

#### 排序顺序
- 升序
- 降序

### 搜索示例

#### 搜索低多边形角色

```bash
# 界面操作
1. 输入搜索词: "character"
2. 类型选择: "3D模型"
3. 类别选择: "Characters"
4. 标签添加: "low-poly"
5. 排序: "评分" 降序
```

#### 搜索科幻武器

```bash
1. 输入搜索词: "weapon"
2. 类型选择: "3D模型"
3. 类别选择: "Weapons"
4. 标签添加: "sci-fi"
```

#### 搜索PBR材质

```bash
1. 输入搜索词: "pbr"
2. 类型选择: "材质"
3. 排序: "评分" 降序
```

## 下载和导入

### 查看资源详情

1. 点击任意资源卡片
2. 右侧滑出详情面板
3. 查看：
   - 完整描述
   - 预览图/视频
   - 文件列表
   - 许可证信息
   - 依赖项
   - 兼容性
   - 用户评价

### 下载资源

#### 方式一：从详情页面下载

1. 点击资源卡片打开详情
2. 点击"Download"按钮
3. 选择导入位置（默认为项目的Assets目录）
4. 等待下载完成

#### 方式二：快速下载

1. 将鼠标悬停在资源卡片上
2. 点击下载图标（⬇️）
3. 自动下载到默认位置

#### 方式三：批量下载（收藏夹）

```typescript
// 下载所有收藏的资源
const favorites = await AssetStoreAPI.getFavorites(userId);
for (const asset of favorites) {
  await AssetStoreAPI.importAsset(asset.id, projectPath);
}
```

### 导入到项目

下载的资源会自动导入到项目结构中：

```
MyGameProject/
└── Assets/
    ├── Models/
    │   ├── character.fbx
    │   └── weapon.obj
    ├── Materials/
    │   └── pbr_material.json
    ├── Textures/
    │   ├── albedo.png
    │   └── normal.png
    └── Audio/
        └── bgm.mp3
```

### 处理依赖

如果资源有依赖项，系统会提示您：

```
⚠️ This asset requires:
  - AdvancedToonShader v2.0
  - RPGCharacterFramework v1.5

[Download all] [Download only this asset]
```

## 收藏管理

### 添加收藏

#### 方式一：从列表

1. 找到想收藏的资源
2. 点击心形图标（♡）
3. 图标变为实心（❤️）表示已收藏

#### 方式二：从详情页面

1. 打开资源详情
2. 点击心形按钮
3. 添加到收藏列表

### 查看收藏

1. 点击过滤器面板的"我的收藏"标签
2. 浏览所有收藏的资源
3. 支持搜索和过滤

### 取消收藏

- 再次点击心形图标（❤️）
- 或在详情页面点击心形按钮

### 同步收藏

收藏可以跨设备同步（需要登录）：

```bash
# CLI 查看收藏
asset-store favorites --user-id your@email.com

# 输出
❤️ Favorites for user: your@email.com

1. Low Poly Character Pack
   ID: asset-123
   Rating: ★ 4.8

2. Sci-Fi Weapons Set
   ID: asset-456
   Rating: ★ 4.6
```

## 命令行工具

### 安装CLI工具

```bash
# 从源码构建
cd asset-store-cli
cargo build --release

# 二进制文件位置
target/release/asset-store
```

### 基本命令

#### 搜索资源

```bash
# 基本搜索
asset-store search "character"

# 高级搜索
asset-store search "weapon" \
  --asset-type model_3d \
  --category weapons \
  --tags sci-fi animated \
  --per-page 50

# 只搜索免费资源
asset-store search "environment" --pricing free
```

#### 下载资源

```bash
# 下载到当前目录
asset-store download asset-id-123

# 指定输出目录
asset-store download asset-id-123 \
  --output ./MyGame/Assets/Models

# 下载并解压
asset-store download asset-id-123 --extract
```

#### 查看资源详情

```bash
asset-store info asset-id-123

# 输出
📦 Asset Information

Name: Sci-Fi Character Pack
ID: asset-id-123
Version: 2.1.0
Author: John Doe

Description:
  A complete sci-fi character pack with
  5 unique characters, 20 weapons, and
  full animation sets.

Type: model_3d
Category: Characters
License: MIT
Size: 125 MB
Rating: ⭐ 4.8
Downloads: 12,345

Tags:
  - sci-fi
  - animated
  - low-poly

Files:
  - character_01.fbx (2.5 MB)
  - character_02.fbx (2.4 MB)
  - texture_atlas.png (8.2 MB)
  - ...
```

#### 查看收藏

```bash
# 使用默认用户ID
asset-store favorites

# 指定用户ID
asset-store favorites --user-id user@example.com
```

#### 查看下载历史

```bash
asset-store history

# 输出
📜 Download history for user: default

1. Sci-Fi Character Pack v2.1.0
   Downloaded: 2024-01-15 10:30:00
   Path: ./Assets/Models

2. Fantasy Terrain Set v1.8.0
   Downloaded: 2024-01-14 15:22:00
   Path: ./Assets/Environments
```

### 批处理脚本

#### 批量下载收藏

```bash
#!/bin/bash
# download_favorites.sh

USER_ID="user@example.com"
OUTPUT_DIR="./MyGame/Assets"

# 获取所有收藏
asset-store favorites --user-id $USER_ID | \
  grep "ID:" | \
  awk '{print $2}' | \
  while read asset_id; do
    echo "Downloading $asset_id..."
    asset-store download "$asset_id" --output "$OUTPUT_DIR"
  done
```

#### 批量更新资源

```bash
#!/bin/bash
# update_assets.sh

# 从历史记录获取所有资源
asset-store history | \
  grep "v" | \
  awk '{for(i=1;i<=NF;i++) if($i ~ /v[0-9]/) print $i}' | \
  while read version; do
    echo "Checking for updates: $version"
    # 检查并下载新版本
  done
```

## 常见问题

### Q: 下载的资源在哪里？

**A**:
- 默认位置：`项目目录/Assets/资源类型/`
- 可在下载时选择自定义位置
- 可在资源浏览器中查看

### Q: 可以离线使用资源商店吗？

**A**:
- 需要网络连接搜索和下载
- 已下载的资源可离线使用
- 系统会缓存浏览历史（有限期）

### Q: 如何更新已下载的资源？

**A**:
1. 在下载历史中找到资源
2. 查看是否有新版本
3. 重新下载新版本
4. 替换旧文件

### Q: 下载失败怎么办？

**A**:
1. 检查网络连接
2. 查看磁盘空间
3. 检查写入权限
4. 查看错误日志：
   ```bash
   # 查看日志
   tail -f ~/.game-engine/logs/asset-store.log
   ```

### Q: 可以下载到其他项目吗？

**A**:
可以！下载时指定输出目录：
```bash
asset-store download asset-id-123 --output /path/to/other/project/Assets
```

### Q: 如何上传自己的资源？

**A**:
目前上传功能仅通过CLI可用：
```bash
asset-store upload manifest.json --files model.fbx texture.png
```

完整指南请参考 [创作者文档](CREATOR_GUIDE.md)

### Q: 免费资源和付费资源有什么区别？

**A**:
- **免费资源**：完全免费使用，需遵守许可证条款
- **付费资源**：一次性购买，永久使用
- **订阅资源**：订阅期间可使用，取消后无法访问新下载

### Q: 许可证是什么意思？

**A**:
常见许可证：
- **MIT**：最宽松，可商用、修改、分发
- **CC0**：公共领域，无限制
- **CC BY**：需署名
- **Proprietary**：专有，需遵守特定条款

## 最佳实践

### 1. 资源组织

```
MyGame/
├── Assets/
│   ├── Characters/
│   │   ├── Player/
│   │   └── NPCs/
│   ├── Environment/
│   │   ├── Buildings/
│   │   └── Nature/
│   └── UI/
│       ├── Menus/
│       └── HUD/
└── Projects/
    └── MyGame.game
```

### 2. 版本管理

使用Git LFS跟踪大文件：
```bash
git lfs track "*.fbx"
git lfs track "*.png"
git add .gitattributes
```

### 3. 文档记录

创建 `Assets.md` 记录资源来源：
```markdown
# Assets

## Characters
- Player Character - asset-123 - MIT License
- NPC Pack - asset-456 - CC BY 4.0

## Environment
- City Buildings - asset-789 - Proprietary
```

### 4. 定期备份

```bash
# 备份资源商店缓存
cp -r ~/.game-engine/asset-store ~/backup/asset-store-$(date +%Y%m%d)
```

## 键盘快捷键

| 快捷键 | 功能 |
|--------|------|
| `Ctrl/Cmd + F` | 聚焦搜索框 |
| `Ctrl/Cmd + Shift + F` | 打开过滤器 |
| `Ctrl/Cmd + D` | 下载选中的资源 |
| `Ctrl/Cmd + L` | 添加/移除收藏 |
| `Esc` | 关闭详情面板 |
| `Ctrl/Cmd + R` | 刷新搜索结果 |
| `←` / `→` | 上一页/下一页 |

## 获取帮助

- **文档**：[完整文档](ASSET_STORE_IMPLEMENTATION.md)
- **论坛**：[社区论坛](https://forum.example.com)
- **支持**：support@example.com
- **Bug报告**：[GitHub Issues](https://github.com/example/game-engine/issues)

## 下一步

- 阅读完整API文档
- 了解创作者工具
- 加入社区讨论
- 分享您的资源

祝您使用愉快！🎮
