import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import {
  Box,
  Boxes,
  FileCode,
  Folder,
  Gauge,
  Layers,
  Play,
  Save,
  Settings,
  Square,
} from "lucide-react";
import ConnectionStatus from "./ConnectionStatus";
import CommandPalette from "./CommandPalette";
import { ReactNode } from "react";
import { useLocation } from "wouter";

interface EditorLayoutProps {
  children: ReactNode;
}

export default function EditorLayout({ children }: EditorLayoutProps) {
  const [location, setLocation] = useLocation();

  const navItems = [
    { icon: Layers, label: "场景", path: "/scene" },
    { icon: Folder, label: "资产", path: "/assets" },
    { icon: Box, label: "实体", path: "/entities" },
    { icon: FileCode, label: "代码", path: "/code" },
    { icon: Gauge, label: "调试", path: "/debug" },
    { icon: Settings, label: "设置", path: "/settings" },
  ];

  return (
    <>
      <CommandPalette />
      <div className="h-screen flex flex-col bg-background text-foreground">
      {/* 顶部工具栏 */}
      <div className="h-12 border-b border-border bg-card flex items-center justify-between px-4">
        <div className="flex items-center gap-3">
          <img src="/engine-logo.png" alt="Engine Logo" className="w-6 h-6" />
          <span className="font-semibold text-sm">Game Engine Editor</span>
        </div>
        <div className="flex items-center gap-2">
          <Button size="sm" variant="ghost" className="h-8 gap-2">
            <Save className="w-4 h-4" />
            <span className="text-xs">保存</span>
          </Button>
          <Button size="sm" variant="ghost" className="h-8 gap-2">
            <Play className="w-4 h-4" />
            <span className="text-xs">运行</span>
          </Button>
          <Button size="sm" variant="ghost" className="h-8 gap-2">
            <Square className="w-4 h-4" />
            <span className="text-xs">停止</span>
          </Button>
          <Separator orientation="vertical" className="h-6 mx-2" />
          <Button size="sm" variant="ghost" className="h-8 gap-2">
            <FileCode className="w-4 h-4" />
            <span className="text-xs">代码</span>
          </Button>
          <Separator orientation="vertical" className="h-6 mx-2" />
          <ConnectionStatus />
        </div>
      </div>

      <div className="flex-1 flex overflow-hidden">
        {/* 左侧导航栏 */}
        <div className="w-16 border-r border-border bg-sidebar flex flex-col items-center py-4 gap-2">
          {navItems.map((item) => {
            const Icon = item.icon;
            const isActive = location === item.path;
            return (
              <button
                key={item.path}
                onClick={() => setLocation(item.path)}
                className={`w-12 h-12 rounded-lg flex flex-col items-center justify-center gap-1 transition-colors ${
                  isActive
                    ? "bg-sidebar-accent text-sidebar-accent-foreground"
                    : "text-sidebar-foreground hover:bg-sidebar-accent/50"
                }`}
                title={item.label}
              >
                <Icon className="w-5 h-5" />
                <span className="text-[10px]">{item.label}</span>
              </button>
            );
          })}
        </div>

        {/* 主内容区域 */}
        <div className="flex-1 flex overflow-hidden">
          <ScrollArea className="flex-1">
            {children}
          </ScrollArea>
        </div>
      </div>
    </div>
    </>
  );
}
