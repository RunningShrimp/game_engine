# 组件迁移开发者指南

> 开发者快速上手Atomic Design组件架构的实用指南

## 目录

1. [快速开始](#快速开始)
2. [迁移步骤](#迁移步骤)
3. [常见场景](#常见场景)
4. [最佳实践](#最佳实践)
5. [故障排除](#故障排除)
6. [API参考](#api参考)

---

## 快速开始

### 安装依赖

```bash
# 确保安装了必要的依赖
npm install clsx @radix-ui/react-icons
npm install -D @testing-library/react @testing-library/user-event
```

### 目录结构概览

```bash
src/components/
├── atoms/          # UI基础元素 (Button的基础部分)
│   ├── Icon/
│   └── Text/
├── molecules/      # 可复用的UI组件 (Button, Input等)
│   ├── Button/
│   └── Input/
└── organisms/      # 复杂的业务组件 (Toolbar, Panel等)
    ├── Toolbar/
    └── PropertyInspector/
```

### 基础导入方式

```typescript
// ✅ 推荐 - 从层级索引导入
import { Button } from '@/components/molecules';
import { Icon, Text } from '@/components/atoms';

// ✅ 可选 - 直接从组件导入
import { Button } from '@/components/molecules/Button';

// ❌ 不推荐 - 从绝对路径导入
import { Button } from '/Users/.../components/molecules/Button/Button';
```

---

## 迁移步骤

### 步骤1: 识别待迁移组件

检查你的代码，找到使用旧组件路径的地方：

```typescript
// 旧的导入路径
import { Button } from '../components/ui/Button';
import { Toolbar } from '../components/Toolbar/Toolbar';
```

**工具**: 使用搜索命令找到所有旧导入

```bash
# 查找所有旧组件导入
grep -r "from.*components/ui" src/
grep -r "from.*Toolbar" src/
```

---

### 步骤2: 更新导入路径

将旧的导入路径更新为新路径：

```typescript
// 旧路径
import { Button } from '../components/ui/Button';

// 新路径
import { Button } from '@/components/molecules';
// 或者
import { Button } from '@/components/molecules/Button';
```

**路径映射表**:

| 旧路径 | 新路径 |
|--------|--------|
| `components/ui/Button` | `components/molecules/Button` |
| `components/ui/Spinner` | `components/atoms/Spinner` |
| `components/ui/Skeleton` | `components/atoms/Skeleton` |
| `components/Toolbar` | `components/organisms/Toolbar` |
| `components/EntityTree` | `components/organisms/EntityTree` |

---

### 步骤3: 检查API变化

一些组件的API可能发生了变化。查看组件的类型定义：

```typescript
// Button 的新 API
import type { ButtonProps } from '@/components/molecules/Button';

interface ButtonProps {
  variant?: 'primary' | 'secondary' | 'ghost' | 'danger';
  size?: 'xs' | 'sm' | 'md' | 'lg' | 'xl';
  leftIcon?: string;      // 新增: 左侧图标
  rightIcon?: string;     // 新增: 右侧图标
  loading?: boolean;      // 新增: 加载状态
  fullWidth?: boolean;    // 新增: 全宽模式
  onClick?: () => void;
  children: React.ReactNode;
}
```

---

### 步骤4: 更新组件使用

根据新的API更新你的代码：

```typescript
// 旧代码
<Button
  variant="primary"
  onClick={handleClick}
>
  Save
</Button>

// 新代码 - 利用新增的功能
<Button
  variant="primary"
  size="md"
  leftIcon="save"
  onClick={handleClick}
>
  Save
</Button>
```

---

### 步骤5: 测试你的更改

运行测试确保一切正常工作：

```bash
# 运行单元测试
npm test

# 运行类型检查
npm run type-check

# 运行应用
npm run dev
```

---

## 常见场景

### 场景1: 简单的Button导入更新

**之前**:
```typescript
import { Button } from '../../ui/Button';

export const MyComponent = () => {
  return (
    <Button onClick={handleSave}>
      Save
    </Button>
  );
};
```

**之后**:
```typescript
import { Button } from '@/components/molecules';

export const MyComponent = () => {
  return (
    <Button
      variant="primary"
      leftIcon="save"
      onClick={handleSave}
    >
      Save
    </Button>
  );
};
```

---

### 场景2: 使用Icon组件

**之前**:
```typescript
// 使用内联SVG或第三方图标库
<svg>
  <path d="..." />
</svg>
```

**之后**:
```typescript
import { Icon } from '@/components/atoms';

// 使用统一的Icon组件
<Icon name="save" size="md" color="currentColor" />
```

**可用图标**:
- Lucide Icons: `save`, `load`, `delete`, `edit`, `search`, etc.
- 自定义图标: 添加到 `icons/index.ts`

---

### 场景3: 复杂的Toolbar迁移

**之前**:
```typescript
import { Toolbar } from '../Toolbar/Toolbar';
import { Button } from '../ui/Button';

export const EditorToolbar = () => {
  return (
    <Toolbar>
      <Button onClick={handleSave}>Save</Button>
      <Button onClick={handleLoad}>Load</Button>
      <Button onClick={handleUndo}>Undo</Button>
    </Toolbar>
  );
};
```

**之后**:
```typescript
import { Toolbar } from '@/components/organisms';
import { Button } from '@/components/molecules';
import { Divider } from '@/components/atoms';

export const EditorToolbar = () => {
  const items = [
    { id: 'save', label: 'Save', icon: 'save', onClick: handleSave },
    { id: 'load', label: 'Load', icon: 'load', onClick: handleLoad },
    { type: 'divider' },
    { id: 'undo', label: 'Undo', icon: 'undo', onClick: handleUndo },
  ];

  return <Toolbar items={items} orientation="horizontal" />;
};
```

---

### 场景4: 输入框组件迁移

**之前**:
```typescript
import { Input } from '../some-lib';

<Input
  value={name}
  onChange={e => setName(e.target.value)}
  placeholder="Enter name"
/>
```

**之后**:
```typescript
import { Input } from '@/components/molecules';

<Input
  value={name}
  onChange={setName}
  placeholder="Enter name"
  prefixIcon="user"
  error={errors.name}
  size="md"
/>
```

---

### 场景5: 实体树项迁移

**之前**:
```typescript
// 直接在EntityTree中实现
<div className="entity-item" onClick={onSelect}>
  <span className="entity-name">{entity.name}</span>
  <span className="entity-count">{entity.children.length}</span>
</div>
```

**之后**:
```typescript
import { EntityTreeItem } from '@/components/organisms';

<EntityTreeItem
  entity={entity}
  level={level}
  isSelected={selectedId === entity.id}
  onSelect={() => setSelectedEntity(entity.id)}
/>
```

---

## 最佳实践

### 1. 使用TypeScript类型

```typescript
// ✅ 推荐 - 导入并使用类型
import type { ButtonProps } from '@/components/molecules/Button';

const MyButton = (props: ButtonProps) => {
  return <Button {...props} />;
};

// ❌ 不推荐 - 重复定义类型
interface MyButtonProps {
  variant?: string;
  onClick?: () => void;
  // ...
}
```

---

### 2. 利用组件组合

```typescript
// ✅ 推荐 - 使用组合模式
<Card>
  <Card.Header>
    <Title>User Profile</Title>
  </Card.Header>
  <Card.Body>
    <UserInfo />
  </Card.Body>
  <Card.Footer>
    <Button>Edit</Button>
  </Card.Footer>
</Card>

// ❌ 不推荐 - 使用大量props
<Card
  title="User Profile"
  body={<UserInfo />}
  footer={<Button>Edit</Button>}
/>
```

---

### 3. 遵循命名约定

```typescript
// ✅ 推荐 - 清晰的命名
const UserAvatar = () => <Avatar src={user.photo} />;
const SaveButton = () => <Button leftIcon="save">Save</Button>;

// ❌ 不推荐 - 模糊的命名
const Component1 = () => <Avatar src={user.photo} />;
const Comp2 = () => <Button leftIcon="save">Save</Button>;
```

---

### 4. 正确处理事件

```typescript
// ✅ 推荐 - 使用事件处理器
const handleClick = () => {
  console.log('Button clicked');
};

<Button onClick={handleClick}>Click me</Button>

// ❌ 不推荐 - 内联函数 (每次渲染创建新函数)
<Button onClick={() => console.log('clicked')}>Click me</Button>
```

---

### 5. 条件渲染

```typescript
// ✅ 推荐 - 清晰的条件渲染
{isLoading && <Spinner />}
{error && <Alert variant="error">{error}</Alert>}

// ❌ 不推荐 - 复杂的三元运算符
{isLoading ? <Spinner /> : error ? <Alert>{error}</Alert> : null}
```

---

### 6. 使用辅助函数

```typescript
// ✅ 推荐 - 使用clsx或cn工具
import { cn } from '@/utils/cn';

<div className={cn(
  'base-class',
  isActive && 'active-class',
  variant && `variant-${variant}`
)} />

// ❌ 不推荐 - 手动拼接字符串
<div className={`base-class ${isActive ? 'active-class' : ''} variant-${variant}`} />
```

---

## 故障排除

### 问题1: 导入错误

**错误信息**:
```
Module not found: Can't resolve '@/components/molecules/Button'
```

**解决方案**:

1. 检查路径别名配置:

```typescript
// tsconfig.json
{
  "compilerOptions": {
    "paths": {
      "@/*": ["./src/*"]
    }
  }
}
```

2. 检查vite配置:

```typescript
// vite.config.ts
import path from 'path';

export default {
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
};
```

3. 使用相对路径作为备选:

```typescript
// 如果路径别名不工作，使用相对路径
import { Button } from '../../../components/molecules/Button';
```

---

### 问题2: TypeScript类型错误

**错误信息**:
```
Type 'string' is not assignable to type 'ButtonVariant'
```

**解决方案**:

使用正确的类型值:

```typescript
// ❌ 错误
<Button variant="invalid">Click</Button>

// ✅ 正确
<Button variant="primary">Click</Button>
```

查看类型定义:

```typescript
import type { ButtonProps } from '@/components/molecules/Button';

// ButtonProps.variant 的类型是:
type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'danger';
```

---

### 问题3: 样式丢失

**错误现象**: 组件渲染了但样式不对

**解决方案**:

1. 确保导入了样式:

```typescript
// main.tsx
import './styles/index.css'; // 全局样式
```

2. 检查CSS Modules配置:

```typescript
// vite.config.ts
export default {
  css: {
    modules: {
      localsConvention: 'camelCase',
    },
  },
};
```

3. 检查组件的className:

```typescript
// 确保正确应用className
<div className={styles.container}>
```

---

### 问题4: Props不匹配

**错误现象**: 组件不接受某些props

**解决方案**:

查看组件的类型定义:

```typescript
// 查看组件支持哪些props
import type { ButtonProps } from '@/components/molecules/Button';

// ButtonProps 包含:
interface ButtonProps {
  variant?: ButtonVariant;
  size?: ButtonSize;
  leftIcon?: string;
  rightIcon?: string;
  onClick?: () => void;
  children: React.ReactNode;
}
```

如果需要额外的props, 扩展组件:

```typescript
interface ExtendedButtonProps extends ButtonProps {
  extraProp?: string;
}

const ExtendedButton = (props: ExtendedButtonProps) => {
  const { extraProp, ...buttonProps } = props;
  return <Button {...buttonProps} />;
};
```

---

### 问题5: 性能问题

**错误现象**: 应用变慢或频繁重新渲染

**解决方案**:

1. 使用React.memo:

```typescript
import { memo } from 'react';

export const MyComponent = memo(({ data }) => {
  return <div>{data.map(...)}</div>;
});
```

2. 使用useMemo和useCallback:

```typescript
const memoizedValue = useMemo(() => {
  return expensiveCalculation(data);
}, [data]);

const handleClick = useCallback(() => {
  console.log('clicked');
}, []); // 空依赖数组，函数不会重新创建
```

3. 懒加载组件:

```typescript
import { lazy, Suspense } from 'react';

const HeavyComponent = lazy(() => import('./HeavyComponent'));

<Suspense fallback={<Spinner />}>
  <HeavyComponent />
</Suspense>
```

---

## API参考

### Button组件

```typescript
import { Button } from '@/components/molecules';

interface ButtonProps {
  // 按钮变体
  variant?: 'primary' | 'secondary' | 'ghost' | 'danger';

  // 按钮尺寸
  size?: 'xs' | 'sm' | 'md' | 'lg' | 'xl';

  // 图标
  leftIcon?: string;
  rightIcon?: string;

  // 状态
  loading?: boolean;
  disabled?: boolean;

  // 布局
  fullWidth?: boolean;

  // 事件
  onClick?: (e: React.MouseEvent) => void;

  // 内容
  children: React.ReactNode;
}
```

**示例**:

```typescript
// 基础按钮
<Button onClick={handleClick}>Click me</Button>

// 带图标的按钮
<Button leftIcon="save" onClick={handleSave}>
  Save
</Button>

// 加载状态
<Button loading={isLoading} disabled>
  Submitting...
</Button>

// 危险操作
<Button variant="danger" onClick={handleDelete}>
  Delete
</Button>
```

---

### Icon组件

```typescript
import { Icon } from '@/components/atoms';

interface IconProps {
  // 图标名称
  name: string;

  // 尺寸
  size?: 'xs' | 'sm' | 'md' | 'lg' | 'xl';

  // 颜色
  color?: string;

  // 旋转动画
  spin?: boolean;

  // 自定义类名
  className?: string;
}
```

**示例**:

```typescript
// 基础图标
<Icon name="save" />

// 自定义大小和颜色
<Icon name="delete" size="lg" color="red" />

// 旋转动画
<Icon name="loading" spin />
```

---

### Input组件

```typescript
import { Input } from '@/components/molecules';

interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  // 尺寸
  size?: 'sm' | 'md' | 'lg';

  // 变体
  variant?: 'default' | 'error' | 'success';

  // 前缀/后缀
  prefix?: React.ReactNode;
  suffix?: React.ReactNode;
  prefixIcon?: string;
  suffixIcon?: string;

  // 状态
  error?: string;
  loading?: boolean;

  // 值变化
  onChange?: (value: string) => void;
}
```

**示例**:

```typescript
// 基础输入
<Input value={name} onChange={setName} />

// 带错误提示
<Input
  value={email}
  onChange={setEmail}
  error="Invalid email format"
/>

// 带图标
<Input
  value={search}
  onChange={setSearch}
  prefixIcon="search"
  placeholder="Search..."
/>
```

---

### Toolbar组件

```typescript
import { Toolbar } from '@/components/organisms';

interface ToolbarProps {
  // 工具栏项
  items: ToolbarItem[];

  // 方向
  orientation?: 'horizontal' | 'vertical';

  // 尺寸
  size?: 'sm' | 'md' | 'lg';

  // 变体
  variant?: 'default' | 'compact';
}

interface ToolbarItem {
  id: string;
  type?: 'button' | 'divider' | 'spacer';
  label?: string;
  icon?: string;
  onClick?: () => void;
  disabled?: boolean;
  active?: boolean;
}
```

**示例**:

```typescript
const toolbarItems = [
  { id: 'save', label: 'Save', icon: 'save', onClick: handleSave },
  { type: 'divider' },
  { id: 'undo', label: 'Undo', icon: 'undo', onClick: handleUndo },
  { id: 'redo', label: 'Redo', icon: 'redo', onClick: handleRedo },
];

<Toolbar items={toolbarItems} orientation="horizontal" />
```

---

## 迁移检查清单

使用这个清单确保迁移完整:

```markdown
## [组件名] 迁移检查清单

### 导入更新
- [ ] 更新导入路径
- [ ] 更新类型导入
- [ ] 移除未使用的导入

### API更新
- [ ] 检查Props变化
- [ ] 更新Props使用
- [ ] 处理废弃的API

### 功能测试
- [ ] 基础功能正常
- [ ] 边界情况处理
- [ ] 错误处理正确

### 视觉检查
- [ ] 样式正确
- [ ] 响应式正常
- [ ] 动画流畅

### 性能检查
- [ ] 无性能退化
- [ ] 无内存泄漏
- [ ] 控制台无错误

### 代码质量
- [ ] TypeScript无错误
- [ ] ESLint无警告
- [ ] 测试通过
```

---

## 获取帮助

### 资源链接

- 📖 [完整架构文档](./COMPONENT_ARCHITECTURE.md)
- 📋 [迁移计划](./COMPONENT_MIGRATION_PLAN.md)
- 💬 [团队讨论区](内部链接)
- 🐛 [问题追踪](内部链接)

### 联系方式

- 架构负责人: [姓名] ([邮箱])
- 技术支持: [邮箱]
- Slack频道: #component-migration

---

**文档版本**: 1.0.0
**最后更新**: 2026-01-04
**维护者**: Game Engine Editor Team
