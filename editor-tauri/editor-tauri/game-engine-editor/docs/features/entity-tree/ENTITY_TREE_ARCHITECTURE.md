# EntityTree 组件架构图

## 组件层次结构

```
EntityTree (Organism)
│
├── Header Section
│   ├── Title ("Scene Hierarchy")
│   ├── Create Button
│   └── Search Input
│       └── EntityTreeSearch (Atom)
│
├── Tree Container
│   └── Entity List (Recursive)
│       └── EntityTreeItem (Organism - Recursive)
│           ├── EntityTreeToggle (Atom) [if has children]
│           ├── EntityTreeDragHandle (Atom) [if no children]
│           ├── EntityTreeIcon (Atom)
│           ├── EntityTreeLabel (Atom)
│           │   └── Edit Mode Input (if editing)
│           └── EntityTreeActions (Molecule)
│               ├── Visibility Button (Atom)
│               └── Lock Button (Atom)
│           └── Children (Recursive EntityTreeItem)
│
└── Context Menu (Portal)
    └── EntityTreeContextMenu (Molecule)
        ├── Rename Item
        ├── Duplicate Item
        ├── Toggle Visibility Item
        ├── Separator
        └── Delete Item
```

## 数据流图

```
User Input
    │
    ├─→ Click
    │   └─→ onEntitySelect (Callback)
    │       └─→ Update selectedEntities
    │           └─→ Re-render EntityTreeItem
    │
    ├─→ Double Click
    │   └─→ Start Editing Mode
    │       └─→ Show EntityTreeLabel Input
    │           ├─→ Enter → onEntityRename
    │           └─→ Escape → Cancel
    │
    ├─→ Right Click
    │   └─→ Show EntityTreeContextMenu
    │       └─→ Click Item
    │           └─→ Execute Action
    │               └─→ Close Menu
    │
    ├─→ Type in Search
    │   └─→ Update searchQuery
    │       └─→ Filter Entities
    │           └─→ Auto-expand Matches
    │
    ├─→ Drag Start
    │   └─→ Set draggedEntity
    │       └─→ Show Visual Feedback
    │
    └─→ Drop
        └─→ onEntityReparent (Callback)
            └─→ Update Entity Hierarchy
                └─→ Auto-expand Target
```

## 状态管理

```
EntityTree State
│
├── expandedEntities: Set<string>
│   └─→ Which entities are expanded
│
├── editingEntity: string | null
│   └─→ Currently editing entity ID
│
├── editName: string
│   └─→ Temporary edit value
│
├── searchQuery: string
│   └─→ Current search filter
│
├── contextMenu
│   ├── visible: boolean
│   ├── x: number
│   ├── y: number
│   └─→ entityId: string | null
│
├── draggedEntity: string | null
│   └─→ Currently dragged entity ID
│
└── dropTarget: string | null
    └─→ Current drop target ID
```

## Props 流转

```
Parent Component
    │
    ├─→ entities: Entity[]
    │   └─→ EntityTree
    │       └─→ filteredEntities (memoized)
    │           └─→ EntityTreeItem (recursive)
    │
├─→ selectedEntities: string[]
│   └─→ EntityTree
│       └─→ EntityTreeItem
│           └─→ Check if selected
│
└─→ Callbacks
    ├── onEntitySelect
    ├── onEntityRename
    ├── onEntityDelete
    ├── onEntityCreate
    ├── onEntityToggleVisibility
    ├── onEntityToggleLock
    └─→ onEntityReparent
```

## 组件职责划分

### EntityTree (Organism)
**职责:**
- 主容器和状态管理
- 协调所有子组件
- 处理复杂的用户交互
- 搜索和过滤逻辑

**不应:**
- 直接渲染UI细节
- 处理单个实体项的样式

### EntityTreeItem (Organism)
**职责:**
- 渲染单个实体项
- 管理单个实体的交互
- 递归渲染子实体

**不应:**
- 管理全局状态
- 处理跨实体逻辑

### EntityTreeIcon (Atom)
**职责:**
- 显示实体图标

**不应:**
- 处理任何交互
- 管理状态

### EntityTreeToggle (Atom)
**职责:**
- 显示展开/折叠按钮
- 触发展开/折叠事件

**不应:**
- 管理展开状态
- 渲染子实体

### EntityTreeActions (Molecule)
**职责:**
- 显示可见性和锁定按钮
- 处理按钮点击事件

**不应:**
- 管理实体状态
- 处理业务逻辑

### EntityTreeLabel (Atom)
**职责:**
- 显示实体名称
- 处理内联编辑UI

**不应:**
- 保存编辑结果（回调给父组件）
- 管理编辑状态

### EntityTreeContextMenu (Molecule)
**职责:**
- 显示上下文菜单
- 处理菜单项点击
- 管理菜单位置和可见性

**不应:**
- 执行业务逻辑（回调给父组件）
- 管理菜单数据

### EntityTreeSearch (Atom)
**职责:**
- 显示搜索输入框
- 提供清除按钮

**不应:**
- 执行搜索过滤（回调给父组件）
- 管理搜索结果

## 设计模式

### 1. 容器/展示模式
```
EntityTree (Container)
    └─→ EntityTreeItem (Presenter)
        └─→ 子组件 (Presenters)
```

### 2. 组合模式
```
EntityTree
    └─→ EntityTreeItem
        └─→ EntityTreeItem (recursive)
```

### 3. 受控组件模式
```
Parent
    └─→ EntityTree (controlled)
        └─→ All state from props
```

### 4. 回调模式
```
Child Component
    └─→ Trigger Callback
        └─→ Parent Handles Logic
            └─→ Update Props
                └─→ Child Re-renders
```

## 性能优化策略

### 1. Memoization
```typescript
const EntityTree = React.memo<EntityTreeProps>((props) => {
  // Component implementation
});
```

### 2. Callback Stability
```typescript
const handleEntityClick = useCallback((e, id) => {
  // Handler logic
}, [dependencies]);
```

### 3. Computed Values
```typescript
const filteredEntities = useMemo(() => {
  return filterEntities(entities, searchQuery);
}, [entities, searchQuery]);
```

### 4. Lazy Rendering
```typescript
{hasChildren && isExpanded && (
  <div>
    {entity.children.map(child => renderEntity(child))}
  </div>
)}
```

## 测试策略

### 单元测试金字塔
```
        /\
       /  \        Integration Tests
      /____\       (25+ scenarios)
     /      \
    /        \     Component Tests
   /__________\    (30+ test cases)
  /            \
 /  Atom Tests  \  (Basic rendering)
/________________\
```

### 测试覆盖范围
- ✅ 所有原子组件
- ✅ 所有分子组件
- ✅ 所有用户交互
- ✅ 所有边界情况
- ✅ 性能测试
- ✅ 无障碍测试

## 扩展点

### 1. 自定义图标
```typescript
<EntityTreeIcon className="custom-icon" />
```

### 2. 自定义菜单项
```typescript
const customItems: ContextMenuItem[] = [
  // Your custom items
];
<EntityTreeContextMenu items={customItems} />
```

### 3. 自定义搜索
```typescript
<EntityTreeSearch
  placeholder="Custom placeholder"
  onChange={handleCustomSearch}
/>
```

### 4. 样式覆盖
```typescript
<EntityTree className="custom-styles" />
```

## 总结

EntityTree的架构遵循以下原则:

1. **单一职责**: 每个组件只做一件事
2. **开闭原则**: 对扩展开放，对修改关闭
3. **依赖倒置**: 依赖抽象（Props），不依赖具体实现
4. **组合优于继承**: 使用组合构建复杂UI
5. **接口隔离**: Props接口最小化
6. **迪米特法则**: 组件间通信通过回调

这种架构使得组件:
- ✅ 易于理解
- ✅ 易于测试
- ✅ 易于维护
- ✅ 易于扩展
- ✅ 易于复用
