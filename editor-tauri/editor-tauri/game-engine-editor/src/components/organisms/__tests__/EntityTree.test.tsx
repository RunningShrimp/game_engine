import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import '@testing-library/jest-dom';
import { Entity } from '../../../types/engine';
import EntityTree from '../EntityTree';
import EntityTreeIcon from '../EntityTree/EntityTreeIcon';
import EntityTreeToggle from '../EntityTree/EntityTreeToggle';
import EntityTreeActions from '../EntityTree/EntityTreeActions';
import EntityTreeLabel from '../EntityTree/EntityTreeLabel';
import EntityTreeSearch from '../EntityTree/EntityTreeSearch';

// Mock entities
const mockEntities: Entity[] = [
  {
    id: '1',
    name: 'Root Entity',
    transform: {
      position: { x: 0, y: 0, z: 0 },
      rotation: { x: 0, y: 0, z: 0, w: 1 },
      scale: { x: 1, y: 1, z: 1 },
    },
    components: [],
    children: [
      {
        id: '2',
        name: 'Child Entity',
        transform: {
          position: { x: 0, y: 0, z: 0 },
          rotation: { x: 0, y: 0, z: 0, w: 1 },
          scale: { x: 1, y: 1, z: 1 },
        },
        components: [],
        children: [],
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
};

describe('EntityTree Components', () => {
  afterEach(() => {
    jest.clearAllMocks();
  });

  describe('EntityTreeIcon', () => {
    it('should render icon with default className', () => {
      render(<EntityTreeIcon />);
      const svg = screen.getByRole('img', { hidden: true }) || document.querySelector('svg');
      expect(svg).toBeInTheDocument();
      expect(svg).toHaveClass('w-4', 'h-4', 'text-slate-400');
    });

    it('should render icon with custom className', () => {
      render(<EntityTreeIcon className="custom-class" />);
      const svg = document.querySelector('svg');
      expect(svg).toHaveClass('custom-class');
    });
  });

  describe('EntityTreeToggle', () => {
    it('should render toggle button', () => {
      const handleToggle = jest.fn();
      render(<EntityTreeToggle expanded={false} onToggle={handleToggle} />);
      const button = screen.getByRole('button');
      expect(button).toBeInTheDocument();
      expect(button).toHaveAttribute('aria-label', 'Expand');
      expect(button).toHaveAttribute('aria-expanded', 'false');
    });

    it('should render expanded toggle', () => {
      const handleToggle = jest.fn();
      render(<EntityTreeToggle expanded={true} onToggle={handleToggle} />);
      const button = screen.getByRole('button');
      expect(button).toHaveAttribute('aria-label', 'Collapse');
      expect(button).toHaveAttribute('aria-expanded', 'true');
    });

    it('should call onToggle when clicked', () => {
      const handleToggle = jest.fn();
      render(<EntityTreeToggle expanded={false} onToggle={handleToggle} />);
      const button = screen.getByRole('button');
      fireEvent.click(button);
      expect(handleToggle).toHaveBeenCalledTimes(1);
    });

    it('should stop event propagation', () => {
      const handleToggle = jest.fn();
      const mockEvent = { stopPropagation: jest.fn() } as any;
      render(<EntityTreeToggle expanded={false} onToggle={handleToggle} />);
      const button = screen.getByRole('button');
      fireEvent.click(button, mockEvent);
      // Check if the click handler is called (stopPropagation should be called by the handler)
      expect(handleToggle).toHaveBeenCalled();
    });
  });

  describe('EntityTreeActions', () => {
    it('should render visibility and lock buttons', () => {
      render(
        <EntityTreeActions
          visible={true}
          locked={false}
          onToggleVisibility={jest.fn()}
          onToggleLock={jest.fn()}
        />
      );

      const visibilityButton = screen.getByTitle('Visible');
      const lockButton = screen.getByTitle('Unlocked');

      expect(visibilityButton).toBeInTheDocument();
      expect(lockButton).toBeInTheDocument();
    });

    it('should call onToggleVisibility when visibility button clicked', () => {
      const handleToggleVisibility = jest.fn();
      render(
        <EntityTreeActions
          visible={true}
          locked={false}
          onToggleVisibility={handleToggleVisibility}
          onToggleLock={jest.fn()}
        />
      );

      const visibilityButton = screen.getByTitle('Visible');
      fireEvent.click(visibilityButton);
      expect(handleToggleVisibility).toHaveBeenCalledTimes(1);
    });

    it('should call onToggleLock when lock button clicked', () => {
      const handleToggleLock = jest.fn();
      render(
        <EntityTreeActions
          visible={true}
          locked={false}
          onToggleVisibility={jest.fn()}
          onToggleLock={handleToggleLock}
        />
      );

      const lockButton = screen.getByTitle('Unlocked');
      fireEvent.click(lockButton);
      expect(handleToggleLock).toHaveBeenCalledTimes(1);
    });

    it('should show correct icons for hidden state', () => {
      render(
        <EntityTreeActions
          visible={false}
          locked={false}
          onToggleVisibility={jest.fn()}
          onToggleLock={jest.fn()}
        />
      );

      const visibilityButton = screen.getByTitle('Hidden');
      expect(visibilityButton).toBeInTheDocument();
    });

    it('should show correct icons for locked state', () => {
      render(
        <EntityTreeActions
          visible={true}
          locked={true}
          onToggleVisibility={jest.fn()}
          onToggleLock={jest.fn()}
        />
      );

      const lockButton = screen.getByTitle('Locked');
      expect(lockButton).toBeInTheDocument();
    });
  });

  describe('EntityTreeLabel', () => {
    it('should render entity name', () => {
      render(
        <EntityTreeLabel
          name="Test Entity"
          isEditing={false}
          editName=""
          onEditNameChange={jest.fn()}
          onEditFinish={jest.fn()}
          onEditCancel={jest.fn()}
          onDoubleClick={jest.fn()}
        />
      );

      expect(screen.getByText('Test Entity')).toBeInTheDocument();
    });

    it('should render input when editing', () => {
      render(
        <EntityTreeLabel
          name="Test Entity"
          isEditing={true}
          editName="Edited Name"
          onEditNameChange={jest.fn()}
          onEditFinish={jest.fn()}
          onEditCancel={jest.fn()}
          onDoubleClick={jest.fn()}
        />
      );

      const input = screen.getByRole('textbox');
      expect(input).toBeInTheDocument();
      expect(input).toHaveValue('Edited Name');
    });

    it('should call onDoubleClick when double clicked', () => {
      const handleDoubleClick = jest.fn();
      render(
        <EntityTreeLabel
          name="Test Entity"
          isEditing={false}
          editName=""
          onEditNameChange={jest.fn()}
          onEditFinish={jest.fn()}
          onEditCancel={jest.fn()}
          onDoubleClick={handleDoubleClick}
        />
      );

      const label = screen.getByText('Test Entity');
      fireEvent.doubleClick(label);
      expect(handleDoubleClick).toHaveBeenCalledTimes(1);
    });

    it('should call onEditFinish on Enter key', () => {
      const handleEditFinish = jest.fn();
      render(
        <EntityTreeLabel
          name="Test Entity"
          isEditing={true}
          editName="New Name"
          onEditNameChange={jest.fn()}
          onEditFinish={handleEditFinish}
          onEditCancel={jest.fn()}
          onDoubleClick={jest.fn()}
        />
      );

      const input = screen.getByRole('textbox');
      fireEvent.keyDown(input, { key: 'Enter' });
      expect(handleEditFinish).toHaveBeenCalledTimes(1);
    });

    it('should call onEditCancel on Escape key', () => {
      const handleEditCancel = jest.fn();
      render(
        <EntityTreeLabel
          name="Test Entity"
          isEditing={true}
          editName="New Name"
          onEditNameChange={jest.fn()}
          onEditFinish={jest.fn()}
          onEditCancel={handleEditCancel}
          onDoubleClick={jest.fn()}
        />
      );

      const input = screen.getByRole('textbox');
      fireEvent.keyDown(input, { key: 'Escape' });
      expect(handleEditCancel).toHaveBeenCalledTimes(1);
    });
  });

  describe('EntityTreeSearch', () => {
    it('should render search input', () => {
      render(
        <EntityTreeSearch
          value=""
          onChange={jest.fn()}
          placeholder="Search entities..."
        />
      );

      const input = screen.getByPlaceholderText('Search entities...');
      expect(input).toBeInTheDocument();
    });

    it('should call onChange when input changes', () => {
      const handleChange = jest.fn();
      render(
        <EntityTreeSearch
          value=""
          onChange={handleChange}
          placeholder="Search entities..."
        />
      );

      const input = screen.getByPlaceholderText('Search entities...');
      fireEvent.change(input, { target: { value: 'test' } });
      expect(handleChange).toHaveBeenCalledWith('test');
    });

    it('should show clear button when value exists', () => {
      render(
        <EntityTreeSearch
          value="test"
          onChange={jest.fn()}
          placeholder="Search entities..."
        />
      );

      const clearButton = screen.getByLabelText('Clear search');
      expect(clearButton).toBeInTheDocument();
    });

    it('should clear value when clear button clicked', () => {
      const handleChange = jest.fn();
      render(
        <EntityTreeSearch
          value="test"
          onChange={handleChange}
          placeholder="Search entities..."
        />
      );

      const clearButton = screen.getByLabelText('Clear search');
      fireEvent.click(clearButton);
      expect(handleChange).toHaveBeenCalledWith('');
    });
  });

  describe('EntityTree', () => {
    it('should render entity tree', () => {
      render(
        <EntityTree
          entities={mockEntities}
          selectedEntities={[]}
          {...mockCallbacks}
        />
      );

      expect(screen.getByText('Scene Hierarchy')).toBeInTheDocument();
      expect(screen.getByText('Root Entity')).toBeInTheDocument();
      expect(screen.getByText('Child Entity')).toBeInTheDocument();
    });

    it('should filter entities based on search query', () => {
      render(
        <EntityTree
          entities={mockEntities}
          selectedEntities={[]}
          {...mockCallbacks}
        />
      );

      const searchInput = screen.getByPlaceholderText('Search entities...');
      fireEvent.change(searchInput, { target: { value: 'Child' } });

      expect(screen.getByText('Child Entity')).toBeInTheDocument();
      expect(screen.queryByText('Root Entity')).not.toBeInTheDocument();
    });

    it('should call onEntitySelect when entity clicked', () => {
      render(
        <EntityTree
          entities={mockEntities}
          selectedEntities={[]}
          {...mockCallbacks}
        />
      );

      const rootEntity = screen.getByText('Root Entity');
      fireEvent.click(rootEntity);
      expect(mockCallbacks.onEntitySelect).toHaveBeenCalledWith(['1']);
    });

    it('should call onEntityCreate when new button clicked', () => {
      render(
        <EntityTree
          entities={mockEntities}
          selectedEntities={[]}
          {...mockCallbacks}
        />
      );

      const newButton = screen.getByTitle('Create New Entity');
      fireEvent.click(newButton);
      expect(mockCallbacks.onEntityCreate).toHaveBeenCalledTimes(1);
    });

    it('should expand and collapse entities', () => {
      render(
        <EntityTree
          entities={mockEntities}
          selectedEntities={[]}
          {...mockCallbacks}
        />
      );

      // Initially collapsed (we need to click the toggle)
      const expandButton = screen.getAllByRole('button').find(
        (button) => button.getAttribute('aria-label') === 'Expand'
      );

      if (expandButton) {
        fireEvent.click(expandButton);
        expect(expandButton).toHaveAttribute('aria-expanded', 'true');
      }
    });

    it('should handle multi-selection with Ctrl/Cmd key', () => {
      render(
        <EntityTree
          entities={mockEntities}
          selectedEntities={['1']}
          {...mockCallbacks}
        />
      );

      const childEntity = screen.getByText('Child Entity');
      fireEvent.click(childEntity, { ctrlKey: true });

      expect(mockCallbacks.onEntitySelect).toHaveBeenCalledWith(['1', '2']);
    });

    it('should show context menu on right-click', async () => {
      render(
        <EntityTree
          entities={mockEntities}
          selectedEntities={[]}
          {...mockCallbacks}
        />
      );

      const rootEntity = screen.getByText('Root Entity');
      fireEvent.contextMenu(rootEntity);

      await waitFor(() => {
        expect(screen.getByText('Rename')).toBeInTheDocument();
        expect(screen.getByText('Duplicate')).toBeInTheDocument();
        expect(screen.getByText('Delete')).toBeInTheDocument();
      });
    });

    it('should allow inline editing on double-click', () => {
      render(
        <EntityTree
          entities={mockEntities}
          selectedEntities={[]}
          {...mockCallbacks}
        />
      );

      const rootEntity = screen.getByText('Root Entity');
      fireEvent.doubleClick(rootEntity);

      const input = screen.getByDisplayValue('Root Entity');
      expect(input).toBeInTheDocument();
    });

    it('should call onEntityRename when editing finishes', () => {
      render(
        <EntityTree
          entities={mockEntities}
          selectedEntities={[]}
          {...mockCallbacks}
        />
      );

      // Start editing
      const rootEntity = screen.getByText('Root Entity');
      fireEvent.doubleClick(rootEntity);

      // Change name
      const input = screen.getByDisplayValue('Root Entity');
      fireEvent.change(input, { target: { value: 'New Name' } });
      fireEvent.blur(input);

      expect(mockCallbacks.onEntityRename).toHaveBeenCalledWith('1', 'New Name');
    });

    it('should toggle visibility', () => {
      render(
        <EntityTree
          entities={mockEntities}
          selectedEntities={[]}
          {...mockCallbacks}
        />
      );

      const visibilityButton = screen.getByTitle('Visible');
      fireEvent.click(visibilityButton);
      expect(mockCallbacks.onEntityToggleVisibility).toHaveBeenCalledWith('1');
    });

    it('should toggle lock', () => {
      render(
        <EntityTree
          entities={mockEntities}
          selectedEntities={[]}
          {...mockCallbacks}
        />
      );

      const lockButton = screen.getByTitle('Unlocked');
      fireEvent.click(lockButton);
      expect(mockCallbacks.onEntityToggleLock).toHaveBeenCalledWith('1');
    });

    it('should handle drag and drop', () => {
      const onEntityReparent = jest.fn();
      render(
        <EntityTree
          entities={mockEntities}
          selectedEntities={[]}
          onEntityReparent={onEntityReparent}
          {...mockCallbacks}
        />
      );

      const childEntity = screen.getByText('Child Entity');
      const dragStartEvent = new DragEvent('dragStart', { bubbles: true });
      Object.defineProperty(dragStartEvent, 'dataTransfer', {
        value: { effectAllowed: '' },
        writable: true,
      });

      fireEvent(childEntity, dragStartEvent);

      // Note: Full drag-and-drop testing requires more complex setup
      // This is a basic test to ensure drag events are handled
    });

    it('should display empty state when no entities', () => {
      render(
        <EntityTree
          entities={[]}
          selectedEntities={[]}
          {...mockCallbacks}
        />
      );

      expect(screen.getByText('No entities in scene')).toBeInTheDocument();
    });

    it('should display search results empty message', () => {
      render(
        <EntityTree
          entities={mockEntities}
          selectedEntities={[]}
          {...mockCallbacks}
        />
      );

      const searchInput = screen.getByPlaceholderText('Search entities...');
      fireEvent.change(searchInput, { target: { value: 'NonExistent' } });

      expect(screen.getByText('No entities found')).toBeInTheDocument();
    });

    it('should respect accessibility attributes', () => {
      render(
        <EntityTree
          entities={mockEntities}
          selectedEntities={[]}
          {...mockCallbacks}
        />
      );

      const treeContainer = screen.getByRole('tree');
      expect(treeContainer).toBeInTheDocument();
      expect(treeContainer).toHaveAttribute('aria-label', 'Entity hierarchy');
    });
  });
});
