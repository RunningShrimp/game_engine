# Accessibility Quick Start Guide

快速实施可访问性功能的简明指南。

## 快速开始 (3步)

### 步骤1: 导入样式

在 `src/main.tsx` 中添加:

```tsx
import './styles/accessibility.css';
```

### 步骤2: 使用可访问组件

替换组件导入:

```tsx
// App.tsx
import App from './App-accessible';

// 或者替换具体组件
import { EntityTree } from './components/EntityTree/EntityTree-accessible';
```

### 步骤3: 使用可访问性工具

在组件中导入工具函数:

```tsx
import { announceToScreenReader, generateId } from './utils/accessibility';

// 使用示例
const handleClick = () => {
  doSomething();
  announceToScreenReader('操作成功', 'polite');
};

const buttonId = generateId('button');
```

## 常用模式

### 1. 为按钮添加可访问性

```tsx
<button
  onClick={handleClick}
  aria-label="创建新实体"
  aria-pressed={isPressed}
  disabled={isDisabled}
>
  {isPressed ? '已按下' : '未按下'}
</button>
```

### 2. 实现模态框

```tsx
import { useFocusTrap } from '../hooks/useFocusTrap';

function Modal({ isOpen, onClose }) {
  const { containerRef } = useFocusTrap(isOpen);

  if (!isOpen) return null;

  return (
    <div
      ref={containerRef}
      role="dialog"
      aria-modal="true"
      aria-labelledby="modal-title"
      className="fixed inset-0 bg-black bg-opacity-50"
    >
      <div className="bg-white rounded-lg p-6">
        <h2 id="modal-title" className="text-xl font-bold mb-4">
          标题
        </h2>
        <p>内容</p>
        <button onClick={onClose}>关闭</button>
      </div>
    </div>
  );
}
```

### 3. 实现键盘导航

```tsx
const handleKeyDown = (e: React.KeyboardEvent) => {
  switch (e.key) {
    case 'Enter':
    case ' ':
      e.preventDefault();
      handleSelect();
      break;
    case 'Escape':
      e.preventDefault();
      handleCancel();
      break;
    case 'ArrowDown':
      e.preventDefault();
      handleNext();
      break;
    case 'ArrowUp':
      e.preventDefault();
      handlePrevious();
      break;
  }
};

<div onKeyDown={handleKeyDown} tabIndex={0} role="listbox">
  {items}
</div>
```

### 4. 通知屏幕阅读器

```tsx
import { announceToScreenReader } from '../utils/accessibility';

// 成功消息
const handleSuccess = () => {
  saveData();
  announceToScreenReader('数据保存成功', 'polite');
};

// 错误消息
const handleError = () => {
  showError();
  announceToScreenReader('发生错误,请重试', 'assertive');
};

// 状态更新
const handleSelectionChange = (count: number) => {
  announceToScreenReader(`已选择${count}个项目`, 'polite');
};
```

### 5. 管理焦点

```tsx
import { setFocus, setFocusToFirst } from '../utils/accessibility';

// 聚焦到特定元素
const openModal = () => {
  setShowModal(true);
  setTimeout(() => {
    const closeButton = document.getElementById('modal-close');
    if (closeButton) {
      setFocus(closeButton);
    }
  }, 100);
};

// 聚焦到第一个元素
const openDropdown = () => {
  setShowDropdown(true);
  setTimeout(() => {
    const container = document.getElementById('dropdown-menu');
    if (container) {
      setFocusToFirst(container);
    }
  }, 100);
};
```

## 键盘快捷键速查表

### 全局快捷键

| 快捷键 | 功能 |
|--------|------|
| `Tab` | 下一个元素 |
| `Shift+Tab` | 上一个元素 |
| `Escape` | 取消/关闭 |
| `Enter` | 确认/选择 |
| `Space` | 激活按钮 |
| `Ctrl/Cmd + Z` | 撤销 |
| `Ctrl/Cmd + Shift + Z` | 重做 |

### 导航快捷键

| 快捷键 | 功能 |
|--------|------|
| `ArrowUp/Down` | 上下导航 |
| `ArrowLeft/Right` | 展开/折叠 |
| `Home` | 第一个项目 |
| `End` | 最后一个项目 |
| `PageUp/Down` | 快速翻页 |

## ARIA速查表

### 常用角色

```tsx
<!-- 应用 -->
<div role="application" aria-label="游戏引擎编辑器">

<!-- 树形结构 -->
<div role="tree" aria-label="场景层级">

<!-- 树节点 -->
<div role="treeitem" aria-expanded="true">

<!-- 对话框 -->
<div role="dialog" aria-modal="true">

<!-- 按钮 -->
<button aria-pressed="true">

<!-- 列表 -->
<div role="listbox">

<!-- 列表项 -->
<div role="option" aria-selected="true">

<!-- 状态 -->
<div role="status" aria-live="polite">

<!-- 警告 -->
<div role="alert" aria-live="assertive">
```

### 常用属性

```tsx
<!-- 标签 -->
aria-label="关闭"
aria-labelledby="modal-title"

<!-- 描述 -->
aria-describedby="modal-description"

<!-- 状态 -->
aria-selected="true"
aria-checked="false"
aria-expanded="true"
aria-pressed="false"
aria-disabled="true"

<!-- 实时区域 -->
aria-live="polite"        // 礼貌模式
aria-live="assertive"     // 紧急模式
aria-atomic="true"        // 整体更新

<!-- 关联 -->
aria-controls="panel-1"
aria-haspopup="true"
aria-haspopup="menu"

<!-- 值 -->
aria-valuenow="50"
aria-valuemin="0"
aria-valuemax="100"

<!-- 级别和位置 -->
aria-level="2"
aria-setsize="10"
aria-posinset="5"
```

## 检查清单

使用此清单快速验证可访问性:

### 基础检查

- [ ] 所有交互元素可通过Tab访问
- [ ] 焦点指示器清晰可见
- [ ] 所有按钮有aria-label或文本
- [ ] 所有输入有对应的label
- [ ] 键盘可完成所有操作
- [ ] Escape键可关闭模态框

### ARIA检查

- [ ] 使用语义化HTML角色
- [ ] 状态变化有ARIA属性
- [ ] 动态内容有aria-live
- [ ] 模态框有aria-modal="true"
- [ ] 隐藏内容有aria-hidden

### 对比度检查

- [ ] 文本对比度 >= 4.5:1
- [ ] 大文本对比度 >= 3:1
- [ ] 交互元素对比度 >= 3:1
- [ ] 焦点指示器对比度 >= 3:1

### 测试检查

- [ ] 在Chrome中测试
- [ ] 在Firefox中测试
- [ ] 在Safari中测试
- [ ] 用屏幕阅读器测试
- [ ] 仅用键盘测试

## 常见问题

### Q: 如何为图标按钮添加标签?

```tsx
<button
  onClick={handleClick}
  aria-label="关闭对话框"
  title="关闭"
>
  <XIcon />
</button>
```

### Q: 如何通知屏幕阅读器状态变化?

```tsx
import { announceToScreenReader } from '../utils/accessibility';

// 状态变化时
const [isLoading, setIsLoading] = useState(false);

useEffect(() => {
  if (isLoading) {
    announceToScreenReader('正在加载', 'polite');
  } else {
    announceToScreenReader('加载完成', 'polite');
  }
}, [isLoading]);
```

### Q: 如何实现可访问的下拉菜单?

```tsx
import { useFocusTrap } from '../hooks/useFocusTrap';

function Dropdown({ isOpen, onClose }) {
  const { containerRef } = useFocusTrap(isOpen);

  return isOpen ? (
    <div
      ref={containerRef}
      role="menu"
      aria-label="选项菜单"
      onKeyDown={(e) => {
        if (e.key === 'Escape') onClose();
      }}
    >
      <button role="menuitem" onClick={() => { /* action 1 */ }}>
        选项1
      </button>
      <button role="menuitem" onClick={() => { /* action 2 */ }}>
        选项2
      </button>
    </div>
  ) : null;
}
```

### Q: 如何为表格添加可访问性?

```tsx
<table>
  <caption>用户列表</caption>
  <thead>
    <tr>
      <th scope="col">姓名</th>
      <th scope="col">邮箱</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <th scope="row">张三</th>
      <td>zhangsan@example.com</td>
    </tr>
  </tbody>
</table>
```

## 工具推荐

### 浏览器扩展

1. **axe DevTools** - 自动检测可访问性问题
2. **WAVE** - 可视化可访问性评估
3. **Lighthouse** - 综合性能和可访问性测试

### 在线工具

1. **WebAIM Contrast Checker** - 颜色对比度检查
2. **Colour Contrast Analyser** - 桌面应用程序
3. **ARIA Validator** - ARIA属性验证

### 屏幕阅读器

1. **NVDA** (Windows) - 免费
2. **VoiceOver** (macOS) - 内置
3. **JAWS** (Windows) - 商业

## 进阶资源

- [完整可访问性指南](./ACCESSIBILITY_README.md)
- [WCAG 2.1 快速参考](https://www.w3.org/WAI/WCAG21/quickref/)
- [ARIA 实践指南](https://www.w3.org/WAI/ARIA/apg/)

## 获取帮助

如需帮助:

1. 查看完整文档: `ACCESSIBILITY_README.md`
2. 查看代码示例: `src/App-accessible.tsx`
3. 查看工具函数: `src/utils/accessibility.ts`
4. 查看样式文件: `src/styles/accessibility.css`

---

**记住**: 可访问性是对所有人的改进,不仅是对残障用户。良好的可访问性实践也改善了所有用户体验!
