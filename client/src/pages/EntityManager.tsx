import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import {
  Box,
  Camera,
  Code,
  Lightbulb,
  Plus,
  Search,
  Sparkles,
  Users,
} from "lucide-react";

export default function EntityManager() {
  const entityTypes = [
    { id: 1, name: "空实体", icon: Box, description: "创建一个空的实体容器" },
    {
      id: 2,
      name: "3D对象",
      icon: Box,
      description: "带有网格渲染器的3D对象",
    },
    { id: 3, name: "摄像机", icon: Camera, description: "场景摄像机" },
    { id: 4, name: "光源", icon: Lightbulb, description: "光照组件" },
    { id: 5, name: "粒子系统", icon: Sparkles, description: "GPU粒子效果" },
    { id: 6, name: "角色", icon: Users, description: "可控制的角色实体" },
  ];

  const components = [
    { id: 1, name: "Transform", category: "核心", icon: Box },
    { id: 2, name: "MeshRenderer", category: "渲染", icon: Box },
    { id: 3, name: "Camera", category: "渲染", icon: Camera },
    { id: 4, name: "Light", category: "渲染", icon: Lightbulb },
    { id: 5, name: "RigidBody", category: "物理", icon: Box },
    { id: 6, name: "Collider", category: "物理", icon: Box },
    { id: 7, name: "Script", category: "脚本", icon: Code },
    { id: 8, name: "Animator", category: "动画", icon: Users },
  ];

  const existingEntities = [
    { id: 1, name: "Player", components: ["Transform", "MeshRenderer", "Script"] },
    { id: 2, name: "Enemy_01", components: ["Transform", "MeshRenderer", "AI"] },
    { id: 3, name: "MainCamera", components: ["Transform", "Camera"] },
    { id: 4, name: "DirectionalLight", components: ["Transform", "Light"] },
  ];

  return (
    <div className="h-full flex">
      {/* 左侧实体类型 */}
      <div className="w-64 border-r border-border bg-card">
        <div className="h-12 border-b border-border flex items-center px-3 justify-between">
          <span className="text-sm font-medium">实体类型</span>
          <Button size="sm" variant="ghost" className="h-7 w-7 p-0">
            <Plus className="w-4 h-4" />
          </Button>
        </div>
        <ScrollArea className="h-[calc(100%-3rem)]">
          <div className="p-2 space-y-1">
            {entityTypes.map((type) => {
              const Icon = type.icon;
              return (
                <button
                  key={type.id}
                  className="w-full flex items-start gap-2 px-2 py-2 rounded hover:bg-accent text-left"
                >
                  <Icon className="w-4 h-4 text-muted-foreground mt-0.5" />
                  <div className="flex-1 min-w-0">
                    <div className="text-sm font-medium">{type.name}</div>
                    <div className="text-xs text-muted-foreground">
                      {type.description}
                    </div>
                  </div>
                </button>
              );
            })}
          </div>
        </ScrollArea>
      </div>

      {/* 中间现有实体列表 */}
      <div className="flex-1 flex flex-col">
        {/* 工具栏 */}
        <div className="h-12 border-b border-border bg-card flex items-center px-4 gap-3">
          <div className="flex-1 relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
            <Input
              type="text"
              placeholder="搜索实体..."
              className="pl-9 h-8 bg-input"
            />
          </div>
          <Button size="sm" variant="default" className="h-8 gap-2">
            <Plus className="w-4 h-4" />
            <span className="text-xs">新建实体</span>
          </Button>
        </div>

        {/* 实体列表 */}
        <ScrollArea className="flex-1">
          <div className="p-4 space-y-2">
            {existingEntities.map((entity) => (
              <Card
                key={entity.id}
                className="p-4 hover:bg-accent/50 cursor-pointer transition-colors"
              >
                <div className="flex items-start justify-between mb-2">
                  <div className="flex items-center gap-2">
                    <Box className="w-5 h-5 text-primary" />
                    <span className="font-medium">{entity.name}</span>
                  </div>
                  <Button size="sm" variant="ghost" className="h-6 px-2">
                    <span className="text-xs">编辑</span>
                  </Button>
                </div>
                <div className="flex flex-wrap gap-1">
                  {entity.components.map((comp, idx) => (
                    <span
                      key={idx}
                      className="text-xs px-2 py-0.5 bg-secondary rounded"
                    >
                      {comp}
                    </span>
                  ))}
                </div>
              </Card>
            ))}
          </div>
        </ScrollArea>
      </div>

      {/* 右侧组件库 */}
      <div className="w-80 border-l border-border bg-card">
        <div className="h-12 border-b border-border flex items-center px-3">
          <span className="text-sm font-medium">组件库</span>
        </div>
        <ScrollArea className="h-[calc(100%-3rem)]">
          <div className="p-4">
            <div className="space-y-4">
              {["核心", "渲染", "物理", "脚本", "动画"].map((category) => (
                <div key={category}>
                  <div className="text-xs font-semibold text-muted-foreground mb-2">
                    {category}
                  </div>
                  <div className="space-y-1">
                    {components
                      .filter((c) => c.category === category)
                      .map((component) => {
                        const Icon = component.icon;
                        return (
                          <button
                            key={component.id}
                            className="w-full flex items-center gap-2 px-2 py-2 rounded hover:bg-accent text-sm text-left"
                          >
                            <Icon className="w-4 h-4 text-muted-foreground" />
                            <span>{component.name}</span>
                          </button>
                        );
                      })}
                  </div>
                </div>
              ))}
            </div>
            <Separator className="my-4" />
            <div>
              <div className="text-xs font-semibold text-muted-foreground mb-2">
                自定义组件
              </div>
              <Button
                size="sm"
                variant="outline"
                className="w-full gap-2"
              >
                <Code className="w-4 h-4" />
                <span className="text-xs">创建脚本组件</span>
              </Button>
            </div>
          </div>
        </ScrollArea>
      </div>
    </div>
  );
}
