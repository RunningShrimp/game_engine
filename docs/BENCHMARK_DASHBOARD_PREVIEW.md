# 性能监控仪表板预览

## 仪表板界面

### 整体布局

```
┌─────────────────────────────────────────────────────────────┐
│  🚀 Game Engine Performance Dashboard                       │
│  Real-time performance monitoring and trend analysis        │
│                                                             │
│  Last updated: 2025-12-28 22:50:00  [Refresh]              │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  📊 Performance Summary                                     │
│                                                             │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐       │
│  │    ✅   │  │    ❌   │  │    ➖   │  │    📊   │       │
│  │    5    │  │    2    │  │   35    │  │   42    │       │
│  │Improved │  │Regressed│  │  Stable │  │  Total  │       │
│  └─────────┘  └─────────┘  └─────────┘  └─────────┘       │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────┐  ┌─────────────────────┐
│  ECS Entity Creation│  │  Physics Step       │
│                     │  │                     │
│  ↓ 5.2%             │  │  ↑ 2.1%             │
│  1,234 ns           │  │  3,456 ns           │
│  Baseline: 1,300 ns │  │  Baseline: 3,385 ns │
│  Status: Improved   │  │  Status: Regressed  │
│  StdDev: ±45 ns     │  │  StdDev: ±120 ns    │
└─────────────────────┘  └─────────────────────┘

┌─────────────────────┐  ┌─────────────────────┐
│  Render Batching    │  │  Serialization      │
│                     │  │                     │
│  → 0.8%             │  │  ↓ 8.3%             │
│  2,100 ns           │  │  850 ns             │
│  Baseline: 2,083 ns │  │  Baseline: 927 ns   │
│  Status: Stable     │  │  Status: Improved   │
│  StdDev: ±78 ns     │  │  StdDev: ±32 ns     │
└─────────────────────┘  └─────────────────────┘

... (more metric cards)

┌─────────────────────────────────────────────────────────────┐
│  📈 Performance Trends                                      │
│                                                             │
│  ┌─────────────────────────────────────────────────┐       │
│  │  ECS Entity Creation Performance Trend         │       │
│  │  1500│                                         │       │
│  │      │    ●                                    │       │
│  │  1400│     ●●                                  │       │
│  │      │      ●●                                 │       │
│  │  1300│       ●●●                               │       │
│  │      │         ●●●●                            │       │
│  │  1200└──────────────────────────────────────   │       │
│  │       2025-12-20    2025-12-25    2025-12-28   │       │
│  └─────────────────────────────────────────────────┘       │
│                                                             │
│  ┌─────────────────────────────────────────────────┐       │
│  │  Physics Step Performance Trend                 │       │
│  │  3600│    ●    ●                                │       │
│  │      │   ● ●●  ●●                               │       │
│  │  3500│  ●   ●●● ●●                               │       │
│  │      │ ●     ●●●●●                              │       │
│  │  3400└──────────────────────────────────────    │       │
│  │       2025-12-20    2025-12-25    2025-12-28    │       │
│  └─────────────────────────────────────────────────┘       │
│                                                             │
│  ... (more trend charts)                                    │
└─────────────────────────────────────────────────────────────┘
```

## 颜色方案

### 状态指示器

| 状态 | Emoji | 颜色 | 说明 |
|------|-------|------|------|
| 改进 | ✅ | 绿色 #48bb78 | 性能提升 >5% |
| 回归 | ❌ | 红色 #f56565 | 性能退化 >10% |
| 稳定 | ➖ | 黄色 #ecc94b | 变化 ±5% 内 |

### 渐变主题

```css
背景渐变: linear-gradient(135deg, #667eea 0%, #764ba2 100%)
成功渐变: linear-gradient(135deg, #48bb78 0%, #38a169 100%)
警告渐变: linear-gradient(135deg, #ecc94b 0%, #d69e2e 100%)
错误渐变: linear-gradient(135deg, #f56565 0%, #e53e3e 100%)
```

## 响应式布局

### 桌面 (>1024px)
- 3列卡片布局
- 2列图表布局
- 完整导航栏

### 平板 (768px - 1024px)
- 2列卡片布局
- 1列图表布局
- 简化导航

### 移动 (<768px)
- 1列卡片布局
- 1列图表布局
- 汉堡菜单

## 交互功能

### 自动刷新
- 间隔: 5分钟
- 显示: 倒计时器
- 手动: [Refresh] 按钮

### 卡片交互
- 悬停: 向上移动4px
- 阴影: 加深效果
- 平滑过渡: 0.2s

### 图表功能
- 响应式缩放
- 高DPI支持 (150dpi)
- 自动标签旋转
- 网格辅助线

## 数据格式

### JSON数据结构

```json
{
  "metadata": {
    "timestamp": "2025-12-28T22:50:00Z",
    "count": 42,
    "version": "1.0"
  },
  "benchmarks": {
    "ecs_create_entity_1000": {
      "mean": 1234.56,
      "stddev": 45.67,
      "median": 1220.00,
      "min": 1100.00,
      "max": 1500.00,
      "baseline": 1200.00,
      "baseline_stddev": 40.00,
      "unit": "ns"
    }
  }
}
```

### 图表命名约定

- 趋势图: `{benchmark_name}_trend.png`
- 数据文件: `benchmark_data.json`
- 历史数据: `{YYYY-MM-DD}.json`

## 性能优化

### 加载策略
1. 并行加载所有资源
2. 懒加载图表
3. 错误优雅降级

### 缓存策略
- 浏览器缓存: 5分钟
- 图表缓存: 1小时
- 数据缓存: 1小时

### 响应时间
- 首次加载: <2秒
- 后续刷新: <500ms
- 图表渲染: <100ms

## 浏览器兼容性

| 浏览器 | 最低版本 | 备注 |
|--------|---------|------|
| Chrome | 90+ | 完全支持 |
| Firefox | 88+ | 完全支持 |
| Safari | 14+ | 完全支持 |
| Edge | 90+ | 完全支持 |
| Mobile | iOS 14+, Android 10+ | 响应式支持 |

## 本地运行

### 方法1: Python HTTP服务器

```bash
cd game_engine/benches/trends
python3 -m http.server 8000
# 访问: http://localhost:8000
```

### 方法2: Node.js HTTP服务器

```bash
npx http-server game_engine/benches/trends -p 8000
# 访问: http://localhost:8000
```

### 方法3: VS Code Live Server

1. 安装 Live Server 扩展
2. 右键 `index.html`
3. 选择 "Open with Live Server"

## 部署到GitHub Pages

### 自动部署 (通过CI)

已在`.github/workflows/benchmark.yml`中配置:

```yaml
- name: Deploy trend report to GitHub Pages
  uses: peaceiris/actions-gh-pages@v3
  with:
    github_token: ${{ secrets.GITHUB_TOKEN }}
    publish_dir: ./game_engine/benches/trends
    publish_branch: gh-pages
```

### 手动部署

```bash
# 1. 生成数据
cargo bench --workspace
python3 scripts/export_benchmark_data.py

# 2. 构建gh-pages分支
git checkout --orphan gh-pages
git add -f game_engine/benches/trends/*
git commit -m "Deploy dashboard"
git push origin gh-pages
```

## 自定义

### 修改颜色主题

编辑`index.html`中的CSS变量:

```css
.background-gradient {
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
}

.success-color { color: #48bb78; }
.danger-color { color: #f56565; }
.warning-color { color: #ecc94b; }
```

### 调整刷新间隔

修改JavaScript:

```javascript
// 从5分钟改为2分钟
setInterval(loadData, 2 * 60 * 1000);
```

### 添加新指标

在`renderDashboard()`函数中添加:

```javascript
const metricsHTML = Object.entries(benchmarkData).map(([name, data]) => {
    // 添加自定义逻辑
    return `...`;
}).join('');
```

## 监控集成

### 添加分析

```html
<!-- Google Analytics -->
<script async src="https://www.googletagmanager.com/gtag/js?id=GA_ID"></script>
<script>
  window.dataLayer = window.dataLayer || [];
  function gtag(){dataLayer.push(arguments);}
  gtag('js', new Date());
  gtag('config', 'GA_ID');
</script>
```

### 错误跟踪

```javascript
window.onerror = function(msg, url, line) {
    // 发送到错误跟踪服务
    console.error('Dashboard error:', msg, url, line);
};
```

## 安全考虑

### CSP Headers

```html
<meta http-equiv="Content-Security-Policy"
      content="default-src 'self'; img-src 'self' data:; script-src 'self' 'unsafe-inline';">
```

### HTTPS Only

生产环境建议使用HTTPS，GitHub Pages默认启用。

## 故障排除

### 问题: 数据不加载

**检查:**
1. `benchmark_data.json`是否存在
2. 控制台错误 (F12)
3. CORS配置 (使用本地服务器)

### 问题: 图表不显示

**检查:**
1. PNG文件路径
2. 文件权限
3. 服务器日志

### 问题: 刷新失败

**检查:**
1. 网络连接
2. JSON格式有效性
3. 浏览器控制台

## 未来增强

### 计划功能

1. **实时更新** - WebSocket推送
2. **对比模式** - 多版本对比
3. **导出功能** - PDF/图片导出
4. **告警配置** - 自定义阈值
5. **注释系统** - 性能变化说明
6. **历史回溯** - 任意时间点查看

### 反馈渠道

如有建议或问题:
- GitHub Issues
- 文档评论区
- 邮件反馈
