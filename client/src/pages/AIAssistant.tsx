import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import {
  Bot,
  Code,
  Image,
  Lightbulb,
  Send,
  Sparkles,
  Wand2,
} from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";

export default function AIAssistant() {
  const [selectedTool, setSelectedTool] = useState<string>("code");
  const [prompt, setPrompt] = useState("");

  const tools = [
    { id: "code", icon: Code, label: "AI代码助手", color: "text-blue-500" },
    { id: "texture", icon: Image, label: "AI纹理生成", color: "text-purple-500" },
    { id: "material", icon: Sparkles, label: "AI材质生成", color: "text-pink-500" },
    { id: "optimize", icon: Lightbulb, label: "AI性能优化", color: "text-yellow-500" },
  ];

  const handleGenerate = () => {
    if (!prompt.trim()) {
      toast.error("请输入提示词");
      return;
    }
    toast.success("AI正在生成内容...");
    setPrompt("");
  };

  const codeExamples = [
    {
      title: "创建旋转立方体",
      description: "生成一个带有旋转动画的立方体实体",
    },
    {
      title: "实现第一人称控制器",
      description: "创建WASD移动和鼠标视角控制",
    },
    {
      title: "添加粒子系统",
      description: "实现火焰粒子效果",
    },
  ];

  const textureExamples = [
    "金属表面纹理，带有划痕和磨损",
    "卡通风格草地纹理，明亮色彩",
    "科幻风格地板纹理，发光线条",
  ];

  return (
    <div className="h-full flex">
      {/* 左侧工具选择 */}
      <div className="w-64 border-r border-border bg-card">
        <div className="h-12 border-b border-border flex items-center px-3">
          <span className="text-sm font-medium">AI工具</span>
        </div>
        <ScrollArea className="h-[calc(100%-3rem)]">
          <div className="p-2 space-y-1">
            {tools.map((tool) => {
              const Icon = tool.icon;
              return (
                <button
                  key={tool.id}
                  className={`w-full text-left px-3 py-2.5 text-sm rounded flex items-center gap-3 transition-colors ${
                    selectedTool === tool.id
                      ? "bg-primary text-primary-foreground"
                      : "hover:bg-accent"
                  }`}
                  onClick={() => setSelectedTool(tool.id)}
                >
                  <Icon className={`w-5 h-5 ${selectedTool === tool.id ? "" : tool.color}`} />
                  <span>{tool.label}</span>
                </button>
              );
            })}
          </div>
        </ScrollArea>
      </div>

      {/* 中间内容区 */}
      <div className="flex-1 flex flex-col">
        {/* 工具栏 */}
        <div className="h-12 border-b border-border bg-card flex items-center justify-between px-4">
          <div className="flex items-center gap-2">
            <Bot className="w-4 h-4 text-primary" />
            <span className="text-sm font-medium">
              {tools.find((t) => t.id === selectedTool)?.label}
            </span>
          </div>
          <div className="flex items-center gap-2">
            <Button size="sm" variant="ghost" className="h-8 gap-2">
              <Wand2 className="w-4 h-4" />
              <span className="text-xs">示例</span>
            </Button>
          </div>
        </div>

        {/* 主内容区 */}
        <ScrollArea className="flex-1 p-6">
          <div className="max-w-3xl mx-auto space-y-6">
            {/* AI介绍卡片 */}
            <Card className="p-6 bg-gradient-to-br from-primary/10 to-purple-500/10 border-primary/20">
              <div className="flex items-start gap-4">
                <div className="w-12 h-12 rounded-lg bg-primary/20 flex items-center justify-center flex-shrink-0">
                  <Bot className="w-6 h-6 text-primary" />
                </div>
                <div>
                  <h3 className="text-lg font-semibold mb-2">AI辅助开发</h3>
                  <p className="text-sm text-muted-foreground">
                    基于NPU加速的AI工具集，帮助您快速生成代码、纹理和材质。
                    提高开发效率，释放创造力。
                  </p>
                </div>
              </div>
            </Card>

            {/* 示例列表 */}
            {selectedTool === "code" && (
              <div className="space-y-3">
                <h3 className="text-sm font-semibold">代码生成示例</h3>
                {codeExamples.map((example, index) => (
                  <Card
                    key={index}
                    className="p-4 hover:bg-accent/50 cursor-pointer transition-colors"
                    onClick={() => setPrompt(example.description)}
                  >
                    <h4 className="text-sm font-medium mb-1">{example.title}</h4>
                    <p className="text-xs text-muted-foreground">
                      {example.description}
                    </p>
                  </Card>
                ))}
              </div>
            )}

            {selectedTool === "texture" && (
              <div className="space-y-3">
                <h3 className="text-sm font-semibold">纹理生成示例</h3>
                {textureExamples.map((example, index) => (
                  <Card
                    key={index}
                    className="p-4 hover:bg-accent/50 cursor-pointer transition-colors"
                    onClick={() => setPrompt(example)}
                  >
                    <p className="text-sm">{example}</p>
                  </Card>
                ))}
              </div>
            )}

            {selectedTool === "material" && (
              <Card className="p-6">
                <h3 className="text-sm font-semibold mb-4">AI材质生成</h3>
                <div className="grid grid-cols-2 gap-4">
                  <div className="aspect-square bg-gradient-to-br from-gray-700 to-gray-900 rounded-lg"></div>
                  <div className="aspect-square bg-gradient-to-br from-blue-500 to-cyan-500 rounded-lg"></div>
                  <div className="aspect-square bg-gradient-to-br from-red-500 to-orange-500 rounded-lg"></div>
                  <div className="aspect-square bg-gradient-to-br from-green-500 to-emerald-500 rounded-lg"></div>
                </div>
              </Card>
            )}

            {selectedTool === "optimize" && (
              <Card className="p-6">
                <h3 className="text-sm font-semibold mb-4">性能优化建议</h3>
                <div className="space-y-3">
                  <div className="flex items-start gap-3">
                    <div className="w-6 h-6 rounded-full bg-yellow-500/20 flex items-center justify-center flex-shrink-0 mt-0.5">
                      <Lightbulb className="w-3 h-3 text-yellow-500" />
                    </div>
                    <div>
                      <h4 className="text-sm font-medium mb-1">减少Draw Call</h4>
                      <p className="text-xs text-muted-foreground">
                        检测到142个Draw Call，建议使用实例化渲染合并相同网格
                      </p>
                    </div>
                  </div>
                  <Separator />
                  <div className="flex items-start gap-3">
                    <div className="w-6 h-6 rounded-full bg-yellow-500/20 flex items-center justify-center flex-shrink-0 mt-0.5">
                      <Lightbulb className="w-3 h-3 text-yellow-500" />
                    </div>
                    <div>
                      <h4 className="text-sm font-medium mb-1">纹理压缩</h4>
                      <p className="text-xs text-muted-foreground">
                        发现23张未压缩纹理，建议使用BC7/ASTC压缩格式
                      </p>
                    </div>
                  </div>
                  <Separator />
                  <div className="flex items-start gap-3">
                    <div className="w-6 h-6 rounded-full bg-yellow-500/20 flex items-center justify-center flex-shrink-0 mt-0.5">
                      <Lightbulb className="w-3 h-3 text-yellow-500" />
                    </div>
                    <div>
                      <h4 className="text-sm font-medium mb-1">LOD优化</h4>
                      <p className="text-xs text-muted-foreground">
                        远处物体使用高精度模型，建议添加LOD级别
                      </p>
                    </div>
                  </div>
                </div>
              </Card>
            )}

            {/* 生成结果 */}
            <Card className="p-6">
              <h3 className="text-sm font-semibold mb-4">生成结果</h3>
              <div className="bg-muted/50 rounded-lg p-4 font-mono text-xs space-y-1">
                <div className="text-green-500">// AI生成的代码将显示在这里</div>
                <div className="text-blue-400">const entity = new Entity('cube');</div>
                <div className="text-blue-400">entity.addComponent('Transform', {"{"}</div>
                <div className="ml-4 text-muted-foreground">position: [0, 0, 0],</div>
                <div className="ml-4 text-muted-foreground">rotation: [0, 0, 0],</div>
                <div className="text-blue-400">{"}"});</div>
              </div>
            </Card>
          </div>
        </ScrollArea>

        {/* 底部输入区 */}
        <div className="border-t border-border bg-card p-4">
          <div className="max-w-3xl mx-auto">
            <div className="flex gap-2">
              <input
                type="text"
                className="flex-1 px-4 py-2 bg-input border border-border rounded-lg text-sm"
                placeholder="描述您想要生成的内容..."
                value={prompt}
                onChange={(e) => setPrompt(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    handleGenerate();
                  }
                }}
              />
              <Button onClick={handleGenerate}>
                <Send className="w-4 h-4 mr-2" />
                生成
              </Button>
            </div>
            <div className="mt-2 text-xs text-muted-foreground">
              提示：使用详细的描述可以获得更好的生成结果
            </div>
          </div>
        </div>
      </div>

      {/* 右侧设置面板 */}
      <div className="w-80 border-l border-border bg-card flex flex-col">
        <div className="h-12 border-b border-border flex items-center px-3">
          <span className="text-sm font-medium">AI设置</span>
        </div>
        <ScrollArea className="flex-1 p-4 space-y-4">
          <Card className="p-4">
            <h3 className="text-sm font-semibold mb-3">模型配置</h3>
            <div className="space-y-3">
              <div>
                <label className="text-xs font-medium text-muted-foreground block mb-1">
                  AI模型
                </label>
                <select className="w-full px-2 py-1 bg-input border border-border rounded text-xs">
                  <option>GPT-4 Turbo</option>
                  <option>Claude 3 Opus</option>
                  <option>Gemini Pro</option>
                  <option>本地模型 (NPU)</option>
                </select>
              </div>
              <div>
                <label className="text-xs font-medium text-muted-foreground block mb-1">
                  创造性
                </label>
                <input
                  type="range"
                  className="w-full"
                  min="0"
                  max="1"
                  step="0.1"
                  defaultValue="0.7"
                />
                <div className="flex justify-between text-[10px] text-muted-foreground mt-1">
                  <span>保守</span>
                  <span>创新</span>
                </div>
              </div>
              <div>
                <label className="text-xs font-medium text-muted-foreground block mb-1">
                  最大长度
                </label>
                <input
                  type="number"
                  className="w-full px-2 py-1 bg-input border border-border rounded text-xs"
                  defaultValue="2048"
                />
              </div>
            </div>
          </Card>

          <Card className="p-4">
            <h3 className="text-sm font-semibold mb-3">NPU加速</h3>
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <label className="text-xs font-medium">启用NPU加速</label>
                <input type="checkbox" defaultChecked />
              </div>
              <Separator />
              <div className="text-xs text-muted-foreground">
                <div className="mb-2">NPU状态: <span className="text-green-500">就绪</span></div>
                <div className="mb-2">算力: 8 TOPS</div>
                <div>加速比: 3.2x</div>
              </div>
            </div>
          </Card>

          <Card className="p-4">
            <h3 className="text-sm font-semibold mb-3">使用统计</h3>
            <div className="space-y-2 text-xs">
              <div className="flex justify-between">
                <span className="text-muted-foreground">今日生成</span>
                <span className="font-medium">42次</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">本月生成</span>
                <span className="font-medium">1,234次</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">节省时间</span>
                <span className="font-medium text-green-500">18.5小时</span>
              </div>
            </div>
          </Card>
        </ScrollArea>
      </div>
    </div>
  );
}
