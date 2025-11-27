import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import {
  Bot,
  MessageCircle,
  MessageSquare,
  Play,
  Plus,
  Save,
  Sparkles,
  User,
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

export default function DialogueSystem() {
  const [selectedNode, setSelectedNode] = useState<string | null>(null);

  const initialNodes: Node[] = [
    {
      id: "1",
      type: "default",
      data: { label: "开始对话" },
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
      data: { label: "NPC: 你好，旅行者！" },
      position: { x: 400, y: 150 },
      style: {
        background: "#60a5fa",
        color: "#000",
        border: "2px solid #3b82f6",
        borderRadius: "8px",
        padding: "10px",
      },
    },
    {
      id: "3",
      type: "default",
      data: { label: "玩家: 你好！" },
      position: { x: 200, y: 250 },
      style: {
        background: "#34d399",
        color: "#000",
        border: "2px solid #10b981",
        borderRadius: "8px",
        padding: "10px",
      },
    },
    {
      id: "4",
      type: "default",
      data: { label: "玩家: 有什么任务吗？" },
      position: { x: 600, y: 250 },
      style: {
        background: "#34d399",
        color: "#000",
        border: "2px solid #10b981",
        borderRadius: "8px",
        padding: "10px",
      },
    },
    {
      id: "5",
      type: "default",
      data: { label: "NPC: 很高兴见到你！" },
      position: { x: 200, y: 350 },
      style: {
        background: "#60a5fa",
        color: "#000",
        border: "2px solid #3b82f6",
        borderRadius: "8px",
        padding: "10px",
      },
    },
    {
      id: "6",
      type: "default",
      data: { label: "NPC: 我需要你帮忙..." },
      position: { x: 600, y: 350 },
      style: {
        background: "#60a5fa",
        color: "#000",
        border: "2px solid #3b82f6",
        borderRadius: "8px",
        padding: "10px",
      },
    },
    {
      id: "7",
      type: "default",
      data: { label: "条件: 任务已完成" },
      position: { x: 600, y: 450 },
      style: {
        background: "#fbbf24",
        color: "#000",
        border: "2px solid #f59e0b",
        borderRadius: "8px",
        padding: "10px",
      },
    },
  ];

  const initialEdges: Edge[] = [
    { id: "e1-2", source: "1", target: "2", animated: true },
    { id: "e2-3", source: "2", target: "3", label: "选项1" },
    { id: "e2-4", source: "2", target: "4", label: "选项2" },
    { id: "e3-5", source: "3", target: "5" },
    { id: "e4-6", source: "4", target: "6" },
    { id: "e6-7", source: "6", target: "7" },
  ];

  const [nodes] = useState<Node[]>(initialNodes);
  const [edges] = useState<Edge[]>(initialEdges);

  const handleSave = () => {
    toast.success("对话已保存");
  };

  const handleTest = () => {
    toast.success("开始测试对话");
  };

  return (
    <div className="h-full flex">
      {/* 左侧NPC列表 */}
      <div className="w-64 border-r border-border bg-card">
        <div className="h-12 border-b border-border flex items-center justify-between px-3">
          <span className="text-sm font-medium">NPC列表</span>
          <Button size="sm" variant="ghost" className="h-7 w-7 p-0">
            <Plus className="w-4 h-4" />
          </Button>
        </div>
        <ScrollArea className="h-[calc(100%-3rem)]">
          <div className="p-2 space-y-1">
            {[
              { id: "npc1", name: "村长", role: "任务发布者" },
              { id: "npc2", name: "铁匠", role: "商人" },
              { id: "npc3", name: "守卫", role: "守卫" },
              { id: "npc4", name: "旅行商人", role: "商人" },
            ].map((npc) => (
              <button
                key={npc.id}
                className="w-full text-left px-3 py-2 text-sm rounded hover:bg-accent transition-colors"
              >
                <div className="flex items-center gap-2 mb-1">
                  <User className="w-3 h-3" />
                  <span className="font-medium">{npc.name}</span>
                </div>
                <div className="text-xs text-muted-foreground">{npc.role}</div>
              </button>
            ))}
          </div>
        </ScrollArea>
      </div>

      {/* 中间编辑区 */}
      <div className="flex-1 flex flex-col">
        {/* 工具栏 */}
        <div className="h-12 border-b border-border bg-card flex items-center justify-between px-4">
          <div className="flex items-center gap-2">
            <MessageCircle className="w-4 h-4 text-primary" />
            <span className="text-sm font-medium">对话系统</span>
          </div>
          <div className="flex items-center gap-2">
            <Button size="sm" variant="ghost" className="h-8 gap-2">
              <Plus className="w-4 h-4" />
              <span className="text-xs">新建节点</span>
            </Button>
            <Separator orientation="vertical" className="h-6" />
            <Button size="sm" variant="ghost" className="h-8 gap-2" onClick={handleSave}>
              <Save className="w-4 h-4" />
              <span className="text-xs">保存</span>
            </Button>
            <Button size="sm" className="h-8 gap-2" onClick={handleTest}>
              <Play className="w-4 h-4" />
              <span className="text-xs">测试</span>
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
          <span className="text-sm font-medium">对话属性</span>
        </div>
        <ScrollArea className="flex-1 p-4 space-y-4">
          {selectedNode ? (
            <>
              <Card className="p-4">
                <h3 className="text-sm font-semibold mb-3">对话内容</h3>
                <div className="space-y-3">
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">
                      说话者
                    </label>
                    <select className="w-full px-2 py-1 bg-input border border-border rounded text-xs">
                      <option>NPC</option>
                      <option>玩家</option>
                      <option>旁白</option>
                    </select>
                  </div>
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">
                      对话文本
                    </label>
                    <textarea
                      className="w-full px-2 py-1 bg-input border border-border rounded text-xs resize-none"
                      rows={4}
                      defaultValue={
                        nodes.find((n) => n.id === selectedNode)?.data.label
                      }
                    />
                  </div>
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">
                      语音文件
                    </label>
                    <input
                      type="text"
                      className="w-full px-2 py-1 bg-input border border-border rounded text-xs"
                      placeholder="voice_001.mp3"
                    />
                  </div>
                </div>
              </Card>

              <Card className="p-4">
                <h3 className="text-sm font-semibold mb-3">表情动画</h3>
                <div className="space-y-3">
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">
                      面部表情
                    </label>
                    <select className="w-full px-2 py-1 bg-input border border-border rounded text-xs">
                      <option>中性</option>
                      <option>开心</option>
                      <option>悲伤</option>
                      <option>愤怒</option>
                      <option>惊讶</option>
                    </select>
                  </div>
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">
                      肢体动作
                    </label>
                    <select className="w-full px-2 py-1 bg-input border border-border rounded text-xs">
                      <option>无</option>
                      <option>点头</option>
                      <option>摇头</option>
                      <option>挥手</option>
                      <option>指向</option>
                    </select>
                  </div>
                </div>
              </Card>

              <Card className="p-4">
                <h3 className="text-sm font-semibold mb-3">条件设置</h3>
                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <label className="text-xs font-medium">启用条件</label>
                    <input type="checkbox" />
                  </div>
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">
                      条件类型
                    </label>
                    <select className="w-full px-2 py-1 bg-input border border-border rounded text-xs">
                      <option>任务状态</option>
                      <option>物品拥有</option>
                      <option>等级要求</option>
                      <option>好感度</option>
                    </select>
                  </div>
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">
                      条件值
                    </label>
                    <input
                      type="text"
                      className="w-full px-2 py-1 bg-input border border-border rounded text-xs"
                      placeholder="quest_001_completed"
                    />
                  </div>
                </div>
              </Card>

              <Card className="p-4">
                <h3 className="text-sm font-semibold mb-3">事件触发</h3>
                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <label className="text-xs font-medium">触发事件</label>
                    <input type="checkbox" />
                  </div>
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">
                      事件类型
                    </label>
                    <select className="w-full px-2 py-1 bg-input border border-border rounded text-xs">
                      <option>接受任务</option>
                      <option>完成任务</option>
                      <option>获得物品</option>
                      <option>增加好感度</option>
                      <option>播放动画</option>
                    </select>
                  </div>
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">
                      事件参数
                    </label>
                    <input
                      type="text"
                      className="w-full px-2 py-1 bg-input border border-border rounded text-xs"
                      placeholder='{"questId": "quest_001"}'
                    />
                  </div>
                </div>
              </Card>

              <Card className="p-4">
                <h3 className="text-sm font-semibold mb-3">AI生成</h3>
                <div className="space-y-3">
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">
                      角色性格
                    </label>
                    <select className="w-full px-2 py-1 bg-input border border-border rounded text-xs">
                      <option>友善</option>
                      <option>严肃</option>
                      <option>幽默</option>
                      <option>神秘</option>
                    </select>
                  </div>
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">
                      对话场景
                    </label>
                    <textarea
                      className="w-full px-2 py-1 bg-input border border-border rounded text-xs resize-none"
                      rows={3}
                      placeholder="描述对话场景和背景..."
                    />
                  </div>
                  <Button size="sm" variant="outline" className="w-full">
                    <Sparkles className="w-3 h-3 mr-2" />
                    AI生成对话
                  </Button>
                </div>
              </Card>

              <Card className="p-4">
                <h3 className="text-sm font-semibold mb-3">本地化</h3>
                <div className="space-y-2">
                  {[
                    { lang: "中文", text: "你好，旅行者！" },
                    { lang: "English", text: "Hello, traveler!" },
                    { lang: "日本語", text: "こんにちは、旅人！" },
                  ].map((translation) => (
                    <div
                      key={translation.lang}
                      className="p-2 bg-muted/50 rounded text-xs"
                    >
                      <div className="font-medium mb-1">{translation.lang}</div>
                      <div className="text-muted-foreground">
                        {translation.text}
                      </div>
                    </div>
                  ))}
                  <Button size="sm" variant="outline" className="w-full mt-2">
                    <Plus className="w-3 h-3 mr-1" />
                    添加语言
                  </Button>
                </div>
              </Card>
            </>
          ) : (
            <Card className="p-6 text-center">
              <MessageSquare className="w-12 h-12 mx-auto mb-3 text-muted-foreground" />
              <p className="text-sm text-muted-foreground">
                选择一个对话节点查看属性
              </p>
            </Card>
          )}
        </ScrollArea>
      </div>
    </div>
  );
}
