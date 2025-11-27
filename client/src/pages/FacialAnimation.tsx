import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import {
  Copy,
  Folder,
  Play,
  Plus,
  Save,
  Smile,
  Sparkles,
} from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";

export default function FacialAnimation() {
  const [selectedExpression, setSelectedExpression] = useState<string>("neutral");

  const expressions = [
    { id: "neutral", name: "中性", emoji: "😐" },
    { id: "happy", name: "开心", emoji: "😊" },
    { id: "sad", name: "悲伤", emoji: "😢" },
    { id: "angry", name: "愤怒", emoji: "😠" },
    { id: "surprised", name: "惊讶", emoji: "😮" },
    { id: "fear", name: "恐惧", emoji: "😨" },
    { id: "disgust", name: "厌恶", emoji: "🤢" },
    { id: "smile", name: "微笑", emoji: "🙂" },
  ];

  const blendShapes = [
    { name: "browInnerUp", value: 0, category: "眉毛" },
    { name: "browDown_L", value: 0, category: "眉毛" },
    { name: "browDown_R", value: 0, category: "眉毛" },
    { name: "eyeBlink_L", value: 0, category: "眼睛" },
    { name: "eyeBlink_R", value: 0, category: "眼睛" },
    { name: "eyeWide_L", value: 0, category: "眼睛" },
    { name: "eyeWide_R", value: 0, category: "眼睛" },
    { name: "jawOpen", value: 0, category: "嘴巴" },
    { name: "mouthSmile_L", value: 0, category: "嘴巴" },
    { name: "mouthSmile_R", value: 0, category: "嘴巴" },
    { name: "mouthFrown_L", value: 0, category: "嘴巴" },
    { name: "mouthFrown_R", value: 0, category: "嘴巴" },
  ];

  const handleSave = () => {
    toast.success("表情已保存");
  };

  const handleCopy = () => {
    toast.success("已复制到剪贴板");
  };

  const handleCreateExpression = () => {
    toast.success("正在创建新表情");
  };

  return (
    <div className="h-full flex">
      {/* 左侧表情库 */}
      <div className="w-64 border-r border-border bg-card">
        <div className="h-12 border-b border-border flex items-center justify-between px-3">
          <span className="text-sm font-medium">表情库</span>
          <Button
            size="sm"
            variant="ghost"
            className="h-7 w-7 p-0"
            onClick={handleCreateExpression}
          >
            <Plus className="w-4 h-4" />
          </Button>
        </div>
        <ScrollArea className="h-[calc(100%-3rem)]">
          <div className="p-2 space-y-1">
            {expressions.map((expr) => (
              <button
                key={expr.id}
                className={`w-full text-left px-3 py-2 text-sm rounded flex items-center gap-3 transition-colors ${
                  selectedExpression === expr.id
                    ? "bg-primary text-primary-foreground"
                    : "hover:bg-accent"
                }`}
                onClick={() => setSelectedExpression(expr.id)}
              >
                <span className="text-2xl">{expr.emoji}</span>
                <span>{expr.name}</span>
              </button>
            ))}
          </div>
        </ScrollArea>
      </div>

      {/* 中间预览区 */}
      <div className="flex-1 flex flex-col">
        {/* 工具栏 */}
        <div className="h-12 border-b border-border bg-card flex items-center justify-between px-4">
          <div className="flex items-center gap-2">
            <Smile className="w-4 h-4 text-primary" />
            <span className="text-sm font-medium">面部动画编辑器</span>
          </div>
          <div className="flex items-center gap-2">
            <Button size="sm" variant="ghost" className="h-8 gap-2" onClick={handleCopy}>
              <Copy className="w-4 h-4" />
              <span className="text-xs">复制</span>
            </Button>
            <Separator orientation="vertical" className="h-6" />
            <Button size="sm" variant="ghost" className="h-8 gap-2" onClick={handleSave}>
              <Save className="w-4 h-4" />
              <span className="text-xs">保存</span>
            </Button>
            <Button size="sm" variant="ghost" className="h-8 gap-2">
              <Play className="w-4 h-4" />
              <span className="text-xs">预览</span>
            </Button>
          </div>
        </div>

        {/* 3D面部预览 */}
        <div className="flex-1 bg-background flex items-center justify-center">
          <div className="text-center space-y-4">
            <div className="w-96 h-96 mx-auto bg-gradient-to-br from-primary/20 to-purple-500/20 rounded-full flex items-center justify-center">
              <div className="text-9xl">
                {expressions.find((e) => e.id === selectedExpression)?.emoji}
              </div>
            </div>
            <div className="text-sm text-muted-foreground">
              {expressions.find((e) => e.id === selectedExpression)?.name}
            </div>
          </div>
        </div>

        {/* 底部混合形状控制 */}
        <div className="h-48 border-t border-border bg-card p-4">
          <div className="flex items-center justify-between mb-3">
            <span className="text-xs font-medium">混合形状 (Blend Shapes)</span>
            <Button size="sm" variant="ghost" className="h-6 px-2 text-xs">
              重置全部
            </Button>
          </div>
          <ScrollArea className="h-[calc(100%-2rem)]">
            <div className="grid grid-cols-3 gap-3">
              {blendShapes.map((shape) => (
                <div key={shape.name} className="space-y-1">
                  <div className="flex items-center justify-between">
                    <label className="text-xs font-medium">{shape.name}</label>
                    <span className="text-xs text-muted-foreground">
                      {shape.value.toFixed(2)}
                    </span>
                  </div>
                  <input
                    type="range"
                    className="w-full"
                    min="0"
                    max="1"
                    step="0.01"
                    defaultValue={shape.value}
                  />
                </div>
              ))}
            </div>
          </ScrollArea>
        </div>
      </div>

      {/* 右侧属性面板 */}
      <div className="w-80 border-l border-border bg-card flex flex-col">
        <div className="h-12 border-b border-border flex items-center px-3">
          <span className="text-sm font-medium">表情属性</span>
        </div>
        <ScrollArea className="flex-1 p-4 space-y-4">
          <Card className="p-4">
            <h3 className="text-sm font-semibold mb-3">基本信息</h3>
            <div className="space-y-3">
              <div>
                <label className="text-xs font-medium text-muted-foreground block mb-1">
                  表情名称
                </label>
                <input
                  type="text"
                  className="w-full px-2 py-1 bg-input border border-border rounded text-xs"
                  defaultValue={
                    expressions.find((e) => e.id === selectedExpression)?.name
                  }
                />
              </div>
              <div>
                <label className="text-xs font-medium text-muted-foreground block mb-1">
                  分类
                </label>
                <select className="w-full px-2 py-1 bg-input border border-border rounded text-xs">
                  <option>基础表情</option>
                  <option>复杂表情</option>
                  <option>口型</option>
                  <option>自定义</option>
                </select>
              </div>
            </div>
          </Card>

          <Card className="p-4">
            <h3 className="text-sm font-semibold mb-3">混合形状分组</h3>
            <div className="space-y-2">
              {["眉毛", "眼睛", "嘴巴", "鼻子", "脸颊"].map((category) => (
                <Card
                  key={category}
                  className="p-3 hover:bg-accent/50 cursor-pointer"
                >
                  <div className="flex items-center justify-between">
                    <span className="text-sm">{category}</span>
                    <span className="text-xs text-muted-foreground">
                      {
                        blendShapes.filter((s) => s.category === category)
                          .length
                      }{" "}
                      个
                    </span>
                  </div>
                </Card>
              ))}
            </div>
          </Card>

          <Card className="p-4">
            <h3 className="text-sm font-semibold mb-3">骨骼驱动</h3>
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <label className="text-xs font-medium">启用骨骼驱动</label>
                <input type="checkbox" />
              </div>
              <div>
                <label className="text-xs font-medium text-muted-foreground block mb-1">
                  驱动骨骼
                </label>
                <select className="w-full px-2 py-1 bg-input border border-border rounded text-xs">
                  <option>jaw_bone</option>
                  <option>eyebrow_L</option>
                  <option>eyebrow_R</option>
                  <option>eyelid_upper_L</option>
                </select>
              </div>
              <div>
                <label className="text-xs font-medium text-muted-foreground block mb-1">
                  驱动强度
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
            <h3 className="text-sm font-semibold mb-3">表情组合</h3>
            <div className="space-y-2">
              <div className="text-xs text-muted-foreground mb-2">
                混合多个表情创建新表情
              </div>
              {[
                { name: "开心", weight: 0.7 },
                { name: "惊讶", weight: 0.3 },
              ].map((expr, index) => (
                <div
                  key={index}
                  className="flex items-center gap-2 text-xs"
                >
                  <span className="flex-1">{expr.name}</span>
                  <input
                    type="range"
                    className="flex-1"
                    min="0"
                    max="1"
                    step="0.01"
                    defaultValue={expr.weight}
                  />
                  <span className="w-10 text-right text-muted-foreground">
                    {expr.weight.toFixed(1)}
                  </span>
                </div>
              ))}
              <Button size="sm" variant="outline" className="w-full mt-2">
                <Plus className="w-3 h-3 mr-1" />
                添加表情
              </Button>
            </div>
          </Card>

          <Card className="p-4">
            <h3 className="text-sm font-semibold mb-3">AI辅助</h3>
            <div className="space-y-3">
              <Button size="sm" variant="outline" className="w-full justify-start">
                <Sparkles className="w-3 h-3 mr-2" />
                AI生成表情
              </Button>
              <Button size="sm" variant="outline" className="w-full justify-start">
                <Folder className="w-3 h-3 mr-2" />
                从图片识别
              </Button>
              <div>
                <label className="text-xs font-medium text-muted-foreground block mb-1">
                  表情描述
                </label>
                <textarea
                  className="w-full px-2 py-1 bg-input border border-border rounded text-xs resize-none"
                  rows={3}
                  placeholder="描述想要的表情..."
                />
              </div>
            </div>
          </Card>

          <Card className="p-4">
            <h3 className="text-sm font-semibold mb-3">导出选项</h3>
            <div className="space-y-3">
              <div>
                <label className="text-xs font-medium text-muted-foreground block mb-1">
                  导出格式
                </label>
                <select className="w-full px-2 py-1 bg-input border border-border rounded text-xs">
                  <option>引擎原生格式</option>
                  <option>FBX</option>
                  <option>GLTF</option>
                  <option>JSON</option>
                </select>
              </div>
              <div className="flex items-center justify-between">
                <label className="text-xs font-medium">包含骨骼数据</label>
                <input type="checkbox" defaultChecked />
              </div>
            </div>
          </Card>
        </ScrollArea>
      </div>
    </div>
  );
}
