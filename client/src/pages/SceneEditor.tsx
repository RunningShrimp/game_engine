import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import Renderer3D from "@/components/Renderer3D";
import HistoryPanel from "@/components/HistoryPanel";
import {
  Box,
  Grid3x3,
  Lightbulb,
  Move,
  RotateCcw,
  Scale,
  Video,
} from "lucide-react";
import { useState } from "react";
import { useDragDrop } from "@/hooks/useDragDrop";
import { useHotkeys, HOTKEYS } from "@/hooks/useHotkeys";
import { usePerformance } from "@/hooks/usePerformance";
import { useUndoRedo } from "@/contexts/UndoRedoContext";
import { TransformCommand } from "@/lib/undoRedo";
import { toast } from "sonner";

export default function SceneEditor() {
  const [selectedTool, setSelectedTool] = useState<string>("move");
  const { handleDragOver, handleDrop } = useDragDrop();
  const perfStats = usePerformance();
  const { execute, undo, redo, canUndo, canRedo, undoDescription, redoDescription } = useUndoRedo();

  // 快捷键配置
  useHotkeys([
    {
      ...HOTKEYS.SAVE,
      handler: () => toast.success('场景已保存'),
    },
    {
      ...HOTKEYS.UNDO,
      handler: () => {
        if (undo()) {
          toast.success(`撤销: ${undoDescription}`);
        } else {
          toast.error('没有可撤销的操作');
        }
      },
    },
    {
      ...HOTKEYS.REDO,
      handler: () => {
        if (redo()) {
          toast.success(`重做: ${redoDescription}`);
        } else {
          toast.error('没有可重做的操作');
        }
      },
    },
    {
      ...HOTKEYS.DELETE,
      handler: () => toast.info('删除选中对象'),
    },
    {
      key: 'q',
      handler: () => setSelectedTool('move'),
    },
    {
      key: 'w',
      handler: () => setSelectedTool('rotate'),
    },
    {
      key: 'e',
      handler: () => setSelectedTool('scale'),
    },
  ]);

  const tools = [
    { id: "move", icon: Move, label: "移动" },
    { id: "rotate", icon: RotateCcw, label: "旋转" },
    { id: "scale", icon: Scale, label: "缩放" },
  ];

  const sceneObjects = [
    { id: 1, name: "主摄像机", type: "camera", icon: Video },
    { id: 2, name: "方向光", type: "light", icon: Lightbulb },
    { id: 3, name: "立方体", type: "mesh", icon: Box },
    { id: 4, name: "地面", type: "mesh", icon: Grid3x3 },
  ];

  return (
    <div className="h-full flex">
      {/* 左侧场景层级 */}
      <div className="w-64 border-r border-border bg-card">
        <div className="h-10 border-b border-border flex items-center px-3">
          <span className="text-sm font-medium">场景层级</span>
        </div>
        <ScrollArea className="h-[calc(100%-2.5rem)]">
          <div className="p-2">
            {sceneObjects.map((obj) => {
              const Icon = obj.icon;
              return (
                <button
                  key={obj.id}
                  className="w-full flex items-center gap-2 px-2 py-1.5 rounded hover:bg-accent text-sm text-left"
                >
                  <Icon className="w-4 h-4 text-muted-foreground" />
                  <span>{obj.name}</span>
                </button>
              );
            })}
          </div>
        </ScrollArea>
      </div>

      {/* 中间视口区域 */}
      <div className="flex-1 flex flex-col">
        {/* 工具栏 */}
        <div className="h-12 border-b border-border bg-card flex items-center px-4 gap-2">
          {tools.map((tool) => {
            const Icon = tool.icon;
            return (
              <Button
                key={tool.id}
                size="sm"
                variant={selectedTool === tool.id ? "secondary" : "ghost"}
                className="h-8 gap-2"
                onClick={() => setSelectedTool(tool.id)}
              >
                <Icon className="w-4 h-4" />
                <span className="text-xs">{tool.label}</span>
              </Button>
            );
          })}
          <Separator orientation="vertical" className="h-6 mx-2" />
          <Button size="sm" variant="ghost" className="h-8 gap-2">
            <Grid3x3 className="w-4 h-4" />
            <span className="text-xs">网格</span>
          </Button>
        </div>

        {/* 3D视口 */}
        <div 
          className="flex-1 bg-background relative overflow-hidden"
          onDragOver={handleDragOver}
          onDrop={(e) => {
            const data = handleDrop(e);
            if (data?.type === 'asset') {
              toast.success(`已添加资产到场景: ${data.data.name}`);
              // 这里可以调用WebSocket发送消息到Rust引擎
            }
          }}
        >
          <Renderer3D className="absolute inset-0" />

          {/* 视口信息叠加层 */}
          <div className="absolute bottom-4 left-4 bg-card/90 backdrop-blur px-3 py-2 rounded text-xs space-y-1">
            <div className="text-muted-foreground">FPS: {perfStats.fps}</div>
            <div className="text-muted-foreground">Frame Time: {perfStats.frameTime.toFixed(2)}ms</div>
            <div className="text-muted-foreground">Memory: {perfStats.memory}MB</div>
          </div>
        </div>
      </div>

      {/* 右侧面板 */}
      <div className="w-80 border-l border-border bg-card flex flex-col">
        <div className="flex-1 flex flex-col">
          <div className="h-12 border-b border-border flex items-center px-3">
            <span className="text-sm font-medium">属性</span>
          </div>
          <ScrollArea className="flex-1 p-4 space-y-4">
            <div>
              <label className="text-xs font-medium text-muted-foreground">
                名称
              </label>
              <input
                type="text"
                className="w-full mt-1 px-3 py-1.5 bg-input border border-border rounded text-sm"
                placeholder="对象名称"
              />
            </div>
            <Separator />
            <div>
              <div className="text-xs font-medium mb-2">变换</div>
              <div className="space-y-2">
                <div>
                  <label className="text-xs text-muted-foreground">位置</label>
                  <div className="grid grid-cols-3 gap-2 mt-1">
                    <input
                      type="number"
                      className="px-2 py-1 bg-input border border-border rounded text-xs"
                      placeholder="X"
                      defaultValue="0"
                    />
                    <input
                      type="number"
                      className="px-2 py-1 bg-input border border-border rounded text-xs"
                      placeholder="Y"
                      defaultValue="0"
                    />
                    <input
                      type="number"
                      className="px-2 py-1 bg-input border border-border rounded text-xs"
                      placeholder="Z"
                      defaultValue="0"
                    />
                  </div>
                </div>
                <div>
                  <label className="text-xs text-muted-foreground">旋转</label>
                  <div className="grid grid-cols-3 gap-2 mt-1">
                    <input
                      type="number"
                      className="px-2 py-1 bg-input border border-border rounded text-xs"
                      placeholder="X"
                      defaultValue="0"
                    />
                    <input
                      type="number"
                      className="px-2 py-1 bg-input border border-border rounded text-xs"
                      placeholder="Y"
                      defaultValue="0"
                    />
                    <input
                      type="number"
                      className="px-2 py-1 bg-input border border-border rounded text-xs"
                      placeholder="Z"
                      defaultValue="0"
                    />
                  </div>
                </div>
                <div>
                  <label className="text-xs text-muted-foreground">缩放</label>
                  <div className="grid grid-cols-3 gap-2 mt-1">
                    <input
                      type="number"
                      className="px-2 py-1 bg-input border border-border rounded text-xs"
                      placeholder="X"
                      defaultValue="1"
                    />
                    <input
                      type="number"
                      className="px-2 py-1 bg-input border border-border rounded text-xs"
                      placeholder="Y"
                      defaultValue="1"
                    />
                    <input
                      type="number"
                      className="px-2 py-1 bg-input border border-border rounded text-xs"
                      placeholder="Z"
                      defaultValue="1"
                    />
                  </div>
                </div>
              </div>
            </div>
          </ScrollArea>
        </div>
        <Separator />
        <div className="h-64">
          <HistoryPanel />
        </div>
      </div>
    </div>
  );
}
