# EntityTree Component

A fully-featured, atomic component hierarchy for displaying and managing entity hierarchies in the game engine editor.

## Architecture

The EntityTree component follows atomic design principles, organized into the following structure:

```
EntityTree (Organism)
├── EntityTreeItem (Organism - Recursive)
│   ├── EntityTreeIcon (Atom)
│   ├── EntityTreeToggle (Atom)
│   ├── EntityTreeDragHandle (Atom)
│   ├── EntityTreeLabel (Atom)
│   └── EntityTreeActions (Molecule)
│       ├── Visibility Button
│       └── Lock Button
├── EntityTreeContextMenu (Molecule)
└── EntityTreeSearch (Atom)
```

## Component Structure

### Main Container: EntityTree

**Location:** `src/components/organisms/EntityTree/index.tsx`

The main container component that manages:
- Entity hierarchy state
- Selection state
- Search/filter functionality
- Context menu state
- Drag and drop state
- Inline editing state

**Props:**
```typescript
interface EntityTreeProps {
  entities: Entity[];                           // Root entities to display
  selectedEntities: string[];                   // Currently selected entity IDs
  onEntitySelect: (entityIds: string[]) => void; // Selection callback
  onEntityRename: (entityId: string, newName: string) => void;
  onEntityDelete: (entityId: string) => void;
  onEntityCreate: () => void;
  onEntityToggleVisibility: (entityId: string) => void;
  onEntityToggleLock: (entityId: string) => void;
  onEntityReorder?: (entityId: string, newIndex: number) => void;
  onEntityReparent?: (entityId: string, newParentId: string | null) => void;
  className?: string;
}
```

### Sub-Components

#### EntityTreeIcon

**Location:** `src/components/organisms/EntityTree/EntityTreeIcon/index.tsx`

Displays the entity icon in the hierarchy tree.

**Props:**
```typescript
interface EntityTreeIconProps {
  className?: string;
}
```

**Usage:**
```tsx
<EntityTreeIcon className="w-4 h-4 text-slate-400" />
```

#### EntityTreeToggle

**Location:** `src/components/organisms/EntityTree/EntityTreeToggle/index.tsx`

Displays expand/collapse toggle for entities with children.

**Props:**
```typescript
interface EntityTreeToggleProps {
  expanded: boolean;
  onToggle: () => void;
  className?: string;
}
```

**Usage:**
```tsx
<EntityTreeToggle
  expanded={isExpanded}
  onToggle={() => console.log('toggle')}
/>
```

#### EntityTreeDragHandle

**Location:** `src/components/organisms/EntityTree/EntityTreeDragHandle/index.tsx`

Visual spacer for alignment in draggable entities.

**Props:**
```typescript
interface EntityTreeDragHandleProps {
  className?: string;
}
```

#### EntityTreeActions

**Location:** `src/components/organisms/EntityTree/EntityTreeActions/index.tsx`

Displays visibility and lock toggle buttons.

**Props:**
```typescript
interface EntityTreeActionsProps {
  visible: boolean;
  locked: boolean;
  onToggleVisibility: (e: React.MouseEvent) => void;
  onToggleLock: (e: React.MouseEvent) => void;
  className?: string;
}
```

**Usage:**
```tsx
<EntityTreeActions
  visible={entity.visible}
  locked={entity.locked}
  onToggleVisibility={handleVisibilityToggle}
  onToggleLock={handleLockToggle}
/>
```

#### EntityTreeLabel

**Location:** `src/components/organisms/EntityTree/EntityTreeLabel/index.tsx`

Displays entity name with inline editing capability.

**Props:**
```typescript
interface EntityTreeLabelProps {
  name: string;
  isEditing: boolean;
  editName: string;
  onEditNameChange: (name: string) => void;
  onEditFinish: () => void;
  onEditCancel: () => void;
  onDoubleClick: () => void;
  className?: string;
}
```

**Usage:**
```tsx
<EntityTreeLabel
  name={entity.name}
  isEditing={isEditing}
  editName={editName}
  onEditNameChange={setEditName}
  onEditFinish={saveEdit}
  onEditCancel={cancelEdit}
  onDoubleClick={startEditing}
/>
```

#### EntityTreeContextMenu

**Location:** `src/components/organisms/EntityTree/EntityTreeContextMenu/index.tsx`

Right-click context menu for entity actions.

**Props:**
```typescript
interface EntityTreeContextMenuProps {
  visible: boolean;
  x: number;
  y: number;
  items: ContextMenuItem[];
  onClose: () => void;
}

interface ContextMenuItem {
  id: string;
  label: string;
  icon: React.ReactNode;
  onClick: () => void;
  danger?: boolean;
}
```

**Usage:**
```tsx
<EntityTreeContextMenu
  visible={contextMenu.visible}
  x={contextMenu.x}
  y={contextMenu.y}
  items={[
    {
      id: 'rename',
      label: 'Rename',
      icon: <Icon />,
      onClick: handleRename,
    },
    {
      id: 'delete',
      label: 'Delete',
      icon: <TrashIcon />,
      onClick: handleDelete,
      danger: true,
    },
  ]}
  onClose={closeContextMenu}
/>
```

#### EntityTreeSearch

**Location:** `src/components/organisms/EntityTree/EntityTreeSearch/index.tsx`

Search input for filtering entities.

**Props:**
```typescript
interface EntityTreeSearchProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  className?: string;
}
```

**Usage:**
```tsx
<EntityTreeSearch
  value={searchQuery}
  onChange={setSearchQuery}
  placeholder="Search entities..."
/>
```

## Features

### 1. Entity Selection
- **Single Selection**: Click on an entity
- **Multi-Selection**: Ctrl/Cmd + Click to toggle selection
- **Range Selection**: Shift + Click to select range
- **Clear Selection**: Click on empty space

### 2. Entity Management
- **Rename**: Double-click entity name or use context menu
- **Delete**: Use context menu or keyboard (Delete key)
- **Duplicate**: Use context menu (TODO: Implement)
- **Create**: Click "+ New" button in header

### 3. Visibility and Lock
- **Toggle Visibility**: Click eye icon
- **Toggle Lock**: Click lock icon
- **Visual Feedback**: Icons change based on state
- **Locked entities** cannot be dragged

### 4. Hierarchy Navigation
- **Expand/Collapse**: Click arrow icon
- **Auto-expand**: When searching or dropping entities
- **Indentation**: Visual depth indicator

### 5. Drag and Drop
- **Reparent**: Drag entity onto another entity
- **Reorder**: Drag to root level to reorder
- **Visual Feedback**: Drop targets highlighted
- **Lock Awareness**: Locked entities not draggable

### 6. Search and Filter
- **Real-time Filtering**: Search as you type
- **Auto-expand**: Entities with matching children auto-expand
- **Clear Button**: Quickly clear search
- **Empty States**: Helpful messages when no results

### 7. Context Menu
- **Rename**: Inline edit entity name
- **Duplicate**: Copy entity (TODO)
- **Toggle Visibility**: Show/hide entity
- **Delete**: Remove entity
- **Keyboard Support**: ESC to close

### 8. Accessibility
- **ARIA Attributes**: Proper roles and labels
- **Keyboard Navigation**: Full keyboard support
- **Screen Reader**: Descriptive labels
- **Focus Management**: Proper focus handling

## Usage Example

```tsx
import React, { useState } from 'react';
import { EntityTree } from './components/organisms/EntityTree';
import { Entity } from './types/engine';

function App() {
  const [entities, setEntities] = useState<Entity[]>([]);
  const [selectedEntities, setSelectedEntities] = useState<string[]>([]);

  const handleEntitySelect = (entityIds: string[]) => {
    setSelectedEntities(entityIds);
  };

  const handleEntityRename = (entityId: string, newName: string) => {
    // Update entity name in your state management
    console.log('Renaming', entityId, 'to', newName);
  };

  const handleEntityDelete = (entityId: string) => {
    // Remove entity from your state management
    console.log('Deleting', entityId);
  };

  const handleEntityCreate = () => {
    // Create new entity
    console.log('Creating new entity');
  };

  const handleEntityToggleVisibility = (entityId: string) => {
    // Toggle entity visibility
    console.log('Toggling visibility', entityId);
  };

  const handleEntityToggleLock = (entityId: string) => {
    // Toggle entity lock
    console.log('Toggling lock', entityId);
  };

  const handleEntityReparent = (entityId: string, newParentId: string | null) => {
    // Reparent entity
    console.log('Reparenting', entityId, 'to', newParentId);
  };

  return (
    <div style={{ height: '100vh' }}>
      <EntityTree
        entities={entities}
        selectedEntities={selectedEntities}
        onEntitySelect={handleEntitySelect}
        onEntityRename={handleEntityRename}
        onEntityDelete={handleEntityDelete}
        onEntityCreate={handleEntityCreate}
        onEntityToggleVisibility={handleEntityToggleVisibility}
        onEntityToggleLock={handleEntityToggleLock}
        onEntityReparent={handleEntityReparent}
      />
    </div>
  );
}

export default App;
```

## Styling

The component uses Tailwind CSS for styling. The color scheme follows the editor's dark theme:

- **Background**: `bg-slate-900`
- **Text**: `text-slate-200`
- **Hover**: `hover:bg-slate-700`
- **Selected**: `bg-blue-600`
- **Border**: `border-slate-700`
- **Icons**: `text-slate-400`

### Custom Styling

You can override styles by passing a `className` prop:

```tsx
<EntityTree
  className="custom-tree-styles"
  {...otherProps}
/>
```

## Performance Optimizations

1. **Memoization**: Uses `useMemo` and `useCallback` for expensive operations
2. **Virtual Scrolling**: Consider implementing for very large trees (TODO)
3. **Lazy Loading**: Children are only rendered when parent is expanded
4. **Filtered Search**: Search results are cached

## Testing

### Unit Tests

Location: `src/components/organisms/__tests__/EntityTree.test.tsx`

Run unit tests:
```bash
npm test -- EntityTree.test.tsx
```

### Integration Tests

Location: `src/components/organisms/__tests__/EntityTree.integration.test.tsx`

Run integration tests:
```bash
npm test -- EntityTree.integration.test.tsx
```

### Test Coverage

The test suite covers:
- Component rendering
- User interactions (click, double-click, right-click)
- Search and filtering
- Drag and drop
- Context menu
- Inline editing
- Multi-selection
- Accessibility
- Performance with large trees

## API Compatibility

The refactored EntityTree maintains 100% API compatibility with the original EntityTree component. All props, callbacks, and behaviors are identical.

## Migration Guide

If you're migrating from the old EntityTree component:

1. **No changes required!** The API is identical
2. Simply update the import path if needed:
   ```tsx
   // Old
   import { EntityTree } from './components/EntityTree/EntityTree';

   // New (optional, old path still works)
   import { EntityTree } from './components/organisms/EntityTree';
   ```

## Future Enhancements

- [ ] Virtual scrolling for large trees
- [ ] Entity drag preview image
- [ ] Custom context menu items
- [ ] Entity icons based on component types
- [ ] Keyboard shortcuts for common actions
- [ ] Entity breadcrumbs for deep hierarchies
- [ ] Batch operations on multiple entities
- [ ] Entity templates/presets

## Troubleshooting

### Entities not expanding
- Check if entities have children
- Verify `expandedEntities` state is being updated

### Drag and drop not working
- Ensure entities are not locked
- Check that `onEntityReparent` callback is provided

### Search not filtering
- Verify search query state is updated
- Check that `filteredEntities` memo is working

### Context menu not appearing
- Ensure `contextMenu.visible` is set to true
- Check z-index of context menu

## Contributing

When modifying the EntityTree component:

1. Maintain atomic component structure
2. Update tests for new features
3. Ensure accessibility compliance
4. Update this README with changes
5. Test with large entity trees (100+ entities)
6. Verify drag and drop still works

## License

Part of the Game Engine Editor project.
