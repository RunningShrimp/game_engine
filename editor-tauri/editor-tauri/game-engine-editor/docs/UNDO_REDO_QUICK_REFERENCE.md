# Undo/Redo System - Quick Reference

## Quick Setup

```typescript
import { HistoryManager } from './utils/HistoryManager';
import { HistoryPersistence } from './utils/HistoryPersistence';

// With persistence
const persistence = new HistoryPersistence({ enabled: true, autoSave: true });
const history = new HistoryManager(100, persistence);

// Without persistence
const history = new HistoryManager(100);
```

## Basic Operations

```typescript
// Execute command
await history.executeCommand(command);

// Undo/Redo
await history.undo();
await history.redo();

// Check availability
history.canUndo(); // boolean
history.canRedo(); // boolean
```

## Advanced Commands

### Batch
```typescript
import { BatchCommand } from './commands';

const batch = new BatchCommand(commands, { mode: 'parallel' });
await history.executeCommand(batch);
```

### Transaction
```typescript
import { TransactionCommand } from './commands';

const txn = new TransactionCommand(commands, {
  rollbackOnFailure: true
});
await history.executeCommand(txn);
```

### Macro
```typescript
import { MacroCommand } from './commands';

const macro = new MacroCommand('Name', commands, params);
await history.executeCommand(macro);
```

### Conditional
```typescript
import { ConditionalCommand } from './commands';

const cond = new ConditionalCommand(
  () => condition,
  thenCommand,
  elseCommand
);
await history.executeCommand(cond);
```

### Delayed
```typescript
import { DelayedCommand } from './commands';

const delayed = new DelayedCommand(command, 5000); // 5s delay
await history.executeCommand(delayed);
```

## Bookmarks

```typescript
// Create
const bookmark = history.createBookmark('Name', 'Desc', ['tag1']);

// Jump
await history.jumpToBookmark(bookmark.id);

// List
const bookmarks = history.getBookmarks();

// Delete
history.deleteBookmark(bookmark.id);
```

## Branches

```typescript
// Create
const branch = history.createBranch('experimental', parentId);

// Switch
await history.switchBranch(branch.id);

// List
const branches = history.getBranches();

// Delete
history.deleteBranch(branchId);
```

## Search & Filter

```typescript
// Search
const results = history.search('query');
console.log(results.commands, results.bookmarks);

// Filter
const filtered = history.filter({
  dateRange: { start, end },
  commandTypes: ['CreateEntityCommand'],
  tags: ['important'],
});

// Compare
const diff = history.compare(stateId1, stateId2);
```

## Persistence

```typescript
// Save/Load
await history.save();
await history.load();

// Export
const blob = await history.export('json');
// Download blob...

// Import
await history.import(file);
```

## Statistics

```typescript
const stats = history.getStatistics();
console.log(stats.totalCommands);
console.log(stats.mostUsedCommandTypes);
console.log(stats.memoryUsage);
```

## Optimization

```typescript
await history.optimize({
  mergeConsecutiveCommands: true,
  removeDuplicateCommands: true,
  keepBookmarks: true,
});
```

## UI Usage

```typescript
import { HistoryPanel } from './components/HistoryPanel';

<HistoryPanel
  historyManager={history}
  onClose={() => setShowHistory(false)}
/>
```

## Subscribe to Changes

```typescript
const unsubscribe = history.subscribe((state) => {
  console.log('History updated:', state);
});

// Later
unsubscribe();
```

## Command Registry

```typescript
import { getGlobalRegistry } from './utils/CommandRegistry';

const registry = getGlobalRegistry();

// Register
registry.register('MyCommand', factory, metadata);

// Create
const cmd = registry.create('MyCommand', ...args);

// Statistics
const stats = registry.getStatistics('MyCommand');
```

## Common Patterns

### Execute with Error Handling
```typescript
try {
  await history.executeCommand(command);
} catch (error) {
  console.error('Command failed:', error);
}
```

### Transaction with Rollback
```typescript
try {
  await history.executeTransaction(commands, {
    rollbackOnFailure: true
  });
} catch (error) {
  console.error('Transaction failed, rolled back');
}
```

### Create Bookmark Before Major Change
```typescript
const snapshot = history.createBookmark('Before change');
try {
  await performMajorChange();
} catch (error) {
  await history.jumpToBookmark(snapshot.id);
  throw error;
}
```

## Keyboard Shortcuts (Recommended)

```typescript
document.addEventListener('keydown', (e) => {
  if (e.ctrlKey || e.metaKey) {
    if (e.key === 'z') {
      e.preventDefault();
      history.undo();
    } else if (e.key === 'y') {
      e.preventDefault();
      history.redo();
    } else if (e.shiftKey && e.key === 'Z') {
      e.preventDefault();
      history.redo();
    }
  }
});
```

## Tips

1. **Set reasonable history limits** (100-500 commands)
2. **Enable auto-save** for important projects
3. **Use bookmarks** before major operations
4. **Regular optimization** (every 10-15 minutes)
5. **Batch related operations** for better performance
6. **Use transactions** for multi-step operations
7. **Create macros** for repetitive workflows

## Troubleshooting

**Not persisting?**
```typescript
const persistence = new HistoryPersistence({ enabled: true });
```

**Memory too high?**
```typescript
const history = new HistoryManager(50); // Reduce limit
await history.optimize();
```

**Slow performance?**
```typescript
await history.executeBatch(commands, 'parallel');
await history.optimize({ mergeConsecutiveCommands: true });
```
