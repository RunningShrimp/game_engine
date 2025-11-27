import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import {
  BookOpen,
  ChevronRight,
  Code,
  FileText,
  Layers,
  Search,
  Settings,
  Zap,
} from "lucide-react";
import { useState } from "react";

export default function Documentation() {
  const [searchQuery, setSearchQuery] = useState("");

  const docSections = [
    {
      title: "快速开始",
      icon: Zap,
      items: [
        "安装与配置",
        "创建第一个项目",
        "编辑器界面介绍",
        "基础概念",
      ],
    },
    {
      title: "场景编辑",
      icon: Layers,
      items: [
        "场景层级管理",
        "3D视口操作",
        "变换工具使用",
        "摄像机控制",
      ],
    },
    {
      title: "资产管理",
      icon: FileText,
      items: [
        "导入资产",
        "资产类型",
        "资产浏览器",
        "资产打包",
      ],
    },
    {
      title: "脚本开发",
      icon: Code,
      items: [
        "脚本系统概述",
        "JavaScript API",
        "Lua API",
        "WASM集成",
      ],
    },
    {
      title: "渲染系统",
      icon: Settings,
      items: [
        "PBR材质",
        "全局光照",
        "后处理效果",
        "粒子系统",
      ],
    },
  ];

  const tutorials = [
    {
      title: "创建一个简单的3D场景",
      description: "学习如何创建基础的3D场景，添加对象和光照",
      duration: "15分钟",
    },
    {
      title: "使用PBR材质系统",
      description: "深入了解物理基础渲染材质的创建和使用",
      duration: "20分钟",
    },
    {
      title: "实现角色控制器",
      description: "使用脚本系统创建可控制的角色",
      duration: "30分钟",
    },
    {
      title: "优化游戏性能",
      description: "学习性能分析工具和优化技巧",
      duration: "25分钟",
    },
  ];

  return (
    <div className="h-full flex">
      {/* 左侧导航 */}
      <div className="w-64 border-r border-border bg-card">
        <div className="h-12 border-b border-border flex items-center px-3">
          <span className="text-sm font-medium">文档目录</span>
        </div>
        <ScrollArea className="h-[calc(100%-3rem)]">
          <div className="p-2 space-y-4">
            {docSections.map((section) => {
              const Icon = section.icon;
              return (
                <div key={section.title}>
                  <div className="flex items-center gap-2 px-2 py-1.5 text-sm font-semibold">
                    <Icon className="w-4 h-4 text-primary" />
                    <span>{section.title}</span>
                  </div>
                  <div className="ml-6 space-y-1">
                    {section.items.map((item) => (
                      <button
                        key={item}
                        className="w-full text-left px-2 py-1 text-sm text-muted-foreground hover:text-foreground hover:bg-accent rounded"
                      >
                        {item}
                      </button>
                    ))}
                  </div>
                </div>
              );
            })}
          </div>
        </ScrollArea>
      </div>

      {/* 主内容区 */}
      <div className="flex-1 flex flex-col">
        {/* 搜索栏 */}
        <div className="h-12 border-b border-border bg-card flex items-center px-4">
          <div className="flex-1 relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
            <Input
              type="text"
              placeholder="搜索文档..."
              className="pl-9 h-8 bg-input"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            />
          </div>
        </div>

        {/* 文档内容 */}
        <ScrollArea className="flex-1">
          <div className="p-8 max-w-4xl mx-auto space-y-8">
            {/* 欢迎部分 */}
            <section>
              <div className="flex items-center gap-3 mb-4">
                <BookOpen className="w-8 h-8 text-primary" />
                <h1 className="text-3xl font-bold">引擎文档</h1>
              </div>
              <p className="text-lg text-muted-foreground">
                欢迎使用游戏引擎编辑器！这里包含了所有你需要的文档和教程。
              </p>
            </section>

            <Separator />

            {/* 教程卡片 */}
            <section>
              <h2 className="text-2xl font-bold mb-4">推荐教程</h2>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                {tutorials.map((tutorial) => (
                  <Card
                    key={tutorial.title}
                    className="p-4 hover:bg-accent/50 cursor-pointer transition-colors"
                  >
                    <h3 className="font-semibold mb-2">{tutorial.title}</h3>
                    <p className="text-sm text-muted-foreground mb-3">
                      {tutorial.description}
                    </p>
                    <div className="flex items-center justify-between">
                      <span className="text-xs text-muted-foreground">
                        {tutorial.duration}
                      </span>
                      <Button size="sm" variant="ghost" className="h-7 gap-1">
                        开始学习
                        <ChevronRight className="w-3 h-3" />
                      </Button>
                    </div>
                  </Card>
                ))}
              </div>
            </section>

            <Separator />

            {/* 示例内容 */}
            <section>
              <h2 className="text-2xl font-bold mb-4">快速开始</h2>
              <Card className="p-6">
                <h3 className="text-xl font-semibold mb-4">安装与配置</h3>
                <div className="space-y-4 text-sm">
                  <p>
                    游戏引擎编辑器是一个基于Rust构建的现代化游戏引擎，提供了强大的编辑器工具和高性能的运行时。
                  </p>
                  <div>
                    <h4 className="font-semibold mb-2">系统要求</h4>
                    <ul className="list-disc list-inside space-y-1 text-muted-foreground">
                      <li>操作系统: Windows 10+, macOS 11+, Linux (Ubuntu 20.04+)</li>
                      <li>CPU: 支持AVX2指令集的处理器</li>
                      <li>GPU: 支持Vulkan 1.2+或DirectX 12的显卡</li>
                      <li>内存: 8GB RAM (推荐16GB)</li>
                      <li>存储: 2GB可用空间</li>
                    </ul>
                  </div>
                  <div>
                    <h4 className="font-semibold mb-2">安装步骤</h4>
                    <div className="bg-muted/50 rounded p-4 font-mono text-xs space-y-2">
                      <div>$ git clone https://github.com/your-repo/game-engine.git</div>
                      <div>$ cd game-engine</div>
                      <div>$ cargo build --release</div>
                      <div>$ cargo run --bin editor</div>
                    </div>
                  </div>
                  <div>
                    <h4 className="font-semibold mb-2">硬件加速配置</h4>
                    <p className="text-muted-foreground">
                      引擎会自动检测并启用可用的硬件加速功能，包括SIMD优化、NPU加速和FSR超分辨率。
                      你可以在设置页面中查看和调整这些选项。
                    </p>
                  </div>
                </div>
              </Card>
            </section>

            <Separator />

            {/* API参考 */}
            <section>
              <h2 className="text-2xl font-bold mb-4">API参考</h2>
              <Card className="p-6">
                <h3 className="text-xl font-semibold mb-4">核心API</h3>
                <div className="space-y-3">
                  <div className="bg-muted/50 rounded p-4">
                    <code className="text-sm">
                      <span className="text-primary">Engine</span>.init(config: EngineConfig)
                    </code>
                    <p className="text-xs text-muted-foreground mt-2">
                      初始化游戏引擎实例
                    </p>
                  </div>
                  <div className="bg-muted/50 rounded p-4">
                    <code className="text-sm">
                      <span className="text-primary">Scene</span>.create(name: string)
                    </code>
                    <p className="text-xs text-muted-foreground mt-2">
                      创建新场景
                    </p>
                  </div>
                  <div className="bg-muted/50 rounded p-4">
                    <code className="text-sm">
                      <span className="text-primary">Entity</span>.spawn(components: Component[])
                    </code>
                    <p className="text-xs text-muted-foreground mt-2">
                      在场景中生成新实体
                    </p>
                  </div>
                </div>
              </Card>
            </section>
          </div>
        </ScrollArea>
      </div>
    </div>
  );
}
