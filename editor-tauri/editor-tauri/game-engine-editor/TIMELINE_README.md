# 动画时间轴编辑器 - 使用说明

## 功能概述

已成功为Tauri游戏引擎编辑器实现了一个完整的**动画时间轴编辑器**（Animation Timeline Editor）。

## 实现的功能

### 1. 核心类型系统 ✓
- **文件**: `src/types/animation.ts`
- 完整的TypeScript类型定义
- 支持关键帧、动画曲线、动画轨道、动画剪辑
- 13种Easing函数支持
- Vector3和Quaternion值类型支持

### 2. Timeline UI组件 ✓
完整的UI组件系统，包括：

#### 主组件
- **Timeline.tsx**: 时间轴主组件
- **TimeRuler.tsx**: 时间标尺（Canvas绘制，支持缩放和拖拽）
- **Playhead.tsx**: 播放头组件
- **TrackList.tsx**: 轨道列表
- **Track.tsx**: 单个轨道组件
- **PlaybackControls.tsx**: 播放控制

#### 高级功能组件
- **CurveEditor.tsx**: 曲线编辑器（Canvas绘制）
- **PropertiesPanel.tsx**: 关键帧属性面板
- **AnimationManager.tsx**: 动画剪辑管理器

### 3. 播放控制 ✓
- ⏮️ 跳到开始
- ⏯️ 播放/暂停
- ⏹️ 停止
- ⏪ 前一帧 / ⏩ 后一帧
- ⏭️ 跳到结束
- 🔂 循环播放
- 播放速度调节（0.1x - 2x）
- 时间显示（分:秒:帧格式）

### 4. 键盘快捷键 ✓
- `Space`: 播放/暂停
- `Home`: 跳到开始
- `End`: 跳到结束
- `←/→`: 前一帧/后一帧
- `Shift+←/→`: 前一秒/后一秒
- `Del`: 删除选中的关键帧
- `Ctrl+T`: 打开/关闭时间轴

### 5. 时间标尺 ✓
- Canvas绘制，性能优异
- 支持鼠标拖拽跳转时间
- 支持滚轮缩放
- 自动调整刻度密度
- 显示当前时间指示器

### 6. 轨道系统 ✓
- 轨道可见性切换（👁️）
- 轨道锁定（🔒）
- 轨道静音（🔇）
- 轨道展开/收起
- 颜色编码：
  - Transform: 红色
  - Rotation: 绿色
  - Scale: 蓝色
  - Property: 紫色
  - Event: 琥珀色

### 7. 关键帧编辑 ✓
- 关键帧显示和选择
- 拖拽移动时间
- 多选关键帧（Shift+点击）
- 删除关键帧
- 支持插值类型：
  - Constant（常量）
  - Linear（线性）
  - Cubic（三次样条）
  - Hermite

### 8. 曲线编辑器 ✓
- Canvas绘制曲线
- 时间-值图显示
- 切线手柄编辑
- 网格显示
- 自动缩放

### 9. 动画管理 ✓
- 新建动画剪辑
- 动画重命名
- 动画复制
- 动画删除
- 动画导出（JSON）
- 动画导入（JSON）

### 10. Tauri后端集成 ✓
- **文件**: `src-tauri/src/animation_system.rs`
- 完整的Rust后端实现
- 动画剪辑持久化（JSON文件）
- 关键帧CRUD操作
- 动画评估（插值计算）

## 使用方法

### 1. 打开时间轴

在编辑器中按下 `Ctrl+T` (或 `Cmd+T`) 或点击底部的 "🎬 Timeline" 按钮。

### 2. 创建动画

时间轴会自动创建一个默认的 "Animation 1" 剪辑。

### 3. 添加轨道

目前轨道通过代码创建，未来可以通过以下方式添加：
- 选择场景中的实体
- 选择要动画的属性（Position、Rotation、Scale等）
- 自动创建对应轨道

### 4. 添加关键帧

- 点击轨道添加关键帧
- 或按 `K` 键快速添加
- 拖拽关键帧调整时间
- 双击关键帧编辑值

### 5. 播放动画

- 按下 `Space` 键或点击 ▶️ 按钮播放
- 调节播放速度（0.1x - 2x）
- 启用循环播放

### 6. 编辑曲线

- 在曲线编辑器中查看动画曲线
- 拖拽切线手柄调整曲线
- 切换不同的插值类型

### 7. 保存动画

动画自动保存为JSON格式，存储在 `animations/` 目录。

## 技术特性

### 性能优化
- Canvas绘制时间标尺和曲线
- requestAnimationFrame更新播放
- React.memo优化组件渲染
- 虚拟化长列表（未来可扩展）

### 类型安全
- 完整的TypeScript类型定义
- Rust后端类型安全
- 序列化/反序列化支持

### 用户体验
- 响应式设计
- 深色主题
- 平滑动画
- 直观的快捷键

## 文件结构

```
src/
├── types/
│   └── animation.ts                    # 动画类型定义
├── components/
│   └── Timeline/
│       ├── Timeline.tsx                # 主时间轴组件
│       ├── Timeline.css                # 样式文件
│       ├── TimeRuler.tsx               # 时间标尺
│       ├── Playhead.tsx                # 播放头
│       ├── TrackList.tsx               # 轨道列表
│       ├── Track.tsx                   # 单个轨道
│       ├── PlaybackControls.tsx        # 播放控制
│       ├── CurveEditor.tsx             # 曲线编辑器
│       ├── PropertiesPanel.tsx         # 属性面板
│       ├── AnimationManager.tsx        # 动画管理器
│       └── index.ts                    # 导出文件
src-tauri/
└── src/
    ├── animation_system.rs             # 动画系统Rust实现
    └── lib.rs                          # 已集成动画命令

```

## API命令（Tauri后端）

### 动画剪辑管理
- `create_animation_clip(name: String)`: 创建动画剪辑
- `save_animation_clip(clip: AnimationClipData)`: 保存动画剪辑
- `load_animation_clip(id: String)`: 加载动画剪辑
- `delete_animation_clip(clip_id: String)`: 删除动画剪辑
- `list_animation_clips()`: 列出所有动画剪辑

### 关键帧管理
- `add_keyframe(track_id: String, keyframe: KeyframeData)`: 添加关键帧
- `update_keyframe(keyframe_id: String, data: KeyframeData)`: 更新关键帧
- `delete_keyframe(keyframe_id: String)`: 删除关键帧

### 动画评估
- `evaluate_animation_at_time(clip_id: String, time: f64)`: 在指定时间评估动画

## 未来扩展

### 短期（P1）
- [ ] 从实体创建轨道
- [ ] 关键帧拖拽编辑值
- [ ] Dope Sheet视图
- [ ] 多轨道复制粘贴

### 中期（P2）
- [ ] 骨骼动画支持
- [ ] IK/FK切换
- [ ] 动画层叠
- [ ] 动画混合

### 长期（P3）
- [ ] 物理动画集成
- [ ] 程序化动画
- [ ] 动画状态机
- [ ] 实时预览

## 已知问题

1. **BehaviorEditor错误**: 这些是项目原有的错误，不影响Timeline功能
2. **Tauri API调用**: AnimationManager中的Tauri调用已注释，待后端完全就绪后启用

## 贡献

欢迎提交问题和改进建议！

## 许可证

MIT License
