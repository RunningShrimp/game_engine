import { useState, DragEvent } from 'react';

export interface DragDropData {
  type: string;
  data: any;
}

export function useDragDrop() {
  const [isDragging, setIsDragging] = useState(false);
  const [dragData, setDragData] = useState<DragDropData | null>(null);

  const handleDragStart = (e: DragEvent, type: string, data: any) => {
    const dragDropData: DragDropData = { type, data };
    e.dataTransfer.setData('application/json', JSON.stringify(dragDropData));
    e.dataTransfer.effectAllowed = 'copy';
    setIsDragging(true);
    setDragData(dragDropData);
  };

  const handleDragEnd = () => {
    setIsDragging(false);
    setDragData(null);
  };

  const handleDragOver = (e: DragEvent) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'copy';
  };

  const handleDrop = (e: DragEvent, onDrop?: (data: DragDropData) => void) => {
    e.preventDefault();
    setIsDragging(false);

    try {
      const jsonData = e.dataTransfer.getData('application/json');
      if (jsonData) {
        const dropData: DragDropData = JSON.parse(jsonData);
        setDragData(null);
        onDrop?.(dropData);
        return dropData;
      }
    } catch (error) {
      console.error('Failed to parse drop data:', error);
    }

    return null;
  };

  return {
    isDragging,
    dragData,
    handleDragStart,
    handleDragEnd,
    handleDragOver,
    handleDrop,
  };
}
