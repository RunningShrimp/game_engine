# Atom Components

基础原子组件库 - 提供最简单、可复用的UI构建块。

## 概述

Atom组件是UI组件库的最小单元,设计简单、高度可复用且可组合。这些组件可以单独使用,也可以组合成更复杂的Molecule和Organism组件。

## 组件列表

### [Icon](./Icon/)
封装Lucide图标库的图标组件。

**特性:**
- 支持所有Lucide图标
- 可自定义大小、颜色、描边宽度
- 完整的可访问性支持

**示例:**
```tsx
<Icon name="Play" size={24} />
<Icon name="Settings" color="red" className="text-red-500" />
<Icon name="User" label="User icon" />
```

---

### [Text](./Text/)
统一的文本排版组件。

**特性:**
- 支持多种文本变体(h1-h6, p, span, label, code等)
- 文本颜色、字重、对齐方式
- 文本截断和多行显示

**示例:**
```tsx
<Text variant="h1">Heading 1</Text>
<Text variant="p">Paragraph text</Text>
<Text variant="label" color="primary">Label</Text>
<Text variant="code" weight="medium">Code snippet</Text>
<Text truncate lines={2}>Long text that will be truncated</Text>
```

---

### [Badge](./Badge/)
徽章组件,用于状态标识和标签。

**特性:**
- 多种颜色变体(default, primary, success, warning, error等)
- 三种尺寸(sm, md, lg)
- 支持图标和圆点指示器

**示例:**
```tsx
<Badge variant="success">Active</Badge>
<Badge variant="error" size="sm">Error</Badge>
<Badge variant="primary" icon={<Icon name="Star" />}>Featured</Badge>
<Badge variant="info" dot>New</Badge>
```

---

### [Avatar](./Avatar/)
头像组件,显示用户图片或首字母。

**特性:**
- 图片、首字母、图标后备方案
- 五种尺寸(xs, md, lg, xl)
- 圆形/方形形状
- 在线状态指示器

**示例:**
```tsx
<Avatar src="/avatar.png" alt="User name" />
<Avatar initials="JD" size="lg" />
<Avatar fallbackIcon={<Icon name="User" />} status="online" />
<Avatar initials="AB" shape="square" variant="ring" />
```

---

### [Divider](./Divider/)
分割线组件,用于视觉分隔。

**特性:**
- 水平/垂直方向
- 支持带文字的分割线
- 标签样式选项

**示例:**
```tsx
<Divider />
<Divider orientation="vertical" className="h-8" />
<Divider>Section Title</Divider>
<Divider label>Important Section</Divider>
```

---

### [Spacer](./Spacer/)
间距组件,创建一致的元素间距。

**特性:**
- 水平/垂直方向
- 预设尺寸(xs, sm, md, lg, xl, 2xl)
- 自定义尺寸支持
- Flex grow选项

**示例:**
```tsx
<Spacer size="md" />
<Spacer axis="vertical" size="lg" />
<Spacer grow /> {/* Flex grow spacer */}
<Spacer size="custom" value={32} />
```

---

### [Tooltip](./Tooltip/)
工具提示组件,悬停时显示额外信息。

**特性:**
- 四个位置(top, bottom, left, right)
- 可配置延迟
- 可禁用
- 最大宽度设置

**示例:**
```tsx
<Tooltip content="This is a tooltip">
  <button>Hover me</button>
</Tooltip>
<Tooltip content="Help text" position="right">
  <Icon name="HelpCircle" />
</Tooltip>
<Tooltip content="Disabled" disabled>
  <button>No tooltip</button>
</Tooltip>
```

---

### [ProgressBar](./ProgressBar/)
进度条组件,显示操作进度。

**特性:**
- 多种颜色变体
- 三种尺寸
- 百分比标签显示
- 条纹动画效果

**示例:**
```tsx
<ProgressBar value={50} />
<ProgressBar value={75} variant="success" showLabel />
<ProgressBar value={30} variant="warning" striped animated />
<ProgressBar value={90} size="lg" label="Processing..." />
```

---

### [Spinner](./Spinner/)
加载指示器组件。

**特性:**
- 五种尺寸(xs, sm, md, lg, xl)
- 四种颜色(primary, secondary, white, currentColor)
- 三种动画速度(slow, normal, fast)

**示例:**
```tsx
<Spinner size="md" color="primary" />
<Spinner size="lg" color="white" />
<Spinner size="sm" speed="fast" />
```

---

### [Skeleton](./Skeleton/)
骨架屏组件,内容加载占位符。

**特性:**
- 三种变体(text, rectangular, circular)
- 自定义尺寸
- 多行文本支持
- 可选动画

**预配置组件:**
- `CardSkeleton` - 卡片骨架屏
- `TableSkeleton` - 表格骨架屏

**示例:**
```tsx
// Text skeleton
<Skeleton variant="text" width="100%" height="20px" />

// Circular avatar skeleton
<Skeleton variant="circular" width="40px" height="40px" />

// Card skeleton
<Skeleton variant="rectangular" width="100%" height="200px" />

// Multiple text lines
<Skeleton variant="text" lines={3} />

// Pre-configured skeletons
<CardSkeleton />
<TableSkeleton rows={5} columns={4} />
```

---

## 使用指南

### 导入组件

```tsx
// 导入单个组件
import { Icon } from '@/components/atoms/Icon';
import { Text } from '@/components/atoms/Text';

// 导入多个组件
import { Icon, Text, Badge, Avatar } from '@/components/atoms';
```

### TypeScript类型

所有组件都导出完整的TypeScript类型定义:

```tsx
import type {
  IconProps,
  TextProps,
  TextVariant,
  BadgeProps,
  BadgeVariant,
  AvatarProps,
  AvatarSize
} from '@/components/atoms';
```

### 可访问性

所有Atom组件都遵循WCAG 2.1 AA级标准:

- **ARIA属性**: 所有组件都有适当的ARIA角色和属性
- **键盘导航**: 支持键盘操作
- **屏幕阅读器**: 完整的屏幕阅读器支持
- **焦点管理**: 正确的焦点指示

### 样式自定义

所有组件都接受`className` prop用于自定义样式:

```tsx
<Badge className="my-custom-badge" variant="primary">
  Custom Badge
</Badge>
```

使用`cn`工具函数组合样式:

```tsx
import { cn } from '@/lib/utils';

<Badge className={cn(
  'base-class',
  condition && 'conditional-class',
  'another-class'
)} />
```

## 设计原则

1. **单一职责**: 每个组件只做一件事,并做好它
2. **可组合性**: 组件可以轻松组合成更复杂的UI
3. **可定制性**: 通过props和className提供灵活的自定义
4. **可访问性**: 内置完整的可访问性支持
5. **类型安全**: 完整的TypeScript类型定义

## 最佳实践

### DO ✅

```tsx
// 使用语义化的变体
<Text variant="h1">Page Title</Text>
<Text variant="p">Body text</Text>

// 提供有意义的标签
<Icon name="Settings" label="Open settings" />

// 使用合适的颜色传递状态
<Badge variant="success">Completed</Badge>
<Badge variant="error">Failed</Badge>
```

### DON'T ❌

```tsx
// 不要嵌套文本组件
<Text variant="h1">
  <Text variant="bold">Title</Text>
</Text>

// 不要滥用图标
<div>
  <Icon name="Star" />
  <Icon name="Star" />
  <Icon name="Star" />
</div>

// 不要使用颜色仅用于装饰
<Badge variant="primary">Normal status</Badge>
```

## 测试

所有组件都有完整的单元测试覆盖:

```bash
# 运行所有Atom组件测试
npm test src/components/atoms

# 运行特定组件测试
npm test src/components/atoms/Icon
```

## 贡献

添加新的Atom组件时,请确保:

1. 创建独立的组件目录(`ComponentName/index.tsx`)
2. 添加完整的TypeScript类型和JSDoc注释
3. 实现`displayName`用于调试
4. 创建单元测试文件(`ComponentName.test.tsx`)
5. 在`atoms/index.ts`中导出组件和类型
6. 更新此README文档

## 相关资源

- [组件库文档](../../README.md)
- [Molecule组件](../molecules/README.md)
- [Organism组件](../organisms/README.md)
- [设计系统](../../../docs/design-system.md)

## License

MIT
