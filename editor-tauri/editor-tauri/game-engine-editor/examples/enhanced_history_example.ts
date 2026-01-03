// Complete Example: Enhanced Undo/Redo System Usage

import { HistoryManager } from '../src/utils/HistoryManager';
import { HistoryPersistence } from '../src/utils/HistoryPersistence';
import {
  BatchCommand,
  TransactionCommand,
  MacroCommand,
  ConditionalCommand,
  DelayedCommand,
} from '../src/commands';
import { Command } from '../src/types/commands';
import { getGlobalRegistry } from '../src/utils/CommandRegistry';

// Example 1: Basic Setup
async function example1_BasicSetup() {
  console.log('=== Example 1: Basic Setup ===');

  // Create persistence
  const persistence = new HistoryPersistence({
    enabled: true,
    autoSave: true,
    autoSaveInterval: 60000, // 1 minute
    storageType: 'localStorage',
    compress: true,
  });

  // Create history manager
  const history = new HistoryManager(100, persistence);

  // Subscribe to changes
  const unsubscribe = history.subscribe((state) => {
    console.log('History updated:');
    console.log('- Can undo:', state.canUndo);
    console.log('- Can redo:', state.canRedo);
    console.log('- Bookmarks:', state.bookmarks.length);
    console.log('- Branches:', state.branches.length);
  });

  // Execute a simple command
  const simpleCommand: Command = {
    id: 'cmd_1',
    timestamp: new Date(),
    description: 'Create entity',
    metadata: { tags: ['entity', 'create'] },

    async execute() {
      console.log('Executing: Create entity');
    },

    async undo() {
      console.log('Undoing: Create entity');
    },

    async redo() {
      console.log('Redoing: Create entity');
    },
  };

  await history.executeCommand(simpleCommand);
  await history.undo();
  await history.redo();

  unsubscribe();
}

// Example 2: Batch Operations
async function example2_BatchOperations() {
  console.log('\n=== Example 2: Batch Operations ===');

  const history = new HistoryManager(100);

  // Create multiple commands
  const commands: Command[] = [
    {
      id: 'cmd_1',
      timestamp: new Date(),
      description: 'Create entity 1',
      async execute() { console.log('Create entity 1'); },
      async undo() { console.log('Undo entity 1'); },
      async redo() { console.log('Redo entity 1'); },
    },
    {
      id: 'cmd_2',
      timestamp: new Date(),
      description: 'Create entity 2',
      async execute() { console.log('Create entity 2'); },
      async undo() { console.log('Undo entity 2'); },
      async redo() { console.log('Redo entity 2'); },
    },
    {
      id: 'cmd_3',
      timestamp: new Date(),
      description: 'Create entity 3',
      async execute() { console.log('Create entity 3'); },
      async undo() { console.log('Undo entity 3'); },
      async redo() { console.log('Redo entity 3'); },
    },
  ];

  // Execute as batch (sequential)
  console.log('Executing batch sequentially...');
  await history.executeBatch(commands, 'sequential');

  // Undo the batch
  await history.undo();

  // Execute as batch (parallel)
  console.log('Executing batch in parallel...');
  await history.executeBatch(commands, 'parallel');
}

// Example 3: Transaction with Rollback
async function example3_Transaction() {
  console.log('\n=== Example 3: Transaction with Rollback ===');

  const history = new HistoryManager(100);

  // Create commands where one will fail
  const commands: Command[] = [
    {
      id: 'cmd_1',
      timestamp: new Date(),
      description: 'Step 1',
      async execute() {
        console.log('Step 1: Success');
      },
      async undo() {
        console.log('Undo Step 1');
      },
      async redo() {
        console.log('Redo Step 1');
      },
    },
    {
      id: 'cmd_2',
      timestamp: new Date(),
      description: 'Step 2 (will fail)',
      async execute() {
        console.log('Step 2: Failing...');
        throw new Error('Intentional failure');
      },
      async undo() {
        console.log('Undo Step 2');
      },
      async redo() {
        console.log('Redo Step 2');
      },
    },
    {
      id: 'cmd_3',
      timestamp: new Date(),
      description: 'Step 3',
      async execute() {
        console.log('Step 3: Success');
      },
      async undo() {
        console.log('Undo Step 3');
      },
      async redo() {
        console.log('Redo Step 3');
      },
    },
  ];

  // Execute with rollback
  try {
    await history.executeTransaction(commands, {
      rollbackOnFailure: true,
      stopOnFirstError: true,
    });
  } catch (error) {
    console.log('Transaction failed and rolled back:', (error as Error).message);
  }

  // Verify nothing was executed
  console.log('Can undo:', history.canUndo()); // Should be false
}

// Example 4: Macro Commands
async function example4_MacroCommands() {
  console.log('\n=== Example 4: Macro Commands ===');

  const history = new HistoryManager(100);

  // Define a macro for creating a scene
  const createSceneMacro = new MacroCommand(
    'Create Scene',
    [
      {
        id: 'macro_1',
        timestamp: new Date(),
        description: 'Create camera',
        async execute() { console.log('Create camera'); },
        async undo() { console.log('Undo camera'); },
        async redo() { console.log('Redo camera'); },
      },
      {
        id: 'macro_2',
        timestamp: new Date(),
        description: 'Create light',
        async execute() { console.log('Create light'); },
        async undo() { console.log('Undo light'); },
        async redo() { console.log('Redo light'); },
      },
      {
        id: 'macro_3',
        timestamp: new Date(),
        description: 'Create ground',
        async execute() { console.log('Create ground'); },
        async undo() { console.log('Undo ground'); },
        async redo() { console.log('Redo ground'); },
      },
    ],
    {
      cameraPosition: { x: 0, y: 5, z: 10 },
      lightIntensity: 1.0,
    }
  );

  // Execute the macro
  console.log('Executing macro: Create Scene');
  await history.executeCommand(createSceneMacro);

  // Undo the entire macro
  await history.undo();
}

// Example 5: Conditional Commands
async function example5_ConditionalCommands() {
  console.log('\n=== Example 5: Conditional Commands ===');

  const history = new HistoryManager(100);

  let shouldExecute = true;

  const thenCommand: Command = {
    id: 'then',
    timestamp: new Date(),
    description: 'Then command',
    async execute() { console.log('Executing THEN branch'); },
    async undo() { console.log('Undo THEN'); },
    async redo() { console.log('Redo THEN'); },
  };

  const elseCommand: Command = {
    id: 'else',
    timestamp: new Date(),
    description: 'Else command',
    async execute() { console.log('Executing ELSE branch'); },
    async undo() { console.log('Undo ELSE'); },
    async redo() { console.log('Redo ELSE'); },
  };

  const conditional = new ConditionalCommand(
    () => shouldExecute,
    thenCommand,
    elseCommand
  );

  // Test with condition true
  console.log('Condition: true');
  await history.executeCommand(conditional);

  // Test with condition false
  shouldExecute = false;
  console.log('Condition: false');
  await history.executeCommand(conditional);
}

// Example 6: Bookmarks
async function example6_Bookmarks() {
  console.log('\n=== Example 6: Bookmarks ===');

  const history = new HistoryManager(100);

  // Execute some commands
  for (let i = 1; i <= 3; i++) {
    const command: Command = {
      id: `cmd_${i}`,
      timestamp: new Date(),
      description: `Command ${i}`,
      async execute() { console.log(`Execute ${i}`); },
      async undo() { console.log(`Undo ${i}`); },
      async redo() { console.log(`Redo ${i}`); },
    };
    await history.executeCommand(command);
  }

  // Create a bookmark
  const bookmark = history.createBookmark(
    'Checkpoint 1',
    'After first 3 commands',
    ['checkpoint', 'initial']
  );

  console.log('Created bookmark:', bookmark.name);
  console.log('All bookmarks:', history.getBookmarks());

  // Execute more commands
  for (let i = 4; i <= 6; i++) {
    const command: Command = {
      id: `cmd_${i}`,
      timestamp: new Date(),
      description: `Command ${i}`,
      async execute() { console.log(`Execute ${i}`); },
      async undo() { console.log(`Undo ${i}`); },
      async redo() { console.log(`Redo ${i}`); },
    };
    await history.executeCommand(command);
  }

  // Jump back to bookmark
  console.log('Jumping back to bookmark...');
  await history.jumpToBookmark(bookmark.id);

  console.log('Current history size:', history.getHistorySize());
}

// Example 7: Branches
async function example7_Branches() {
  console.log('\n=== Example 7: Branches ===');

  const history = new HistoryManager(100);

  // Create main branch
  const mainBranch = history.createBranch('main', null);
  console.log('Created branch:', mainBranch.name);

  // Execute some commands on main branch
  const command1: Command = {
    id: 'main_1',
    timestamp: new Date(),
    description: 'Main branch command',
    async execute() { console.log('Main: Execute'); },
    async undo() { console.log('Main: Undo'); },
    async redo() { console.log('Main: Redo'); },
  };
  await history.executeCommand(command1);

  // Create experimental branch
  const expBranch = history.createBranch('experimental', mainBranch.id);
  console.log('Created experimental branch');

  // Execute different commands on experimental branch
  const command2: Command = {
    id: 'exp_1',
    timestamp: new Date(),
    description: 'Experimental command',
    async execute() { console.log('Experimental: Execute'); },
    async undo() { console.log('Experimental: Undo'); },
    async redo() { console.log('Experimental: Redo'); },
  };

  // Switch to experimental branch
  await history.switchBranch(expBranch.id);
  await history.executeCommand(command2);

  console.log('All branches:', history.getBranches());
  console.log('Current branch:', expBranch.name);

  // Switch back to main
  await history.switchBranch(mainBranch.id);
  console.log('Switched back to:', mainBranch.name);
}

// Example 8: Search and Filter
async function example8_SearchAndFilter() {
  console.log('\n=== Example 8: Search and Filter ===');

  const history = new HistoryManager(100);

  // Execute various commands
  const commands: Command[] = [
    {
      id: 'create_1',
      timestamp: new Date(),
      description: 'Create entity',
      metadata: { tags: ['entity', 'create'] },
      async execute() {},
      async undo() {},
      async redo() {},
    },
    {
      id: 'delete_1',
      timestamp: new Date(),
      description: 'Delete entity',
      metadata: { tags: ['entity', 'delete'] },
      async execute() {},
      async undo() {},
      async redo() {},
    },
    {
      id: 'transform_1',
      timestamp: new Date(),
      description: 'Transform entity',
      metadata: { tags: ['entity', 'transform'] },
      async execute() {},
      async undo() {},
      async redo() {},
    },
  ];

  for (const cmd of commands) {
    await history.executeCommand(cmd);
  }

  // Create a bookmark
  history.createBookmark('Test checkpoint', 'For testing search', ['test']);

  // Search
  const searchResults = history.search('entity');
  console.log('Search results for "entity":', searchResults.totalResults);

  // Filter
  const filtered = history.filter({
    tags: ['entity', 'create'],
  });
  console.log('Filtered commands:', filtered.length);
}

// Example 9: Statistics
async function example9_Statistics() {
  console.log('\n=== Example 9: Statistics ===');

  const history = new HistoryManager(100);

  // Execute some commands
  for (let i = 0; i < 10; i++) {
    const command: Command = {
      id: `stat_${i}`,
      timestamp: new Date(),
      description: i % 2 === 0 ? 'Create entity' : 'Delete entity',
      async execute() {},
      async undo() {},
      async redo() {},
    };
    await history.executeCommand(command);
  }

  // Create bookmarks and branches
  history.createBookmark('Bookmark 1', 'Test', ['test']);
  history.createBranch('experimental', null);

  // Get statistics
  const stats = history.getStatistics();
  console.log('Statistics:');
  console.log('- Total commands:', stats.totalCommands);
  console.log('- Undo depth:', stats.undoStackDepth);
  console.log('- Redo depth:', stats.redoStackDepth);
  console.log('- Bookmarks:', stats.totalBookmarks);
  console.log('- Branches:', stats.totalBranches);
  console.log('- Memory usage:', stats.memoryUsage, 'bytes');
  console.log('- Most used commands:', stats.mostUsedCommandTypes);
}

// Example 10: Optimization
async function example10_Optimization() {
  console.log('\n=== Example 10: Optimization ===');

  const history = new HistoryManager(100);

  // Execute many similar commands
  for (let i = 0; i < 20; i++) {
    const command: Command = {
      id: `opt_${i}`,
      timestamp: new Date(),
      description: 'Move entity',
      async execute() {},
      async undo() {},
      async redo() {},
    };
    await history.executeCommand(command);
  }

  console.log('Before optimization:');
  let statsBefore = history.getStatistics();
  console.log('- Commands:', statsBefore.totalCommands);
  console.log('- Memory:', statsBefore.memoryUsage);

  // Optimize
  await history.optimize({
    mergeConsecutiveCommands: true,
    removeDuplicateCommands: true,
    keepBookmarks: true,
  });

  console.log('After optimization:');
  let statsAfter = history.getStatistics();
  console.log('- Commands:', statsAfter.totalCommands);
  console.log('- Memory:', statsAfter.memoryUsage);
}

// Example 11: Persistence
async function example11_Persistence() {
  console.log('\n=== Example 11: Persistence ===');

  const persistence = new HistoryPersistence({
    enabled: true,
    autoSave: false,
    storageType: 'localStorage',
  });

  const history = new HistoryManager(50, persistence);

  // Execute commands
  const command: Command = {
    id: 'persist_1',
    timestamp: new Date(),
    description: 'Persistent command',
    async execute() { console.log('Execute'); },
    async undo() { console.log('Undo'); },
    async redo() { console.log('Redo'); },
  };
  await history.executeCommand(command);

  // Save
  await history.save();
  console.log('History saved');

  // Export
  const blob = await history.export('json');
  console.log('Exported size:', blob.size, 'bytes');

  // Clear and load
  history.clear();
  console.log('History cleared');

  await history.load();
  console.log('History loaded');
  console.log('Can undo:', history.canUndo());
}

// Example 12: Command Registry
async function example12_CommandRegistry() {
  console.log('\n=== Example 12: Command Registry ===');

  const registry = getGlobalRegistry();

  // Register a command type
  registry.register(
    'CustomCommand',
    (name: string) => ({
      id: `custom_${Date.now()}`,
      timestamp: new Date(),
      description: name,
      async execute() { console.log(`Execute ${name}`); },
      async undo() { console.log(`Undo ${name}`); },
      async redo() { console.log(`Redo ${name}`); },
    }),
    {
      type: 'CustomCommand',
      category: 'custom',
      description: 'A custom command',
      version: '1.0.0',
      tags: ['custom'],
    }
  );

  // Create command from registry
  const cmd = registry.create('CustomCommand', 'Test Command');

  // Track execution
  const startTime = Date.now();
  await cmd.execute();
  const executionTime = Date.now() - startTime;
  registry.trackExecution('CustomCommand', true, executionTime);

  // Get statistics
  const stats = registry.getStatistics('CustomCommand');
  console.log('Command statistics:', stats);
}

// Run all examples
async function runAllExamples() {
  await example1_BasicSetup();
  await example2_BatchOperations();
  await example3_Transaction();
  await example4_MacroCommands();
  await example5_ConditionalCommands();
  await example6_Bookmarks();
  await example7_Branches();
  await example8_SearchAndFilter();
  await example9_Statistics();
  await example10_Optimization();
  await example11_Persistence();
  await example12_CommandRegistry();

  console.log('\n=== All examples completed ===');
}

// Export for use
export {
  example1_BasicSetup,
  example2_BatchOperations,
  example3_Transaction,
  example4_MacroCommands,
  example5_ConditionalCommands,
  example6_Bookmarks,
  example7_Branches,
  example8_SearchAndFilter,
  example9_Statistics,
  example10_Optimization,
  example11_Persistence,
  example12_CommandRegistry,
  runAllExamples,
};

// Run if executed directly
if (require.main === module) {
  runAllExamples().catch(console.error);
}
