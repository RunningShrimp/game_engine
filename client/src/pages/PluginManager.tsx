import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import {
  Book,
  Code,
  Download,
  ExternalLink,
  Package,
  Plus,
  RefreshCw,
  Search,
  Settings,
  Star,
  Trash2,
} from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";

interface Plugin {
  id: string;
  name: string;
  version: string;
  author: string;
  description: string;
  enabled: boolean;
  installed: boolean;
  downloads: number;
  rating: number;
  category: string;
}

export default function PluginManager() {
  const [selectedTab, setSelectedTab] = useState<string>("installed");
  const [searchQuery, setSearchQuery] = useState("");

  const plugins: Plugin[] = [
    {
      id: "terrain-tools",
      name: "高级地形工具",
      version: "1.2.0",
      author: "TerrainStudio",
      description: "提供高级地形雕刻、侵蚀模拟和植被分布工具",
      enabled: true,
      installed: true,
      downloads: 12500,
      rating: 4.8,
      category: "工具",
    },
    {
      id: "water-shader",
      name: "真实水体着色器",
      version: "2.0.1",
      author: "ShaderLab",
      description: "逼真的水体渲染，支持波浪、反射和折射效果",
      enabled: true,
      installed: true,
      downloads: 8900,
      rating: 4.9,
      category: "渲染",
    },
    {
      id: "ai-animator",
      name: "AI动画生成器",
      version: "0.9.5",
      author: "AIMotion",
      description: "使用AI自动生成角色动画和过渡",
      enabled: false,
      installed: true,
      downloads: 5600,
      rating: 4.5,
      category: "动画",
    },
    {
      id: "particle-fx",
      name: "粒子特效库",
      version: "3.1.2",
      author: "FXMaster",
      description: "包含200+预制粒子特效，火焰、烟雾、魔法等",
      enabled: false,
      installed: false,
      downloads: 15200,
      rating: 4.7,
      category: "特效",
    },
    {
      id: "sound-manager",
      name: "音频管理器",
      version: "1.5.0",
      author: "AudioPro",
      description: "高级音频管理和3D空间音效系统",
      enabled: false,
      installed: false,
      downloads: 7800,
      rating: 4.6,
      category: "音频",
    },
  ];

  const installedPlugins = plugins.filter((p) => p.installed);
  const availablePlugins = plugins.filter((p) => !p.installed);

  const handleTogglePlugin = (pluginId: string) => {
    const plugin = plugins.find((p) => p.id === pluginId);
    if (plugin) {
      toast.success(
        plugin.enabled ? `已禁用 ${plugin.name}` : `已启用 ${plugin.name}`
      );
    }
  };

  const handleInstallPlugin = (pluginId: string) => {
    const plugin = plugins.find((p) => p.id === pluginId);
    if (plugin) {
      toast.success(`正在安装 ${plugin.name}...`);
    }
  };

  const handleUninstallPlugin = (pluginId: string) => {
    const plugin = plugins.find((p) => p.id === pluginId);
    if (plugin) {
      toast.success(`已卸载 ${plugin.name}`);
    }
  };

  return (
    <div className="h-full flex">
      {/* 左侧分类 */}
      <div className="w-64 border-r border-border bg-card">
        <div className="h-12 border-b border-border flex items-center px-3">
          <span className="text-sm font-medium">分类</span>
        </div>
        <ScrollArea className="h-[calc(100%-3rem)]">
          <div className="p-2 space-y-1">
            {["全部", "工具", "渲染", "动画", "特效", "音频", "AI"].map(
              (category) => (
                <button
                  key={category}
                  className="w-full text-left px-3 py-2 text-sm rounded hover:bg-accent transition-colors"
                >
                  {category}
                </button>
              )
            )}
          </div>
        </ScrollArea>
      </div>

      {/* 中间内容区 */}
      <div className="flex-1 flex flex-col">
        {/* 工具栏 */}
        <div className="h-12 border-b border-border bg-card flex items-center justify-between px-4">
          <div className="flex items-center gap-2">
            <Package className="w-4 h-4 text-primary" />
            <span className="text-sm font-medium">插件管理器</span>
          </div>
          <div className="flex items-center gap-2">
            <div className="flex gap-1 mr-4">
              <Button
                size="sm"
                variant={selectedTab === "installed" ? "default" : "ghost"}
                className="h-7 px-3 text-xs"
                onClick={() => setSelectedTab("installed")}
              >
                已安装
              </Button>
              <Button
                size="sm"
                variant={selectedTab === "available" ? "default" : "ghost"}
                className="h-7 px-3 text-xs"
                onClick={() => setSelectedTab("available")}
              >
                可用插件
              </Button>
              <Button
                size="sm"
                variant={selectedTab === "develop" ? "default" : "ghost"}
                className="h-7 px-3 text-xs"
                onClick={() => setSelectedTab("develop")}
              >
                开发文档
              </Button>
            </div>
            <Button size="sm" variant="ghost" className="h-8 gap-2">
              <RefreshCw className="w-4 h-4" />
              <span className="text-xs">刷新</span>
            </Button>
          </div>
        </div>

        {/* 搜索栏 */}
        <div className="p-4 border-b border-border">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
            <input
              type="text"
              className="w-full pl-10 pr-4 py-2 bg-input border border-border rounded-lg text-sm"
              placeholder="搜索插件..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            />
          </div>
        </div>

        {/* 插件列表 */}
        <ScrollArea className="flex-1 p-4">
          {selectedTab === "installed" && (
            <div className="space-y-3">
              {installedPlugins.map((plugin) => (
                <Card key={plugin.id} className="p-4">
                  <div className="flex items-start justify-between">
                    <div className="flex-1">
                      <div className="flex items-center gap-2 mb-2">
                        <h3 className="text-sm font-semibold">{plugin.name}</h3>
                        <span className="text-xs text-muted-foreground">
                          v{plugin.version}
                        </span>
                        {plugin.enabled && (
                          <span className="px-2 py-0.5 bg-green-500/20 text-green-500 text-[10px] font-medium rounded">
                            已启用
                          </span>
                        )}
                      </div>
                      <p className="text-xs text-muted-foreground mb-2">
                        {plugin.description}
                      </p>
                      <div className="flex items-center gap-4 text-xs text-muted-foreground">
                        <span>作者: {plugin.author}</span>
                        <span className="flex items-center gap-1">
                          <Download className="w-3 h-3" />
                          {plugin.downloads.toLocaleString()}
                        </span>
                        <span className="flex items-center gap-1">
                          <Star className="w-3 h-3 fill-current text-yellow-500" />
                          {plugin.rating}
                        </span>
                      </div>
                    </div>
                    <div className="flex gap-2 ml-4">
                      <Button
                        size="sm"
                        variant="ghost"
                        className="h-8 w-8 p-0"
                        onClick={() => handleTogglePlugin(plugin.id)}
                      >
                        <Settings className="w-4 h-4" />
                      </Button>
                      <Button
                        size="sm"
                        variant="ghost"
                        className="h-8 w-8 p-0"
                        onClick={() => handleUninstallPlugin(plugin.id)}
                      >
                        <Trash2 className="w-4 h-4" />
                      </Button>
                    </div>
                  </div>
                </Card>
              ))}
            </div>
          )}

          {selectedTab === "available" && (
            <div className="space-y-3">
              {availablePlugins.map((plugin) => (
                <Card key={plugin.id} className="p-4">
                  <div className="flex items-start justify-between">
                    <div className="flex-1">
                      <div className="flex items-center gap-2 mb-2">
                        <h3 className="text-sm font-semibold">{plugin.name}</h3>
                        <span className="text-xs text-muted-foreground">
                          v{plugin.version}
                        </span>
                        <span className="px-2 py-0.5 bg-primary/20 text-primary text-[10px] font-medium rounded">
                          {plugin.category}
                        </span>
                      </div>
                      <p className="text-xs text-muted-foreground mb-2">
                        {plugin.description}
                      </p>
                      <div className="flex items-center gap-4 text-xs text-muted-foreground">
                        <span>作者: {plugin.author}</span>
                        <span className="flex items-center gap-1">
                          <Download className="w-3 h-3" />
                          {plugin.downloads.toLocaleString()}
                        </span>
                        <span className="flex items-center gap-1">
                          <Star className="w-3 h-3 fill-current text-yellow-500" />
                          {plugin.rating}
                        </span>
                      </div>
                    </div>
                    <Button
                      size="sm"
                      className="ml-4"
                      onClick={() => handleInstallPlugin(plugin.id)}
                    >
                      <Download className="w-3 h-3 mr-1" />
                      安装
                    </Button>
                  </div>
                </Card>
              ))}
            </div>
          )}

          {selectedTab === "develop" && (
            <div className="max-w-3xl mx-auto space-y-6">
              <Card className="p-6">
                <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
                  <Book className="w-5 h-5 text-primary" />
                  插件开发文档
                </h3>
                <div className="space-y-4">
                  <div>
                    <h4 className="text-sm font-semibold mb-2">快速开始</h4>
                    <p className="text-xs text-muted-foreground mb-2">
                      创建一个新的插件项目：
                    </p>
                    <div className="bg-muted/50 rounded p-3 font-mono text-xs">
                      <div className="text-green-400">
                        $ npm create nexus-plugin my-plugin
                      </div>
                      <div className="text-blue-400">$ cd my-plugin</div>
                      <div className="text-blue-400">$ npm install</div>
                      <div className="text-blue-400">$ npm run dev</div>
                    </div>
                  </div>

                  <Separator />

                  <div>
                    <h4 className="text-sm font-semibold mb-2">插件结构</h4>
                    <div className="bg-muted/50 rounded p-3 font-mono text-xs space-y-1">
                      <div>my-plugin/</div>
                      <div className="ml-2">├── src/</div>
                      <div className="ml-4">│   ├── index.ts</div>
                      <div className="ml-4">│   └── components/</div>
                      <div className="ml-2">├── package.json</div>
                      <div className="ml-2">└── plugin.config.json</div>
                    </div>
                  </div>

                  <Separator />

                  <div>
                    <h4 className="text-sm font-semibold mb-2">API示例</h4>
                    <div className="bg-muted/50 rounded p-3 font-mono text-xs space-y-1">
                      <div className="text-purple-400">
                        import {"{"} Plugin {"}"} from '@nexus/plugin-api';
                      </div>
                      <div className="text-blue-400 mt-2">
                        export default class MyPlugin extends Plugin {"{"}
                      </div>
                      <div className="ml-2 text-green-400">
                        onLoad() {"{"}
                      </div>
                      <div className="ml-4 text-muted-foreground">
                        console.log('Plugin loaded!');
                      </div>
                      <div className="ml-2 text-green-400">{"}"}</div>
                      <div className="text-blue-400">{"}"}</div>
                    </div>
                  </div>

                  <Button className="w-full" variant="outline">
                    <ExternalLink className="w-4 h-4 mr-2" />
                    查看完整文档
                  </Button>
                </div>
              </Card>

              <Card className="p-6">
                <h3 className="text-lg font-semibold mb-4">可用API</h3>
                <div className="space-y-3">
                  {[
                    {
                      name: "Scene API",
                      desc: "场景管理和实体操作",
                    },
                    {
                      name: "Renderer API",
                      desc: "自定义渲染管线和着色器",
                    },
                    {
                      name: "Animation API",
                      desc: "动画控制和混合",
                    },
                    {
                      name: "Physics API",
                      desc: "物理模拟和碰撞检测",
                    },
                    {
                      name: "UI API",
                      desc: "编辑器UI扩展",
                    },
                  ].map((api) => (
                    <Card
                      key={api.name}
                      className="p-3 hover:bg-accent/50 cursor-pointer"
                    >
                      <div className="flex items-center justify-between">
                        <div>
                          <div className="text-sm font-medium">{api.name}</div>
                          <div className="text-xs text-muted-foreground">
                            {api.desc}
                          </div>
                        </div>
                        <Code className="w-4 h-4 text-muted-foreground" />
                      </div>
                    </Card>
                  ))}
                </div>
              </Card>
            </div>
          )}
        </ScrollArea>
      </div>

      {/* 右侧详情面板 */}
      <div className="w-80 border-l border-border bg-card flex flex-col">
        <div className="h-12 border-b border-border flex items-center px-3">
          <span className="text-sm font-medium">插件详情</span>
        </div>
        <ScrollArea className="flex-1 p-4 space-y-4">
          <Card className="p-4">
            <h3 className="text-sm font-semibold mb-3">统计信息</h3>
            <div className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span className="text-muted-foreground">已安装</span>
                <span className="font-medium">{installedPlugins.length}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">已启用</span>
                <span className="font-medium">
                  {installedPlugins.filter((p) => p.enabled).length}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">可用更新</span>
                <span className="font-medium text-primary">2</span>
              </div>
            </div>
          </Card>

          <Card className="p-4">
            <h3 className="text-sm font-semibold mb-3">推荐插件</h3>
            <div className="space-y-2">
              {availablePlugins.slice(0, 3).map((plugin) => (
                <Card
                  key={plugin.id}
                  className="p-3 hover:bg-accent/50 cursor-pointer"
                >
                  <div className="text-sm font-medium mb-1">{plugin.name}</div>
                  <div className="flex items-center justify-between text-xs text-muted-foreground">
                    <span className="flex items-center gap-1">
                      <Star className="w-3 h-3 fill-current text-yellow-500" />
                      {plugin.rating}
                    </span>
                    <Button size="sm" variant="ghost" className="h-6 px-2">
                      <Plus className="w-3 h-3" />
                    </Button>
                  </div>
                </Card>
              ))}
            </div>
          </Card>

          <Card className="p-4">
            <h3 className="text-sm font-semibold mb-3">开发资源</h3>
            <div className="space-y-2">
              <Button size="sm" variant="outline" className="w-full justify-start">
                <Book className="w-3 h-3 mr-2" />
                API文档
              </Button>
              <Button size="sm" variant="outline" className="w-full justify-start">
                <Code className="w-3 h-3 mr-2" />
                示例代码
              </Button>
              <Button size="sm" variant="outline" className="w-full justify-start">
                <ExternalLink className="w-3 h-3 mr-2" />
                社区论坛
              </Button>
            </div>
          </Card>
        </ScrollArea>
      </div>
    </div>
  );
}
