# 组件迁移计划

> 基于Atomic Design架构的现有组件迁移到新架构体系的详细计划

## 目录

1. [迁移概述](#迁移概述)
2. [现状分析](#现状分析)
3. [迁移策略](#迁移策略)
4. [详细迁移计划](#详细迁移计划)
5. [测试策略](#测试策略)
6. [风险评估](#风险评估)
7. [回滚计划](#回滚计划)

---

## 迁移概述

### 目标

将现有组件迁移到Atomic Design架构体系，实现：
- 清晰的组件层级划分
- 统一的代码规范和命名约定
- 提高组件复用性和可维护性
- 完善的测试覆盖率

### 迁移范围

**影响范围**:
- `/src/components/` 下所有现有组件
- `/src/App.tsx` 及其他使用这些组件的文件
- 相关的测试文件和类型定义

**不包括**:
- `/src/utils/` - 工具函数
- `/src/hooks/` - 自定义Hooks
- `/src/types/` - 全局类型定义
- 第三方库组件

### 预计时间

- 总计: **8周**
- Phase 1-2: **4周** (基础组件)
- Phase 3-4: **4周** (业务组件和页面)

---

## 现状分析

### 现有组件清单

#### UI组件 (8个)

```
components/ui/
├── Button.tsx              → molecules/Button/
├── EmptyState.tsx          → organisms/EmptyState/
├── Spinner.tsx             → atoms/Spinner/ (已完成)
├── Skeleton.tsx            → atoms/Skeleton/ (已完成)
└── examples.tsx            → 删除
```

#### 功能模块组件 (约100+个)

```
components/
├── Toolbar/
│   ├── Toolbar.tsx         → organisms/Toolbar/
│   └── BatchToolbar.tsx    → organisms/BatchToolbar/
├── EntityTree/
│   ├── EntityTree.tsx      → organisms/EntityTree/
│   └── VirtualEntityTree.tsx → organisms/VirtualEntityTree/
├── PropertyInspector/
│   └── PropertyInspector.tsx → organisms/PropertyInspector/
├── Timeline/               → organisms/Timeline/ + pages/TimelinePage/
├── AssetBrowser/           → organisms/AssetBrowser/ + pages/AssetBrowserPage/
├── AssetStorePanel/        → organisms/AssetStorePanel/
├── MaterialEditor/         → pages/MaterialEditorPage/
├── BehaviorEditor/         → pages/BehaviorEditorPage/
├── PerformanceDashboard/   → pages/PerformanceDashboardPage/
├── HistoryPanel/           → organisms/HistoryPanel/
├── ShortcutEditor/         → organisms/ShortcutEditor/
├── Toast/                  → organisms/Toast/
├── ResizablePanel/         → molecules/ResizablePanel/
├── tutorial/               → organisms/TutorialSystem/
├── loading/                → atoms/ (各种Loading组件)
├── ShortcutOverlay/        → organisms/ShortcutOverlay/
└── Viewport/               → organisms/Viewport/
```

### 依赖关系图

```
App.tsx
├── Toolbar/ (organisms)
│   └── ui/Button (molecules - 需要迁移)
├── EntityTree/ (organisms)
│   └── ui/Icon (atoms - 需要创建)
├── PropertyInspector/ (organisms)
│   └── ui/Input (molecules - 需要创建)
├── Timeline/ (organisms)
│   └── ui/Slider (molecules - 需要创建)
└── AssetBrowser/ (organisms)
    ├── ui/Card (organisms - 需要创建)
    └── ui/Badge (atoms - 需要创建)
```

---

## 迁移策略

### 迁移原则

1. **自底向上**: 从Atoms开始，逐步向上迁移
2. **增量迁移**: 每次迁移一个组件，保持系统可用
3. **向后兼容**: 保留旧接口，提供迁移警告
4. **测试先行**: 先写测试，再迁移代码
5. **文档同步**: 代码和文档同步更新

### 迁移流程

```
1. 创建新组件目录结构
   ↓
2. 编写/更新测试用例
   ↓
3. 实现新组件（迁移旧代码）
   ↓
4. 添加导出和类型定义
   ↓
5. 更新引用路径
   ↓
6. 运行测试和手动验证
   ↓
7. 标记旧组件为deprecated
   ↓
8. 更新文档
   ↓
9. 代码审查
   ↓
10. 合并到主分支
```

### 兼容性策略

#### 阶段1: 双轨运行

```typescript
// 保留旧路径，指向新实现
export { Button as ButtonLegacy } from './Button';

// 同时导出新路径
export { Button } from '../molecules/Button';

// 添加废弃警告
/**
 * @deprecated 请使用 'components/molecules/Button' 代替
 * 此导出将在v2.0.0中移除
 */
```

#### 阶段2: 警告期

```typescript
// 运行时警告
if (process.env.NODE_ENV === 'development') {
  console.warn(
    '[Button] Legacy import detected. Please update to: ' +
    'import { Button } from "components/molecules/Button"'
  );
}
```

#### 阶段3: 完全移除

```bash
# 2个版本后完全移除旧代码
git rm components/ui/Button.tsx
```

---

## 详细迁移计划

### Phase 1: 基础原子组件 (Week 1-2)

#### 目标组件

```bash
atoms/
├── Icon/              # 图标组件 (新建)
├── Text/              # 文本组件 (新建)
├── Badge/             # 徽章组件 (新建)
├── Avatar/            # 头像组件 (新建)
├── Divider/           # 分割线组件 (新建)
├── Spacer/            # 间距组件 (新建)
├── Tooltip/           # 提示组件 (新建)
└── ProgressBar/       # 进度条组件 (新建)
```

#### 迁移清单

##### 1. Icon 组件

**文件**: `atoms/Icon/Icon.tsx`

```typescript
// 功能需求
- 支持多种图标库 (Lucide, Radix Icons, 自定义SVG)
- 支持尺寸: xs, sm, md, lg, xl
- 支持颜色继承
- 支持旋转动画
- 性能优化: SVG缓存

// 类型定义
interface IconProps {
  name: string;
  size?: 'xs' | 'sm' | 'md' | 'lg' | 'xl';
  color?: string;
  className?: string;
  spin?: boolean;
  iconSet?: 'lucide' | 'radix' | 'custom';
}

// 测试用例
- 渲染正确的图标
- 应用正确的尺寸类
- 支持自定义颜色
- 支持旋转动画
- 性能测试: 渲染1000个图标
```

**依赖**: 无

**优先级**: P0

**预计工时**: 2天

---

##### 2. Text 组件

**文件**: `atoms/Text/Text.tsx`

```typescript
// 功能需求
- 支持多种变体: heading, title, body, caption, code
- 支持多种字体大小
- 支持粗细、颜色、对齐
- 支持截断和省略号
- 支持多行限制

// 类型定义
interface TextProps {
  variant?: 'h1' | 'h2' | 'h3' | 'body' | 'caption' | 'code';
  size?: 'xs' | 'sm' | 'md' | 'lg' | 'xl' | '2xl' | '3xl';
  weight?: 'normal' | 'medium' | 'semibold' | 'bold';
  color?: string;
  align?: 'left' | 'center' | 'right' | 'justify';
  truncate?: boolean;
  maxLines?: number;
  children: React.ReactNode;
}

// 测试用例
- 渲染正确的HTML标签
- 应用正确的样式类
- 支持文本截断
- 支持多行限制
- 支持代码高亮
```

**依赖**: 无

**优先级**: P0

**预计工时**: 1天

---

##### 3. Badge 组件

**文件**: `atoms/Badge/Badge.tsx`

```typescript
// 功能需求
- 支持多种颜色变体
- 支持圆点样式
- 支持自定义内容
- 支持图标组合

// 类型定义
interface BadgeProps {
  variant?: 'default' | 'primary' | 'success' | 'warning' | 'danger';
  size?: 'sm' | 'md' | 'lg';
  dot?: boolean;
  count?: number;
  maxCount?: number;
  children?: React.ReactNode;
}

// 测试用例
- 渲染正确的样式
- 显示正确的计数
- 支持圆点样式
- 支持最大计数限制
```

**依赖**: 无

**优先级**: P0

**预计工时**: 1天

---

##### 4. Divider 组件

**文件**: `atoms/Divider/Divider.tsx`

```typescript
// 功能需求
- 支持水平和垂直方向
- 支持虚线样式
- 支持文字标签
- 支持自定义间距

// 类型定义
interface DividerProps {
  orientation?: 'horizontal' | 'vertical';
  variant?: 'solid' | 'dashed';
  label?: string;
  spacing?: 'sm' | 'md' | 'lg';
}

// 测试用例
- 渲染正确的方向
- 应用正确的样式
- 支持文字标签
```

**依赖**: 无

**优先级**: P0

**预计工时**: 0.5天

---

##### 5. Tooltip 组件

**文件**: `atoms/Tooltip/Tooltip.tsx`

```typescript
// 功能需求
- 支持多方向弹出
- 支持延迟显示
- 支持自动定位
- 支持多种触发方式
- 可访问性支持

// 类型定义
interface TooltipProps {
  content: React.ReactNode;
  placement?: 'top' | 'bottom' | 'left' | 'right';
  delay?: number;
  trigger?: 'hover' | 'click' | 'focus';
  disabled?: boolean;
  children: React.ReactElement;
}

// 测试用例
- 正确显示提示内容
- 正确的弹出方向
- 支持延迟显示
- 支持键盘导航
```

**依赖**: 无

**优先级**: P0

**预计工时**: 2天

---

### Phase 2: 基础分子组件 (Week 3-4)

#### 目标组件

```bash
molecules/
├── Button/            # 从 ui/Button 迁移
├── Input/             # 新建
├── Select/            # 新建
├── Checkbox/          # 新建
├── Radio/             # 新建
├── Switch/            # 新建
├── TextArea/          # 新建
├── SearchInput/       # 新建
├── Dropdown/          # 新建
└── MenuItem/          # 新建
```

#### 迁移清单

##### 1. Button 组件 (迁移)

**源文件**: `components/ui/Button.tsx`
**目标文件**: `components/molecules/Button/Button.tsx`

**当前问题**:
- 缺少系统化的变体管理
- 缺少加载状态
- 缺少图标支持
- 缺少尺寸变体

**改进计划**:

```typescript
// 新增功能
+ 尺寸变体: xs, sm, md, lg, xl
+ 图标支持: leftIcon, rightIcon
+ 加载状态: loading
+ 全宽模式: fullWidth
+ 危险按钮: variant="danger"
+ 幽灵按钮: variant="ghost"
+ 禁用状态优化

// 迁移步骤
1. 创建 molecules/Button/ 目录
2. 定义 ButtonTypes.ts (变体、尺寸、状态)
3. 实现 Button.tsx (迁移现有逻辑)
4. 添加 Button.variants.tsx (样式变体)
5. 编写 Button.test.tsx
6. 创建 Button.examples.tsx
7. 更新所有引用

// 兼容性处理
// components/ui/Button.tsx (保留，标记废弃)
export { Button } from '../molecules/Button';
/**
 * @deprecated 使用 'components/molecules/Button' 代替
 */
```

**依赖**: Icon, Text (atoms)

**优先级**: P0

**预计工时**: 3天

**影响范围**:
- Toolbar/Toolbar.tsx
- PropertyInspector/PropertyInspector.tsx
- AssetBrowser/AssetBrowser.tsx
- 其他所有使用Button的地方

---

##### 2. Input 组件 (新建)

**文件**: `molecules/Input/Input.tsx`

```typescript
// 功能需求
- 基础输入功能
- 前缀/后缀图标
- 前缀/后缀文本
- 错误状态
- 禁用状态
- 尺寸变体
- 密码输入
- 清除按钮

// 类型定义
interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  size?: 'sm' | 'md' | 'lg';
  variant?: 'default' | 'error' | 'success';
  prefix?: React.ReactNode;
  suffix?: React.ReactNode;
  prefixIcon?: string;
  suffixIcon?: string;
  onClear?: () => void;
  error?: string;
}

// 测试用例
- 基础输入功能
- 前缀/后缀显示
- 错误状态样式
- 清除按钮功能
- 密码显示/隐藏
```

**依赖**: Icon, Text (atoms)

**优先级**: P0

**预计工时**: 2天

---

##### 3. Select 组件 (新建)

**文件**: `molecules/Select/Select.tsx`

```typescript
// 功能需求
- 单选/多选
- 搜索过滤
- 虚拟滚动
- 键盘导航
- 异步数据加载
- 自定义选项渲染

// 类型定义
interface SelectProps<T> {
  options: Option<T>[];
  value?: T | T[];
  onChange?: (value: T | T[]) => void;
  multiple?: boolean;
  searchable?: boolean;
  disabled?: boolean;
  placeholder?: string;
  loading?: boolean;
  renderOption?: (option: Option<T>) => React.ReactNode;
}

interface Option<T> {
  label: string;
  value: T;
  disabled?: boolean;
}

// 测试用例
- 单选功能
- 多选功能
- 搜索过滤
- 键盘导航
- 异步加载
```

**依赖**: Input, Icon, MenuItem (molecules)

**优先级**: P1

**预计工时**: 3天

---

### Phase 3: 核心有机体组件 (Week 5-6)

#### 目标组件

```bash
organisms/
├── Toolbar/           # 从 Toolbar/ 迁移
├── EntityTreeItem/    # 从 EntityTree/ 提取
├── PropertyGroup/     # 从 PropertyInspector/ 提取
├── AssetCard/         # 从 AssetBrowser/ 提取
├── CommandPalette/    # 新建
└── NotificationPanel/ # 新建
```

#### 迁移清单

##### 1. Toolbar 组件 (迁移)

**源文件**: `components/Toolbar/Toolbar.tsx`
**目标文件**: `components/organisms/Toolbar/Toolbar.tsx`

**当前代码分析**:

```typescript
// 现有功能
✅ 工具栏布局
✅ 按钮分组
✅ 工具提示

// 需要改进
+ 使用新的 Button 组件
+ 添加 Divider 组件
+ 支持自定义渲染
+ 添加工具栏项变体
+ 支持溢出菜单
+ 可访问性增强
```

**迁移步骤**:

```typescript
// 1. 提取 ToolbarItem 组件
interface ToolbarItemProps {
  id: string;
  type: 'button' | 'divider' | 'spacer';
  props?: any;
}

// 2. 重构 Toolbar 组件
interface ToolbarProps {
  items: ToolbarItemProps[];
  orientation?: 'horizontal' | 'vertical';
  size?: 'sm' | 'md' | 'lg';
  variant?: 'default' | 'compact';
}

// 3. 使用新的子组件
import { Button } from '../../molecules/Button';
import { Divider } from '../../atoms/Divider';

// 4. 更新引用
- import { Toolbar } from './Toolbar/Toolbar';
+ import { Toolbar } from './components/organisms/Toolbar';
```

**依赖**: Button, Divider, Icon, Tooltip (atoms/molecules)

**优先级**: P0

**预计工时**: 2天

**影响范围**:
- App.tsx
- EditorPage
- 所有使用工具栏的页面

---

##### 2. EntityTreeItem 组件 (提取)

**源文件**: `components/EntityTree/EntityTree.tsx`
**目标文件**: `components/organisms/EntityTreeItem/EntityTreeItem.tsx`

**提取理由**:
- EntityTreeItem 是独立的树节点组件
- 可以在多处复用
- 需要独立测试

**组件功能**:

```typescript
// 功能需求
- 显示实体名称和图标
- 显示子实体数量徽章
- 支持展开/折叠
- 支持选中状态
- 支持拖拽
- 支持上下文菜单

// 类型定义
interface EntityTreeItemProps {
  entity: Entity;
  level: number;
  isExpanded: boolean;
  isSelected: boolean;
  onToggle: () => void;
  onSelect: () => void;
  onContextMenu: (e: React.MouseEvent) => void;
}

// 测试用例
- 渲染实体信息
- 展开/折叠动画
- 选中状态样式
- 拖拽功能
- 上下文菜单触发
```

**依赖**: Icon, Badge, Checkbox (atoms/molecules)

**优先级**: P0

**预计工时**: 2天

---

##### 3. CommandPalette 组件 (新建)

**文件**: `organisms/CommandPalette/CommandPalette.tsx`

```typescript
// 功能需求
- 全局命令搜索 (类似 VS Code Cmd+Shift+P)
- 快捷键触发
- 模糊搜索
- 命令分类
- 最近使用
- 键盘导航
- 主题支持

// 类型定义
interface Command {
  id: string;
  label: string;
  description?: string;
  icon?: string;
  category?: string;
  shortcut?: string;
  action: () => void;
}

interface CommandPaletteProps {
  commands: Command[];
  recentCommands?: string[];
  placeholder?: string;
}

// 测试用例
- 快捷键触发
- 搜索过滤
- 键盘导航
- 命令执行
- 分类显示
```

**依赖**: Input, Icon, MenuItem (molecules)

**优先级**: P0

**预计工时**: 4天

---

### Phase 4: 模板和页面 (Week 7-8)

#### 目标组件

```bash
templates/
├── EditorLayout/      # 新建
├── PanelLayout/       # 新建
└── SplitViewLayout/   # 新建

pages/
├── HomePage/          # 从 App.tsx 提取
├── EditorPage/        # 新建
└── SettingsPage/      # 新建
```

#### 迁移清单

##### 1. EditorLayout 模板 (新建)

**文件**: `templates/EditorLayout/EditorLayout.tsx`

```typescript
// 功能需求
- 顶部工具栏
- 左侧实体树
- 中间视口
- 右侧属性面板
- 底部时间轴/控制台
- 可调整大小的面板
- 响应式布局

// 类型定义
interface EditorLayoutProps {
  header?: React.ReactNode;
  sidebar?: React.ReactNode;
  main?: React.ReactNode;
  inspector?: React.ReactNode;
  footer?: React.ReactNode;
  sidebarWidth?: number;
  inspectorWidth?: number;
}

// 布局结构
<div className="editor-layout">
  <header className="editor-header">...</header>
  <div className="editor-body">
    <aside className="editor-sidebar">...</aside>
    <main className="editor-viewport">...</main>
    <aside className="editor-inspector">...</aside>
  </div>
  <footer className="editor-footer">...</footer>
</div>
```

**依赖**: ResizablePanel (molecules)

**优先级**: P0

**预计工时**: 3天

---

##### 2. EditorPage 页面 (新建)

**文件**: `pages/EditorPage/EditorPage.tsx`

**迁移来源**: `App.tsx`

**步骤**:

```typescript
// 1. 从 App.tsx 提取编辑器相关逻辑
- 选择系统
- 工具栏
- 实体树
- 属性面板
- 视口
- 时间轴

// 2. 使用 EditorLayout 模板
import { EditorLayout } from '../../templates/EditorLayout';

// 3. 组装页面
<EditorLayout
  header={<Toolbar />}
  sidebar={<EntityTree />}
  main={<Viewport />}
  inspector={<PropertyInspector />}
  footer={<Timeline />}
/>

// 4. 页面级状态管理
const EditorPage = () => {
  const [selectedEntity, setSelectedEntity] = useState(null);
  const [history, setHistory] = useState([]);

  // ...业务逻辑
};
```

**依赖**: EditorLayout, Toolbar, EntityTree, Viewport, PropertyInspector, Timeline

**优先级**: P0

**预计工时**: 4天

---

## 测试策略

### 测试金字塔

```
        /\
       /  \         E2E Tests (5%)
      /    \        - 关键用户流程
     /------\       - 跨组件集成
    /        \
   /          \     Integration Tests (15%)
  /            \    - 组件间交互
 /              \   - 状态管理
/----------------\  - 数据流
/                  \
/                    \
/     Unit Tests     \ (80%)
/   (组件单元测试)    \
/                      \
- 组件渲染
- Props传递
- 事件处理
- 边界情况
```

### 测试覆盖率目标

| 层级 | 目标覆盖率 | 必测项 |
|------|-----------|--------|
| Atoms | 95%+ | 所有组件 |
| Molecules | 90%+ | 所有组件 |
| Organisms | 85%+ | 核心组件 |
| Templates | 80%+ | 布局结构 |
| Pages | 70%+ | 关键页面 |

### 测试工具栈

```json
{
  "testing-library": "@testing-library/react",
  "user-event": "@testing-library/user-event",
  "test-runner": "vitest",
  "coverage": "c8",
  "e2e": "playwright"
}
```

### 单元测试模板

```typescript
// ComponentName.test.tsx
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { ComponentName } from './ComponentName';

describe('ComponentName', () => {
  // 基础渲染测试
  it('renders correctly', () => {
    render(<ComponentName />);
    expect(screen.getByTestId('component-name')).toBeInTheDocument();
  });

  // Props测试
  it('applies correct variant class', () => {
    render(<ComponentName variant="primary" />);
    expect(screen.getByTestId('component-name')).toHaveClass('variant-primary');
  });

  // 交互测试
  it('calls onClick when clicked', () => {
    const handleClick = vi.fn();
    render(<ComponentName onClick={handleClick} />);
    fireEvent.click(screen.getByTestId('component-name'));
    expect(handleClick).toHaveBeenCalledTimes(1);
  });

  // 边界情况测试
  it('renders empty state correctly', () => {
    render(<ComponentName items={[]} />);
    expect(screen.getByText('No items')).toBeInTheDocument();
  });

  // 可访问性测试
  it('is accessible via keyboard', () => {
    render(<ComponentName />);
    const element = screen.getByTestId('component-name');
    element.focus();
    expect(element).toHaveFocus();
  });
});
```

---

## 风险评估

### 高风险项

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|---------|
| 破坏现有功能 | 高 | 中 | 1. 完整的测试覆盖<br>2. 增量迁移<br>3. 保留旧代码作为回退 |
| 性能下降 | 中 | 低 | 1. 性能基准测试<br>2. 代码分割<br>3. 懒加载 |
| 迁移周期过长 | 中 | 中 | 1. 并行开发<br>2. 分阶段发布<br>3. 优先级管理 |
| 团队适应期 | 低 | 高 | 1. 详细文档<br>2. 培训<br>3. 代码审查 |

### 回滚计划

#### 阶段性回滚

```bash
# 如果某个组件迁移失败，立即回滚
git revert <commit-hash>

# 恢复旧版本
git checkout <old-version-tag> -- components/ui/Button.tsx

# 重新发布
npm run build
npm run publish
```

#### 完整回滚

```bash
# 如果整个迁移失败，回滚到迁移前的版本
git checkout pre-migration-tag

# 或者使用功能开关
const FEATURE_ATOMIC_DESIGN = false;

// 使用旧组件
if (!FEATURE_ATOMIC_DESIGN) {
  module.exports = require('./legacy');
} else {
  module.exports = require('./new');
}
```

---

## 实施时间表

### Week 1-2: Phase 1 - 基础原子组件

```
Week 1:
├── Day 1-2: Icon 组件
├── Day 3: Text 组件
├── Day 4: Badge 组件
└── Day 5: Divider 组件

Week 2:
├── Day 1-2: Tooltip 组件
├── Day 3: Avatar 组件
├── Day 4: Spacer 组件
└── Day 5: ProgressBar 组件 + 集成测试
```

### Week 3-4: Phase 2 - 基础分子组件

```
Week 3:
├── Day 1-3: Button 组件 (迁移)
├── Day 4-5: Input 组件

Week 4:
├── Day 1-3: Select 组件
├── Day 4: Checkbox 组件
└── Day 5: Radio, Switch 组件
```

### Week 5-6: Phase 3 - 核心有机体组件

```
Week 5:
├── Day 1-2: Toolbar 组件 (迁移)
├── Day 3-4: EntityTreeItem 组件 (提取)
└── Day 5: PropertyGroup 组件

Week 6:
├── Day 1-4: CommandPalette 组件
└── Day 5: AssetCard 组件
```

### Week 7-8: Phase 4 - 模板和页面

```
Week 7:
├── Day 1-3: EditorLayout 模板
├── Day 4-5: PanelLayout, SplitViewLayout

Week 8:
├── Day 1-4: EditorPage 页面 (迁移)
└── Day 5: HomePage, SettingsPage + 最终测试
```

---

## 成功指标

### 代码质量指标

- [ ] 测试覆盖率 > 85%
- [ ] TypeScript 编译零错误
- [ ] ESLint 零警告
- [ ] 所有组件通过可访问性测试
- [ ] 性能基准测试通过

### 用户体验指标

- [ ] 首屏加载时间 < 2s
- [ ] 交互响应时间 < 100ms
- [ ] 无运行时错误
- [ ] 所有功能正常工作

### 开发效率指标

- [ ] 新组件开发时间减少 30%
- [ ] 组件复用率 > 60%
- [ ] 代码审查时间减少 40%
- [ ] Bug修复时间减少 50%

---

## 附录

### A. 迁移检查清单

每个组件迁移时使用：

```markdown
## [组件名] 迁移检查清单

### 准备阶段
- [ ] 阅读现有代码
- [ ] 绘制组件依赖图
- [ ] 识别需要改进的地方
- [ ] 编写迁移计划

### 实施阶段
- [ ] 创建新目录结构
- [ ] 编写/更新测试用例
- [ ] 实现新组件
- [ ] 添加类型定义
- [ ] 添加样式文件
- [ ] 添加导出文件
- [ ] 添加示例代码

### 验证阶段
- [ ] 运行单元测试
- [ ] 运行集成测试
- [ ] 手动功能测试
- [ ] 可访问性测试
- [ ] 性能测试
- [ ] 跨浏览器测试

### 集成阶段
- [ ] 更新引用路径
- [ ] 更新文档
- [ ] 代码审查
- [ ] 合并到主分支
- [ ] 标记旧组件为废弃

### 发布阶段
- [ ] 更新 CHANGELOG
- [ ] 发布新版本
- [ ] 通知团队
- [ ] 监控错误日志
```

### B. 相关文档

- [COMPONENT_ARCHITECTURE.md](./COMPONENT_ARCHITECTURE.md) - 组件架构设计
- [MIGRATION_GUIDE.md](./MIGRATION_GUIDE.md) - 开发者迁移指南
- [TESTING_GUIDE.md](./TESTING_GUIDE.md) - 测试指南

---

**文档版本**: 1.0.0
**最后更新**: 2026-01-04
**维护者**: Game Engine Editor Team
**审核状态**: 待审核
