/**
 * 撤销/重做系统
 * 使用命令模式实现操作历史管理
 */

export interface Command {
  execute(): void;
  undo(): void;
  redo(): void;
  description: string;
}

export class UndoRedoManager {
  private history: Command[] = [];
  private currentIndex: number = -1;
  private maxHistorySize: number = 100;
  private listeners: Set<() => void> = new Set();

  /**
   * 执行命令并添加到历史记录
   */
  execute(command: Command): void {
    // 执行命令
    command.execute();

    // 清除当前索引之后的历史记录
    this.history = this.history.slice(0, this.currentIndex + 1);

    // 添加到历史记录
    this.history.push(command);
    this.currentIndex++;

    // 限制历史记录大小
    if (this.history.length > this.maxHistorySize) {
      this.history.shift();
      this.currentIndex--;
    }

    this.notifyListeners();
  }

  /**
   * 撤销操作
   */
  undo(): boolean {
    if (!this.canUndo()) {
      return false;
    }

    const command = this.history[this.currentIndex];
    command.undo();
    this.currentIndex--;

    this.notifyListeners();
    return true;
  }

  /**
   * 重做操作
   */
  redo(): boolean {
    if (!this.canRedo()) {
      return false;
    }

    this.currentIndex++;
    const command = this.history[this.currentIndex];
    command.redo();

    this.notifyListeners();
    return true;
  }

  /**
   * 是否可以撤销
   */
  canUndo(): boolean {
    return this.currentIndex >= 0;
  }

  /**
   * 是否可以重做
   */
  canRedo(): boolean {
    return this.currentIndex < this.history.length - 1;
  }

  /**
   * 获取当前可撤销的命令描述
   */
  getUndoDescription(): string | null {
    if (!this.canUndo()) {
      return null;
    }
    return this.history[this.currentIndex].description;
  }

  /**
   * 获取当前可重做的命令描述
   */
  getRedoDescription(): string | null {
    if (!this.canRedo()) {
      return null;
    }
    return this.history[this.currentIndex + 1].description;
  }

  /**
   * 清空历史记录
   */
  clear(): void {
    this.history = [];
    this.currentIndex = -1;
    this.notifyListeners();
  }

  /**
   * 获取历史记录
   */
  getHistory(): Array<{ description: string; isCurrent: boolean }> {
    return this.history.map((cmd, index) => ({
      description: cmd.description,
      isCurrent: index === this.currentIndex,
    }));
  }

  /**
   * 订阅历史记录变化
   */
  subscribe(listener: () => void): () => void {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  }

  /**
   * 通知所有监听器
   */
  private notifyListeners(): void {
    this.listeners.forEach((listener) => listener());
  }

  /**
   * 设置最大历史记录大小
   */
  setMaxHistorySize(size: number): void {
    this.maxHistorySize = size;
    if (this.history.length > size) {
      const removeCount = this.history.length - size;
      this.history.splice(0, removeCount);
      this.currentIndex = Math.max(-1, this.currentIndex - removeCount);
      this.notifyListeners();
    }
  }
}

// 导出单例实例
export const undoRedoManager = new UndoRedoManager();

// 常用命令实现示例

/**
 * 场景对象变换命令
 */
export class TransformCommand implements Command {
  description: string;
  private entityId: string;
  private oldTransform: any;
  private newTransform: any;
  private applyTransform: (entityId: string, transform: any) => void;

  constructor(
    entityId: string,
    oldTransform: any,
    newTransform: any,
    applyTransform: (entityId: string, transform: any) => void,
    description?: string
  ) {
    this.entityId = entityId;
    this.oldTransform = oldTransform;
    this.newTransform = newTransform;
    this.applyTransform = applyTransform;
    this.description = description || `变换实体 ${entityId}`;
  }

  execute(): void {
    this.applyTransform(this.entityId, this.newTransform);
  }

  undo(): void {
    this.applyTransform(this.entityId, this.oldTransform);
  }

  redo(): void {
    this.execute();
  }
}

/**
 * 创建实体命令
 */
export class CreateEntityCommand implements Command {
  description: string;
  private entity: any;
  private onCreate: (entity: any) => void;
  private onDelete: (entityId: string) => void;

  constructor(
    entity: any,
    onCreate: (entity: any) => void,
    onDelete: (entityId: string) => void,
    description?: string
  ) {
    this.entity = entity;
    this.onCreate = onCreate;
    this.onDelete = onDelete;
    this.description = description || `创建实体 ${entity.name}`;
  }

  execute(): void {
    this.onCreate(this.entity);
  }

  undo(): void {
    this.onDelete(this.entity.id);
  }

  redo(): void {
    this.execute();
  }
}

/**
 * 删除实体命令
 */
export class DeleteEntityCommand implements Command {
  description: string;
  private entity: any;
  private onCreate: (entity: any) => void;
  private onDelete: (entityId: string) => void;

  constructor(
    entity: any,
    onCreate: (entity: any) => void,
    onDelete: (entityId: string) => void,
    description?: string
  ) {
    this.entity = entity;
    this.onCreate = onCreate;
    this.onDelete = onDelete;
    this.description = description || `删除实体 ${entity.name}`;
  }

  execute(): void {
    this.onDelete(this.entity.id);
  }

  undo(): void {
    this.onCreate(this.entity);
  }

  redo(): void {
    this.execute();
  }
}
