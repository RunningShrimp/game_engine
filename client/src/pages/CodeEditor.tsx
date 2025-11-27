import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import Editor from "@monaco-editor/react";
import {
  FileCode,
  FileText,
  FolderOpen,
  Play,
  Save,
  Search,
} from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";

export default function CodeEditor() {
  const [selectedFile, setSelectedFile] = useState<string | null>("main.js");
  const [code, setCode] = useState<string>(`// 游戏主脚本
import { Engine, Scene, Entity } from 'game-engine';

// 初始化引擎
const engine = new Engine({
  width: 1920,
  height: 1080,
  title: 'My Game',
});

// 创建场景
const scene = new Scene('main');

// 创建玩家实体
const player = new Entity('player');
player.addComponent('Transform', {
  position: [0, 0, 0],
  rotation: [0, 0, 0],
  scale: [1, 1, 1],
});
player.addComponent('MeshRenderer', {
  mesh: 'cube',
  material: 'default',
});
player.addComponent('RigidBody', {
  mass: 1.0,
  useGravity: true,
});

scene.addEntity(player);

// 游戏循环
engine.onUpdate((deltaTime) => {
  // 更新游戏逻辑
  console.log('Frame time:', deltaTime);
});

// 启动引擎
engine.run();
`);

  const files = [
    { name: "main.js", type: "javascript", icon: FileCode },
    { name: "player.js", type: "javascript", icon: FileCode },
    { name: "enemy.js", type: "javascript", icon: FileCode },
    { name: "config.json", type: "json", icon: FileText },
    { name: "README.md", type: "markdown", icon: FileText },
  ];

  const handleSave = () => {
    toast.success("文件已保存");
  };

  const handleRun = () => {
    toast.info("运行脚本...");
  };

  return (
    <div className="h-full flex">
      {/* 左侧文件树 */}
      <div className="w-64 border-r border-border bg-card">
        <div className="h-12 border-b border-border flex items-center justify-between px-3">
          <span className="text-sm font-medium">文件</span>
          <Button size="sm" variant="ghost" className="h-7 w-7 p-0">
            <Search className="w-4 h-4" />
          </Button>
        </div>
        <ScrollArea className="h-[calc(100%-3rem)]">
          <div className="p-2">
            <div className="mb-2">
              <div className="flex items-center gap-2 px-2 py-1.5 text-sm font-semibold">
                <FolderOpen className="w-4 h-4 text-primary" />
                <span>scripts</span>
              </div>
            </div>
            <div className="ml-4 space-y-1">
              {files.map((file) => {
                const Icon = file.icon;
                return (
                  <button
                    key={file.name}
                    className={`w-full text-left px-2 py-1.5 text-sm rounded flex items-center gap-2 transition-colors ${
                      selectedFile === file.name
                        ? "bg-accent text-accent-foreground"
                        : "text-muted-foreground hover:text-foreground hover:bg-accent/50"
                    }`}
                    onClick={() => setSelectedFile(file.name)}
                  >
                    <Icon className="w-4 h-4" />
                    <span>{file.name}</span>
                  </button>
                );
              })}
            </div>
          </div>
        </ScrollArea>
      </div>

      {/* 编辑器区域 */}
      <div className="flex-1 flex flex-col">
        {/* 工具栏 */}
        <div className="h-12 border-b border-border bg-card flex items-center justify-between px-4">
          <div className="flex items-center gap-2">
            <FileCode className="w-4 h-4 text-muted-foreground" />
            <span className="text-sm font-medium">{selectedFile}</span>
          </div>
          <div className="flex items-center gap-2">
            <Button size="sm" variant="ghost" className="h-8 gap-2" onClick={handleSave}>
              <Save className="w-4 h-4" />
              <span className="text-xs">保存</span>
            </Button>
            <Separator orientation="vertical" className="h-6" />
            <Button size="sm" variant="ghost" className="h-8 gap-2" onClick={handleRun}>
              <Play className="w-4 h-4" />
              <span className="text-xs">运行</span>
            </Button>
          </div>
        </div>

        {/* Monaco Editor */}
        <div className="flex-1">
          <Editor
            height="100%"
            defaultLanguage="javascript"
            value={code}
            onChange={(value) => setCode(value || "")}
            theme="vs-dark"
            options={{
              fontSize: 14,
              minimap: { enabled: true },
              scrollBeyondLastLine: false,
              automaticLayout: true,
              tabSize: 2,
              wordWrap: "on",
              lineNumbers: "on",
              renderWhitespace: "selection",
              bracketPairColorization: {
                enabled: true,
              },
            }}
          />
        </div>
      </div>

      {/* 右侧输出面板 */}
      <div className="w-80 border-l border-border bg-card flex flex-col">
        <div className="h-12 border-b border-border flex items-center px-3">
          <span className="text-sm font-medium">输出</span>
        </div>
        <ScrollArea className="flex-1">
          <div className="p-4 space-y-2 font-mono text-xs">
            <Card className="p-3 bg-muted/50">
              <div className="text-green-500">[INFO] 脚本加载成功</div>
              <div className="text-muted-foreground text-[10px] mt-1">
                23:30:15
              </div>
            </Card>
            <Card className="p-3 bg-muted/50">
              <div className="text-blue-500">[DEBUG] Engine initialized</div>
              <div className="text-muted-foreground text-[10px] mt-1">
                23:30:16
              </div>
            </Card>
            <Card className="p-3 bg-muted/50">
              <div className="text-yellow-500">[WARN] Texture not found: player.png</div>
              <div className="text-muted-foreground text-[10px] mt-1">
                23:30:17
              </div>
            </Card>
          </div>
        </ScrollArea>
      </div>
    </div>
  );
}
