import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import {
  AlertCircle,
  CheckCircle,
  Circle,
  GitBranch,
  Play,
  Plus,
  Save,
  Shuffle,
  Zap,
} from "lucide-react";
import { useState } from "react";
import ReactFlow, {
  Background,
  Controls,
  MiniMap,
  Node,
  Edge,
} from "reactflow";
import "reactflow/dist/style.css";
import { toast } from "sonner";

export default function BehaviorTree() {
  const [selectedNode, setSelectedNode] = useState<string | null>(null);

  const initialNodes: Node[] = [
    {
      id: "1",
      type: "default",
      data: { label: "根节点 (Root)" },
      position: { x: 400, y: 50 },
      style: {
        background: "hsl(var(--primary))",
        color: "hsl(var(--primary-foreground))",
        border: "2px solid hsl(var(--primary))",
        borderRadius: "8px",
        padding: "10px",
      },
    },
    {
      id: "2",
      type: "default",
      data: { label: "选择器 (Selector)" },
      position: { x: 400, y: 150 },
      style: {
        background: "hsl(var(--card))",
        color: "hsl(var(--card-foreground))",
        border: "2px solid hsl(var(--border))",
        borderRadius: "8px",
        padding: "10px",
      },
    },
    {
      id: "3",
      type: "default",
      data: { label: "序列 (Sequence)" },
      position: { x: 200, y: 250 },
      style: {
        background: "hsl(var(--card))",
        color: "hsl(var(--card-foreground))",
        border: "2px solid hsl(var(--border))",
        borderRadius: "8px",
        padding: "10px",
      },
    },
    {
      id: "4",
      type: "default",
      data: { label: "条件: 敌人在视野内" },
      position: { x: 600, y: 250 },
      style: {
        background: "#fbbf24",
        color: "#000",
        border: "2px solid #f59e0b",
        borderRadius: "8px",
        padding: "10px",
      },
    },
    {
      id: "5",
      type: "default",
      data: { label: "动作: 巡逻" },
      position: { x: 100, y: 350 },
      style: {
        background: "#34d399",
        color: "#000",
        border: "2px solid #10b981",
        borderRadius: "8px",
        padding: "10px",
      },
    },
    {
      id: "6",
      type: "default",
      data: { label: "动作: 追击" },
      position: { x: 300, y: 350 },
      style: {
        background: "#34d399",
        color: "#000",
        border: "2px solid #10b981",
        borderRadius: "8px",
        padding: "10px",
      },
    },
    {
      id: "7",
      type: "default",
      data: { label: "动作: 攻击" },
      position: { x: 600, y: 350 },
      style: {
        background: "#f87171",
        color: "#000",
        border: "2px solid #ef4444",
        borderRadius: "8px",
        padding: "10px",
      },
    },
  ];

  const initialEdges: Edge[] = [
    { id: "e1-2", source: "1", target: "2", animated: true },
    { id: "e2-3", source: "2", target: "3" },
    { id: "e2-4", source: "2", target: "4" },
    { id: "e3-5", source: "3", target: "5" },
    { id: "e3-6", source: "3", target: "6" },
    { id: "e4-7", source: "4", target: "7" },
  ];

  const [nodes] = useState<Node[]>(initialNodes);
  const [edges] = useState<Edge[]>(initialEdges);

  const nodeTypes = [
    {
      category: "复合节点",
      items: [
        { name: "选择器", icon: Shuffle, color: "text-blue-500" },
        { name: "序列", icon: GitBranch, color: "text-purple-500" },
        { name: "并行", icon: Circle, color: "text-cyan-500" },
      ],
    },
    {
      category: "装饰节点",
      items: [
        { name: "反转", icon: Zap, color: "text-orange-500" },
        { name: "重复", icon: Circle, color: "text-pink-500" },
        { name: "直到成功", icon: CheckCircle, color: "text-green-500" },
      ],
    },
    {
      category: "条件节点",
      items: [
        { name: "检查距离", icon: AlertCircle, color: "text-yellow-500" },
        { name: "检查生命值", icon: AlertCircle, color: "text-yellow-500" },
        { name: "检查视野", icon: AlertCircle, color: "text-yellow-500" },
      ],
    },
    {
      category: "动作节点",
      items: [
        { name: "移动到", icon: Play, color: "text-green-500" },
        { name: "攻击", icon: Zap, color: "text-red-500" },
        { name: "等待", icon: Circle, color: "text-gray-500" },
      ],
    },
  ];

  const handleSave = () => {
    toast.success("行为树已保存");
  };

  const handleRun = () => {
    toast.success("开始运行行为树");
  };

  return (
    <div className="h-full flex">
      {/* 左侧节点库 */}
      <div className="w-64 border-r border-border bg-card">
        <div className="h-12 border-b border-border flex items-center justify-between px-3">
          <span className="text-sm font-medium">节点库</span>
        </div>
        <ScrollArea className="h-[calc(100%-3rem)]">
          <div className="p-2 space-y-4">
            {nodeTypes.map((category) => (
              <div key={category.category}>
                <div className="text-xs font-semibold text-muted-foreground mb-2 px-2">
                  {category.category}
                </div>
                <div className="space-y-1">
                  {category.items.map((node) => (
                    <button
                      key={node.name}
                      className="w-full text-left px-3 py-2 text-sm rounded hover:bg-accent transition-colors flex items-center gap-2"
                      draggable
                    >
                      <node.icon className={`w-4 h-4 ${node.color}`} />
                      <span>{node.name}</span>
                    </button>
                  ))}
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
            <GitBranch className="w-4 h-4 text-primary" />
            <span className="text-sm font-medium">行为树编辑器</span>
          </div>
          <div className="flex items-center gap-2">
            <Button size="sm" variant="ghost" className="h-8 gap-2">
              <Plus className="w-4 h-4" />
              <span className="text-xs">新建</span>
            </Button>
            <Separator orientation="vertical" className="h-6" />
            <Button size="sm" variant="ghost" className="h-8 gap-2" onClick={handleSave}>
              <Save className="w-4 h-4" />
              <span className="text-xs">保存</span>
            </Button>
            <Button size="sm" className="h-8 gap-2" onClick={handleRun}>
              <Play className="w-4 h-4" />
              <span className="text-xs">运行</span>
            </Button>
          </div>
        </div>

        {/* React Flow 画布 */}
        <div className="flex-1 bg-background">
          <ReactFlow
            nodes={nodes}
            edges={edges}
            onNodeClick={(_, node) => setSelectedNode(node.id)}
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
          {selectedNode ? (
            <>
              <Card className="p-4">
                <h3 className="text-sm font-semibold mb-3">基本信息</h3>
                <div className="space-y-3">
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">
                      节点名称
                    </label>
                    <input
                      type="text"
                      className="w-full px-2 py-1 bg-input border border-border rounded text-xs"
                      defaultValue={
                        nodes.find((n) => n.id === selectedNode)?.data.label
                      }
                    />
                  </div>
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">
                      节点类型
                    </label>
                    <select className="w-full px-2 py-1 bg-input border border-border rounded text-xs">
                      <option>复合节点</option>
                      <option>装饰节点</option>
                      <option>条件节点</option>
                      <option>动作节点</option>
                    </select>
                  </div>
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">
                      描述
                    </label>
                    <textarea
                      className="w-full px-2 py-1 bg-input border border-border rounded text-xs resize-none"
                      rows={3}
                      placeholder="节点描述..."
                    />
                  </div>
                </div>
              </Card>

              <Card className="p-4">
                <h3 className="text-sm font-semibold mb-3">执行设置</h3>
                <div className="space-y-3">
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">
                      执行策略
                    </label>
                    <select className="w-full px-2 py-1 bg-input border border-border rounded text-xs">
                      <option>顺序执行</option>
                      <option>随机执行</option>
                      <option>优先级执行</option>
                    </select>
                  </div>
                  <div className="flex items-center justify-between">
                    <label className="text-xs font-medium">失败时中断</label>
                    <input type="checkbox" defaultChecked />
                  </div>
                  <div className="flex items-center justify-between">
                    <label className="text-xs font-medium">成功时继续</label>
                    <input type="checkbox" defaultChecked />
                  </div>
                </div>
              </Card>

              <Card className="p-4">
                <h3 className="text-sm font-semibold mb-3">条件参数</h3>
                <div className="space-y-3">
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">
                      检查目标
                    </label>
                    <select className="w-full px-2 py-1 bg-input border border-border rounded text-xs">
                      <option>敌人</option>
                      <option>玩家</option>
                      <option>友军</option>
                      <option>物品</option>
                    </select>
                  </div>
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">
                      检查距离 (米)
                    </label>
                    <input
                      type="number"
                      className="w-full px-2 py-1 bg-input border border-border rounded text-xs"
                      defaultValue="10"
                    />
                  </div>
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">
                      视野角度 (度)
                    </label>
                    <input
                      type="range"
                      className="w-full"
                      min="0"
                      max="360"
                      defaultValue="90"
                    />
                  </div>
                </div>
              </Card>

              <Card className="p-4">
                <h3 className="text-sm font-semibold mb-3">调试信息</h3>
                <div className="space-y-2 text-xs">
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">执行次数</span>
                    <span className="font-medium">42</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">成功次数</span>
                    <span className="font-medium text-green-500">38</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">失败次数</span>
                    <span className="font-medium text-red-500">4</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">平均耗时</span>
                    <span className="font-medium">12ms</span>
                  </div>
                </div>
              </Card>

              <Card className="p-4">
                <h3 className="text-sm font-semibold mb-3">黑板变量</h3>
                <div className="space-y-2">
                  {[
                    { name: "targetEnemy", type: "Object", value: "null" },
                    { name: "patrolPoint", type: "Vector3", value: "(0,0,0)" },
                    { name: "isAlert", type: "Boolean", value: "false" },
                  ].map((variable) => (
                    <div
                      key={variable.name}
                      className="p-2 bg-muted/50 rounded text-xs"
                    >
                      <div className="flex items-center justify-between mb-1">
                        <span className="font-medium">{variable.name}</span>
                        <span className="text-muted-foreground">
                          {variable.type}
                        </span>
                      </div>
                      <div className="text-muted-foreground">
                        {variable.value}
                      </div>
                    </div>
                  ))}
                  <Button size="sm" variant="outline" className="w-full mt-2">
                    <Plus className="w-3 h-3 mr-1" />
                    添加变量
                  </Button>
                </div>
              </Card>
            </>
          ) : (
            <Card className="p-6 text-center">
              <GitBranch className="w-12 h-12 mx-auto mb-3 text-muted-foreground" />
              <p className="text-sm text-muted-foreground">
                选择一个节点查看属性
              </p>
            </Card>
          )}
        </ScrollArea>
      </div>
    </div>
  );
}
