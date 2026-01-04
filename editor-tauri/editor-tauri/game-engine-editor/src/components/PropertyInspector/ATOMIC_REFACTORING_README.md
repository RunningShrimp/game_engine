# PropertyInspector 原子化重构文档

## 概述

PropertyInspector组件已完成原子化重构，将其从一个大型单体组件拆分为多个可复用的原子(Ampl)和分子(Molecule)组件。这一重构提升了代码的可维护性、可测试性和可复用性。

## 架构设计

### 组件层次结构

```
PropertyInspector (Organism)
├── EntityInfo (Molecule)
│   ├── Input (Molecule)
│   ├── Checkbox (Molecule)
│   └── Text (Atom)
├── TransformEditor (Molecule)
│   ├── Vector3Input (Molecule)
│   │   ├── NumberInput (Molecule)
│   │   └── Text (Atom)
│   └── Divider (Atom)
└── ComponentList (Molecule)
    ├── ComponentItem (Molecule)
    │   ├── Badge (Atom)
    │   ├── Icon (Atom)
    │   ├── Checkbox (Molecule)
    │   └── Text (Atom)
    └── Button (Molecule)
```

## 新组件详解

### 1. Vector3Input (Molecule)

**文件位置**: `/src/components/molecules/Vector3Input/`

**功能**: 用于编辑3D向量值（位置、旋转、缩放）

**特性**:
- 三个带颜色编码的输入框（X=红色, Y=绿色, Z=蓝色）
- 可配置的步进值(step)
- 支持最小/最大值约束
- 可选的精度控制
- 禁用状态支持
- 两种颜色方案：RGB 或 Slate

**Props**:
```typescript
interface Vector3InputProps {
  label: string;
  value: { x: number; y: number; z: number };
  onChange: (axis: 'x' | 'y' | 'z', newValue: number) => void;
  step?: number;
  disabled?: boolean;
  colorScheme?: 'rgb' | 'slate';
  min?: number;
  max?: number;
  precision?: number;
}
```

**使用示例**:
```tsx
<Vector3Input
  label="Position"
  value={{ x: 0, y: 0, z: 0 }}
  onChange={(axis, value) => console.log(`${axis}: ${value}`)}
  step={0.1}
  colorScheme="rgb"
/>
```

**Stories**:
- `Position` - 位置编辑（默认步进0.1）
- `Rotation` - 旋转编辑（步进1）
- `Scale` - 缩放编辑（步进0.01）
- `Disabled` - 禁用状态
- `WithPrecision` - 带精度控制
- `WithConstraints` - 带约束
- `Interactive` - 交互式示例

---

### 2. TransformEditor (Molecule)

**文件位置**: `/src/components/molecules/TransformEditor/`

**功能**: 编辑实体的完整变换属性（位置、旋转、缩放）

**特性**:
- 使用Vector3Input显示三个变换属性
- 支持世界/局部坐标系切换
- 可配置的步进值
- 显示坐标系指示器
- 禁用状态支持

**Props**:
```typescript
interface TransformEditorProps {
  transform: Transform;
  onChange: (transform: Transform) => void;
  coordinateSpace?: 'world' | 'local';
  disabled?: boolean;
  stepValues?: {
    position?: number;
    rotation?: number;
    scale?: number;
  };
  showCoordinateSpace?: boolean;
}
```

**使用示例**:
```tsx
<TransformEditor
  transform={entity.transform}
  onChange={(newTransform) => setEntityTransform(newTransform)}
  coordinateSpace="world"
  showCoordinateSpace={true}
  stepValues={{
    position: 0.1,
    rotation: 1,
    scale: 0.01
  }}
/>
```

**Stories**:
- `WorldSpace` - 世界坐标系
- `LocalSpace` - 局部坐标系
- `WithTransformValues` - 带变换值
- `Disabled` - 禁用状态
- `CustomStepValues` - 自定义步进值
- `Interactive` - 交互式示例

---

### 3. EntityInfo (Molecule)

**文件位置**: `/src/components/molecules/EntityInfo/`

**功能**: 显示和编辑实体基本信息（名称、ID、可见性、锁定状态）

**特性**:
- 实体名称内联编辑（Enter提交，Escape取消）
- 显示只读实体ID
- 可见性和锁定状态显示
- 可选字段显示控制
- 支持名称变更回调

**Props**:
```typescript
interface EntityInfoProps {
  entity: Entity;
  onNameChange?: (newName: string) => void;
  allowNameEdit?: boolean;
  showId?: boolean;
  showVisibility?: boolean;
  showLocked?: boolean;
}
```

**使用示例**:
```tsx
<EntityInfo
  entity={selectedEntity}
  onNameChange={(newName) => renameEntity(entity.id, newName)}
  allowNameEdit={true}
  showId={true}
  showVisibility={true}
  showLocked={true}
/>
```

**Stories**:
- `Default` - 默认状态
- `NoNameEdit` - 禁止名称编辑
- `Minimal` - 仅显示名称和ID
- `HiddenAndLocked` - 隐藏和锁定实体
- `LongName` - 长名称
- `SpecialCharacters` - 特殊字符
- `Interactive` - 交互式示例

---

### 4. ComponentItem (Molecule)

**文件位置**: `/src/components/molecules/ComponentItem/`

**功能**: 显示和管理单个组件

**特性**:
- 可折叠/展开的面板
- 显示组件名称和类型徽章
- 启用/禁用切换
- 属性列表显示
- 支持属性编辑（可选）
- 根据属性类型自动渲染输入控件

**Props**:
```typescript
interface ComponentItemProps {
  component: Component;
  onToggle?: (enabled: boolean) => void;
  onPropertyChange?: (propertyKey: string, value: any) => void;
  allowPropertyEdit?: boolean;
  defaultExpanded?: boolean;
  showTypeBadge?: boolean;
  showEnabledToggle?: boolean;
}
```

**使用示例**:
```tsx
<ComponentItem
  component={meshRendererComponent}
  onToggle={(enabled) => toggleComponent(component.id, enabled)}
  onPropertyChange={(key, value) => updateProperty(component.id, key, value)}
  allowPropertyEdit={true}
  defaultExpanded={true}
/>
```

**Stories**:
- `Default` - 默认状态
- `Collapsed` - 折叠状态
- `Disabled` - 禁用组件
- `EditableProperties` - 可编辑属性
- `NoProperties` - 无属性
- `MixedPropertyTypes` - 混合属性类型
- `NestedProperties` - 嵌套属性
- `Interactive` - 交互式示例

---

### 5. ComponentList (Molecule)

**文件位置**: `/src/components/molecules/ComponentList/`

**功能**: 显示和管理实体的所有组件

**特性**:
- 组件数量统计
- 添加组件按钮
- 删除组件功能（悬停显示）
- 空状态提示
- 组件列表管理

**Props**:
```typescript
interface ComponentListProps {
  components: Component[];
  onToggle?: (componentId: string, enabled: boolean) => void;
  onPropertyChange?: (componentId: string, propertyKey: string, value: any) => void;
  onAddComponent?: () => void;
  onRemoveComponent?: (componentId: string) => void;
  allowPropertyEdit?: boolean;
  showAddButton?: boolean;
  showRemoveButtons?: boolean;
  emptyText?: string;
}
```

**使用示例**:
```tsx
<ComponentList
  components={entity.components}
  onToggle={(componentId, enabled) => toggleComponent(componentId, enabled)}
  onPropertyChange={(componentId, key, value) => updateProperty(componentId, key, value)}
  onAddComponent={() => addNewComponent()}
  onRemoveComponent={(componentId) => removeComponent(componentId)}
  allowPropertyEdit={true}
  showAddButton={true}
  showRemoveButtons={true}
/>
```

**Stories**:
- `Default` - 默认状态
- `Empty` - 空列表
- `EditableProperties` - 可编辑属性
- `WithRemoveButtons` - 带删除按钮
- `SingleComponent` - 单个组件
- `ManyComponents` - 多个组件
- `AllDisabled` - 全部禁用
- `Interactive` - 交互式示例

---

### 6. PropertyInspector (Organism - 重构版)

**文件位置**: `/src/components/PropertyInspector/PropertyInspector.refactored.tsx`

**功能**: 实体属性检查器主组件

**架构改进**:
- 使用EntityInfo显示实体信息
- 使用TransformEditor编辑变换
- 使用ComponentList管理组件
- 清晰的职责分离
- 更好的代码组织

**Props**:
```typescript
interface PropertyInspectorProps {
  entities: Entity[];
  selectedEntities: string[];
  onTransformChange: (entityId: string, transform: Transform) => void;
  onComponentToggle?: (entityId: string, componentId: string, enabled: boolean) => void;
  onEntityRename?: (entityId: string, name: string) => void;
  onComponentPropertyChange?: (
    entityId: string,
    componentId: string,
    propertyKey: string,
    value: any
  ) => void;
  allowPropertyEdit?: boolean;
  coordinateSpace?: 'world' | 'local';
}
```

**使用示例**:
```tsx
<PropertyInspector
  entities={sceneEntities}
  selectedEntities={selectedEntityIds}
  onTransformChange={handleTransformChange}
  onComponentToggle={handleComponentToggle}
  onEntityRename={handleEntityRename}
  onComponentPropertyChange={handlePropertyChange}
  allowPropertyEdit={true}
  coordinateSpace="world"
/>
```

**Stories**:
- `Default` - 默认状态
- `NoSelection` - 无选中实体
- `MultipleSelection` - 多选（显示第一个）
- `EditableProperties` - 可编辑属性
- `NoComponents` - 无组件
- `LocalSpace` - 局部坐标系
- `CustomTransform` - 自定义变换
- `HiddenAndLocked` - 隐藏和锁定
- `Interactive` - 完整交互式示例

---

## 重构收益

### 1. 可复用性
- Vector3Input可用于任何需要3D向量编辑的场景
- EntityInfo可用于任何显示实体信息的UI
- TransformEditor可作为独立组件嵌入其他面板

### 2. 可维护性
- 每个组件职责单一，易于理解
- 修改某个功能不影响其他部分
- 更容易定位和修复bug

### 3. 可测试性
- 每个组件可独立测试
- 更容易编写单元测试
- 测试覆盖率提升

### 4. 可扩展性
- 添加新功能更容易（如新类型的属性编辑器）
- 组件组合灵活
- 支持不同的使用场景

### 5. 代码质量
- 减少代码重复
- 提高代码可读性
- 遵循SOLID原则

## 使用示例

### 完整集成示例

```tsx
import { PropertyInspector } from '@/components/PropertyInspector';

function SceneEditor() {
  const [entities, setEntities] = useState<Entity[]>([]);
  const [selectedIds, setSelectedIds] = useState<string[]>([]);

  const handleTransformChange = (entityId: string, transform: Transform) => {
    setEntities(
      entities.map(e =>
        e.id === entityId ? { ...e, transform } : e
      )
    );
  };

  const handleComponentToggle = (
    entityId: string,
    componentId: string,
    enabled: boolean
  ) => {
    setEntities(
      entities.map(e =>
        e.id === entityId
          ? {
              ...e,
              components: e.components.map(c =>
                c.id === componentId ? { ...c, enabled } : c
              )
            }
          : e
      )
    );
  };

  return (
    <PropertyInspector
      entities={entities}
      selectedEntities={selectedIds}
      onTransformChange={handleTransformChange}
      onComponentToggle={handleComponentToggle}
      allowPropertyEdit={true}
    />
  );
}
```

### 独立使用Vector3Input

```tsx
import { Vector3Input } from '@/components/molecules';

function VelocityEditor() {
  const [velocity, setVelocity] = useState({ x: 0, y: 0, z: 0 });

  return (
    <Vector3Input
      label="Velocity"
      value={velocity}
      onChange={(axis, value) =>
        setVelocity({ ...velocity, [axis]: value })
      }
      step={0.1}
      colorScheme="slate"
    />
  );
}
```

### 自定义组件列表

```tsx
import { ComponentList } from '@/components/molecules';

function ComponentManager() {
  return (
    <ComponentList
      components={entity.components}
      onToggle={(componentId, enabled) => {
        console.log('Toggle component:', componentId, enabled);
      }}
      onAddComponent={() => {
        console.log('Add new component');
      }}
      onRemoveComponent={(componentId) => {
        console.log('Remove component:', componentId);
      }}
      showRemoveButtons={true}
      emptyText="No components - add one to get started"
    />
  );
}
```

## 迁移指南

### 从旧版PropertyInspector迁移

**旧代码**:
```tsx
import { PropertyInspector } from './components/PropertyInspector';

<PropertyInspector
  entities={entities}
  selectedEntities={selectedIds}
  onTransformChange={handleChange}
  onComponentToggle={handleToggle}
  onEntityRename={handleRename}
/>
```

**新代码**:
```tsx
import { PropertyInspector } from './components/PropertyInspector';

// API保持兼容，无需修改使用代码
<PropertyInspector
  entities={entities}
  selectedEntities={selectedIds}
  onTransformChange={handleChange}
  onComponentToggle={handleToggle}
  onEntityRename={handleRename}
  allowPropertyEdit={true}  // 新增：启用属性编辑
  coordinateSpace="world"    // 新增：指定坐标系
/>
```

**新功能**:
- 添加了`onComponentPropertyChange`回调用于属性编辑
- 添加了`allowPropertyEdit`标志控制是否允许编辑属性
- 添加了`coordinateSpace`选择坐标系

## 测试

所有组件都包含完整的Storybook Stories，涵盖：
- 默认状态
- 空状态
- 禁用状态
- 交互状态
- 边界情况
- 完整的交互式示例

运行Storybook查看所有示例：
```bash
npm run storybook
```

访问：
- PropertyInspector: http://localhost:6006/?path=/story/organisms-propertyinspector
- TransformEditor: http://localhost:6006/?path=/story/molecules-transformeditor
- Vector3Input: http://localhost:6006/?path=/story/molecules-vector3input
- EntityInfo: http://localhost:6006/?path=/story/molecules-entityinfo
- ComponentList: http://localhost:6006/?path=/story/molecules-componentlist
- ComponentItem: http://localhost:6006/?path=/story/molecules-componentitem

## 性能优化

1. **组件memoization**: 所有子组件使用React.memo包装
2. **回调优化**: 使用useCallback优化回调函数
3. **状态管理**: 本地状态最小化，通过props传递
4. **条件渲染**: 仅在需要时渲染复杂组件

## 未来改进

1. **批量编辑**: 支持同时编辑多个实体
2. **撤销/重做**: 集成撤销/重做功能
3. **复制/粘贴**: 支持属性值的复制粘贴
4. **预设值**: 添加常用变换预设
5. **动画曲线**: 为属性值添加动画编辑器
6. **验证**: 添加属性值验证和错误提示
7. **搜索过滤**: 为组件列表添加搜索功能
8. **拖拽排序**: 支持组件拖拽重排

## 文件结构

```
src/components/
├── PropertyInspector/
│   ├── PropertyInspector.tsx              (旧版，保留用于兼容)
│   ├── PropertyInspector.refactored.tsx   (新版重构)
│   ├── PropertyInspector.stories.tsx      (Stories)
│   └── BulkEditor.tsx                     (批量编辑器)
├── molecules/
│   ├── Vector3Input/
│   │   ├── Vector3Input.tsx
│   │   ├── Vector3Input.stories.tsx
│   │   └── index.ts
│   ├── TransformEditor/
│   │   ├── TransformEditor.tsx
│   │   ├── TransformEditor.stories.tsx
│   │   └── index.ts
│   ├── EntityInfo/
│   │   ├── EntityInfo.tsx
│   │   ├── EntityInfo.stories.tsx
│   │   └── index.ts
│   ├── ComponentItem/
│   │   ├── ComponentItem.tsx
│   │   ├── ComponentItem.stories.tsx
│   │   └── index.ts
│   └── ComponentList/
│       ├── ComponentList.tsx
│       ├── ComponentList.stories.tsx
│       └── index.ts
└── atoms/
    ├── (已存在的原子组件)
```

## 贡献指南

如果您想对这些组件进行改进：

1. 遵循原子设计原则
2. 保持组件单一职责
3. 添加完整的TypeScript类型
4. 编写Storybook Stories
5. 确保可访问性
6. 保持API一致性

## 许可证

MIT License - 详见项目根目录LICENSE文件

## 作者

Game Engine Editor Team

---

**最后更新**: 2026-01-04
**版本**: 1.0.0
**状态**: 已完成 ✅
