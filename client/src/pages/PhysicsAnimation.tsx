import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import {
  Activity,
  Anchor,
  Box,
  Layers,
  Play,
  Save,
  Settings,
  Zap,
} from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";

export default function PhysicsAnimation() {
  const [selectedBone, setSelectedBone] = useState<string>("spine");
  const [ragdollEnabled, setRagdollEnabled] = useState(false);

  const skeleton = [
    { id: "root", name: "根骨骼", parent: null, level: 0 },
    { id: "pelvis", name: "骨盆", parent: "root", level: 1 },
    { id: "spine", name: "脊柱", parent: "pelvis", level: 2 },
    { id: "chest", name: "胸部", parent: "spine", level: 3 },
    { id: "neck", name: "颈部", parent: "chest", level: 4 },
    { id: "head", name: "头部", parent: "neck", level: 5 },
    { id: "l_shoulder", name: "左肩", parent: "chest", level: 4 },
    { id: "l_arm", name: "左上臂", parent: "l_shoulder", level: 5 },
    { id: "l_forearm", name: "左前臂", parent: "l_arm", level: 6 },
    { id: "l_hand", name: "左手", parent: "l_forearm", level: 7 },
    { id: "r_shoulder", name: "右肩", parent: "chest", level: 4 },
    { id: "r_arm", name: "右上臂", parent: "r_shoulder", level: 5 },
    { id: "r_forearm", name: "右前臂", parent: "r_arm", level: 6 },
    { id: "r_hand", name: "右手", parent: "r_forearm", level: 7 },
    { id: "l_thigh", name: "左大腿", parent: "pelvis", level: 2 },
    { id: "l_calf", name: "左小腿", parent: "l_thigh", level: 3 },
    { id: "l_foot", name: "左脚", parent: "l_calf", level: 4 },
    { id: "r_thigh", name: "右大腿", parent: "pelvis", level: 2 },
    { id: "r_calf", name: "右小腿", parent: "r_thigh", level: 3 },
    { id: "r_foot", name: "右脚", parent: "r_calf", level: 4 },
  ];

  const handleSave = () => {
    toast.success("物理动画配置已保存");
  };

  const handleToggleRagdoll = () => {
    setRagdollEnabled(!ragdollEnabled);
    toast.info(ragdollEnabled ? "已禁用布娃娃模式" : "已启用布娃娃模式");
  };

  const handleSimulate = () => {
    toast.success("开始物理模拟");
  };

  return (
    <div className="h-full flex">
      {/* 左侧骨骼层级 */}
      <div className="w-64 border-r border-border bg-card">
        <div className="h-12 border-b border-border flex items-center px-3">
          <span className="text-sm font-medium">骨骼层级</span>
        </div>
        <ScrollArea className="h-[calc(100%-3rem)]">
          <div className="p-2 space-y-0.5">
            {skeleton.map((bone) => (
              <button
                key={bone.id}
                className={`w-full text-left px-2 py-1.5 text-sm rounded flex items-center gap-2 transition-colors ${
                  selectedBone === bone.id
                    ? "bg-primary text-primary-foreground"
                    : "hover:bg-accent"
                }`}
                style={{ paddingLeft: `${bone.level * 12 + 8}px` }}
                onClick={() => setSelectedBone(bone.id)}
              >
                <Box className="w-3 h-3" />
                <span>{bone.name}</span>
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
            <Activity className="w-4 h-4 text-primary" />
            <span className="text-sm font-medium">角色物理动画</span>
          </div>
          <div className="flex items-center gap-2">
            <Button
              size="sm"
              variant={ragdollEnabled ? "default" : "ghost"}
              className="h-8 gap-2"
              onClick={handleToggleRagdoll}
            >
              <Zap className="w-4 h-4" />
              <span className="text-xs">布娃娃模式</span>
            </Button>
            <Separator orientation="vertical" className="h-6" />
            <Button size="sm" variant="ghost" className="h-8 gap-2" onClick={handleSave}>
              <Save className="w-4 h-4" />
              <span className="text-xs">保存</span>
            </Button>
            <Button size="sm" variant="ghost" className="h-8 gap-2" onClick={handleSimulate}>
              <Play className="w-4 h-4" />
              <span className="text-xs">模拟</span>
            </Button>
          </div>
        </div>

        {/* 3D预览区 */}
        <div className="flex-1 bg-background flex items-center justify-center">
          <div className="text-center space-y-4">
            <div className="w-64 h-64 mx-auto bg-gradient-to-br from-primary/20 to-purple-500/20 rounded-lg flex items-center justify-center">
              <Activity className="w-24 h-24 text-primary/50" />
            </div>
            <div className="text-sm text-muted-foreground">
              角色骨骼预览
              {ragdollEnabled && (
                <div className="text-primary mt-2">布娃娃模式已启用</div>
              )}
            </div>
          </div>
        </div>

        {/* 底部时间轴 */}
        <div className="h-24 border-t border-border bg-card p-4">
          <div className="flex items-center justify-between mb-2">
            <span className="text-xs font-medium">物理模拟时间轴</span>
            <div className="flex items-center gap-2">
              <Button size="sm" variant="ghost" className="h-6 px-2 text-xs">
                重置
              </Button>
            </div>
          </div>
          <div className="h-8 bg-muted rounded relative">
            <div className="absolute inset-0 flex items-center px-2">
              <div className="flex-1 h-1 bg-primary/20 rounded-full">
                <div className="h-full w-1/3 bg-primary rounded-full"></div>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* 右侧属性面板 */}
      <div className="w-80 border-l border-border bg-card flex flex-col">
        <div className="h-12 border-b border-border flex items-center px-3">
          <span className="text-sm font-medium">物理属性</span>
        </div>
        <ScrollArea className="flex-1 p-4 space-y-4">
          <Card className="p-4">
            <h3 className="text-sm font-semibold mb-3 flex items-center gap-2">
              <Settings className="w-4 h-4" />
              骨骼物理
            </h3>
            <div className="space-y-3">
              <div>
                <label className="text-xs font-medium text-muted-foreground block mb-1">
                  选中骨骼
                </label>
                <div className="text-sm font-medium">
                  {skeleton.find((b) => b.id === selectedBone)?.name}
                </div>
              </div>
              <Separator />
              <div className="flex items-center justify-between">
                <label className="text-xs font-medium">启用物理</label>
                <input type="checkbox" defaultChecked />
              </div>
              <div>
                <label className="text-xs font-medium text-muted-foreground block mb-1">
                  质量 (kg)
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
                  阻尼
                </label>
                <input
                  type="range"
                  className="w-full"
                  min="0"
                  max="1"
                  step="0.01"
                  defaultValue="0.1"
                />
              </div>
            </div>
          </Card>

          <Card className="p-4">
            <h3 className="text-sm font-semibold mb-3 flex items-center gap-2">
              <Anchor className="w-4 h-4" />
              关节约束
            </h3>
            <div className="space-y-3">
              <div>
                <label className="text-xs font-medium text-muted-foreground block mb-1">
                  约束类型
                </label>
                <select className="w-full px-2 py-1 bg-input border border-border rounded text-xs">
                  <option>铰链关节</option>
                  <option>球形关节</option>
                  <option>固定关节</option>
                  <option>弹簧关节</option>
                </select>
              </div>
              <div>
                <label className="text-xs font-medium text-muted-foreground block mb-1">
                  角度限制 (度)
                </label>
                <div className="grid grid-cols-2 gap-2">
                  <input
                    type="number"
                    className="px-2 py-1 bg-input border border-border rounded text-xs"
                    placeholder="最小"
                    defaultValue="-90"
                  />
                  <input
                    type="number"
                    className="px-2 py-1 bg-input border border-border rounded text-xs"
                    placeholder="最大"
                    defaultValue="90"
                  />
                </div>
              </div>
              <div className="flex items-center justify-between">
                <label className="text-xs font-medium">启用碰撞</label>
                <input type="checkbox" defaultChecked />
              </div>
            </div>
          </Card>

          <Card className="p-4">
            <h3 className="text-sm font-semibold mb-3 flex items-center gap-2">
              <Layers className="w-4 h-4" />
              碰撞体
            </h3>
            <div className="space-y-3">
              <div>
                <label className="text-xs font-medium text-muted-foreground block mb-1">
                  形状
                </label>
                <select className="w-full px-2 py-1 bg-input border border-border rounded text-xs">
                  <option>胶囊体</option>
                  <option>球体</option>
                  <option>盒体</option>
                  <option>凸包</option>
                </select>
              </div>
              <div>
                <label className="text-xs font-medium text-muted-foreground block mb-1">
                  半径
                </label>
                <input
                  type="number"
                  className="w-full px-2 py-1 bg-input border border-border rounded text-xs"
                  defaultValue="0.1"
                  step="0.01"
                />
              </div>
              <div>
                <label className="text-xs font-medium text-muted-foreground block mb-1">
                  高度
                </label>
                <input
                  type="number"
                  className="w-full px-2 py-1 bg-input border border-border rounded text-xs"
                  defaultValue="0.5"
                  step="0.01"
                />
              </div>
            </div>
          </Card>

          <Card className="p-4">
            <h3 className="text-sm font-semibold mb-3">布娃娃配置</h3>
            <div className="space-y-3">
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
                  defaultValue="0"
                />
                <div className="flex justify-between text-[10px] text-muted-foreground mt-1">
                  <span>动画</span>
                  <span>物理</span>
                </div>
              </div>
              <div>
                <label className="text-xs font-medium text-muted-foreground block mb-1">
                  激活阈值
                </label>
                <input
                  type="number"
                  className="w-full px-2 py-1 bg-input border border-border rounded text-xs"
                  defaultValue="50"
                  placeholder="伤害值"
                />
              </div>
              <div>
                <label className="text-xs font-medium text-muted-foreground block mb-1">
                  恢复时间 (秒)
                </label>
                <input
                  type="number"
                  className="w-full px-2 py-1 bg-input border border-border rounded text-xs"
                  defaultValue="2.0"
                  step="0.1"
                />
              </div>
              <div className="flex items-center justify-between">
                <label className="text-xs font-medium">自动恢复</label>
                <input type="checkbox" defaultChecked />
              </div>
            </div>
          </Card>

          <Card className="p-4">
            <h3 className="text-sm font-semibold mb-3">物理材质</h3>
            <div className="space-y-3">
              <div>
                <label className="text-xs font-medium text-muted-foreground block mb-1">
                  摩擦力
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
                  弹性
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
            </div>
          </Card>
        </ScrollArea>
      </div>
    </div>
  );
}
