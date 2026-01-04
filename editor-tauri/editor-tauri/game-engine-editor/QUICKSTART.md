# 快速开始指南 (Quick Start Guide)

欢迎使用游戏引擎编辑器！本指南将帮助您快速上手并开始使用本编辑器。

---

## 📋 目录

- [系统要求](#系统要求)
- [安装](#安装)
- [快速启动](#快速启动)
- [基本概念](#基本概念)
- [常用功能](#常用功能)
- [下一步](#下一步)
- [获取帮助](#获取帮助)

---

## 系统要求

### 最低要求
- **操作系统**: Windows 10+, macOS 10.15+, 或 Linux
- **内存**: 8 GB RAM
- **存储**: 500 MB 可用空间
- **浏览器**: 最新版本的 Chrome, Firefox, Safari, 或 Edge

### 推荐配置
- **操作系统**: Windows 11, macOS 13+, 或 Linux (最新版本)
- **内存**: 16 GB RAM 或更高
- **存储**: 2 GB 可用空间或更多
- **显卡**: 支持WebGPU的现代显卡
- **处理器**: 多核处理器（4核或更多）

---

## 安装

### 方式一：从源代码构建

#### 1. 克隆仓库

```bash
git clone https://github.com/your-org/game-engine-editor.git
cd game-engine-editor
```

#### 2. 安装依赖

```bash
# 安装 Node.js 依赖
npm install

# 或使用 pnpm
pnpm install

# 或使用 yarn
yarn install
```

#### 3. 构建项目

```bash
# 开发模式运行
npm run dev

# 生产构建
npm run build
```

### 方式二：下载预编译版本

1. 访问 [Releases](https://github.com/your-org/game-engine-editor/releases) 页面
2. 下载适合您操作系统的最新版本
3. 解压并运行应用程序

---

## 快速启动

### 启动开发服务器

```bash
npm run dev
```

这将启动：
- **前端开发服务器**: http://localhost:1420
- **后端API服务器**: http://localhost:3000
- **热重载**: 自动检测文件变化并重新加载

### 构建生产版本

```bash
npm run build
```

构建产物将输出到 `dist/` 目录。

---

## 基本概念

### 编辑器界面

编辑器采用现代化的布局设计，主要包含以下区域：

```
┌─────────────────────────────────────────────────────────┐
│  菜单栏 (Menu Bar)                                        │
├──────────┬──────────────────────────────┬───────────────┤
│          │                              │               │
│  资源    │     场景视图 (Scene View)     │   属性检查器  │
│  浏览器  │                              │               │
│          │                              │               │
├──────────┴──────────────────────────────┴───────────────┤
│  时间轴 (Timeline) / 动画编辑器                            │
├─────────────────────────────────────────────────────────┤
│  状态栏 (Status Bar) / 控制台                            │
└─────────────────────────────────────────────────────────┘
```

### 核心组件

#### 1. **资源浏览器 (Asset Browser)**
浏览和管理项目中的所有资源文件（模型、材质、纹理等）。

**快速入门**: [资产浏览器快速开始](docs/user/ASSET_BROWSER_QUICK_START.md)

#### 2. **场景视图 (Scene View)**
可视化编辑游戏场景，包括实体放置、变换和编辑。

**快速入门**: [场景编辑指南](docs/features/entity-tree/)

#### 3. **属性检查器 (Property Inspector)**
查看和编辑选中实体或组件的属性。

**快速入门**: [属性检查器指南](docs/features/property-inspector/)

#### 4. **时间轴 (Timeline)**
创建和编辑动画序列。

**快速入门**: [动画系统快速开始](docs/user/ANIMATION_QUICKSTART.md)

#### 5. **实体树 (Entity Tree)**
管理场景中的实体层级关系。

**快速入门**: [虚拟实体树快速开始](docs/user/VIRTUAL_ENTITY_TREE_QUICKSTART.md)

---

## 常用功能

### 创建新项目

1. 点击 **文件 → 新建项目**
2. 选择项目模板
3. 设置项目名称和位置
4. 点击 **创建**

### 添加实体

1. 在实体树中右键点击
2. 选择 **创建实体**
3. 选择实体类型
4. 配置实体属性

### 导入资源

1. 在资源浏览器中点击 **导入**
2. 选择要导入的文件
3. 配置导入设置
4. 点击 **导入**

**支持的格式**:
- 3D模型: `.gltf`, `.glb`, `.fbx`, `.obj`
- 纹理: `.png`, `.jpg`, `.webp`, `.ktx2`
- 音频: `.mp3`, `.wav`, `.ogg`
- 材质: `.mat` (自定义格式)

### 创建动画

1. 选择要动画化的实体
2. 在时间轴中点击 **添加动画轨道**
3. 移动时间指针到关键帧位置
4. 修改实体属性
5. 点击 **添加关键帧**

**详细指南**: [动画系统指南](docs/user/ANIMATION_SYSTEM_GUIDE.md)

### 性能监控

1. 点击 **窗口 → 性能仪表板**
2. 查看实时性能指标
3. 分析性能瓶颈
4. 应用优化建议

**详细指南**: [性能仪表板用户指南](docs/user/PERFORMANCE_DASHBOARD_USER_GUIDE.md)

---

## 下一步

### 学习资源

- **[动画系统](docs/user/ANIMATION_SYSTEM_GUIDE.md)** - 深入了解动画功能
- **[插件系统](docs/user/PLUGIN_SYSTEM_QUICK_START.md)** - 开发自定义插件
- **[基准测试](docs/user/BENCHMARK_QUICK_REF.md)** - 性能测试和优化
- **[可访问性](docs/user/ACCESSIBILITY_QUICKSTART.md)** - 无障碍功能使用

### 开发者资源

- **[贡献指南](docs/developer/CONTRIBUTING.md)** - 如何贡献代码
- **[组件架构](docs/developer/COMPONENT_ARCHITECTURE.md)** - 理解系统架构
- **[Storybook](docs/developer/STORYBOOK.md)** - 组件开发文档

### 高级功能

- **[Nanite系统](docs/features/nanite/NANITE_GUIDE.md)** - 高保真渲染
- **[全局光照](docs/api/GI_GUIDE.md)** - 实时全局光照
- **[WebGPU渲染](docs/features/rendering/WEBGPU_3D_RENDERING_IMPLEMENTATION.md)** - 现代图形API

---

## 获取帮助

### 文档资源

- **[文档索引](DOCS_INDEX.md)** - 完整文档目录
- **[README](README.md)** - 项目概述
- **[待办事项](todolist.md)** - 已知问题和计划

### 社区支持

- **GitHub Issues**: 报告问题和请求功能
- **Discussions**: 讨论和交流
- **Wiki**: 社区维护的知识库

### 常见问题

<details>
<summary><b>Q: 如何升级到新版本？</b></summary>

A: 查看[升级指南](docs/developer/UPGRADE.md)了解详细的升级步骤。
</details>

<details>
<summary><b>Q: 编辑器运行缓慢怎么办？</b></summary>

A: 查看[性能优化最佳实践](docs/features/performance/PERFORMANCE_OPTIMIZATION_BEST_PRACTICES.md)获取优化建议。
</details>

<details>
<summary><b>Q: 如何开发自定义插件？</b></summary>

A: 从[插件系统快速开始](docs/user/PLUGIN_SYSTEM_QUICK_START.md)开始学习插件开发。
</details>

<details>
<summary><b>Q: 支持哪些3D模型格式？</b></summary>

A: 支持 `.gltf`, `.glb`, `.fbx`, `.obj` 格式。推荐使用 glTF 2.0 格式以获得最佳兼容性。
</details>

---

## 键盘快捷键

### 常用快捷键

| 操作 | Windows/Linux | macOS |
|------|--------------|-------|
| 新建项目 | `Ctrl+N` | `Cmd+N` |
| 打开项目 | `Ctrl+O` | `Cmd+O` |
| 保存 | `Ctrl+S` | `Cmd+S` |
| 撤销 | `Ctrl+Z` | `Cmd+Z` |
| 重做 | `Ctrl+Shift+Z` | `Cmd+Shift+Z` |
| 删除 | `Delete` | `Delete` |
| 复制 | `Ctrl+C` | `Cmd+C` |
| 粘贴 | `Ctrl+V` | `Cmd+V` |
| 全选 | `Ctrl+A` | `Cmd+A` |

### 视图快捷键

| 操作 | 快捷键 |
|------|--------|
| 切换资源浏览器 | `Ctrl+1` / `Cmd+1` |
| 切换实体树 | `Ctrl+2` / `Cmd+2` |
| 切换属性检查器 | `Ctrl+3` / `Cmd+3` |
| 切换时间轴 | `Ctrl+4` / `Cmd+4` |
| 切换性能仪表板 | `Ctrl+5` / `Cmd+5` |
| 全屏模式 | `F11` |

**完整快捷键列表**: [快捷键参考](docs/SHORTCUTS_REFERENCE.md)

---

## 反馈和贡献

我们欢迎您的反馈和贡献！

- 🐛 **报告问题**: [GitHub Issues](https://github.com/your-org/game-engine-editor/issues)
- 💡 **功能请求**: [Feature Requests](https://github.com/your-org/game-engine-editor/discussions)
- 🔧 **贡献代码**: [贡献指南](docs/developer/CONTRIBUTING.md)

---

## 许可证

本项目采用 MIT 许可证。详见 [LICENSE](LICENSE) 文件。

---

**祝您使用愉快！** 🎮

---

**最后更新**: 2026-01-04
