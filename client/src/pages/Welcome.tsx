import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { useProject } from "@/contexts/ProjectContext";
import {
  BookOpen,
  FileCode,
  FolderOpen,
  Github,
  Sparkles,
} from "lucide-react";
import { useLocation } from "wouter";

export default function Welcome() {
  const [, setLocation] = useLocation();
  const { recentProjects } = useProject();

  const features = [
    {
      title: "场景编辑器",
      description: "强大的3D场景编辑工具，支持实时预览和多种变换工具",
      icon: Sparkles,
    },
    {
      title: "资产管理",
      description: "高效的资产浏览器，支持多种格式的模型、纹理和音频",
      icon: FolderOpen,
    },
    {
      title: "实体系统",
      description: "基于ECS架构的实体组件系统，灵活且高性能",
      icon: FileCode,
    },
    {
      title: "调试工具",
      description: "实时性能监控、日志查看和性能分析工具",
      icon: BookOpen,
    },
  ];

  const formatDate = (timestamp: number) => {
    return new Date(timestamp).toLocaleDateString('zh-CN');
  };

  return (
    <div className="h-full overflow-auto">
      {/* Hero Section */}
      <div className="relative h-96 overflow-hidden">
        <img
          src="/hero-bg.png"
          alt="Hero Background"
          className="absolute inset-0 w-full h-full object-cover"
        />
        <div className="absolute inset-0 bg-gradient-to-b from-background/50 to-background"></div>
        <div className="relative h-full flex flex-col items-center justify-center text-center px-4">
          <img src="/engine-logo.png" alt="Engine Logo" className="w-24 h-24 mb-6" />
          <h1 className="text-4xl font-bold mb-4">Game Engine Editor</h1>
          <p className="text-lg text-muted-foreground max-w-2xl mb-8">
            现代化的游戏引擎编辑器，基于Rust构建，支持多平台、高性能渲染和AI加速
          </p>
          <div className="flex gap-4">
            <Button size="lg" onClick={() => setLocation("/scene")}>
              开始创作
            </Button>
            <Button size="lg" variant="outline" onClick={() => setLocation("/docs")}>
              <BookOpen className="w-4 h-4 mr-2" />
              查看文档
            </Button>
            <Button size="lg" variant="outline">
              <Github className="w-4 h-4 mr-2" />
              查看源码
            </Button>
          </div>
        </div>
      </div>

      <div className="container py-12 space-y-12">
        {/* Features Grid */}
        <section>
          <h2 className="text-2xl font-bold mb-6">核心功能</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
            {features.map((feature) => {
              const Icon = feature.icon;
              return (
                <Card key={feature.title} className="p-6 hover:bg-accent/50 transition-colors">
                  <Icon className="w-8 h-8 text-primary mb-4" />
                  <h3 className="text-lg font-semibold mb-2">{feature.title}</h3>
                  <p className="text-sm text-muted-foreground">
                    {feature.description}
                  </p>
                </Card>
              );
            })}
          </div>
        </section>

        {/* Recent Projects */}
        <section>
          <h2 className="text-2xl font-bold mb-6">最近项目</h2>
          {recentProjects.length > 0 ? (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {recentProjects.map((project) => (
                <Card
                  key={project.id}
                  className="p-4 hover:bg-accent/50 cursor-pointer transition-colors"
                  onClick={() => setLocation('/scene')}
                >
                  <div className="flex items-start justify-between mb-2">
                    <h3 className="font-semibold">{project.name}</h3>
                    <FolderOpen className="w-5 h-5 text-muted-foreground" />
                  </div>
                  <p className="text-sm text-muted-foreground">
                    最后打开: {formatDate(project.lastOpened)}
                  </p>
                </Card>
              ))}
            </div>
          ) : (
            <Card className="p-8 text-center">
              <p className="text-muted-foreground mb-4">还没有最近项目</p>
              <Button onClick={() => setLocation('/scene')}>
                创建新项目
              </Button>
            </Card>
          )}
        </section>

        {/* Tech Stack */}
        <section>
          <h2 className="text-2xl font-bold mb-6">技术栈</h2>
          <Card className="p-6">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div>
                <h3 className="text-lg font-semibold mb-3">核心技术</h3>
                <ul className="space-y-2 text-sm">
                  <li className="flex items-center gap-2">
                    <div className="w-2 h-2 bg-primary rounded-full"></div>
                    <span>Rust - 高性能游戏引擎核心</span>
                  </li>
                  <li className="flex items-center gap-2">
                    <div className="w-2 h-2 bg-primary rounded-full"></div>
                    <span>WGPU - 跨平台图形API</span>
                  </li>
                  <li className="flex items-center gap-2">
                    <div className="w-2 h-2 bg-primary rounded-full"></div>
                    <span>ECS - 实体组件系统架构</span>
                  </li>
                  <li className="flex items-center gap-2">
                    <div className="w-2 h-2 bg-primary rounded-full"></div>
                    <span>React - 现代化编辑器UI</span>
                  </li>
                </ul>
              </div>
              <div>
                <h3 className="text-lg font-semibold mb-3">硬件加速</h3>
                <ul className="space-y-2 text-sm">
                  <li className="flex items-center gap-2">
                    <div className="w-2 h-2 bg-primary rounded-full"></div>
                    <span>SIMD优化 (AVX/AVX2/NEON)</span>
                  </li>
                  <li className="flex items-center gap-2">
                    <div className="w-2 h-2 bg-primary rounded-full"></div>
                    <span>多厂商NPU SDK集成</span>
                  </li>
                  <li className="flex items-center gap-2">
                    <div className="w-2 h-2 bg-primary rounded-full"></div>
                    <span>AMD FSR超分辨率</span>
                  </li>
                  <li className="flex items-center gap-2">
                    <div className="w-2 h-2 bg-primary rounded-full"></div>
                    <span>GPU粒子系统</span>
                  </li>
                </ul>
              </div>
            </div>
          </Card>
        </section>
      </div>
    </div>
  );
}
