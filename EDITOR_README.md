# Game Engine Editor

现代化的游戏引擎编辑器Web界面，为Rust游戏引擎提供强大的可视化编辑工具。

## 功能特性

### 🎨 场景编辑器
- **3D视口**: 实时预览和编辑3D场景
- **场景层级**: 树形结构管理场景对象
- **变换工具**: 移动、旋转、缩放工具
- **属性面板**: 实时编辑对象属性

### 📦 资产管理
- **资产浏览器**: 网格/列表视图切换
- **多格式支持**: 模型、纹理、音频、视频、脚本
- **资产预览**: 实时预览资产内容
- **搜索过滤**: 快速查找资产

### 🎮 实体系统
- **ECS架构**: 基于实体组件系统
- **组件库**: 丰富的预制组件
- **实体管理**: 创建和编辑游戏实体
- **自定义组件**: 支持脚本组件

### 🔧 调试工具
- **性能监控**: FPS、帧时间、Draw Calls等指标
- **日志系统**: 分级日志查看和过滤
- **性能分析**: CPU时间分配和调用堆栈
- **系统信息**: 硬件信息和加速状态

### ⚙️ 设置面板
- **项目设置**: 项目名称、版本等配置
- **渲染设置**: 图形API、质量预设、特效开关
- **物理设置**: 物理引擎参数配置
- **编辑器设置**: 主题、字体、快捷键等

### 📚 文档系统
- **完整文档**: 涵盖所有功能的详细文档
- **教程指南**: 分步教程帮助快速上手
- **API参考**: 完整的API文档
- **搜索功能**: 快速查找文档内容

## 技术栈

### 前端
- **React 19**: 现代化的UI框架
- **TypeScript**: 类型安全的开发体验
- **Tailwind CSS 4**: 实用优先的CSS框架
- **shadcn/ui**: 高质量的UI组件库
- **Wouter**: 轻量级路由库

### 后端（Rust引擎）
- **Rust**: 高性能游戏引擎核心
- **WGPU**: 跨平台图形API
- **ECS**: 实体组件系统架构
- **多线程**: 并行计算和渲染

### 硬件加速
- **SIMD优化**: AVX/AVX2/AVX-512 (Intel/AMD)、ARM NEON
- **NPU加速**: Intel OpenVINO、AMD ROCm、华为CANN、高通SNPE等
- **超分辨率**: AMD FSR集成
- **GPU粒子**: GPU加速粒子系统

## 设计理念

### 深色主题
编辑器采用深色主题设计，紫蓝色调为主色，提供舒适的长时间使用体验：
- 背景色: `oklch(0.12 0.01 265)` - 深紫蓝色
- 主色调: 紫色渐变 (`--color-purple-*`)
- 强调色: 紫蓝色 (`oklch(0.28 0.08 280)`)

### 布局结构
- **顶部工具栏**: 全局操作按钮（保存、运行、停止等）
- **左侧导航**: 图标式导航栏，快速切换功能模块
- **主内容区**: 各功能模块的主要工作区域
- **右侧面板**: 属性编辑、预览等辅助面板

### 交互设计
- **响应式**: 适配不同屏幕尺寸
- **快捷键**: 支持常用操作的键盘快捷键
- **拖放**: 支持资产拖放到场景
- **实时预览**: 所见即所得的编辑体验

## 项目结构

```
client/
├── public/              # 静态资源
│   ├── engine-logo.png  # 引擎Logo
│   ├── hero-bg.png      # 首页背景
│   ├── 3d-scene-preview.png  # 3D场景预览
│   ├── asset-texture.png     # 资产纹理示例
│   └── particle-effect.png   # 粒子效果示例
├── src/
│   ├── components/      # 可复用组件
│   │   ├── ui/         # shadcn/ui组件
│   │   └── EditorLayout.tsx  # 编辑器布局
│   ├── pages/          # 页面组件
│   │   ├── Welcome.tsx       # 欢迎页
│   │   ├── SceneEditor.tsx   # 场景编辑器
│   │   ├── AssetBrowser.tsx  # 资产浏览器
│   │   ├── EntityManager.tsx # 实体管理器
│   │   ├── DebugTools.tsx    # 调试工具
│   │   ├── Settings.tsx      # 设置页面
│   │   └── Documentation.tsx # 文档页面
│   ├── App.tsx         # 应用根组件
│   ├── main.tsx        # 入口文件
│   └── index.css       # 全局样式
└── index.html          # HTML模板
```

## 开发指南

### 本地开发
```bash
# 安装依赖
pnpm install

# 启动开发服务器
pnpm dev

# 构建生产版本
pnpm build
```

### 代码规范
- 使用TypeScript进行类型检查
- 遵循React最佳实践
- 使用Tailwind CSS工具类
- 优先使用shadcn/ui组件

### 添加新功能
1. 在 `client/src/pages/` 创建新页面组件
2. 在 `client/src/App.tsx` 添加路由
3. 在 `client/src/components/EditorLayout.tsx` 添加导航项（如需要）
4. 使用shadcn/ui组件保持UI一致性

## 与Rust引擎集成

编辑器通过WebSocket或HTTP API与Rust引擎后端通信：

### 通信协议
- **场景数据**: JSON格式的场景序列化
- **资产加载**: 二进制资产数据传输
- **实时更新**: WebSocket推送引擎状态
- **命令执行**: HTTP POST执行编辑器命令

### 数据流
```
编辑器UI → API调用 → Rust引擎 → 更新场景 → WebSocket推送 → 编辑器更新
```

## 性能优化

### 前端优化
- **虚拟滚动**: 大列表使用虚拟滚动
- **懒加载**: 按需加载组件和资源
- **防抖节流**: 频繁操作使用防抖/节流
- **Memo优化**: 使用React.memo减少重渲染

### 后端优化
- **SIMD加速**: 向量化计算
- **多线程**: 并行处理场景更新
- **GPU加速**: 渲染和物理计算
- **NPU加速**: AI相关任务

## 浏览器支持

- Chrome/Edge 90+
- Firefox 88+
- Safari 14+

需要支持以下特性：
- ES2020
- CSS Grid & Flexbox
- WebGL 2.0 (用于3D预览)

## 许可证

本项目采用MIT许可证。

## 贡献

欢迎提交Issue和Pull Request！

## 相关链接

- [Rust游戏引擎仓库](https://github.com/your-repo/game-engine)
- [在线文档](https://docs.your-engine.com)
- [社区论坛](https://forum.your-engine.com)
