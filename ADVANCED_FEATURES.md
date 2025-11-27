# 游戏引擎编辑器 - 高级功能

本文档介绍编辑器的高级功能和技术实现。

## 🔌 WebSocket实时通信

### 功能概述
编辑器通过WebSocket与Rust游戏引擎后端建立双向实时通信，实现场景数据同步、性能监控和日志传输。

### 技术实现
- **WebSocket服务**: `client/src/lib/websocket.ts`
- **React上下文**: `client/src/contexts/WebSocketContext.tsx`
- **连接状态指示器**: `client/src/components/ConnectionStatus.tsx`

### 使用方法
```typescript
import { useWebSocket } from '@/contexts/WebSocketContext';

function MyComponent() {
  const { isConnected, send, subscribe } = useWebSocket();

  // 发送消息到引擎
  send('scene_update', { entityId: 123, position: [0, 1, 0] });

  // 订阅引擎消息
  useEffect(() => {
    const unsubscribe = subscribe('performance_stats', (message) => {
      console.log('Performance:', message.data);
    });
    return unsubscribe;
  }, []);
}
```

### 消息类型
- `scene_update`: 场景数据更新
- `entity_update`: 实体状态变化
- `asset_loaded`: 资产加载完成
- `performance_stats`: 性能统计数据
- `log_message`: 日志消息
- `command_response`: 命令执行响应

### 连接管理
- **自动重连**: 断线后自动尝试重连（最多5次）
- **指数退避**: 重连延迟逐次增加（1s, 2s, 4s, 8s, 16s）
- **状态监控**: 实时显示连接状态（connecting, connected, disconnected）

## 🎨 WebGPU 3D渲染

### 功能概述
使用WebGPU API实现高性能的3D场景渲染，支持现代图形特性。

### 技术实现
- **渲染器组件**: `client/src/components/Renderer3D.tsx`
- **GPU适配器检测**: 自动检测WebGPU支持
- **渲染循环**: 60fps实时渲染

### 浏览器支持
- Chrome 113+ (需启用WebGPU实验性功能)
- Edge 113+
- 其他浏览器正在逐步支持中

### 启用WebGPU
1. 打开 `chrome://flags`
2. 搜索 "WebGPU"
3. 启用 "Unsafe WebGPU" 选项
4. 重启浏览器

### 渲染特性
- **清屏颜色**: 深紫蓝色 `rgba(0.1, 0.1, 0.15, 1.0)`
- **Alpha混合**: 预乘Alpha模式
- **自适应大小**: 自动适配容器尺寸

## 🖱️ 拖放交互

### 功能概述
支持从资产浏览器拖放资产到场景编辑器，实现快速场景构建。

### 技术实现
- **拖放Hook**: `client/src/hooks/useDragDrop.ts`
- **资产拖动**: 资产浏览器中的卡片可拖动
- **场景接收**: 场景编辑器接收拖放的资产

### 使用示例
```typescript
import { useDragDrop } from '@/hooks/useDragDrop';

// 拖动源
function DraggableAsset({ asset }) {
  const { handleDragStart, handleDragEnd } = useDragDrop();
  
  return (
    <div
      draggable
      onDragStart={(e) => handleDragStart(e, 'asset', asset)}
      onDragEnd={handleDragEnd}
    >
      {asset.name}
    </div>
  );
}

// 拖放目标
function DropTarget() {
  const { handleDragOver, handleDrop } = useDragDrop();
  
  return (
    <div
      onDragOver={handleDragOver}
      onDrop={(e) => {
        const data = handleDrop(e);
        console.log('Dropped:', data);
      }}
    >
      Drop here
    </div>
  );
}
```

### 拖放数据格式
```typescript
interface DragDropData {
  type: string;      // 数据类型（如 'asset', 'entity'）
  data: any;         // 实际数据
}
```

## ⌨️ 快捷键系统

### 功能概述
全局快捷键支持，提高编辑效率。

### 技术实现
- **快捷键Hook**: `client/src/hooks/useHotkeys.ts`
- **预定义快捷键**: `HOTKEYS` 常量

### 全局快捷键
| 快捷键 | 功能 |
|--------|------|
| Ctrl+S | 保存 |
| Ctrl+Z | 撤销 |
| Ctrl+Y | 重做 |
| Ctrl+D | 复制 |
| Ctrl+A | 全选 |
| Ctrl+K | 打开命令面板 |
| Ctrl+P | 打开命令面板 |
| F5 | 运行 |
| Shift+F5 | 停止 |
| Delete | 删除 |

### 场景编辑器快捷键
| 快捷键 | 功能 |
|--------|------|
| Q | 切换到移动工具 |
| W | 切换到旋转工具 |
| E | 切换到缩放工具 |

### 自定义快捷键
```typescript
import { useHotkeys } from '@/hooks/useHotkeys';

function MyComponent() {
  useHotkeys([
    {
      key: 'f',
      ctrl: true,
      handler: () => console.log('Find'),
      description: '查找',
    },
    {
      key: 'g',
      ctrl: true,
      shift: true,
      handler: () => console.log('Go to line'),
      description: '跳转到行',
    },
  ]);
}
```

## 🔍 命令面板

### 功能概述
快速访问所有编辑器功能的命令面板，支持模糊搜索。

### 技术实现
- **命令面板组件**: `client/src/components/CommandPalette.tsx`
- **触发方式**: Ctrl+K 或 Ctrl+P

### 可用命令
- 打开场景编辑器
- 打开资产浏览器
- 打开实体管理器
- 打开调试工具
- 打开设置
- 打开文档
- 保存
- 运行
- 停止

### 搜索功能
- **模糊匹配**: 支持部分关键词匹配
- **多语言**: 支持中英文搜索
- **关键词**: 每个命令配置多个关键词

## 📊 性能监控

### 功能概述
实时监控编辑器性能指标，显示在场景视口左下角。

### 技术实现
- **性能Hook**: `client/src/hooks/usePerformance.ts`
- **监控指标**:
  - **FPS**: 每秒帧数
  - **Frame Time**: 每帧耗时（毫秒）
  - **Memory**: JavaScript堆内存使用（MB）

### 使用方法
```typescript
import { usePerformance } from '@/hooks/usePerformance';

function MyComponent() {
  const stats = usePerformance(1000); // 每1秒更新一次
  
  return (
    <div>
      <div>FPS: {stats.fps}</div>
      <div>Frame Time: {stats.frameTime}ms</div>
      <div>Memory: {stats.memory}MB</div>
    </div>
  );
}
```

### 性能优化建议
1. **保持60fps**: FPS低于60时考虑优化渲染
2. **控制帧时间**: 单帧耗时应小于16.67ms
3. **监控内存**: 内存持续增长可能存在内存泄漏

## 🔧 与Rust引擎集成

### 通信架构
```
┌─────────────────┐         WebSocket          ┌─────────────────┐
│                 │◄──────────────────────────►│                 │
│  React编辑器    │                             │  Rust游戏引擎   │
│                 │         HTTP API            │                 │
│  (前端UI)       │◄──────────────────────────►│  (后端逻辑)     │
└─────────────────┘                             └─────────────────┘
```

### WebSocket消息流
1. **编辑器 → 引擎**:
   - 场景编辑操作
   - 实体创建/修改/删除
   - 资产加载请求
   - 配置更改

2. **引擎 → 编辑器**:
   - 场景状态更新
   - 性能统计数据
   - 日志消息
   - 资产加载完成通知

### HTTP API
用于非实时操作：
- 资产上传
- 项目保存/加载
- 配置读写
- 构建/打包

### 数据序列化
- **格式**: JSON
- **场景数据**: 包含实体、组件、资源引用
- **二进制资产**: Base64编码或独立HTTP传输

## 🚀 性能优化

### 前端优化
1. **React优化**:
   - 使用 `React.memo` 减少重渲染
   - 使用 `useMemo` 和 `useCallback` 缓存计算结果
   - 虚拟滚动处理大列表

2. **渲染优化**:
   - WebGPU硬件加速
   - 按需渲染（仅在场景变化时重绘）
   - LOD（细节层次）管理

3. **资产优化**:
   - 纹理压缩（BC7, ASTC）
   - 模型简化
   - 懒加载

### 后端优化
1. **多线程**:
   - 渲染线程
   - 物理线程
   - 资产加载线程

2. **SIMD加速**:
   - 向量运算
   - 矩阵运算
   - 碰撞检测

3. **GPU加速**:
   - 粒子系统
   - 后处理效果
   - 物理模拟

## 📝 开发指南

### 添加新的WebSocket消息类型
1. 在 `websocket.ts` 中添加类型定义:
```typescript
export type MessageType = 
  | 'scene_update'
  | 'your_new_type';  // 添加新类型
```

2. 在组件中订阅消息:
```typescript
const { subscribe } = useWebSocket();

useEffect(() => {
  return subscribe('your_new_type', (message) => {
    // 处理消息
  });
}, []);
```

### 添加新的快捷键
在 `useHotkeys.ts` 的 `HOTKEYS` 中添加:
```typescript
export const HOTKEYS = {
  // ...existing hotkeys
  YOUR_ACTION: { key: 'x', ctrl: true, description: '你的操作' },
};
```

### 添加新的命令
在 `CommandPalette.tsx` 的 `commands` 数组中添加:
```typescript
{
  id: 'your_command',
  label: '你的命令',
  icon: YourIcon,
  action: () => {
    // 执行操作
  },
  keywords: ['keyword1', 'keyword2'],
}
```

## 🐛 调试技巧

### WebSocket调试
```typescript
// 在浏览器控制台查看WebSocket消息
websocketService.on('*', (message) => {
  console.log('WebSocket:', message);
});
```

### 性能分析
1. 打开Chrome DevTools
2. 切换到Performance标签
3. 录制性能数据
4. 分析帧率和CPU使用

### WebGPU调试
1. 打开 `chrome://gpu`
2. 查看WebGPU状态
3. 检查GPU信息和驱动版本

## 📚 相关资源

- [WebSocket API文档](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket)
- [WebGPU规范](https://www.w3.org/TR/webgpu/)
- [React性能优化](https://react.dev/learn/render-and-commit)
- [Rust游戏引擎开发](https://arewegameyet.rs/)
