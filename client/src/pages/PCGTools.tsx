import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import {
  Mountain,
  Play,
  RefreshCw,
  Save,
  Settings,
  Trees,
  Waves,
} from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";

export default function PCGTools() {
  const [selectedTool, setSelectedTool] = useState<string>("terrain");

  const tools = [
    { id: "terrain", icon: Mountain, label: "地形生成" },
    { id: "vegetation", icon: Trees, label: "植被生成" },
    { id: "water", icon: Waves, label: "水体生成" },
  ];

  const handleGenerate = () => {
    toast.success("正在生成程序化内容...");
  };

  const handleSave = () => {
    toast.success("PCG配置已保存");
  };

  return (
    <div className="h-full flex">
      {/* 左侧工具选择 */}
      <div className="w-64 border-r border-border bg-card">
        <div className="h-12 border-b border-border flex items-center px-3">
          <span className="text-sm font-medium">PCG工具</span>
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
                  <Icon className="w-5 h-5" />
                  <span>{tool.label}</span>
                </button>
              );
            })}
          </div>
        </ScrollArea>
      </div>

      {/* 中间预览区 */}
      <div className="flex-1 flex flex-col">
        {/* 工具栏 */}
        <div className="h-12 border-b border-border bg-card flex items-center justify-between px-4">
          <div className="flex items-center gap-2">
            <Mountain className="w-4 h-4 text-primary" />
            <span className="text-sm font-medium">
              {tools.find((t) => t.id === selectedTool)?.label}
            </span>
          </div>
          <div className="flex items-center gap-2">
            <Button
              size="sm"
              variant="ghost"
              className="h-8 gap-2"
              onClick={handleGenerate}
            >
              <Play className="w-4 h-4" />
              <span className="text-xs">生成</span>
            </Button>
            <Button size="sm" variant="ghost" className="h-8 gap-2">
              <RefreshCw className="w-4 h-4" />
              <span className="text-xs">重新生成</span>
            </Button>
            <Separator orientation="vertical" className="h-6" />
            <Button
              size="sm"
              variant="ghost"
              className="h-8 gap-2"
              onClick={handleSave}
            >
              <Save className="w-4 h-4" />
              <span className="text-xs">保存</span>
            </Button>
          </div>
        </div>

        {/* 预览区域 */}
        <div className="flex-1 bg-background relative overflow-hidden">
          <img
            src="/3d-scene-preview.png"
            alt="PCG Preview"
            className="absolute inset-0 w-full h-full object-cover opacity-40"
          />
          <div className="absolute inset-0 flex items-center justify-center">
            <Card className="p-8 text-center bg-card/80 backdrop-blur-md">
              <div className="w-16 h-16 mx-auto mb-4 rounded-lg bg-primary/20 flex items-center justify-center">
                <Mountain className="w-8 h-8 text-primary" />
              </div>
              <h3 className="text-lg font-semibold mb-2">程序化内容生成</h3>
              <p className="text-sm text-muted-foreground max-w-sm mb-4">
                使用噪声函数和算法生成地形、植被和水体。支持实时预览和参数调整。
              </p>
              <Button onClick={handleGenerate}>
                <Play className="w-4 h-4 mr-2" />
                开始生成
              </Button>
            </Card>
          </div>

          {/* 统计信息 */}
          <div className="absolute bottom-4 left-4 bg-card/90 backdrop-blur px-3 py-2 rounded text-xs space-y-1">
            <div className="text-muted-foreground">顶点数: 1,048,576</div>
            <div className="text-muted-foreground">三角形: 2,097,152</div>
            <div className="text-muted-foreground">生成时间: 2.3s</div>
          </div>
        </div>
      </div>

      {/* 右侧参数面板 */}
      <div className="w-80 border-l border-border bg-card flex flex-col">
        <div className="h-12 border-b border-border flex items-center px-3">
          <span className="text-sm font-medium">生成参数</span>
        </div>
        <ScrollArea className="flex-1 p-4 space-y-4">
          {selectedTool === "terrain" && (
            <>
              <Card className="p-4">
                <h3 className="text-sm font-semibold mb-3 flex items-center gap-2">
                  <Settings className="w-4 h-4" />
                  地形参数
                </h3>
                <div className="space-y-3">
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">
                      尺寸 (m)
                    </label>
                    <input
                      type="number"
                      className="w-full px-2 py-1 bg-input border border-border rounded text-xs"
                      defaultValue="1000"
                    />
                  </div>
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">
                      高度范围 (m)
                    </label>
                    <div className="flex gap-2">
                      <input
                        type="number"
                        className="flex-1 px-2 py-1 bg-input border border-border rounded text-xs"
                        placeholder="最小"
                        defaultValue="0"
                      />
                      <input
                        type="number"
                        className="flex-1 px-2 py-1 bg-input border border-border rounded text-xs"
                        placeholder="最大"
                        defaultValue="200"
                      />
                    </div>
                  </div>
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">
                      细节级别
                    </label>
                    <input
                      type="range"
                      className="w-full"
                      min="1"
                      max="10"
                      defaultValue="5"
                    />
                  </div>
                </div>
              </Card>

              <Card className="p-4">
                <h3 className="text-sm font-semibold mb-3">噪声设置</h3>
                <div className="space-y-3">
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">
                      噪声类型
                    </label>
                    <select className="w-full px-2 py-1 bg-input border border-border rounded text-xs">
                      <option>Perlin噪声</option>
                      <option>Simplex噪声</option>
                      <option>Worley噪声</option>
                      <option>分形噪声</option>
                    </select>
                  </div>
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">
                      频率
                    </label>
                    <input
                      type="range"
                      className="w-full"
                      min="0.1"
                      max="5"
                      step="0.1"
                      defaultValue="1"
                    />
                  </div>
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">
                      振幅
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
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">
                      八度数
                    </label>
                    <input
                      type="range"
                      className="w-full"
                      min="1"
                      max="8"
                      defaultValue="4"
                    />
                  </div>
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">
                      种子值
                    </label>
                    <input
                      type="number"
                      className="w-full px-2 py-1 bg-input border border-border rounded text-xs"
                      defaultValue="12345"
                    />
                  </div>
                </div>
              </Card>

              <Card className="p-4">
                <h3 className="text-sm font-semibold mb-3">侵蚀效果</h3>
                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <label className="text-xs font-medium">启用侵蚀</label>
                    <input type="checkbox" />
                  </div>
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">
                      侵蚀强度
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
                      迭代次数
                    </label>
                    <input
                      type="number"
                      className="w-full px-2 py-1 bg-input border border-border rounded text-xs"
                      defaultValue="100"
                    />
                  </div>
                </div>
              </Card>
            </>
          )}

          {selectedTool === "vegetation" && (
            <>
              <Card className="p-4">
                <h3 className="text-sm font-semibold mb-3 flex items-center gap-2">
                  <Trees className="w-4 h-4" />
                  植被参数
                </h3>
                <div className="space-y-3">
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">
                      密度
                    </label>
                    <input
                      type="range"
                      className="w-full"
                      min="0"
                      max="100"
                      defaultValue="50"
                    />
                  </div>
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">
                      最小高度 (m)
                    </label>
                    <input
                      type="number"
                      className="w-full px-2 py-1 bg-input border border-border rounded text-xs"
                      defaultValue="0"
                    />
                  </div>
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">
                      最大高度 (m)
                    </label>
                    <input
                      type="number"
                      className="w-full px-2 py-1 bg-input border border-border rounded text-xs"
                      defaultValue="150"
                    />
                  </div>
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">
                      坡度限制 (度)
                    </label>
                    <input
                      type="range"
                      className="w-full"
                      min="0"
                      max="90"
                      defaultValue="45"
                    />
                  </div>
                </div>
              </Card>

              <Card className="p-4">
                <h3 className="text-sm font-semibold mb-3">植被类型</h3>
                <div className="space-y-2">
                  {["树木", "灌木", "草地", "花卉"].map((type) => (
                    <div key={type} className="flex items-center justify-between">
                      <label className="text-xs">{type}</label>
                      <input type="checkbox" defaultChecked />
                    </div>
                  ))}
                </div>
              </Card>
            </>
          )}

          {selectedTool === "water" && (
            <Card className="p-4">
              <h3 className="text-sm font-semibold mb-3 flex items-center gap-2">
                <Waves className="w-4 h-4" />
                水体参数
              </h3>
              <div className="space-y-3">
                <div>
                  <label className="text-xs font-medium text-muted-foreground block mb-1">
                    水面高度 (m)
                  </label>
                  <input
                    type="number"
                    className="w-full px-2 py-1 bg-input border border-border rounded text-xs"
                    defaultValue="0"
                  />
                </div>
                <div>
                  <label className="text-xs font-medium text-muted-foreground block mb-1">
                    波浪强度
                  </label>
                  <input
                    type="range"
                    className="w-full"
                    min="0"
                    max="1"
                    step="0.01"
                    defaultValue="0.3"
                  />
                </div>
                <div>
                  <label className="text-xs font-medium text-muted-foreground block mb-1">
                    水体颜色
                  </label>
                  <div className="flex gap-2">
                    <input
                      type="color"
                      className="w-10 h-8 rounded border border-border"
                      defaultValue="#0ea5e9"
                    />
                    <input
                      type="text"
                      className="flex-1 px-2 py-1 bg-input border border-border rounded text-xs"
                      defaultValue="#0ea5e9"
                    />
                  </div>
                </div>
              </div>
            </Card>
          )}
        </ScrollArea>
      </div>
    </div>
  );
}
