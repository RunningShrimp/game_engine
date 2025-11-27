import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import {
  Activity,
  ArrowRight,
  Circle,
  Play,
  Plus,
  Save,
  Settings,
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
    id: "entry",
    type: "input",
    data: { label: "入口" },
    position: { x: 100, y: 200 },
  },
  {
    id: "idle",
    data: { label: "待机" },
    position: { x: 300, y: 200 },
    style: { backgroundColor: "#3b82f6", color: "white" },
  },
  {
    id: "walk",
    data: { label: "行走" },
    position: { x: 500, y: 100 },
  },
  {
    id: "run",
    data: { label: "跑步" },
    position: { x: 500, y: 200 },
  },
  {
    id: "jump",
    data: { label: "跳跃" },
    position: { x: 500, y: 300 },
  },
  {
    id: "fall",
    data: { label: "下落" },
    position: { x: 700, y: 300 },
  },
];

const initialEdges: Edge[] = [
  {
    id: "e-entry-idle",
    source: "entry",
    target: "idle",
    animated: true,
    label: "开始",
  },
  {
    id: "e-idle-walk",
    source: "idle",
    target: "walk",
    label: "速度 > 0.1",
  },
  {
    id: "e-walk-idle",
    source: "walk",
    target: "idle",
    label: "速度 < 0.1",
  },
  {
    id: "e-walk-run",
    source: "walk",
    target: "run",
    label: "速度 > 5",
  },
  {
    id: "e-run-walk",
    source: "run",
    target: "walk",
    label: "速度 < 5",
  },
  {
    id: "e-idle-jump",
    source: "idle",
    target: "jump",
    label: "按下空格",
  },
  {
    id: "e-walk-jump",
    source: "walk",
    target: "jump",
    label: "按下空格",
  },
  {
    id: "e-jump-fall",
    source: "jump",
    target: "fall",
    label: "垂直速度 < 0",
  },
  {
    id: "e-fall-idle",
    source: "fall",
    target: "idle",
    label: "着地",
  },
];

export default function AnimationStateMachine() {
  const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);
  const [selectedState, setSelectedState] = useState<string | null>("idle");

  const onConnect = useCallback(
    (params: Connection) => {
      const newEdge = {
        ...params,
        label: "条件",
      };
      setEdges((eds) => addEdge(newEdge, eds));
      toast.success("已添加转换");
    },
    [setEdges]
  );

  const handleSave = () => {
    toast.success("状态机已保存");
  };

  const handleAddState = () => {
    const newNode: Node = {
      id: `state-${nodes.length}`,
      data: { label: "新状态" },
      position: { x: Math.random() * 400 + 200, y: Math.random() * 300 + 100 },
    };
    setNodes((nds) => [...nds, newNode]);
    toast.info("已添加新状态");
  };

  return (
    <div className="h-full flex">
      {/* 左侧状态列表 */}
      <div className="w-64 border-r border-border bg-card">
        <div className="h-12 border-b border-border flex items-center justify-between px-3">
          <span className="text-sm font-medium">状态列表</span>
          <Button
            size="sm"
            variant="ghost"
            className="h-7 w-7 p-0"
            onClick={handleAddState}
          >
            <Plus className="w-4 h-4" />
          </Button>
        </div>
        <ScrollArea className="h-[calc(100%-3rem)]">
          <div className="p-2 space-y-1">
            {nodes
              .filter((node) => node.id !== "entry")
              .map((node) => (
                <button
                  key={node.id}
                  className={`w-full text-left px-3 py-2 text-sm rounded flex items-center gap-2 transition-colors ${
                    selectedState === node.id
                      ? "bg-primary text-primary-foreground"
                      : "hover:bg-accent"
                  }`}
                  onClick={() => setSelectedState(node.id)}
                >
                  <Circle
                    className={`w-3 h-3 ${
                      node.id === "idle" ? "fill-current" : ""
                    }`}
                  />
                  <span>{node.data.label as string}</span>
                </button>
              ))}
          </div>
        </ScrollArea>
      </div>

      {/* 中间状态机编辑区 */}
      <div className="flex-1 flex flex-col">
        {/* 工具栏 */}
        <div className="h-12 border-b border-border bg-card flex items-center justify-between px-4">
          <div className="flex items-center gap-2">
            <Activity className="w-4 h-4 text-primary" />
            <span className="text-sm font-medium">角色动画状态机</span>
          </div>
          <div className="flex items-center gap-2">
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

        {/* React Flow 状态机编辑器 */}
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
          <span className="text-sm font-medium">状态属性</span>
        </div>
        <ScrollArea className="flex-1 p-4 space-y-4">
          {selectedState && (
            <>
              <Card className="p-4">
                <h3 className="text-sm font-semibold mb-3 flex items-center gap-2">
                  <Settings className="w-4 h-4" />
                  基本信息
                </h3>
                <div className="space-y-3">
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">
                      状态名称
                    </label>
                    <input
                      type="text"
                      className="w-full px-2 py-1 bg-input border border-border rounded text-xs"
                      defaultValue={
                        nodes.find((n) => n.id === selectedState)?.data
                          .label as string
                      }
                    />
                  </div>
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">
                      动画资产
                    </label>
                    <select className="w-full px-2 py-1 bg-input border border-border rounded text-xs">
                      <option>idle_anim</option>
                      <option>walk_anim</option>
                      <option>run_anim</option>
                      <option>jump_anim</option>
                    </select>
                  </div>
                  <div className="flex items-center justify-between">
                    <label className="text-xs font-medium">循环播放</label>
                    <input type="checkbox" defaultChecked />
                  </div>
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">
                      播放速度
                    </label>
                    <input
                      type="range"
                      className="w-full"
                      min="0.1"
                      max="2"
                      step="0.1"
                      defaultValue="1"
                    />
                  </div>
                </div>
              </Card>

              <Card className="p-4">
                <h3 className="text-sm font-semibold mb-3 flex items-center gap-2">
                  <ArrowRight className="w-4 h-4" />
                  转换规则
                </h3>
                <div className="space-y-2">
                  {edges
                    .filter((edge) => edge.source === selectedState)
                    .map((edge) => (
                      <Card
                        key={edge.id}
                        className="p-3 bg-muted/50 hover:bg-accent/50 cursor-pointer"
                      >
                        <div className="flex items-center justify-between mb-2">
                          <span className="text-xs font-medium">
                            → {nodes.find((n) => n.id === edge.target)?.data.label as string}
                          </span>
                          <Circle className="w-2 h-2 fill-current text-primary" />
                        </div>
                        <div className="text-[10px] text-muted-foreground">
                          {edge.label as string}
                        </div>
                      </Card>
                    ))}
                  {edges.filter((edge) => edge.source === selectedState)
                    .length === 0 && (
                    <div className="text-xs text-muted-foreground text-center py-4">
                      暂无转换规则
                    </div>
                  )}
                </div>
                <Button size="sm" variant="outline" className="w-full mt-3">
                  <Plus className="w-3 h-3 mr-1" />
                  添加转换
                </Button>
              </Card>

              <Card className="p-4">
                <h3 className="text-sm font-semibold mb-3">事件</h3>
                <div className="space-y-2">
                  <div className="text-xs">
                    <div className="font-medium mb-1">进入时</div>
                    <div className="bg-muted/50 rounded p-2 font-mono text-[10px]">
                      onEnter() {"{ }"}
                    </div>
                  </div>
                  <div className="text-xs">
                    <div className="font-medium mb-1">更新时</div>
                    <div className="bg-muted/50 rounded p-2 font-mono text-[10px]">
                      onUpdate() {"{ }"}
                    </div>
                  </div>
                  <div className="text-xs">
                    <div className="font-medium mb-1">退出时</div>
                    <div className="bg-muted/50 rounded p-2 font-mono text-[10px]">
                      onExit() {"{ }"}
                    </div>
                  </div>
                </div>
              </Card>

              <Card className="p-4">
                <h3 className="text-sm font-semibold mb-3">混合设置</h3>
                <div className="space-y-3">
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">
                      混合时间 (秒)
                    </label>
                    <input
                      type="number"
                      className="w-full px-2 py-1 bg-input border border-border rounded text-xs"
                      defaultValue="0.2"
                      step="0.1"
                    />
                  </div>
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">
                      混合模式
                    </label>
                    <select className="w-full px-2 py-1 bg-input border border-border rounded text-xs">
                      <option>线性</option>
                      <option>缓入</option>
                      <option>缓出</option>
                      <option>平滑</option>
                    </select>
                  </div>
                </div>
              </Card>
            </>
          )}
        </ScrollArea>
      </div>
    </div>
  );
}
