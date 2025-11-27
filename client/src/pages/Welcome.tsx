import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { useProject } from "@/contexts/ProjectContext";
import {
  Activity,
  BookOpen,
  Bot,
  Box,
  FileCode,
  FolderOpen,
  GitBranch,
  Github,
  MessageCircle,
  RefreshCw,
  Smile,
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
      path: "/scene",
    },
    {
      title: "可视化着色器编辑器",
      description: "节点式材质编辑器，赋能美术师创建复杂着色器",
      icon: Sparkles,
      path: "/shader",
      badge: "P0",
    },
    {
      title: "PCG工具",
      description: "程序化生成地形、植被和水体，打造大规模开放世界",
      icon: FolderOpen,
      path: "/pcg",
      badge: "P0",
    },
    {
      title: "AI辅助工具",
      description: "基于NPU的AI代码助手、纹理生成和性能优化",
      icon: FileCode,
      path: "/ai",
      badge: "P0",
    },
    {
      title: "可视化脚本系统",
      description: "类似UE蓝图的节点式脚本编辑器，降低开发门槛",
      icon: Sparkles,
      path: "/blueprint",
      badge: "P1",
    },
    {
      title: "动画蓝图系统",
      description: "高级动画系统，IK、根运动和状态机可视化编辑",
      icon: FolderOpen,
      path: "/animation",
      badge: "P1",
    },
    {
      title: "物理动画系统",
      description: "布娃娃、碰撞响应、骨骼物理模拟和混合权重",
      icon: Sparkles,
      path: "/physics",
      badge: "NEW",
    },
    {
      title: "插件管理器",
      description: "第三方插件安装、配置、启用/禁用，插件开发API",
      icon: Box,
      path: "/plugins",
      badge: "NEW",
    },
    {
      title: "动作捕捉集成",
      description: "导入Mocap数据（BVH/FBX），实时预览和重定向",
      icon: Activity,
      path: "/mocap",
      badge: "NEW",
    },
    {
      title: "面部动画系统",
      description: "混合形状、骨骼驱动和表情库管理",
      icon: Smile,
      path: "/facial",
      badge: "NEW",
    },
    {
      title: "动画重定向工具",
      description: "不同骨骼结构间的动画迁移，自动骨骼映射",
      icon: RefreshCw,
      path: "/retarget",
      badge: "NEW",
    },
    {
      title: "行为树编辑器",
      description: "AI逻辑设计，复合/装饰/条件/动作节点，黑板变量",
      icon: GitBranch,
      path: "/behavior",
      badge: "AI",
    },
    {
      title: "NPC对话系统",
      description: "智能NPC对话，表情动画，条件分支，AI生成",
      icon: MessageCircle,
      path: "/dialogue",
      badge: "AI",
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
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {features.map((feature) => {
              const Icon = feature.icon;
              return (
                <Card
                  key={feature.title}
                  className="p-6 hover:bg-accent/50 transition-colors cursor-pointer relative"
                  onClick={() => setLocation(feature.path)}
                >
                  {feature.badge && (
                    <div className="absolute top-3 right-3 px-2 py-0.5 bg-primary text-primary-foreground text-[10px] font-bold rounded">
                      {feature.badge}
                    </div>
                  )}
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
