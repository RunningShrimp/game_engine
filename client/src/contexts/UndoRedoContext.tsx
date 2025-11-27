import { createContext, useContext, useEffect, useState, ReactNode } from 'react';
import { undoRedoManager, Command } from '@/lib/undoRedo';

interface UndoRedoContextType {
  execute: (command: Command) => void;
  undo: () => boolean;
  redo: () => boolean;
  canUndo: boolean;
  canRedo: boolean;
  undoDescription: string | null;
  redoDescription: string | null;
  history: Array<{ description: string; isCurrent: boolean }>;
  clear: () => void;
}

const UndoRedoContext = createContext<UndoRedoContextType | undefined>(undefined);

export function UndoRedoProvider({ children }: { children: ReactNode }) {
  const [canUndo, setCanUndo] = useState(false);
  const [canRedo, setCanRedo] = useState(false);
  const [undoDescription, setUndoDescription] = useState<string | null>(null);
  const [redoDescription, setRedoDescription] = useState<string | null>(null);
  const [history, setHistory] = useState<Array<{ description: string; isCurrent: boolean }>>([]);

  const updateState = () => {
    setCanUndo(undoRedoManager.canUndo());
    setCanRedo(undoRedoManager.canRedo());
    setUndoDescription(undoRedoManager.getUndoDescription());
    setRedoDescription(undoRedoManager.getRedoDescription());
    setHistory(undoRedoManager.getHistory());
  };

  useEffect(() => {
    updateState();
    const unsubscribe = undoRedoManager.subscribe(updateState);
    return unsubscribe;
  }, []);

  const execute = (command: Command) => {
    undoRedoManager.execute(command);
  };

  const undo = () => {
    return undoRedoManager.undo();
  };

  const redo = () => {
    return undoRedoManager.redo();
  };

  const clear = () => {
    undoRedoManager.clear();
  };

  return (
    <UndoRedoContext.Provider
      value={{
        execute,
        undo,
        redo,
        canUndo,
        canRedo,
        undoDescription,
        redoDescription,
        history,
        clear,
      }}
    >
      {children}
    </UndoRedoContext.Provider>
  );
}

export function useUndoRedo() {
  const context = useContext(UndoRedoContext);
  if (!context) {
    throw new Error('useUndoRedo must be used within UndoRedoProvider');
  }
  return context;
}
