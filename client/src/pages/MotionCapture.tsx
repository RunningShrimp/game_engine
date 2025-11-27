import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import {
  Activity,
  Download,
  FileVideo,
  Filter,
  Play,
  Save,
  Upload,
  User,
  Wand2,
} from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";

export default function MotionCapture() {
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [isPlaying, setIsPlaying] = useState(false);

  const mocapFiles = [
    {
      id: "walk-01",
      name: "walk_cycle_01.bvh",
      duration: "2.5s",
      fps: 60,
      bones: 52,
      size: "1.2 MB",
    },
    {
      id: "run-01",
      name: "run_forward_01.fbx",
      duration: "1.8s",
      fps: 30,
      bones: 52,
      size: "2.4 MB",
    },
    {
      id: "jump-01",
      name: "jump_up_01.bvh",
      duration: "1.2s",
      fps: 60,
      bones: 52,
      size: "0.8 MB",
    },
  ];

  const handleImport = () => {
    toast.success("正在导入动作捕捉数据...");
  };

  const handleRetarget = () => {
    toast.success("开始重定向到目标骨骼");
  };

  const handleClean = () => {
    toast.success("正在清理数据...");
  };

  const handleSmooth = () => {
    toast.success("正在平滑数据...");
  };

  const handleExport = () => {
    toast.success("正在导出动画...");
  };

  return (
    <div className="h-full flex">
      {/* 左侧文件列表 */}
      <div className="w-64 border-r border-border bg-card">
        <div className="h-12 border-b border-border flex items-center justify-between px-3">
          <span className="text-sm font-medium">Mocap文件</span>
          <Button
            size="sm"
            variant="ghost"
            className="h-7 w-7 p-0"
            onClick={handleImport}
          >
            <Upload className="w-4 h-4" />
          </Button>
        </div>
        <ScrollArea className="h-[calc(100%-3rem)]">
          <div className="p-2 space-y-1">
            {mocapFiles.map((file) => (
              <button
                key={file.id}
                className={`w-full text-left px-3 py-2 text-sm rounded transition-colors ${
                  selectedFile === file.id
                    ? "bg-primary text-primary-foreground"
                    : "hover:bg-accent"
                }`}
                onClick={() => setSelectedFile(file.id)}
              >
                <div className="flex items-center gap-2 mb-1">
                  <FileVideo className="w-3 h-3" />
                  <span className="font-medium truncate">{file.name}</span>
                </div>
                <div className="text-xs text-muted-foreground">
                  {file.duration} · {file.fps}fps
                </div>
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
            <span className="text-sm font-medium">动作捕捉集成</span>
          </div>
          <div className="flex items-center gap-2">
            <Button size="sm" variant="ghost" className="h-8 gap-2" onClick={handleClean}>
              <Filter className="w-4 h-4" />
              <span className="text-xs">清理</span>
            </Button>
            <Button size="sm" variant="ghost" className="h-8 gap-2" onClick={handleSmooth}>
              <Wand2 className="w-4 h-4" />
              <span className="text-xs">平滑</span>
            </Button>
            <Separator orientation="vertical" className="h-6" />
            <Button size="sm" variant="ghost" className="h-8 gap-2" onClick={handleRetarget}>
              <User className="w-4 h-4" />
              <span className="text-xs">重定向</span>
            </Button>
            <Button size="sm" variant="ghost" className="h-8 gap-2" onClick={handleExport}>
              <Save className="w-4 h-4" />
              <span className="text-xs">导出</span>
            </Button>
          </div>
        </div>

        {/* 3D预览区 */}
        <div className="flex-1 bg-background flex items-center justify-center">
          <div className="text-center space-y-4">
            <div className="w-96 h-96 mx-auto bg-gradient-to-br from-primary/20 to-purple-500/20 rounded-lg flex items-center justify-center">
              <User className="w-32 h-32 text-primary/50" />
            </div>
            <div className="text-sm text-muted-foreground">
              {selectedFile
                ? mocapFiles.find((f) => f.id === selectedFile)?.name
                : "选择一个Mocap文件"}
            </div>
            {selectedFile && (
              <Button
                onClick={() => {
                  setIsPlaying(!isPlaying);
                  toast.info(isPlaying ? "已暂停" : "正在播放");
                }}
              >
                <Play className="w-4 h-4 mr-2" />
                {isPlaying ? "暂停" : "播放"}
              </Button>
            )}
          </div>
        </div>

        {/* 底部时间轴 */}
        <div className="h-32 border-t border-border bg-card p-4">
          <div className="flex items-center justify-between mb-2">
            <span className="text-xs font-medium">时间轴</span>
            <div className="flex items-center gap-2">
              <span className="text-xs text-muted-foreground">0:00 / 2:30</span>
              <Button size="sm" variant="ghost" className="h-6 px-2 text-xs">
                重置
              </Button>
            </div>
          </div>
          <div className="h-16 bg-muted rounded relative overflow-hidden">
            <div className="absolute inset-0 flex items-center px-2">
              <div className="flex-1 h-2 bg-primary/20 rounded-full relative">
                <div className="h-full w-1/3 bg-primary rounded-full"></div>
                <div className="absolute top-1/2 left-1/3 -translate-y-1/2 w-3 h-3 bg-primary rounded-full border-2 border-background"></div>
              </div>
            </div>
            {/* 关键帧标记 */}
            <div className="absolute bottom-2 left-0 right-0 flex justify-around px-4">
              {[...Array(10)].map((_, i) => (
                <div
                  key={i}
                  className="w-0.5 h-2 bg-muted-foreground/30 rounded"
                ></div>
              ))}
            </div>
          </div>
        </div>
      </div>

      {/* 右侧属性面板 */}
      <div className="w-80 border-l border-border bg-card flex flex-col">
        <div className="h-12 border-b border-border flex items-center px-3">
          <span className="text-sm font-medium">Mocap属性</span>
        </div>
        <ScrollArea className="flex-1 p-4 space-y-4">
          {selectedFile && (
            <>
              <Card className="p-4">
                <h3 className="text-sm font-semibold mb-3">文件信息</h3>
                <div className="space-y-2 text-sm">
                  {(() => {
                    const file = mocapFiles.find((f) => f.id === selectedFile);
                    return (
                      <>
                        <div className="flex justify-between">
                          <span className="text-muted-foreground">文件名</span>
                          <span className="font-medium">{file?.name}</span>
                        </div>
                        <div className="flex justify-between">
                          <span className="text-muted-foreground">时长</span>
                          <span className="font-medium">{file?.duration}</span>
                        </div>
                        <div className="flex justify-between">
                          <span className="text-muted-foreground">帧率</span>
                          <span className="font-medium">{file?.fps} fps</span>
                        </div>
                        <div className="flex justify-between">
                          <span className="text-muted-foreground">骨骼数</span>
                          <span className="font-medium">{file?.bones}</span>
                        </div>
                        <div className="flex justify-between">
                          <span className="text-muted-foreground">文件大小</span>
                          <span className="font-medium">{file?.size}</span>
                        </div>
                      </>
                    );
                  })()}
                </div>
              </Card>

              <Card className="p-4">
                <h3 className="text-sm font-semibold mb-3">数据清理</h3>
                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <label className="text-xs font-medium">移除抖动</label>
                    <input type="checkbox" defaultChecked />
                  </div>
                  <div className="flex items-center justify-between">
                    <label className="text-xs font-medium">填充缺失帧</label>
                    <input type="checkbox" defaultChecked />
                  </div>
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">
                      噪声阈值
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
                <h3 className="text-sm font-semibold mb-3">平滑设置</h3>
                <div className="space-y-3">
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">
                      平滑强度
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
                      平滑窗口 (帧)
                    </label>
                    <input
                      type="number"
                      className="w-full px-2 py-1 bg-input border border-border rounded text-xs"
                      defaultValue="5"
                    />
                  </div>
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">
                      平滑算法
                    </label>
                    <select className="w-full px-2 py-1 bg-input border border-border rounded text-xs">
                      <option>高斯滤波</option>
                      <option>移动平均</option>
                      <option>Savitzky-Golay</option>
                      <option>卡尔曼滤波</option>
                    </select>
                  </div>
                </div>
              </Card>

              <Card className="p-4">
                <h3 className="text-sm font-semibold mb-3">重定向设置</h3>
                <div className="space-y-3">
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">
                      目标骨骼
                    </label>
                    <select className="w-full px-2 py-1 bg-input border border-border rounded text-xs">
                      <option>默认人形骨骼</option>
                      <option>UE4骨骼</option>
                      <option>Unity骨骼</option>
                      <option>自定义骨骼</option>
                    </select>
                  </div>
                  <div className="flex items-center justify-between">
                    <label className="text-xs font-medium">自动映射</label>
                    <input type="checkbox" defaultChecked />
                  </div>
                  <div className="flex items-center justify-between">
                    <label className="text-xs font-medium">保持比例</label>
                    <input type="checkbox" defaultChecked />
                  </div>
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">
                      缩放因子
                    </label>
                    <input
                      type="number"
                      className="w-full px-2 py-1 bg-input border border-border rounded text-xs"
                      defaultValue="1.0"
                      step="0.1"
                    />
                  </div>
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
                      <option>BVH</option>
                      <option>GLB/GLTF</option>
                      <option>引擎原生格式</option>
                    </select>
                  </div>
                  <div>
                    <label className="text-xs font-medium text-muted-foreground block mb-1">
                      帧率
                    </label>
                    <select className="w-full px-2 py-1 bg-input border border-border rounded text-xs">
                      <option>30 fps</option>
                      <option>60 fps</option>
                      <option>120 fps</option>
                      <option>保持原始</option>
                    </select>
                  </div>
                  <div className="flex items-center justify-between">
                    <label className="text-xs font-medium">压缩动画</label>
                    <input type="checkbox" />
                  </div>
                </div>
              </Card>
            </>
          )}

          {!selectedFile && (
            <Card className="p-6 text-center">
              <Download className="w-12 h-12 mx-auto mb-3 text-muted-foreground" />
              <p className="text-sm text-muted-foreground">
                选择或导入Mocap文件
              </p>
              <Button className="mt-4" onClick={handleImport}>
                <Upload className="w-4 h-4 mr-2" />
                导入文件
              </Button>
            </Card>
          )}
        </ScrollArea>
      </div>
    </div>
  );
}
