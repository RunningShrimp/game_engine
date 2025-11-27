import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import {
  Activity,
  Box,
  Layers,
  Play,
  Save,
  Shuffle,
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
    data: { label: "输入姿势" },
    position: { x: 100, y: 150 },
  },
  {
    id: "2",
    data: { label: "混合空间" },
    position: { x: 300, y: 150 },
  },
  {
    id: "3",
    data: { label: "IK调整" },
    position: { x: 500, y: 150 },
  },
  {
    id: "4",
    type: "output",
    data: { label: "输出姿势" },
    position: { x: 700, y: 150 },
  },
];

const initialEdges: Edge[] = [
  { id: "e1-2", source: "1", target: "2", animated: true },
  { id: "e2-3", source: "2", target: "3" },
  { id: "e3-4", source: "3", target: "4", animated: true },
];

export default function AnimationBlueprint() {
  const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);
  const [selectedTab, setSelectedTab] = useState<string>("graph");

  const onConnect = useCallback(
    (params: Connection) => setEdges((eds) => addEdge(params, eds)),
    [setEdges]
  );

  const nodeTypes = [
    {
      category: "动画",
      items: [
        { name: "播放动画", icon: Play, color: "text-blue-500" },
        { name: "混合空间", icon: Shuffle, color: "text-purple-500" },
        { name: "动画序列", icon: Layers, color: "text-cyan-500" },
      ],
    },
    {
      category: "混合",
      items: [
        { name: "混合两个动画", icon: Zap, color: "text-yellow-500" },
        { name: "叠加混合", icon: Activity, color: "text-orange-500" },
      ],
    },
    {
      category: "IK",
      items: [
        { name: "双骨骼 IK", icon: Box, color: "text-green-500" },
        { name: "FABRIK", icon: Box, color: "text-green-500" },
      ],
    },
    {
      category: "物理",
      items: [
        { name: "布娃娃", icon: Zap, color: "text-red-500" },
        { name: "物理混合", icon: Activity, color: "text-orange-500" },
        { name: "碰撞响应", icon: Layers, color: "text-yellow-500" },
      ],
    },
  ];

  const handleSave = () => {
    toast.success("动画蓝图已保存");
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
          <span className="text-sm font-medium">动画节点</span>
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

      {/* 中间编辑区 */}
      <div className="flex-1 flex flex-col">
        {/* 工具栏 */}
        <div className="h-12 border-b border-border bg-card flex items-center justify-between px-4">
          <div className="flex items-center gap-2">
            <Activity className="w-4 h-4 text-primary" />
            <span className="text-sm font-medium">角色动画蓝图</span>
          </div>
          <div className="flex items-center gap-2">
            <div className="flex gap-1 mr-4">
              <Button
                size="sm"
                variant={selectedTab === "graph" ? "default" : "ghost"}
                className="h-7 px-3 text-xs"
                onClick={() => setSelectedTab("graph")}
              >
                动画图
              </Button>
              <Button
                size="sm"
                variant={selectedTab === "state" ? "default" : "ghost"}
                className="h-7 px-3 text-xs"
                onClick={() => setSelectedTab("state")}
              >
                状态机
              </Button>
            </div>
            <Button size="sm" variant="ghost" className="h-8 gap-2" onClick={handleSave}>
              <Save className="w-4 h-4" />
              <span className="text-xs">保存</span>
            </Button>
            <Separator orientation="vertical" className="h-6" />
            <Button size="sm" variant="ghost" className="h-8 gap-2">
              <Play className="w-4 h-4" />
              <span className="text-xs">预览</span>
            </Button>
          </div>
        </div>

        {/* 动画图编辑器 */}
        {selectedTab === "graph" && (
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
        )}

        {/* 状态机视图 */}
        {selectedTab === "state" && (
          <div className="flex-1 bg-background p-8">
            <div className="max-w-4xl mx-auto">
              <Card className="p-8">
                <h3 className="text-lg font-semibold mb-6 text-center">
                  动画状态机
                </h3>
                <div className="space-y-6">
                  <div className="flex items-center justify-center gap-8">
                    <Card className="p-6 bg-primary/10 border-primary">
                      <div className="text-center">
                        <div className="w-16 h-16 mx-auto mb-3 rounded-lg bg-primary/20 flex items-center justify-center">
                          <Activity className="w-8 h-8 text-primary" />
                        </div>
                        <div className="font-semibold">待机</div>
                      </div>
                    </Card>
                    <div className="text-2xl text-muted-foreground">→</div>
                    <Card className="p-6 hover:bg-accent/50 cursor-pointer">
                      <div className="text-center">
                        <div className="w-16 h-16 mx-auto mb-3 rounded-lg bg-muted flex items-center justify-center">
                          <Activity className="w-8 h-8" />
                        </div>
                        <div className="font-semibold">行走</div>
                      </div>
                    </Card>
                    <div className="text-2xl text-muted-foreground">→</div>
                    <Card className="p-6 hover:bg-accent/50 cursor-pointer">
                      <div className="text-center">
                        <div className="w-16 h-16 mx-auto mb-3 rounded-lg bg-muted flex items-center justify-center">
                          <Activity className="w-8 h-8" />
                        </div>
                        <div className="font-semibold">跑步</div>
                      </div>
                    </Card>
                  </div>
                  <div className="flex items-center justify-center gap-8">
                    <Card className="p-6 hover:bg-accent/50 cursor-pointer">
                      <div className="text-center">
                        <div className="w-16 h-16 mx-auto mb-3 rounded-lg bg-muted flex items-center justify-center">
                          <Activity className="w-8 h-8" />
                        </div>
                        <div className="font-semibold">跳跃</div>
                      </div>
                    </Card>
                    <div className="text-2xl text-muted-foreground">→</div>
                    <Card className="p-6 hover:bg-accent/50 cursor-pointer">
                      <div className="text-center">
                        <div className="w-16 h-16 mx-auto mb-3 rounded-lg bg-muted flex items-center justify-center">
                          <Activity className="w-8 h-8" />
                        </div>
                        <div className="font-semibold">下落</div>
                      </div>
                    </Card>
                    <div className="text-2xl text-muted-foreground">→</div>
                    <Card className="p-6 hover:bg-accent/50 cursor-pointer">
                      <div className="text-center">
                        <div className="w-16 h-16 mx-auto mb-3 rounded-lg bg-muted flex items-center justify-center">
                          <Activity className="w-8 h-8" />
                        </div>
                        <div className="font-semibold">着陆</div>
                      </div>
                    </Card>
                  </div>
                </div>
              </Card>
            </div>
          </div>
        )}
      </div>

      {/* 右侧属性面板 */}
      <div className="w-80 border-l border-border bg-card flex flex-col">
        <div className="h-12 border-b border-border flex items-center px-3">
          <span className="text-sm font-medium">节点属性</span>
        </div>
        <ScrollArea className="flex-1 p-4 space-y-4">
          <Card className="p-4">
            <h3 className="text-sm font-semibold mb-3">混合空间</h3>
            <div className="space-y-3">
              <div>
                <label className="text-xs font-medium text-muted-foreground block mb-1">
                  混合参数
                </label>
                <select className="w-full px-2 py-1 bg-input border border-border rounded text-xs">
                  <option>速度</option>
                  <option>方向</option>
                  <option>高度</option>
                </select>
              </div>
              <div>
                <label className="text-xs font-medium text-muted-foreground block mb-1">
                  混合权重
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
            <h3 className="text-sm font-semibold mb-3">动画列表</h3>
            <div className="space-y-2">
              {["idle", "walk", "run", "jump"].map((anim) => (
                <div
                  key={anim}
                  className="flex items-center justify-between text-sm p-2 rounded hover:bg-accent cursor-pointer"
                >
                  <span>{anim}</span>
                  <Play className="w-3 h-3 text-muted-foreground" />
                </div>
              ))}
            </div>
          </Card>

          <Card className="p-4">
            <h3 className="text-sm font-semibold mb-3">IK设置</h3>
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <label className="text-xs font-medium">启用手部IK</label>
                <input type="checkbox" defaultChecked />
              </div>
              <div className="flex items-center justify-between">
                <label className="text-xs font-medium">启用脚部IK</label>
                <input type="checkbox" defaultChecked />
              </div>
              <div>
                <label className="text-xs font-medium text-muted-foreground block mb-1">
                  IK强度
                </label>
                <input
                  type="range"
                  className="w-full"
                  min="0"
                  max="1"
                  step="0.01"
                  defaultValue="1"
                />
              </div>
            </div>
          </Card>

          <Card className="p-4">
            <h3 className="text-sm font-semibold mb-3">物理动画</h3>
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <label className="text-xs font-medium">启用物理</label>
                <input type="checkbox" />
              </div>
              <div>
                <label className="text-xs font-medium text-muted-foreground block mb-1">
                  物理混合权重
                </label>
                <input
                  type="range"
                  className="w-full"
                  min="0"
                  max="1"
                  step="0.01"
                  defaultValue="0"
                />
              </div>
              <div>
                <label className="text-xs font-medium text-muted-foreground block mb-1">
                  碰撞响应强度
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
            <h3 className="text-sm font-semibold mb-3">根运动</h3>
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <label className="text-xs font-medium">启用根运动</label>
                <input type="checkbox" defaultChecked />
              </div>
              <div>
                <label className="text-xs font-medium text-muted-foreground block mb-1">
                  根骨骼
                </label>
                <select className="w-full px-2 py-1 bg-input border border-border rounded text-xs">
                  <option>Root</option>
                  <option>Hips</option>
                  <option>Pelvis</option>
                </select>
              </div>
            </div>
          </Card>
        </ScrollArea>
      </div>
    </div>
  );
}
