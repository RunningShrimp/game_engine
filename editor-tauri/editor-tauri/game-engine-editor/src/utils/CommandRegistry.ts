// Command Registry Implementation

import { Command } from '../types/commands';
import { CommandTypeStats } from '../types/history';

/**
 * Command metadata for registration
 */
export interface CommandMetadata {
  type: string;
  category: string;
  description: string;
  version: string;
  author?: string;
  tags: string[];
  schema?: any;
  validator?: (command: Command) => boolean;
}

/**
 * Registry entry for a command type
 */
interface RegistryEntry {
  metadata: CommandMetadata;
  factory: (...args: any[]) => Command;
  instances: Set<Command>;
  statistics: {
    totalExecutions: number;
    successfulExecutions: number;
    failedExecutions: number;
    averageExecutionTime: number;
    lastExecution?: Date;
  };
}

/**
 * Command Registry for managing and tracking command types
 */
export class CommandRegistry {
  private registry: Map<string, RegistryEntry> = new Map();
  private commandAliases: Map<string, string> = new Map();

  /**
   * Register a new command type
   */
  register(
    type: string,
    factory: (...args: any[]) => Command,
    metadata: CommandMetadata
  ): void {
    if (this.registry.has(type)) {
      throw new Error(`Command type '${type}' is already registered`);
    }

    const entry: RegistryEntry = {
      metadata,
      factory,
      instances: new Set(),
      statistics: {
        totalExecutions: 0,
        successfulExecutions: 0,
        failedExecutions: 0,
        averageExecutionTime: 0,
      },
    };

    this.registry.set(type, entry);

    // Register aliases
    metadata.tags.forEach((tag) => {
      this.commandAliases.set(tag, type);
    });
  }

  /**
   * Unregister a command type
   */
  unregister(type: string): void {
    const entry = this.registry.get(type);
    if (!entry) {
      throw new Error(`Command type '${type}' is not registered`);
    }

    // Check for active instances
    if (entry.instances.size > 0) {
      throw new Error(
        `Cannot unregister '${type}': ${entry.instances.size} active instances exist`
      );
    }

    this.registry.delete(type);

    // Remove aliases
    entry.metadata.tags.forEach((tag) => {
      if (this.commandAliases.get(tag) === type) {
        this.commandAliases.delete(tag);
      }
    });
  }

  /**
   * Create a command instance from registered type
   */
  create(type: string, ...args: any[]): Command {
    const entry = this.registry.get(type);
    if (!entry) {
      // Try to resolve through alias
      const resolvedType = this.commandAliases.get(type);
      if (resolvedType) {
        return this.create(resolvedType, ...args);
      }
      throw new Error(`Command type '${type}' is not registered`);
    }

    const command = entry.factory(...args);
    entry.instances.add(command);

    return command;
  }

  /**
   * Track command execution for statistics
   */
  trackExecution(type: string, success: boolean, executionTime: number): void {
    const entry = this.registry.get(type);
    if (!entry) {
      return;
    }

    entry.statistics.totalExecutions++;
    entry.statistics.lastExecution = new Date();

    if (success) {
      entry.statistics.successfulExecutions++;
    } else {
      entry.statistics.failedExecutions++;
    }

    // Update average execution time
    const currentAvg = entry.statistics.averageExecutionTime;
    const totalExec = entry.statistics.totalExecutions;
    entry.statistics.averageExecutionTime =
      (currentAvg * (totalExec - 1) + executionTime) / totalExec;
  }

  /**
   * Remove command instance tracking
   */
  removeInstance(type: string, command: Command): void {
    const entry = this.registry.get(type);
    if (entry) {
      entry.instances.delete(command);
    }
  }

  /**
   * Get command metadata
   */
  getMetadata(type: string): CommandMetadata | undefined {
    const entry = this.registry.get(type);
    return entry?.metadata;
  }

  /**
   * Get all registered command types
   */
  getRegisteredTypes(): string[] {
    return Array.from(this.registry.keys());
  }

  /**
   * Get command types by category
   */
  getTypesByCategory(category: string): string[] {
    return Array.from(this.registry.entries())
      .filter(([_, entry]) => entry.metadata.category === category)
      .map(([type, _]) => type);
  }

  /**
   * Get command types by tag
   */
  getTypesByTag(tag: string): string[] {
    return Array.from(this.registry.entries())
      .filter(([_, entry]) => entry.metadata.tags.includes(tag))
      .map(([type, _]) => type);
  }

  /**
   * Get statistics for a command type
   */
  getStatistics(type: string): RegistryEntry['statistics'] | undefined {
    const entry = this.registry.get(type);
    return entry?.statistics;
  }

  /**
   * Get all command statistics
   */
  getAllStatistics(): CommandTypeStats[] {
    const totalExecutions = Array.from(this.registry.values()).reduce(
      (sum, entry) => sum + entry.statistics.totalExecutions,
      0
    );

    return Array.from(this.registry.entries()).map(([type, entry]) => ({
      type,
      count: entry.statistics.totalExecutions,
      percentage: totalExecutions > 0
        ? (entry.statistics.totalExecutions / totalExecutions) * 100
        : 0,
    }));
  }

  /**
   * Validate a command instance
   */
  validate(command: Command): boolean {
    const type = command.constructor.name;
    const entry = this.registry.get(type);

    if (!entry || !entry.metadata.validator) {
      return true;
    }

    return entry.metadata.validator(command);
  }

  /**
   * Get active instance count for a type
   */
  getInstanceCount(type: string): number {
    const entry = this.registry.get(type);
    return entry?.instances.size || 0;
  }

  /**
   * Get all categories
   */
  getCategories(): string[] {
    const categories = new Set<string>();
    for (const entry of this.registry.values()) {
      categories.add(entry.metadata.category);
    }
    return Array.from(categories);
  }

  /**
   * Clear all statistics
   */
  clearStatistics(): void {
    for (const entry of this.registry.values()) {
      entry.statistics = {
        totalExecutions: 0,
        successfulExecutions: 0,
        failedExecutions: 0,
        averageExecutionTime: 0,
      };
    }
  }

  /**
   * Export registry state
   */
  exportRegistry(): any {
    const data: any = {
      types: {},
      statistics: {},
    };

    for (const [type, entry] of this.registry.entries()) {
      data.types[type] = {
        metadata: entry.metadata,
        instanceCount: entry.instances.size,
      };
      data.statistics[type] = entry.statistics;
    }

    return data;
  }

  /**
   * Get registry health metrics
   */
  getHealthMetrics(): {
    totalTypes: number;
    totalInstances: number;
    totalExecutions: number;
    successRate: number;
    averageExecutionTime: number;
  } {
    let totalInstances = 0;
    let totalExecutions = 0;
    let successfulExecutions = 0;
    let totalExecutionTime = 0;

    for (const entry of this.registry.values()) {
      totalInstances += entry.instances.size;
      totalExecutions += entry.statistics.totalExecutions;
      successfulExecutions += entry.statistics.successfulExecutions;
      totalExecutionTime +=
        entry.statistics.averageExecutionTime * entry.statistics.totalExecutions;
    }

    return {
      totalTypes: this.registry.size,
      totalInstances,
      totalExecutions,
      successRate: totalExecutions > 0 ? (successfulExecutions / totalExecutions) * 100 : 0,
      averageExecutionTime: totalExecutions > 0 ? totalExecutionTime / totalExecutions : 0,
    };
  }
}

// Global registry instance
let globalRegistry: CommandRegistry | null = null;

/**
 * Get or create the global command registry
 */
export function getGlobalRegistry(): CommandRegistry {
  if (!globalRegistry) {
    globalRegistry = new CommandRegistry();
  }
  return globalRegistry;
}

/**
 * Reset the global registry (mainly for testing)
 */
export function resetGlobalRegistry(): void {
  globalRegistry = null;
}
