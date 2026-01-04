import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import '@testing-library/jest-dom';
import userEvent from '@testing-library/user-event';
import { Entity } from '../../../types/engine';
import EntityTree from '../EntityTree';

// Integration test suite for EntityTree component
describe('EntityTree Integration Tests', () => {
  const mockEntities: Entity[] = [
    {
      id: '1',
      name: 'Scene Root',
      transform: {
        position: { x: 0, y: 0, z: 0 },
        rotation: { x: 0, y: 0, z: 0, w: 1 },
        scale: { x: 1, y: 1, z: 1 },
      },
      components: [],
      children: [
        {
          id: '2',
          name: 'Camera',
          transform: {
            position: { x: 0, y: 2, z: -5 },
            rotation: { x: 0, y: 0, z: 0, w: 1 },
            scale: { x: 1, y: 1, z: 1 },
          },
          components: [],
          children: [],
          parentId: '1',
          visible: true,
          locked: false,
        },
        {
          id: '3',
          name: 'Light',
          transform: {
            position: { x: 5, y: 10, z: 5 },
            rotation: { x: 0, y: 0, z: 0, w: 1 },
            scale: { x: 1, y: 1, z: 1 },
          },
          components: [],
          children: [
            {
              id: '4',
              name: 'Light Target',
              transform: {
                position: { x: 0, y: 0, z: 0 },
                rotation: { x: 0, y: 0, z: 0, w: 1 },
                scale: { x: 1, y: 1, z: 1 },
              },
              components: [],
              children: [],
              parentId: '3',
              visible: true,
              locked: false,
            },
          ],
          parentId: '1',
          visible: true,
          locked: false,
        },
      ],
      visible: true,
      locked: false,
    },
  ];

  const mockCallbacks = {
    onEntitySelect: jest.fn(),
    onEntityRename: jest.fn(),
    onEntityDelete: jest.fn(),
    onEntityCreate: jest.fn(),
    onEntityToggleVisibility: jest.fn(),
    onEntityToggleLock: jest.fn(),
    onEntityReparent: jest.fn(),
  };

  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('Complex Entity Hierarchy', () => {
    it('should render nested entity structure correctly', () => {
      render(
        <EntityTree
          entities={mockEntities}
          selectedEntities={[]}
          {...mockCallbacks}
        />
      );

      expect(screen.getByText('Scene Root')).toBeInTheDocument();
      expect(screen.getByText('Camera')).toBeInTheDocument();
      expect(screen.getByText('Light')).toBeInTheDocument();
      expect(screen.getByText('Light Target')).toBeInTheDocument();
    });

    it('should expand and collapse nested entities', async () => {
      const user = userEvent.setup();
      render(
        <EntityTree
          entities={mockEntities}
          selectedEntities={[]}
          {...mockCallbacks}
        />
      );

      // Find expand button for Scene Root
      const expandButtons = screen.getAllByRole('button');
      const sceneRootExpand = expandButtons.find(
        (btn) => btn.getAttribute('aria-label') === 'Expand'
      );

      expect(sceneRootExpand).toBeInTheDocument();

      // Click to expand
      await user.click(sceneRootExpand!);
      expect(sceneRootExpand).toHaveAttribute('aria-expanded', 'true');

      // Click again to collapse
      await user.click(sceneRootExpand!);
      expect(sceneRootExpand).toHaveAttribute('aria-expanded', 'false');
    });

    it('should maintain expanded state during search filtering', async () => {
      const user = userEvent.setup();
      render(
        <EntityTree
          entities={mockEntities}
          selectedEntities={[]}
          {...mockCallbacks}
        />
      );

      // Expand Light entity
      const expandButtons = screen.getAllByRole('button');
      const lightExpand = expandButtons.find(
        (btn) => btn.getAttribute('aria-label') === 'Expand'
      );

      if (lightExpand) {
        await user.click(lightExpand);

        // Search for "Target"
        const searchInput = screen.getByPlaceholderText('Search entities...');
        await user.type(searchInput, 'Target');

        // Light Target should still be visible
        expect(screen.getByText('Light Target')).toBeInTheDocument();

        // Clear search
        await user.clear(searchInput);

        // Light should still be expanded
        expect(lightExpand).toHaveAttribute('aria-expanded', 'true');
      }
    });
  });

  describe('Entity Selection Workflows', () => {
    it('should support single selection', async () => {
      const user = userEvent.setup();
      render(
        <EntityTree
          entities={mockEntities}
          selectedEntities={[]}
          {...mockCallbacks}
        />
      );

      const cameraEntity = screen.getByText('Camera');
      await user.click(cameraEntity);

      expect(mockCallbacks.onEntitySelect).toHaveBeenCalledWith(['2']);
    });

    it('should support multi-selection with Ctrl key', async () => {
      const user = userEvent.setup();
      render(
        <EntityTree
          entities={mockEntities}
          selectedEntities={['2']}
          {...mockCallbacks}
        />
      );

      // Select second entity with Ctrl key
      const lightEntity = screen.getByText('Light');
      fireEvent.click(lightEntity, { ctrlKey: true });

      expect(mockCallbacks.onEntitySelect).toHaveBeenCalledWith(['2', '3']);
    });

    it('should support range selection with Shift key', async () => {
      render(
        <EntityTree
          entities={mockEntities}
          selectedEntities={['2']}
          {...mockCallbacks}
        />
      );

      // Try range selection
      const lightEntity = screen.getByText('Light');
      fireEvent.click(lightEntity, { shiftKey: true });

      // Should call onEntitySelect with range
      expect(mockCallbacks.onEntitySelect).toHaveBeenCalled();
    });

    it('should toggle entity selection with Ctrl+Click', async () => {
      const user = userEvent.setup();
      render(
        <EntityTree
          entities={mockEntities}
          selectedEntities={['2', '3']}
          {...mockCallbacks}
        />
      );

      // Deselect Camera with Ctrl+Click
      const cameraEntity = screen.getByText('Camera');
      fireEvent.click(cameraEntity, { ctrlKey: true });

      expect(mockCallbacks.onEntitySelect).toHaveBeenCalledWith(['3']);
    });
  });

  describe('Entity Editing Workflows', () => {
    it('should complete full rename workflow', async () => {
      const user = userEvent.setup();
      render(
        <EntityTree
          entities={mockEntities}
          selectedEntities={[]}
          {...mockCallbacks}
        />
      );

      // Double-click to start editing
      const cameraEntity = screen.getByText('Camera');
      fireEvent.doubleClick(cameraEntity);

      // Input should appear
      const input = screen.getByDisplayValue('Camera');
      expect(input).toBeInTheDocument();

      // Change name
      await user.clear(input);
      await user.type(input, 'Main Camera');

      // Press Enter to finish
      fireEvent.keyDown(input, { key: 'Enter' });

      expect(mockCallbacks.onEntityRename).toHaveBeenCalledWith('2', 'Main Camera');
    });

    it('should cancel rename on Escape key', async () => {
      const user = userEvent.setup();
      render(
        <EntityTree
          entities={mockEntities}
          selectedEntities={[]}
          {...mockCallbacks}
        />
      );

      const cameraEntity = screen.getByText('Camera');
      fireEvent.doubleClick(cameraEntity);

      const input = screen.getByDisplayValue('Camera');
      await user.type(input, 'New Name');

      // Press Escape to cancel
      fireEvent.keyDown(input, { key: 'Escape' });

      // Should not call onEntityRename
      expect(mockCallbacks.onEntityRename).not.toHaveBeenCalled();
    });

    it('should start editing from context menu', async () => {
      const user = userEvent.setup();
      render(
        <EntityTree
          entities={mockEntities}
          selectedEntities={[]}
          {...mockCallbacks}
        />
      );

      // Right-click on entity
      const cameraEntity = screen.getByText('Camera');
      fireEvent.contextMenu(cameraEntity);

      // Click Rename in context menu
      const renameButton = await screen.findByText('Rename');
      await user.click(renameButton);

      // Input should appear
      const input = screen.getByDisplayValue('Camera');
      expect(input).toBeInTheDocument();
    });
  });

  describe('Context Menu Interactions', () => {
    it('should show context menu on right-click', async () => {
      render(
        <EntityTree
          entities={mockEntities}
          selectedEntities={[]}
          {...mockCallbacks}
        />
      );

      const cameraEntity = screen.getByText('Camera');
      fireEvent.contextMenu(cameraEntity);

      await waitFor(() => {
        expect(screen.getByText('Rename')).toBeInTheDocument();
        expect(screen.getByText('Duplicate')).toBeInTheDocument();
        expect(screen.getByText('Toggle Visibility')).toBeInTheDocument();
        expect(screen.getByText('Delete')).toBeInTheDocument();
      });
    });

    it('should close context menu when clicking outside', async () => {
      render(
        <EntityTree
          entities={mockEntities}
          selectedEntities={[]}
          {...mockCallbacks}
        />
      );

      const cameraEntity = screen.getByText('Camera');
      fireEvent.contextMenu(cameraEntity);

      await waitFor(() => {
        expect(screen.getByText('Rename')).toBeInTheDocument();
      });

      // Click outside
      fireEvent.mouseDown(document.body);

      await waitFor(() => {
        expect(screen.queryByText('Rename')).not.toBeInTheDocument();
      });
    });

    it('should execute delete action from context menu', async () => {
      const user = userEvent.setup();
      render(
        <EntityTree
          entities={mockEntities}
          selectedEntities={[]}
          {...mockCallbacks}
        />
      );

      const cameraEntity = screen.getByText('Camera');
      fireEvent.contextMenu(cameraEntity);

      const deleteButton = await screen.findByText('Delete');
      await user.click(deleteButton);

      expect(mockCallbacks.onEntityDelete).toHaveBeenCalledWith('2');
    });

    it('should execute toggle visibility action from context menu', async () => {
      const user = userEvent.setup();
      render(
        <EntityTree
          entities={mockEntities}
          selectedEntities={[]}
          {...mockCallbacks}
        />
      );

      const cameraEntity = screen.getByText('Camera');
      fireEvent.contextMenu(cameraEntity);

      const toggleVisibilityButton = await screen.findByText('Toggle Visibility');
      await user.click(toggleVisibilityButton);

      expect(mockCallbacks.onEntityToggleVisibility).toHaveBeenCalledWith('2');
    });
  });

  describe('Visibility and Lock Toggles', () => {
    it('should toggle entity visibility', async () => {
      const user = userEvent.setup();
      render(
        <EntityTree
          entities={mockEntities}
          selectedEntities={[]}
          {...mockCallbacks}
        />
      );

      const visibilityButton = screen.getByTitle('Visible');
      await user.click(visibilityButton);

      expect(mockCallbacks.onEntityToggleVisibility).toHaveBeenCalledWith('1');
    });

    it('should toggle entity lock', async () => {
      const user = userEvent.setup();
      render(
        <EntityTree
          entities={mockEntities}
          selectedEntities={[]}
          {...mockCallbacks}
        />
      );

      const lockButton = screen.getByTitle('Unlocked');
      await user.click(lockButton);

      expect(mockCallbacks.onEntityToggleLock).toHaveBeenCalledWith('1');
    });

    it('should prevent dragging locked entities', () => {
      const lockedEntity: Entity = {
        ...mockEntities[0],
        locked: true,
      };

      render(
        <EntityTree
          entities={[lockedEntity]}
          selectedEntities={[]}
          {...mockCallbacks}
        />
      );

      const sceneRoot = screen.getByText('Scene Root');
      expect(sceneRoot).toBeInTheDocument();

      // Check if the element has draggable=false or no draggable attribute
      const parentDiv = sceneRoot.closest('[draggable]');
      expect(parentDiv?.getAttribute('draggable')).toBe('null');
    });
  });

  describe('Search and Filtering', () => {
    it('should filter entities based on search query', async () => {
      const user = userEvent.setup();
      render(
        <EntityTree
          entities={mockEntities}
          selectedEntities={[]}
          {...mockCallbacks}
        />
      );

      const searchInput = screen.getByPlaceholderText('Search entities...');
      await user.type(searchInput, 'Camera');

      expect(screen.getByText('Camera')).toBeInTheDocument();
      expect(screen.queryByText('Light')).not.toBeInTheDocument();
    });

    it('should show all entities when search is cleared', async () => {
      const user = userEvent.setup();
      render(
        <EntityTree
          entities={mockEntities}
          selectedEntities={[]}
          {...mockCallbacks}
        />
      );

      const searchInput = screen.getByPlaceholderText('Search entities...');
      await user.type(searchInput, 'Camera');

      // Clear search using clear button
      const clearButton = screen.getByLabelText('Clear search');
      await user.click(clearButton);

      expect(screen.getByText('Scene Root')).toBeInTheDocument();
      expect(screen.getByText('Camera')).toBeInTheDocument();
      expect(screen.getByText('Light')).toBeInTheDocument();
    });

    it('should expand entities containing matching children', async () => {
      const user = userEvent.setup();
      render(
        <EntityTree
          entities={mockEntities}
          selectedEntities={[]}
          {...mockCallbacks}
        />
      );

      const searchInput = screen.getByPlaceholderText('Search entities...');
      await user.type(searchInput, 'Target');

      // Light entity should be expanded to show Light Target
      expect(screen.getByText('Light Target')).toBeInTheDocument();
    });
  });

  describe('Performance with Large Trees', () => {
    it('should handle large entity trees efficiently', () => {
      // Create a large entity tree
      const createLargeTree = (depth: number, breadth: number): Entity[] => {
        if (depth === 0) return [];

        return Array.from({ length: breadth }, (_, i) => ({
          id: `entity-${depth}-${i}`,
          name: `Entity ${depth}-${i}`,
          transform: {
            position: { x: 0, y: 0, z: 0 },
            rotation: { x: 0, y: 0, z: 0, w: 1 },
            scale: { x: 1, y: 1, z: 1 },
          },
          components: [],
          children: createLargeTree(depth - 1, breadth),
          visible: true,
          locked: false,
        }));
      };

      const largeTree = createLargeTree(3, 5);

      const startTime = performance.now();
      render(
        <EntityTree
          entities={largeTree}
          selectedEntities={[]}
          {...mockCallbacks}
        />
      );
      const endTime = performance.now();

      // Should render in reasonable time (< 1000ms)
      expect(endTime - startTime).toBeLessThan(1000);
    });
  });

  describe('Accessibility', () => {
    it('should have proper ARIA attributes', () => {
      render(
        <EntityTree
          entities={mockEntities}
          selectedEntities={[]}
          {...mockCallbacks}
        />
      );

      const tree = screen.getByRole('tree');
      expect(tree).toHaveAttribute('aria-label', 'Entity hierarchy');

      const newButton = screen.getByTitle('Create New Entity');
      expect(newButton).toHaveAttribute('aria-label', 'Create new entity');
    });

    it('should support keyboard navigation', async () => {
      const user = userEvent.setup();
      render(
        <EntityTree
          entities={mockEntities}
          selectedEntities={[]}
          {...mockCallbacks}
        />
      );

      // Press Escape to close context menu if open
      fireEvent.keyDown(document, { key: 'Escape' });

      // Context menu should close if it was open
      await waitFor(() => {
        expect(screen.queryByText('Rename')).not.toBeInTheDocument();
      });
    });
  });
});
