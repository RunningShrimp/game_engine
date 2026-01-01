# Game Engine Performance Dashboard

实时性能分析和可视化仪表板，用于游戏引擎性能监控和优化。

## 功能特性

### 实时监控
- ✅ FPS（帧率）监控
- ✅ 帧时间分析
- ✅ CPU使用率
- ✅ 内存占用
- ✅ GPU使用率
- ✅ Draw calls统计
- ✅ 三角形数量

### 数据可视化
- ✅ 实时折线图
- ✅ 历史趋势图
- ✅ 性能对比
- ✅ 热力图
- ✅ 性能报告

### 告警系统
- ✅ FPS过低告警
- ✅ 内存泄漏检测
- ✅ CPU/GPU瓶颈识别
- ✅ 自定义告警阈值

### 优化建议
- ✅ 自动瓶颈检测
- ✅ 优化建议生成
- ✅ 性能回归检测
- ✅ 最佳实践推荐

## 快速开始

### 安装依赖

\`\`\`bash
cd performance-dashboard
npm install
\`\`\`

### 开发模式

\`\`\`bash
npm run dev
\`\`\`

访问 http://localhost:3000

### 生产构建

\`\`\`bash
npm run build
\`\`\`

### 预览构建

\`\`\`bash
npm run preview
\`\`\`

## 技术栈

- **React 18** - UI框架
- **TypeScript** - 类型安全
- **Vite** - 构建工具
- **TailwindCSS** - 样式框架
- **Recharts** - 图表库
- **WebSocket** - 实时通信

## 项目结构

\`\`\`
performance-dashboard/
├── src/
│   ├── components/         # React组件
│   │   ├── RealTimeMetrics.tsx
│   │   ├── PerformanceCharts.tsx
│   │   ├── AlertPanel.tsx
│   │   └── OptimizationSuggestions.tsx
│   ├── hooks/              # 自定义Hooks
│   ├── types/             # TypeScript类型
│   ├── utils/             # 工具函数
│   ├── App.tsx            # 主应用组件
│   ├── main.tsx           # 入口文件
│   └── index.css          # 全局样式
├── public/                # 静态资源
├── index.html             # HTML模板
├── package.json           # 项目配置
├── vite.config.ts         # Vite配置
├── tailwind.config.js     # Tailwind配置
└── tsconfig.json          # TypeScript配置
\`\`\`

## WebSocket API

### 连接

\`\`\`javascript
const ws = new WebSocket('ws://localhost:8080/api/performance/stream')
\`\`\`

### 数据格式

\`\`\`json
{
  "fps": 60.0,
  "frameTime": 16.67,
  "cpu": 45.2,
  "memory": 268435456,
  "gpu": 62.5,
  "drawCalls": 150,
  "triangleCount": 250000
}
\`\`\`

## 开发指南

### 添加新组件

1. 在 \`src/components/\` 创建组件
2. 使用 TypeScript 定义 Props
3. 使用 TailwindCSS 样式
4. 在 App.tsx 中导入使用

### 自定义主题

编辑 \`tailwind.config.js\`:

\`\`\`javascript
export default {
  theme: {
    extend: {
      colors: {
        'custom': '#value',
      }
    }
  }
}
\`\`\`

## 性能优化

- ✅ 组件懒加载
- ✅ 虚拟化长列表
- ✅ useMemo/useCallback优化
- ✅ WebSocket连接复用

## 浏览器支持

- ✅ Chrome 90+
- ✅ Firefox 88+
- ✅ Safari 14+
- ✅ Edge 90+

## 贡献指南

1. Fork项目
2. 创建功能分支
3. 提交Pull Request

## 许可证

MIT License

---

**最后更新**: 2025-01-01
**维护者**: Game Engine Development Team
