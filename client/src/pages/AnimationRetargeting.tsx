import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import {
  ArrowLeftRight,
  Check,
  Play,
  RefreshCw,
  Save,
  User,
  Users,
  Wand2,
  X,
} from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";

export default function AnimationRetargeting() {
  const [autoMapping, setAutoMapping] = useState(true);

  const sourceSkeleton = [
    { id: "root", name: "根骨骼", mapped: true },
    { id: "pelvis", name: "骨盆", mapped: true },
    { id: "spine", name: "脊柱", mapped: true },
    { id: "chest", name: "胸部", mapped: true },
    { id: "neck", name: "颈部", mapped: true },
    { id: "head", name: "头部", mapped: true },
    { id: "l_shoulder", name: "左肩", mapped: true },
    { id: "l_arm", name: "左上臂", mapped: true },
    { id: "l_forearm", name: "左前臂", mapped: true },
    { id: "l_hand", name: "左手", mapped: true },
    { id: "r_shoulder", name: "右肩", mapped: true },
    { id: "r_arm", name: "右上臂", mapped: true },
    { id: "r_forearm", name: "右前臂", mapped: true },
    { id: "r_hand", name: "右手", mapped: true },
    { id: "l_thigh", name: "左大腿", mapped: true },
    { id: "l_calf", name: "左小腿", mapped: true },
    { id: "l_foot", name: "左脚", mapped: true },
    { id: "r_thigh", name: "右大腿", mapped: true },
    { id: "r_calf", name: "右小腿", mapped: true },
    { id: "r_foot", name: "右脚", mapped: true },
  ];

  const handleAutoMap = () => {
    setAutoMapping(true);
    toast.success("自动映射完成");
  };

  const handleRetarget = () => {
    toast.success("开始重定向动画");
  };

  const handleSave = () => {
    toast.success("映射配置已保存");
  };

  return (
    <div className="h-full flex">
      {/* 左侧源骨骼 */}
      <div className="w-80 border-r border-border bg-card">
        <div className="h-12 border-b border-border flex items-center px-3">
          <User className="w-4 h-4 mr-2 text-primary" />
          <span className="text-sm font-medium">源骨骼</span>
        </div>
        <div className="p-3 border-b border-border">
          <div className="text-xs text-muted-foreground mb-2">角色模型</div>
          <select className="w-full px-2 py-1 bg-input border border-border rounded text-xs">
            <option>标准人形骨骼 (52骨骼)</option>
            <option>UE4 Mannequin</option>
            <option>Unity Humanoid</option>
            <option>自定义骨骼</option>
          </select>
        </div>
        <ScrollArea className="h-[calc(100%-7rem)]">
          <div className="p-2 space-y-0.5">
            {sourceSkeleton.map((bone) => (
              <div
                key={bone.id}
                className="flex items-center justify-between px-2 py-1.5 text-sm rounded hover:bg-accent"
              >
                <span>{bone.name}</span>
                {bone.mapped ? (
                  <Check className="w-3 h-3 text-green-500" />
                ) : (
                  <X className="w-3 h-3 text-red-500" />
                )}
              </div>
            ))}
          </div>
        </ScrollArea>
      </div>

      {/* 中间映射区 */}
      <div className="flex-1 flex flex-col">
        {/* 工具栏 */}
        <div className="h-12 border-b border-border bg-card flex items-center justify-between px-4">
          <div className="flex items-center gap-2">
            <ArrowLeftRight className="w-4 h-4 text-primary" />
            <span className="text-sm font-medium">动画重定向</span>
          </div>
          <div className="flex items-center gap-2">
            <Button size="sm" variant="ghost" className="h-8 gap-2" onClick={handleAutoMap}>
              <Wand2 className="w-4 h-4" />
              <span className="text-xs">自动映射</span>
            </Button>
            <Separator orientation="vertical" className="h-6" />
            <Button size="sm" variant="ghost" className="h-8 gap-2" onClick={handleSave}>
              <Save className="w-4 h-4" />
              <span className="text-xs">保存</span>
            </Button>
            <Button size="sm" className="h-8 gap-2" onClick={handleRetarget}>
              <Play className="w-4 h-4" />
              <span className="text-xs">重定向</span>
            </Button>
          </div>
        </div>

        {/* 映射可视化 */}
        <div className="flex-1 bg-background p-8">
          <div className="max-w-4xl mx-auto">
            <div className="grid grid-cols-2 gap-8">
              {/* 源模型 */}
              <Card className="p-6">
                <div className="text-center space-y-4">
                  <div className="w-48 h-48 mx-auto bg-gradient-to-br from-blue-500/20 to-cyan-500/20 rounded-lg flex items-center justify-center">
                    <User className="w-24 h-24 text-blue-500/50" />
                  </div>
                  <div>
                    <div className="text-sm font-medium">源角色</div>
                    <div className="text-xs text-muted-foreground">
                      标准人形骨骼
                    </div>
                  </div>
                  <div className="text-xs text-muted-foreground">
                    <div>骨骼数: 52</div>
                    <div>高度: 180cm</div>
                    <div>比例: 1:1:1</div>
                  </div>
                </div>
              </Card>

              {/* 目标模型 */}
              <Card className="p-6">
                <div className="text-center space-y-4">
                  <div className="w-48 h-48 mx-auto bg-gradient-to-br from-purple-500/20 to-pink-500/20 rounded-lg flex items-center justify-center">
                    <Users className="w-24 h-24 text-purple-500/50" />
                  </div>
                  <div>
                    <div className="text-sm font-medium">目标角色</div>
                    <div className="text-xs text-muted-foreground">
                      UE4 Mannequin
                    </div>
                  </div>
                  <div className="text-xs text-muted-foreground">
                    <div>骨骼数: 68</div>
                    <div>高度: 175cm</div>
                    <div>比例: 1.2:1:0.9</div>
                  </div>
                </div>
              </Card>
            </div>

            {/* 映射状态 */}
            <Card className="mt-8 p-6">
              <div className="flex items-center justify-between mb-4">
                <h3 className="text-sm font-semibold">映射状态</h3>
                <div className="flex items-center gap-4 text-xs">
                  <span className="flex items-center gap-1">
                    <div className="w-2 h-2 bg-green-500 rounded-full"></div>
                    已映射: 20
                  </span>
                  <span className="flex items-center gap-1">
                    <div className="w-2 h-2 bg-yellow-500 rounded-full"></div>
                    部分: 0
                  </span>
                  <span className="flex items-center gap-1">
                    <div className="w-2 h-2 bg-red-500 rounded-full"></div>
                    未映射: 0
                  </span>
                </div>
              </div>
              <div className="space-y-2">
                {[
                  { from: "头部", to: "Head", confidence: 100 },
                  { from: "左手", to: "LeftHand", confidence: 95 },
                  { from: "右脚", to: "RightFoot", confidence: 98 },
                ].map((mapping, index) => (
                  <div
                    key={index}
                    className="flex items-center gap-4 text-xs p-2 bg-muted/50 rounded"
                  >
                    <span className="flex-1">{mapping.from}</span>
                    <ArrowLeftRight className="w-3 h-3 text-muted-foreground" />
                    <span className="flex-1">{mapping.to}</span>
                    <span className="text-muted-foreground">
                      {mapping.confidence}%
                    </span>
                  </div>
                ))}
              </div>
            </Card>
          </div>
        </div>
      </div>

      {/* 右侧目标骨骼 */}
      <div className="w-80 border-l border-border bg-card flex flex-col">
        <div className="h-12 border-b border-border flex items-center px-3">
          <Users className="w-4 h-4 mr-2 text-primary" />
          <span className="text-sm font-medium">目标骨骼</span>
        </div>
        <div className="p-3 border-b border-border">
          <div className="text-xs text-muted-foreground mb-2">角色模型</div>
          <select className="w-full px-2 py-1 bg-input border border-border rounded text-xs">
            <option>UE4 Mannequin (68骨骼)</option>
            <option>Unity Humanoid</option>
            <option>标准人形骨骼</option>
            <option>自定义骨骼</option>
          </select>
        </div>
        <ScrollArea className="flex-1 p-4 space-y-4">
          <Card className="p-4">
            <h3 className="text-sm font-semibold mb-3">重定向设置</h3>
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <label className="text-xs font-medium">自动映射</label>
                <input
                  type="checkbox"
                  checked={autoMapping}
                  onChange={(e) => setAutoMapping(e.target.checked)}
                />
              </div>
              <div className="flex items-center justify-between">
                <label className="text-xs font-medium">保持比例</label>
                <input type="checkbox" defaultChecked />
              </div>
              <div className="flex items-center justify-between">
                <label className="text-xs font-medium">匹配姿势</label>
                <input type="checkbox" defaultChecked />
              </div>
            </div>
          </Card>

          <Card className="p-4">
            <h3 className="text-sm font-semibold mb-3">比例调整</h3>
            <div className="space-y-3">
              <div>
                <label className="text-xs font-medium text-muted-foreground block mb-1">
                  整体缩放
                </label>
                <input
                  type="number"
                  className="w-full px-2 py-1 bg-input border border-border rounded text-xs"
                  defaultValue="1.0"
                  step="0.1"
                />
              </div>
              <div>
                <label className="text-xs font-medium text-muted-foreground block mb-1">
                  高度比例
                </label>
                <input
                  type="range"
                  className="w-full"
                  min="0.5"
                  max="2"
                  step="0.01"
                  defaultValue="0.97"
                />
              </div>
              <div>
                <label className="text-xs font-medium text-muted-foreground block mb-1">
                  手臂长度
                </label>
                <input
                  type="range"
                  className="w-full"
                  min="0.5"
                  max="2"
                  step="0.01"
                  defaultValue="1.2"
                />
              </div>
              <div>
                <label className="text-xs font-medium text-muted-foreground block mb-1">
                  腿部长度
                </label>
                <input
                  type="range"
                  className="w-full"
                  min="0.5"
                  max="2"
                  step="0.01"
                  defaultValue="0.9"
                />
              </div>
            </div>
          </Card>

          <Card className="p-4">
            <h3 className="text-sm font-semibold mb-3">姿势匹配</h3>
            <div className="space-y-3">
              <div>
                <label className="text-xs font-medium text-muted-foreground block mb-1">
                  T-Pose对齐
                </label>
                <select className="w-full px-2 py-1 bg-input border border-border rounded text-xs">
                  <option>自动检测</option>
                  <option>T-Pose</option>
                  <option>A-Pose</option>
                  <option>自定义姿势</option>
                </select>
              </div>
              <div>
                <label className="text-xs font-medium text-muted-foreground block mb-1">
                  旋转偏移 (度)
                </label>
                <div className="grid grid-cols-3 gap-2">
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
            </div>
          </Card>

          <Card className="p-4">
            <h3 className="text-sm font-semibold mb-3">高级选项</h3>
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <label className="text-xs font-medium">传递根运动</label>
                <input type="checkbox" defaultChecked />
              </div>
              <div className="flex items-center justify-between">
                <label className="text-xs font-medium">保留手指动画</label>
                <input type="checkbox" defaultChecked />
              </div>
              <div className="flex items-center justify-between">
                <label className="text-xs font-medium">保留面部动画</label>
                <input type="checkbox" />
              </div>
              <div>
                <label className="text-xs font-medium text-muted-foreground block mb-1">
                  IK处理
                </label>
                <select className="w-full px-2 py-1 bg-input border border-border rounded text-xs">
                  <option>自动</option>
                  <option>启用IK</option>
                  <option>禁用IK</option>
                </select>
              </div>
            </div>
          </Card>

          <Card className="p-4">
            <h3 className="text-sm font-semibold mb-3">预设配置</h3>
            <div className="space-y-2">
              <Button size="sm" variant="outline" className="w-full justify-start">
                <RefreshCw className="w-3 h-3 mr-2" />
                UE4 → Unity
              </Button>
              <Button size="sm" variant="outline" className="w-full justify-start">
                <RefreshCw className="w-3 h-3 mr-2" />
                Unity → UE4
              </Button>
              <Button size="sm" variant="outline" className="w-full justify-start">
                <RefreshCw className="w-3 h-3 mr-2" />
                标准 → Mixamo
              </Button>
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
                  <option>FBX</option>
                  <option>GLTF</option>
                  <option>引擎原生格式</option>
                </select>
              </div>
              <div className="flex items-center justify-between">
                <label className="text-xs font-medium">包含网格</label>
                <input type="checkbox" />
              </div>
              <div className="flex items-center justify-between">
                <label className="text-xs font-medium">烘焙动画</label>
                <input type="checkbox" defaultChecked />
              </div>
            </div>
          </Card>
        </ScrollArea>
      </div>
    </div>
  );
}
