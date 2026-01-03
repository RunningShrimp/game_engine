// Selection Manager

import { Entity, Vector3 } from '../types/engine';
import {
  SelectionState,
  SelectionFilter,
  Bounds3D,
  Rect,
  SelectionMode,
  SelectionHistoryEntry,
  EntityScreenPos,
} from '../types/selection';

type SelectionListener = (state: SelectionState) => void;

export class SelectionManager {
  private state: SelectionState;
  private listeners: Set<SelectionListener> = new Set();
  private history: SelectionHistoryEntry[] = [];
  private maxHistorySize: number = 50;
  private entities: Map<string, Entity> = new Map();
  private selectionBox: Rect | null = null;
  private isBoxSelecting: boolean = false;

  constructor() {
    this.state = {
      selectedIds: [],
      primarySelection: null,
      selectionOrder: [],
      selectionBounds: this.createEmptyBounds(),
    };
  }

  /**
   * Initialize with entities
   */
  initialize(entities: Entity[]): void {
    this.entities.clear();
    entities.forEach((entity) => {
      this.entities.set(entity.id, entity);
      this.addEntityToMap(entity);
    });
  }

  /**
   * Add entity to map
   */
  addEntity(entity: Entity): void {
    this.entities.set(entity.id, entity);
    this.addEntityToMap(entity);
  }

  /**
   * Remove entity from map
   */
  removeEntity(entityId: string): void {
    this.deselect(entityId);
    this.entities.delete(entityId);
  }

  /**
   * Update entity in map
   */
  updateEntity(entity: Entity): void {
    this.entities.set(entity.id, entity);
    this.updateBounds();
  }

  // ==================== Basic Selection ====================

  /**
   * Select an entity
   */
  select(id: string, additive: boolean = false): void {
    if (!this.entities.has(id)) {
      console.warn(`Entity ${id} not found`);
      return;
    }

    if (additive) {
      if (!this.state.selectedIds.includes(id)) {
        this.state.selectedIds.push(id);
        this.state.selectionOrder.push(id);
        if (this.state.selectedIds.length === 1) {
          this.state.primarySelection = id;
        }
      }
    } else {
      this.state.selectedIds = [id];
      this.state.selectionOrder = [id];
      this.state.primarySelection = id;
    }

    this.updateBounds();
    this.saveToHistory();
    this.notifyListeners();
  }

  /**
   * Deselect an entity
   */
  deselect(id: string): void {
    const index = this.state.selectedIds.indexOf(id);
    if (index !== -1) {
      this.state.selectedIds.splice(index, 1);
      const orderIndex = this.state.selectionOrder.indexOf(id);
      if (orderIndex !== -1) {
        this.state.selectionOrder.splice(orderIndex, 1);
      }

      // Update primary selection
      if (this.state.primarySelection === id) {
        this.state.primarySelection =
          this.state.selectedIds.length > 0
            ? this.state.selectedIds[this.state.selectedIds.length - 1]
            : null;
      }

      this.updateBounds();
      this.saveToHistory();
      this.notifyListeners();
    }
  }

  /**
   * Toggle selection for an entity
   */
  toggleSelect(id: string): void {
    if (this.isSelected(id)) {
      this.deselect(id);
    } else {
      this.select(id, true);
    }
  }

  /**
   * Select range of entities (from last selected to current)
   */
  selectRange(from: string, to: string): void {
    const entityList = Array.from(this.entities.values());
    const fromIndex = entityList.findIndex((e) => e.id === from);
    const toIndex = entityList.findIndex((e) => e.id === to);

    if (fromIndex === -1 || toIndex === -1) return;

    const start = Math.min(fromIndex, toIndex);
    const end = Math.max(fromIndex, toIndex);

    this.deselectAll();

    for (let i = start; i <= end; i++) {
      const entity = entityList[i];
      if (entity) {
        this.state.selectedIds.push(entity.id);
        this.state.selectionOrder.push(entity.id);
      }
    }

    if (this.state.selectedIds.length > 0) {
      this.state.primarySelection = this.state.selectedIds[0];
    }

    this.updateBounds();
    this.saveToHistory();
    this.notifyListeners();
  }

  /**
   * Select all entities
   */
  selectAll(): void {
    this.state.selectedIds = Array.from(this.entities.keys());
    this.state.selectionOrder = [...this.state.selectedIds];
    this.state.primarySelection =
      this.state.selectedIds.length > 0 ? this.state.selectedIds[0] : null;

    this.updateBounds();
    this.saveToHistory();
    this.notifyListeners();
  }

  /**
   * Deselect all entities
   */
  deselectAll(): void {
    if (this.state.selectedIds.length === 0) return;

    this.state.selectedIds = [];
    this.state.selectionOrder = [];
    this.state.primarySelection = null;
    this.state.selectionBounds = this.createEmptyBounds();

    this.saveToHistory();
    this.notifyListeners();
  }

  /**
   * Invert selection
   */
  invertSelection(): void {
    const allIds = Array.from(this.entities.keys());
    const selectedSet = new Set(this.state.selectedIds);

    this.state.selectedIds = allIds.filter((id) => !selectedSet.has(id));
    this.state.selectionOrder = [...this.state.selectedIds];
    this.state.primarySelection =
      this.state.selectedIds.length > 0 ? this.state.selectedIds[0] : null;

    this.updateBounds();
    this.saveToHistory();
    this.notifyListeners();
  }

  // ==================== Advanced Selection ====================

  /**
   * Select entities by filter
   */
  selectByFilter(filter: SelectionFilter): void {
    const matchingEntities = Array.from(this.entities.values()).filter(
      (entity) => this.matchesFilter(entity, filter)
    );

    this.state.selectedIds = matchingEntities.map((e) => e.id);
    this.state.selectionOrder = [...this.state.selectedIds];
    this.state.primarySelection =
      this.state.selectedIds.length > 0 ? this.state.selectedIds[0] : null;

    this.updateBounds();
    this.saveToHistory();
    this.notifyListeners();
  }

  /**
   * Select all children of a parent entity
   */
  selectChildren(parentId: string): void {
    const parent = this.entities.get(parentId);
    if (!parent) return;

    const childrenIds = this.getEntityChildrenIds(parent);
    childrenIds.forEach((id) => {
      if (!this.state.selectedIds.includes(id)) {
        this.state.selectedIds.push(id);
        this.state.selectionOrder.push(id);
      }
    });

    this.updateBounds();
    this.saveToHistory();
    this.notifyListeners();
  }

  /**
   * Select all siblings of an entity
   */
  selectSiblings(id: string): void {
    const entity = this.entities.get(id);
    if (!entity || !entity.parentId) return;

    const parent = this.entities.get(entity.parentId);
    if (!parent) return;

    const siblingIds = parent.children.map((child) => child.id);
    siblingIds.forEach((siblingId) => {
      if (!this.state.selectedIds.includes(siblingId)) {
        this.state.selectedIds.push(siblingId);
        this.state.selectionOrder.push(siblingId);
      }
    });

    this.updateBounds();
    this.saveToHistory();
    this.notifyListeners();
  }

  /**
   * Select similar entities (same components)
   */
  selectSimilar(entityId: string): void {
    const entity = this.entities.get(entityId);
    if (!entity) return;

    const componentTypes = entity.components.map((c) => c.type);

    const similarEntities = Array.from(this.entities.values()).filter((e) => {
      if (e.id === entityId) return false;
      const eComponentTypes = e.components.map((c) => c.type);
      return (
        eComponentTypes.length === componentTypes.length &&
        eComponentTypes.every((ct) => componentTypes.includes(ct))
      );
    });

    this.state.selectedIds = similarEntities.map((e) => e.id);
    this.state.selectionOrder = [...this.state.selectedIds];
    this.state.primarySelection =
      this.state.selectedIds.length > 0 ? this.state.selectedIds[0] : null;

    this.updateBounds();
    this.saveToHistory();
    this.notifyListeners();
  }

  // ==================== Box Selection ====================

  /**
   * Start box selection
   */
  startBoxSelection(point: { x: number; y: number }): void {
    this.selectionBox = {
      start: point,
      end: point,
    };
    this.isBoxSelecting = true;
  }

  /**
   * Update box selection
   */
  updateBoxSelection(point: { x: number; y: number }): void {
    if (!this.selectionBox) return;

    this.selectionBox.end = point;
  }

  /**
   * End box selection
   */
  endBoxSelection(entityPositions: EntityScreenPos[], additive: boolean = true): void {
    if (!this.selectionBox) return;

    const entitiesInBox = this.getEntitiesInBox(entityPositions);

    if (!additive) {
      this.deselectAll();
    }

    entitiesInBox.forEach((id) => {
      if (!this.state.selectedIds.includes(id)) {
        this.state.selectedIds.push(id);
        this.state.selectionOrder.push(id);
      }
    });

    if (this.state.selectedIds.length > 0 && !this.state.primarySelection) {
      this.state.primarySelection = this.state.selectedIds[0];
    }

    this.selectionBox = null;
    this.isBoxSelecting = false;

    this.updateBounds();
    this.saveToHistory();
    this.notifyListeners();
  }

  /**
   * Get entities inside selection box
   */
  private getEntitiesInBox(entityPositions: EntityScreenPos[]): string[] {
    if (!this.selectionBox) return [];

    const box = this.selectionBox;
    const minX = Math.min(box.start.x, box.end.x);
    const maxX = Math.max(box.start.x, box.end.x);
    const minY = Math.min(box.start.y, box.end.y);
    const maxY = Math.max(box.start.y, box.end.y);

    return entityPositions
      .filter((pos) => {
        const x = pos.position.x;
        const y = pos.position.y;
        return x >= minX && x <= maxX && y >= minY && y <= maxY;
      })
      .map((pos) => pos.entityId);
  }

  // ==================== Query ====================

  /**
   * Check if an entity is selected
   */
  isSelected(id: string): boolean {
    return this.state.selectedIds.includes(id);
  }

  /**
   * Get all selected entity IDs
   */
  getSelectedIds(): string[] {
    return [...this.state.selectedIds];
  }

  /**
   * Get selected entities
   */
  getSelectedEntities(): Entity[] {
    return this.state.selectedIds
      .map((id) => this.entities.get(id))
      .filter((e): e is Entity => e !== undefined);
  }

  /**
   * Get selected entity count
   */
  getSelectedCount(): number {
    return this.state.selectedIds.length;
  }

  /**
   * Get primary selection
   */
  getPrimarySelection(): string | null {
    return this.state.primarySelection;
  }

  /**
   * Set primary selection
   */
  setPrimarySelection(id: string): void {
    if (this.state.selectedIds.includes(id)) {
      this.state.primarySelection = id;
      this.notifyListeners();
    }
  }

  /**
   * Get selection bounds
   */
  getSelectionBounds(): Bounds3D {
    return { ...this.state.selectionBounds };
  }

  /**
   * Get current selection state
   */
  getState(): SelectionState {
    return {
      selectedIds: [...this.state.selectedIds],
      primarySelection: this.state.primarySelection,
      selectionOrder: [...this.state.selectionOrder],
      selectionBounds: { ...this.state.selectionBounds },
    };
  }

  /**
   * Get selection box
   */
  getSelectionBox(): Rect | null {
    return this.selectionBox;
  }

  /**
   * Check if box selecting
   */
  isBoxSelectingActive(): boolean {
    return this.isBoxSelecting;
  }

  // ==================== History ====================

  /**
   * Undo selection change
   */
  undo(): void {
    if (this.history.length <= 1) return;

    this.history.pop(); // Remove current state
    const previousState = this.history[this.history.length - 1];

    if (previousState) {
      this.state.selectedIds = [...previousState.ids];
      this.state.primarySelection = previousState.primarySelection;
      this.state.selectionOrder = [...previousState.ids];

      this.updateBounds();
      this.notifyListeners();
    }
  }

  // ==================== Events ====================

  /**
   * Subscribe to selection changes
   */
  onSelectionChange(listener: SelectionListener): () => void {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  }

  /**
   * Notify all listeners
   */
  private notifyListeners(): void {
    const state = this.getState();
    this.listeners.forEach((listener) => {
      try {
        listener(state);
      } catch (error) {
        console.error('Error in selection listener:', error);
      }
    });
  }

  // ==================== Private Helpers ====================

  /**
   * Add entity to map recursively
   */
  private addEntityToMap(entity: Entity): void {
    this.entities.set(entity.id, entity);
    entity.children.forEach((child) => {
      this.addEntityToMap(child);
    });
  }

  /**
   * Get entity children IDs recursively
   */
  private getEntityChildrenIds(entity: Entity): string[] {
    const ids: string[] = [];
    entity.children.forEach((child) => {
      ids.push(child.id);
      ids.push(...this.getEntityChildrenIds(child));
    });
    return ids;
  }

  /**
   * Check if entity matches filter
   */
  private matchesFilter(entity: Entity, filter: SelectionFilter): boolean {
    // Type filter
    if (filter.types && filter.types.length > 0) {
      const entityType = this.getEntityType(entity);
      if (!filter.types.includes(entityType)) return false;
    }

    // Name pattern filter
    if (filter.namePattern) {
      if (!filter.namePattern.test(entity.name)) return false;
    }

    // Layer filter (if you have layer system)
    if (filter.layer) {
      // Implement layer check when layer system is ready
    }

    // Component filter
    if (filter.components && filter.components.length > 0) {
      const entityComponentTypes = entity.components.map((c) => c.type);
      const hasAllComponents = filter.components.every((ct) =>
        entityComponentTypes.includes(ct)
      );
      if (!hasAllComponents) return false;
    }

    // Custom filter
    if (filter.custom) {
      if (!filter.custom(entity)) return false;
    }

    return true;
  }

  /**
   * Get entity type (based on components or other criteria)
   */
  private getEntityType(entity: Entity): string {
    // Implement type detection logic
    // For now, return a default type
    return 'entity';
  }

  /**
   * Update selection bounds
   */
  private updateBounds(): void {
    if (this.state.selectedIds.length === 0) {
      this.state.selectionBounds = this.createEmptyBounds();
      return;
    }

    const selectedEntities = this.getSelectedEntities();
    if (selectedEntities.length === 0) {
      this.state.selectionBounds = this.createEmptyBounds();
      return;
    }

    let minX = Infinity,
      minY = Infinity,
      minZ = Infinity;
    let maxX = -Infinity,
      maxY = -Infinity,
      maxZ = -Infinity;

    selectedEntities.forEach((entity) => {
      const pos = entity.transform.position;
      minX = Math.min(minX, pos.x);
      minY = Math.min(minY, pos.y);
      minZ = Math.min(minZ, pos.z);
      maxX = Math.max(maxX, pos.x);
      maxY = Math.max(maxY, pos.y);
      maxZ = Math.max(maxZ, pos.z);
    });

    const center: Vector3 = {
      x: (minX + maxX) / 2,
      y: (minY + maxY) / 2,
      z: (minZ + maxZ) / 2,
    };

    const size: Vector3 = {
      x: maxX - minX,
      y: maxY - minY,
      z: maxZ - minZ,
    };

    this.state.selectionBounds = {
      min: { x: minX, y: minY, z: minZ },
      max: { x: maxX, y: maxY, z: maxZ },
      center,
      size,
    };
  }

  /**
   * Create empty bounds
   */
  private createEmptyBounds(): Bounds3D {
    return {
      min: { x: 0, y: 0, z: 0 },
      max: { x: 0, y: 0, z: 0 },
      center: { x: 0, y: 0, z: 0 },
      size: { x: 0, y: 0, z: 0 },
    };
  }

  /**
   * Save current selection to history
   */
  private saveToHistory(): void {
    const entry: SelectionHistoryEntry = {
      ids: [...this.state.selectedIds],
      primarySelection: this.state.primarySelection,
      timestamp: Date.now(),
    };

    this.history.push(entry);

    // Limit history size
    if (this.history.length > this.maxHistorySize) {
      this.history.shift();
    }
  }
}

// Export singleton instance
export const selectionManager = new SelectionManager();
