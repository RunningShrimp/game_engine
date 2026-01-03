# 实时性能监控仪表板

游戏引擎编辑器的专业性能分析工具，提供实时FPS、CPU、GPU、内存监控和性能瓶颈分析。

## 快速开始

### 打开方式
- **快捷键**: 按 `F12`
- **按钮**: 点击状态栏的 "📊 Performance" 按钮

## 核心功能

### 1. 实时指标监控
- ✅ FPS帧率（颜色编码）
- ✅ 帧时间
- ✅ CPU/GPU/内存使用率
- ✅ Draw Calls和三角形数
- ✅ 物理和脚本统计

### 2. 交互式图表
- ✅ FPS实时曲线图（60秒历史）
- ✅ 系统使用率饼图（渲染/物理/脚本等）
- ✅ 内存趋势图（带泄漏检测）

### 3. 性能热点分析
- ✅ Top 5性能瓶颈排行
- ✅ 点击查看详细信息
- ✅ 调用栈和调用次数

### 4. 智能告警系统
- ✅ FPS过低告警
- ✅ 内存/GPU/CPU过高告警
- ✅ 帧时间过长告警
- ✅ 告警确认和清除
- ✅ 严重告警顶部闪烁横幅

### 5. 数据导出
- ✅ JSON格式
- ✅ CSV格式
- ✅ 可配置时间范围

## 文件结构

### 后端 (Rust)
```
src-tauri/src/
├── performance_monitor.rs     # 核心监控器
└── performance_commands.rs    # Tauri命令接口
```

### 前端 (TypeScript/React)
```
src/
├── types/performance.ts              # 类型定义
├── api/performance.ts                # API包装
└── components/PerformanceDashboard/
    ├── PerformanceDashboard.tsx      # 主仪表板
    ├── MetricsPanel.tsx              # 指标面板
    ├── HotspotPanel.tsx              # 热点面板
    ├── AlertSystem.tsx               # 告警系统
    ├── Charts/
    │   ├── FPSChart.tsx              # FPS图表
    │   ├── UsagePieChart.tsx         # 使用率饼图
    │   └── MemoryTrendChart.tsx      # 内存趋势图
    └── index.ts                      # 导出
```

## 技术栈

- **后端**: Rust + Tauri + serde + chrono
- **前端**: React 19 + TypeScript + Tailwind CSS 4
- **图表**: Recharts (轻量、高性能)

## 配置选项

### 更新频率
- 100ms: 最实时
- 250ms: 推荐（默认）
- 500ms: 平衡
- 1s: 低开销

### 告警阈值（可配置）
```typescript
FPS:       警告 <50, 严重 <30
内存:      警告 >85%, 严重 >95%
GPU:       警告 >85%, 严重 >95%
帧时间:    警告 >20ms, 严重 >33ms
CPU:       警告 >85%, 严重 >95%
```

## 使用示例

### 性能优化工作流
1. 打开仪表板 (F12)
2. 运行游戏场景
3. 观察热点面板
4. 优化Top 3耗时系统
5. 重新测试验证

### 内存泄漏检测
1. 观察内存趋势图
2. 如显示"Possible Memory Leak"警告
3. 点击热点查看详情
4. 检查对象生命周期

## 文档

- **实现报告**: [PERFORMANCE_DASHBOARD_IMPLEMENTATION_REPORT.md](./PERFORMANCE_DASHBOARD_IMPLEMENTATION_REPORT.md)
- **使用指南**: [PERFORMANCE_DASHBOARD_USER_GUIDE.md](./PERFORMANCE_DASHBOARD_USER_GUIDE.md)

## 依赖

### npm
```json
{
  "recharts": "^2.x"
}
```

### Cargo
```toml
[dependencies]
chrono = "0.4"
```

## 性能影响

- CPU开销: <1%
- 内存占用: ~10MB
- UI渲染: 60 FPS

## 快捷键

- `F12`: 打开/关闭性能仪表板

## 许可

MIT License - 详见项目根目录

---

**享受流畅的性能监控体验！** 🚀
