# 批量操作功能集成指南

本文档说明如何在编辑器中集成和使用批量操作功能。

## 已实现的功能

### 1. 核心类型定义
**文件**: `/src/types/selection.ts`

定义了批量操作所需的所有类型：
- `SelectionState`: 选择状态
- `SelectionFilter`: 选择过滤器
- `BatchOperationOptions`: 批量操作选项
- `BulkEditResult`: 批量编辑结果
- `RenamePattern`: 重命名模式
- `AlignmentOptions`: 对齐选项
- `DistributionOptions`: 分布选项
- `MaterialBatchOperation`: 材质批量操作
- `ComponentBatchOperation`: 组件批量操作

### 2. 选择管理器
**文件**: `/src/utils/SelectionManager.ts`

提供完整的选择管理功能：
- 单选/多选/范围选择
- 框选（矩形选择）
- 全选/反选/取消选择
- 按过滤器选择
- 选择子实体/同级实体/相似实体
- 选择历史记录

### 3. 批量操作管理器
**文件**: `/src/utils/BatchOperation.ts`

实现所有批量操作：
- 批量删除
- 批量重命名（前缀、后缀、替换、编号）
- 批量移动/旋转/缩放
- 批量显示/隐藏
- 批量锁定/解锁
- 批量应用材质
- 批量组件操作

### 4. 对齐工具
**文件**: `/src/utils/AlignmentUtils.ts`

提供对齐和分布功能：
- 对齐到目标实体
- 对齐到网格
- 等距分布
- 网格布局
- 圆形布局
- 线性布局
- 匹配旋转/缩放
- 随机化位置/旋转/缩放

### 5. UI组件

#### 框选组件
**文件**: `/src/components/Viewport/SelectionBox.tsx`

- Shift+拖动进行框选
- 可视化选择框
- 自动筛选框内实体

#### 多选Gizmo
**文件**: `/src/components/Viewport/SelectionGizmo.tsx`

- 显示多选边界
- 中心点标记
- 坐标轴显示
- 批量变换控制

#### 批量属性编辑器
**文件**: `/src/components/PropertyInspector/BulkEditor.tsx`

- 批量变换编辑（位置、旋转、缩放）
- 批量组件操作
- 批量材质应用
- 批量重命名
- 进度显示

#### 批量操作工具栏
**文件**: `/src/components/Toolbar/BatchToolbar.tsx`

- 对齐工具（X/Y/Z轴，最小/最大/中心）
- 分布工具（等距分布）
- 布局工具（网格/圆形）
- 匹配工具（旋转/缩放）
- 选择工具（全选/反选）

### 6. 实体树更新
**文件**: `/src/components/EntityTree/EntityTree.tsx`

- Ctrl+Click: 多选
- Shift+Click: 范围选择
- 右键菜单保持单选行为

### 7. Rust后端
**文件**: `/src-tauri/src/batch_operations.rs`

高性能批量操作实现：
- 所有批量操作的Rust实现
- Tauri命令封装
- 类型安全的API
- 单元测试

## 集成步骤

### 1. 在App.tsx中初始化管理器

```typescript
import { SelectionManager } from './utils/SelectionManager';
import { BatchOperationManager } from './utils/BatchOperation';
import { HistoryManager } from './utils/HistoryManager';

function App() {
  const selectionManager = useMemo(() => new SelectionManager(), []);
  const historyManager = useMemo(() => new HistoryManager(), []);

  const batchOperationManager = useMemo(() => {
    return new BatchOperationManager(
      selectionManager,
      historyManager,
      getEntity,           // 从state获取实体的函数
      updateEntity,        // 更新实体的函数
      addEntity,           // 添加实体的函数
      removeEntity         // 删除实体的函数
    );
  }, [selectionManager, historyManager]);

  // ...
}
```

### 2. 在Viewport中集成框选

```tsx
import { SelectionBox } from './components/Viewport/SelectionBox';

function Viewport() {
  const viewportRef = useRef<HTMLDivElement>(null);

  return (
    <div ref={viewportRef} className="viewport">
      <SelectionBox
        viewportRef={viewportRef}
        onSelectionStart={() => console.log('Selection started')}
        onSelectionEnd={(ids) => console.log('Selected:', ids)}
      />
      {/* 其他视口内容 */}
    </div>
  );
}
```

### 3. 在PropertyInspector中使用批量编辑器

```tsx
import { BulkEditor } from './components/PropertyInspector/BulkEditor';

function PropertyInspector() {
  const selectedCount = selectionManager.getSelectedCount();

  if (selectedCount > 1) {
    return (
      <BulkEditor
        selectionManager={selectionManager}
        batchOperationManager={batchOperationManager}
        entities={entities}
        onEntityChange={handleEntityChange}
      />
    );
  }

  // 单个实体的属性编辑器
  return <SingleEntityEditor />;
}
```

### 4. 添加批量操作工具栏

```tsx
import { BatchToolbar, useBatchShortcuts } from './components/Toolbar/BatchToolbar';

function Editor() {
  // 启用快捷键
  useBatchShortcuts(selectionManager, batchOperationManager);

  return (
    <div className="editor">
      <BatchToolbar
        selectionManager={selectionManager}
        batchOperationManager={batchOperationManager}
        entities={entities}
      />
      {/* 其他编辑器UI */}
    </div>
  );
}
```

## 快捷键

- `Ctrl+A`: 全选
- `Ctrl+D`: 取消选择
- `Ctrl+I`: 反选
- `Shift+Click`: 范围选择（实体树）
- `Ctrl+Click`: 多选（实体树）
- `Shift+Drag`: 框选（视口）
- `Delete`: 删除选中
- `F2`: 批量重命名

## 使用示例

### 批量重命名

```typescript
// 批量添加前缀
await batchOperationManager.batchRename({
  mode: 'prefix',
  value: 'Enemy',
  startNumber: 1,
  padding: 3,
});

// 结果: Entity1 -> Enemy_001, Entity2 -> Enemy_002, ...

// 批量编号
await batchOperationManager.batchRename({
  mode: 'number',
  value: 'Waypoint',
  startNumber: 1,
  padding: 2,
});

// 结果: Waypoint_01, Waypoint_02, Waypoint_03, ...
```

### 批量对齐

```typescript
import { AlignmentUtils } from './utils/AlignmentUtils';

const selectedEntities = selectionManager.getSelectedEntities();

// 对齐到第一个实体的X位置
const updates = AlignmentUtils.alignEntities(selectedEntities, {
  axis: 'x',
  mode: 'min',
});

// 应用更新
Object.entries(updates).forEach(([id, update]) => {
  updateEntity(id, update);
});
```

### 批量分布

```typescript
// 等距分布（X轴）
const updates = AlignmentUtils.distributeEntities(selectedEntities, {
  axis: 'x',
  mode: 'equal',
});

// 自定义间距分布
const updates = AlignmentUtils.distributeEntities(selectedEntities, {
  axis: 'y',
  mode: 'custom',
  spacing: 2.0,
});
```

### 网格布局

```typescript
// 自动排列成网格
const columns = Math.ceil(Math.sqrt(selectedEntities.length));
const updates = AlignmentUtils.arrangeInGrid(
  selectedEntities,
  columns,
  { x: 2, y: 0, z: 2 }  // 间距
);
```

### 圆形布局

```typescript
// 排列成圆形
const updates = AlignmentUtils.arrangeInCircle(
  selectedEntities,
  5.0,  // 半径
  'y'   // 轴向
);
```

## 性能优化

### 大量实体选择（1000+）

1. **虚拟滚动**: 在实体树中使用虚拟滚动
2. **延迟渲染**: Gizmo和边界框延迟渲染
3. **批量更新**: 合并多次更新为单次操作
4. **Web Worker**: 将计算密集型操作移到Worker

```typescript
// 批量操作时显示进度
await batchOperationManager.batchMove(offset, {
  progressCallback: (current, total) => {
    console.log(`Processing ${current}/${total}`);
  },
});
```

### 确认阈值

```typescript
// 超过100个实体时要求确认
await batchOperationManager.batchDelete({
  confirmThreshold: 100,
});
```

## Rust后端调用

```typescript
import { invoke } from '@tauri-apps/api/tauri';

// 直接调用Rust批量操作
const result = await invoke<BulkEditResult>('batch_delete', {
  ids: selectedIds,
  options: {
    confirmThreshold: 100,
  },
});

console.log(`Deleted: ${result.succeeded.length}`);
console.log(`Failed: ${result.failed.length}`);
console.log(`Skipped: ${result.skipped.length}`);
```

## 测试

运行Rust后端测试：

```bash
cd src-tauri
cargo test batch_operations
```

测试文件位置：`src-tauri/src/batch_operations.rs`（测试模块在文件末尾）

## 文件结构总览

```
src/
├── types/
│   └── selection.ts                          # 选择系统类型（~150行）
├── utils/
│   ├── SelectionManager.ts                   # 选择管理器（~400行）
│   ├── BatchOperation.ts                     # 批量操作工具（~500行）
│   └── AlignmentUtils.ts                     # 对齐工具（~400行）
├── components/
│   ├── EntityTree/
│   │   └── EntityTree.tsx                    # 实体树（已更新支持多选）
│   ├── Viewport/
│   │   ├── SelectionBox.tsx                  # 框选组件（~200行）
│   │   └── SelectionGizmo.tsx                # 多选Gizmo（~300行）
│   ├── PropertyInspector/
│   │   └── BulkEditor.tsx                    # 批量属性编辑器（~400行）
│   └── Toolbar/
│       └── BatchToolbar.tsx                  # 批量操作工具栏（~300行）
└── App.tsx                                   # 主应用（需要集成）

src-tauri/src/
└── batch_operations.rs                       # Rust批量操作（~600行）

总计: ~3250行代码
```

## 未来扩展

### 待实现功能
1. 组/取消组（Ctrl+G / Ctrl+Shift+G）
2. 批量复制粘贴
3. 选择预设（保存和加载选择集）
4. 撤销/重做优化（合并连续操作）
5. 批量动画操作
6. 批量物理属性设置

### 性能优化
1. WebGL实例化渲染
2. 八叉树空间索引
3. LOD（细节级别）系统
4. 多线程实体更新

## 总结

批量操作功能已全面实现，包括：
- ✅ 完整的选择系统
- ✅ 所有基础批量操作
- ✅ 对齐和分布工具
- ✅ UI组件和交互
- ✅ Rust后端支持
- ✅ 撤销/重做集成
- ✅ 进度显示
- ✅ 快捷键支持

可以立即开始在编辑器中使用这些功能提升编辑效率！
