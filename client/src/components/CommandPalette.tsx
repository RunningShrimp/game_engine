import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import { useHotkeys } from "@/hooks/useHotkeys";
import {
  Activity,
  Bot,
  Box,
  FileCode,
  Folder,
  FolderOpen,
  Gauge,
  GitBranch,
  Home,
  Layers,
  MessageCircle,
  Mountain,
  Play,
  Save,
  Search,
  Settings,
  Sparkles,
  Square,
} from "lucide-react";
import { useState } from "react";
import { useLocation } from "wouter";

interface Command {
  id: string;
  label: string;
  icon: any;
  action: () => void;
  keywords?: string[];
}

export default function CommandPalette() {
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");
  const [, setLocation] = useLocation();

  useHotkeys([
    {
      key: 'k',
      ctrl: true,
      handler: () => setOpen(true),
    },
    {
      key: 'p',
      ctrl: true,
      handler: () => setOpen(true),
    },
  ]);

  const commands: Command[] = [
    {
      id: 'scene',
      label: '打开场景编辑器',
      icon: Layers,
      action: () => {
        setLocation('/scene');
        setOpen(false);
      },
      keywords: ['scene', 'editor', '场景', '编辑器'],
    },
    {
      id: 'assets',
      label: '打开资产浏览器',
      icon: Folder,
      action: () => {
        setLocation('/assets');
        setOpen(false);
      },
      keywords: ['assets', 'browser', '资产', '浏览器'],
    },
    {
      id: 'entities',
      label: '打开实体管理器',
      icon: Box,
      action: () => {
        setLocation('/entities');
        setOpen(false);
      },
      keywords: ['entities', 'manager', '实体', '管理器'],
    },
    {
      id: 'debug',
      label: '打开调试工具',
      icon: Gauge,
      action: () => {
        setLocation('/debug');
        setOpen(false);
      },
      keywords: ['debug', 'tools', '调试', '工具'],
    },
    {
      id: 'settings',
      label: '打开设置',
      icon: Settings,
      action: () => {
        setLocation('/settings');
        setOpen(false);
      },
      keywords: ['settings', '设置'],
    },
    {
      id: 'docs',
      label: '打开文档',
      icon: FileCode,
      action: () => {
        setLocation('/docs');
        setOpen(false);
      },
      keywords: ['docs', 'documentation', '文档'],
    },
    {
      id: 'code',
      label: '打开代码编辑器',
      icon: FileCode,
      action: () => {
        setLocation('/code');
        setOpen(false);
      },
      keywords: ['code', 'editor', '代码', '编辑器'],
    },
    {
      id: 'shader',
      label: '打开着色器编辑器',
      icon: Sparkles,
      action: () => {
        setLocation('/shader');
        setOpen(false);
      },
      keywords: ['shader', 'material', '着色器', '材质'],
    },
    {
      id: 'pcg',
      label: '打开PCG工具',
      icon: Mountain,
      action: () => {
        setLocation('/pcg');
        setOpen(false);
      },
      keywords: ['pcg', 'terrain', 'procedural', '程序化', '地形'],
    },
    {
      id: 'ai',
      label: '打开AI助手',
      icon: Bot,
      action: () => {
        setLocation('/ai');
        setOpen(false);
      },
      keywords: ['ai', 'assistant', 'copilot', '助手', '智能'],
    },
    {
      id: 'physics',
      label: '打开物理动画',
      icon: Activity,
      action: () => {
        setLocation('/physics');
        setOpen(false);
      },
      keywords: ['physics', 'ragdoll', 'animation', '物理', '布娃娃', '动画'],
    },
    {
      id: 'plugins',
      label: '打开插件管理器',
      icon: Box,
      action: () => {
        setLocation('/plugins');
        setOpen(false);
      },
      keywords: ['plugin', 'extension', 'addon', '插件', '扩展'],
    },
    {
      id: 'mocap',
      label: '打开动作捕捉',
      icon: FileCode,
      action: () => {
        setLocation('/mocap');
        setOpen(false);
      },
      keywords: ['mocap', 'motion', 'capture', '动捕', '动作'],
    },
    {
      id: 'facial',
      label: '打开面部动画',
      icon: FolderOpen,
      action: () => {
        setLocation('/facial');
        setOpen(false);
      },
      keywords: ['facial', 'expression', 'face', '面部', '表情'],
    },
    {
      id: 'retarget',
      label: '打开动画重定向',
      icon: Layers,
      action: () => {
        setLocation('/retarget');
        setOpen(false);
      },
      keywords: ['retarget', 'remap', 'skeleton', '重定向', '骨骼'],
    },
    {
      id: 'behavior',
      label: '打开行为树',
      icon: GitBranch,
      action: () => {
        setLocation('/behavior');
        setOpen(false);
      },
      keywords: ['behavior', 'tree', 'ai', '行为树', 'AI'],
    },
    {
      id: 'dialogue',
      label: '打开对话系统',
      icon: MessageCircle,
      action: () => {
        setLocation('/dialogue');
        setOpen(false);
      },
      keywords: ['dialogue', 'npc', 'conversation', '对话', 'NPC'],
    },
    {
      id: 'save',
      label: '保存',
      icon: Save,
      action: () => {
        console.log('Save action');
        setOpen(false);
      },
      keywords: ['save', '保存'],
    },
    {
      id: 'play',
      label: '运行',
      icon: Play,
      action: () => {
        console.log('Play action');
        setOpen(false);
      },
      keywords: ['play', 'run', '运行'],
    },
    {
      id: 'stop',
      label: '停止',
      icon: Square,
      action: () => {
        console.log('Stop action');
        setOpen(false);
      },
      keywords: ['stop', '停止'],
    },
  ];

  const filteredCommands = commands.filter((cmd) => {
    const searchLower = search.toLowerCase();
    return (
      cmd.label.toLowerCase().includes(searchLower) ||
      cmd.keywords?.some((kw) => kw.toLowerCase().includes(searchLower))
    );
  });

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogContent className="max-w-2xl">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Search className="w-5 h-5" />
            命令面板
          </DialogTitle>
        </DialogHeader>
        <div className="space-y-4">
          <Input
            placeholder="搜索命令... (Ctrl+K 或 Ctrl+P 打开)"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="bg-input"
            autoFocus
          />
          <ScrollArea className="h-96">
            <div className="space-y-1">
              {filteredCommands.map((cmd) => {
                const Icon = cmd.icon;
                return (
                  <Button
                    key={cmd.id}
                    variant="ghost"
                    className="w-full justify-start gap-3 h-12"
                    onClick={cmd.action}
                  >
                    <Icon className="w-5 h-5 text-muted-foreground" />
                    <span>{cmd.label}</span>
                  </Button>
                );
              })}
              {filteredCommands.length === 0 && (
                <div className="text-center py-8 text-muted-foreground">
                  未找到匹配的命令
                </div>
              )}
            </div>
          </ScrollArea>
        </div>
      </DialogContent>
    </Dialog>
  );
}
