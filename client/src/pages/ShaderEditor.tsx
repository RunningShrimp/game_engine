import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import {
  Box,
  Circle,
  Eye,
  Image,
  Layers,
  Plus,
  Save,
  Sparkles,
  Zap,
} from "lucide-react";
import { useCallback, useState } from "react";
import ReactFlow, {
  addEdge,
  Background,
  Connection,
  Controls,
  Edge,
  MiniMap,
  Node,
  useEdgesState,
  useNodesState,
} from "reactflow";
import "reactflow/dist/style.css";
import { toast } from "sonner";

const initialNodes: Node[] = [
  {
    id: "1",
    type: "input",
    data: { label: "纹理输入" },
    position: { x: 100, y: 100 },
  },
  {
    id: "2",
    data: { label: "颜色混合" },
    position: { x: 300, y: 100 },
  },
  {
    id: "3",
    data: { label: "法线贴图" },
    position: { x: 300, y: 200 },
  },
  {
    id: "4",
    data: { label: "光照计算" },
    position: { x: 500, y: 150 },
  },
  {
    id: "5",
    type: "output",
    data: { label: "最终输出" },
    position: { x: 700, y: 150 },
  },
];

const initialEdges: Edge[] = [
  { id: "e1-2", source: "1", target: "2", animated: true },
  { id: "e2-4", source: "2", target: "4" },
  { id: "e3-4", source: "3", target: "4" },
  { id: "e4-5", source: "4", target: "5", animated: true },
];

export default function ShaderEditor() {
  const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);

  const onConnect = useCallback(
    (params: Connection) => setEdges((eds) => addEdge(params, eds)),
    [setEdges]
  );

  const nodeTypes = [
    {
      category: "输入",
      items: [
        { name: "纹理输入", icon: Image, color: "text-blue-500" },
        { name: "颜色输入", icon: Circle, color: "text-green-500" },
        { name: "UV坐标", icon: Layers, color: "text-purple-500" },
      ],
    },
    {
      category: "数学",
      items: [
        { name: "加法", icon: Plus, color: "text-yellow-500" },
        { name: "乘法", icon: Zap, color: "text-orange-500" },
        { name: "混合", icon: Sparkles, color: "text-pink-500" },
      ],
    },
    {
      category: "效果",
      items: [
        { name: "法线贴图", icon: Box, color: "text-cyan-500" },
        { name: "光照", icon: Eye, color: "text-amber-500" },
      ],
    },
  ];

  const handleSave = () => {
    toast.success("着色器已保存");
  };

  const handleAddNode = (nodeName: string) => {
    const newNode: Node = {
      id: `${nodes.length + 1}`,
      data: { label: nodeName },
      position: { x: Math.random() * 400 + 100, y: Math.random() * 300 + 100 },
    };
    setNodes((nds) => [...nds, newNode]);
    toast.info(`已添加节点: ${nodeName}`);
  };

  return (
    <div className="h-full flex">
      {/* 左侧节点库 */}
      <div className="w-64 border-r border-border bg-card">
        <div className="h-12 border-b border-border flex items-center px-3">
          <span className="text-sm font-medium">节点库</span>
        </div>
        <ScrollArea className="h-[calc(100%-3rem)]">
          <div className="p-2 space-y-4">
            {nodeTypes.map((category) => (
              <div key={category.category}>
                <div className="px-2 py-1.5 text-xs font-semibold text-muted-foreground">
                  {category.category}
                </div>
                <div className="space-y-1">
                  {category.items.map((item) => {
                    const Icon = item.icon;
                    return (
                      <button
                        key={item.name}
                        className="w-full text-left px-2 py-2 text-sm rounded flex items-center gap-2 hover:bg-accent transition-colors"
                        onClick={() => handleAddNode(item.name)}
                      >
                        <Icon className={`w-4 h-4 ${item.color}`} />
                        <span>{item.name}</span>
                      </button>
                    );
                  })}
                </div>
              </div>
            ))}
          </div>
        </ScrollArea>
      </div>

      {/* 中间节点编辑区 */}
      <div className="flex-1 flex flex-col">
        {/* 工具栏 */}
        <div className="h-12 border-b border-border bg-card flex items-center justify-between px-4">
          <div className="flex items-center gap-2">
            <Sparkles className="w-4 h-4 text-primary" />
            <span className="text-sm font-medium">PBR材质</span>
          </div>
          <div className="flex items-center gap-2">
            <Button size="sm" variant="ghost" className="h-8 gap-2" onClick={handleSave}>
              <Save className="w-4 h-4" />
              <span className="text-xs">保存</span>
            </Button>
            <Separator orientation="vertical" className="h-6" />
            <Button size="sm" variant="ghost" className="h-8 gap-2">
              <Eye className="w-4 h-4" />
              <span className="text-xs">预览</span>
            </Button>
          </div>
        </div>

        {/* React Flow 节点编辑器 */}
        <div className="flex-1 bg-background">
          <ReactFlow
            nodes={nodes}
            edges={edges}
            onNodesChange={onNodesChange}
            onEdgesChange={onEdgesChange}
            onConnect={onConnect}
            fitView
          >
            <Background />
            <Controls />
            <MiniMap />
          </ReactFlow>
        </div>
      </div>

      {/* 右侧属性面板 */}
      <div className="w-80 border-l border-border bg-card flex flex-col">
        <div className="h-12 border-b border-border flex items-center px-3">
          <span className="text-sm font-medium">节点属性</span>
        </div>
        <ScrollArea className="flex-1 p-4 space-y-4">
          <Card className="p-4">
            <h3 className="text-sm font-semibold mb-3">预览</h3>
            <div className="aspect-square bg-gradient-to-br from-purple-500 to-pink-500 rounded-lg mb-3"></div>
            <div className="text-xs text-muted-foreground">
              实时材质预览
            </div>
          </Card>

          <Card className="p-4">
            <h3 className="text-sm font-semibold mb-3">参数</h3>
            <div className="space-y-3">
              <div>
                <label className="text-xs font-medium text-muted-foreground block mb-1">
                  基础颜色
                </label>
                <div className="flex gap-2">
                  <input
                    type="color"
                    className="w-10 h-8 rounded border border-border"
                    defaultValue="#8b5cf6"
                  />
                  <input
                    type="text"
                    className="flex-1 px-2 py-1 bg-input border border-border rounded text-xs"
                    defaultValue="#8b5cf6"
                  />
                </div>
              </div>
              <div>
                <label className="text-xs font-medium text-muted-foreground block mb-1">
                  金属度
                </label>
                <input
                  type="range"
                  className="w-full"
                  min="0"
                  max="1"
                  step="0.01"
                  defaultValue="0.5"
                />
              </div>
              <div>
                <label className="text-xs font-medium text-muted-foreground block mb-1">
                  粗糙度
                </label>
                <input
                  type="range"
                  className="w-full"
                  min="0"
                  max="1"
                  step="0.01"
                  defaultValue="0.5"
                />
              </div>
            </div>
          </Card>

          <Card className="p-4">
            <h3 className="text-sm font-semibold mb-3">生成代码</h3>
            <div className="bg-muted/50 rounded p-3 font-mono text-xs space-y-1">
              <div className="text-blue-400">struct Material {"{"}</div>
              <div className="ml-2 text-green-400">vec3 albedo;</div>
              <div className="ml-2 text-green-400">float metallic;</div>
              <div className="ml-2 text-green-400">float roughness;</div>
              <div className="text-blue-400">{"}"}</div>
            </div>
          </Card>
        </ScrollArea>
      </div>
    </div>
  );
}
