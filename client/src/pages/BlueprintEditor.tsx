import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import {
  Box,
  Calculator,
  Circle,
  Code,
  Eye,
  Gamepad2,
  Keyboard,
  Lightbulb,
  Mouse,
  Play,
  Plus,
  RotateCw,
  Save,
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
    data: { label: "开始游戏" },
    position: { x: 100, y: 150 },
  },
  {
    id: "2",
    data: { label: "获取玩家输入" },
    position: { x: 300, y: 100 },
  },
  {
    id: "3",
    data: { label: "移动角色" },
    position: { x: 500, y: 100 },
  },
  {
    id: "4",
    data: { label: "播放动画" },
    position: { x: 700, y: 100 },
  },
  {
    id: "5",
    data: { label: "检测碰撞" },
    position: { x: 300, y: 200 },
  },
  {
    id: "6",
    data: { label: "触发事件" },
    position: { x: 500, y: 200 },
  },
];

const initialEdges: Edge[] = [
  { id: "e1-2", source: "1", target: "2", animated: true },
  { id: "e2-3", source: "2", target: "3" },
  { id: "e3-4", source: "3", target: "4" },
  { id: "e1-5", source: "1", target: "5", animated: true },
  { id: "e5-6", source: "5", target: "6" },
];

export default function BlueprintEditor() {
  const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);

  const onConnect = useCallback(
    (params: Connection) => setEdges((eds) => addEdge(params, eds)),
    [setEdges]
  );

  const nodeCategories = [
    {
      category: "事件",
      items: [
        { name: "开始游戏", icon: Play, color: "text-red-500" },
        { name: "按键按下", icon: Keyboard, color: "text-red-500" },
        { name: "鼠标点击", icon: Mouse, color: "text-red-500" },
        { name: "碰撞触发", icon: Circle, color: "text-red-500" },
      ],
    },
    {
      category: "逻辑",
      items: [
        { name: "分支", icon: Zap, color: "text-yellow-500" },
        { name: "循环", icon: RotateCw, color: "text-yellow-500" },
        { name: "延迟", icon: Lightbulb, color: "text-yellow-500" },
      ],
    },
    {
      category: "数学",
      items: [
        { name: "加法", icon: Plus, color: "text-green-500" },
        { name: "计算", icon: Calculator, color: "text-green-500" },
      ],
    },
    {
      category: "游戏",
      items: [
        { name: "移动角色", icon: Gamepad2, color: "text-blue-500" },
        { name: "播放动画", icon: Box, color: "text-blue-500" },
        { name: "生成物体", icon: Plus, color: "text-blue-500" },
      ],
    },
  ];

  const handleSave = () => {
    toast.success("蓝图已保存");
  };

  const handleCompile = () => {
    toast.success("蓝图编译成功");
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
            {nodeCategories.map((category) => (
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

      {/* 中间蓝图编辑区 */}
      <div className="flex-1 flex flex-col">
        {/* 工具栏 */}
        <div className="h-12 border-b border-border bg-card flex items-center justify-between px-4">
          <div className="flex items-center gap-2">
            <Code className="w-4 h-4 text-primary" />
            <span className="text-sm font-medium">角色控制器蓝图</span>
          </div>
          <div className="flex items-center gap-2">
            <Button size="sm" variant="ghost" className="h-8 gap-2" onClick={handleSave}>
              <Save className="w-4 h-4" />
              <span className="text-xs">保存</span>
            </Button>
            <Separator orientation="vertical" className="h-6" />
            <Button size="sm" variant="ghost" className="h-8 gap-2" onClick={handleCompile}>
              <Zap className="w-4 h-4" />
              <span className="text-xs">编译</span>
            </Button>
            <Button size="sm" variant="ghost" className="h-8 gap-2">
              <Play className="w-4 h-4" />
              <span className="text-xs">运行</span>
            </Button>
          </div>
        </div>

        {/* React Flow 蓝图编辑器 */}
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

      {/* 右侧详情面板 */}
      <div className="w-80 border-l border-border bg-card flex flex-col">
        <div className="h-12 border-b border-border flex items-center px-3">
          <span className="text-sm font-medium">节点详情</span>
        </div>
        <ScrollArea className="flex-1 p-4 space-y-4">
          <Card className="p-4">
            <h3 className="text-sm font-semibold mb-3">选中节点</h3>
            <div className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span className="text-muted-foreground">类型</span>
                <span className="font-medium">事件</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">名称</span>
                <span className="font-medium">开始游戏</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">连接数</span>
                <span className="font-medium">2</span>
              </div>
            </div>
          </Card>

          <Card className="p-4">
            <h3 className="text-sm font-semibold mb-3">参数</h3>
            <div className="space-y-3">
              <div>
                <label className="text-xs font-medium text-muted-foreground block mb-1">
                  延迟时间 (秒)
                </label>
                <input
                  type="number"
                  className="w-full px-2 py-1 bg-input border border-border rounded text-xs"
                  defaultValue="0"
                  step="0.1"
                />
              </div>
              <div>
                <label className="text-xs font-medium text-muted-foreground block mb-1">
                  执行条件
                </label>
                <select className="w-full px-2 py-1 bg-input border border-border rounded text-xs">
                  <option>始终执行</option>
                  <option>条件为真</option>
                  <option>条件为假</option>
                </select>
              </div>
              <div className="flex items-center justify-between">
                <label className="text-xs font-medium">启用节点</label>
                <input type="checkbox" defaultChecked />
              </div>
            </div>
          </Card>

          <Card className="p-4">
            <h3 className="text-sm font-semibold mb-3">变量</h3>
            <div className="space-y-2">
              <div className="flex items-center justify-between text-sm">
                <span className="text-muted-foreground">PlayerSpeed</span>
                <span className="font-mono text-xs">5.0</span>
              </div>
              <div className="flex items-center justify-between text-sm">
                <span className="text-muted-foreground">JumpHeight</span>
                <span className="font-mono text-xs">2.5</span>
              </div>
              <div className="flex items-center justify-between text-sm">
                <span className="text-muted-foreground">IsGrounded</span>
                <span className="font-mono text-xs">true</span>
              </div>
              <Button size="sm" variant="outline" className="w-full mt-2">
                <Plus className="w-3 h-3 mr-1" />
                添加变量
              </Button>
            </div>
          </Card>

          <Card className="p-4">
            <h3 className="text-sm font-semibold mb-3">生成代码</h3>
            <div className="bg-muted/50 rounded p-3 font-mono text-xs space-y-1">
              <div className="text-blue-400">function onGameStart() {"{"}</div>
              <div className="ml-2 text-green-400">const input = getPlayerInput();</div>
              <div className="ml-2 text-green-400">moveCharacter(input);</div>
              <div className="ml-2 text-green-400">playAnimation("walk");</div>
              <div className="text-blue-400">{"}"}</div>
            </div>
            <Button size="sm" variant="outline" className="w-full mt-3">
              <Eye className="w-3 h-3 mr-1" />
              查看完整代码
            </Button>
          </Card>
        </ScrollArea>
      </div>
    </div>
  );
}
