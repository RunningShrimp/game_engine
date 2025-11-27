import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { useUndoRedo } from "@/contexts/UndoRedoContext";
import { History, RotateCcw, RotateCw } from "lucide-react";

export default function HistoryPanel() {
  const { history, canUndo, canRedo, undo, redo } = useUndoRedo();

  return (
    <Card className="w-full h-full flex flex-col">
      <div className="h-12 border-b border-border flex items-center justify-between px-4">
        <div className="flex items-center gap-2">
          <History className="w-4 h-4 text-primary" />
          <span className="text-sm font-medium">历史记录</span>
        </div>
        <div className="flex items-center gap-1">
          <Button
            size="sm"
            variant="ghost"
            className="h-7 w-7 p-0"
            disabled={!canUndo}
            onClick={() => undo()}
          >
            <RotateCcw className="w-4 h-4" />
          </Button>
          <Button
            size="sm"
            variant="ghost"
            className="h-7 w-7 p-0"
            disabled={!canRedo}
            onClick={() => redo()}
          >
            <RotateCw className="w-4 h-4" />
          </Button>
        </div>
      </div>
      <ScrollArea className="flex-1">
        <div className="p-2 space-y-1">
          {history.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground text-sm">
              暂无历史记录
            </div>
          ) : (
            history.map((item, index) => (
              <div
                key={index}
                className={`px-3 py-2 rounded text-sm transition-colors ${
                  item.isCurrent
                    ? "bg-primary/20 text-primary font-medium"
                    : "text-muted-foreground hover:bg-accent/50"
                }`}
              >
                {item.description}
              </div>
            ))
          )}
        </div>
      </ScrollArea>
    </Card>
  );
}
