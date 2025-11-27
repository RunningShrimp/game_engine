import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  Activity,
  AlertCircle,
  CheckCircle,
  Cpu,
  HardDrive,
  Info,
  MemoryStick,
  Trash2,
  XCircle,
} from "lucide-react";

export default function DebugTools() {
  const performanceMetrics = [
    { label: "FPS", value: "60", unit: "fps", icon: Activity, color: "text-green-500" },
    { label: "帧时间", value: "16.7", unit: "ms", icon: Activity, color: "text-blue-500" },
    { label: "Draw Calls", value: "124", unit: "", icon: Cpu, color: "text-purple-500" },
    { label: "三角形", value: "45,678", unit: "", icon: Cpu, color: "text-orange-500" },
    { label: "内存使用", value: "512", unit: "MB", icon: MemoryStick, color: "text-yellow-500" },
    { label: "GPU使用", value: "65", unit: "%", icon: HardDrive, color: "text-red-500" },
  ];

  const logs = [
    { id: 1, level: "info", message: "引擎初始化完成", time: "10:23:45", icon: Info },
    { id: 2, level: "success", message: "场景加载成功", time: "10:23:46", icon: CheckCircle },
    { id: 3, level: "warning", message: "纹理压缩格式不支持，使用默认格式", time: "10:23:47", icon: AlertCircle },
    { id: 4, level: "error", message: "着色器编译失败: syntax error at line 42", time: "10:23:48", icon: XCircle },
    { id: 5, level: "info", message: "资产热重载: player_model.fbx", time: "10:23:49", icon: Info },
    { id: 6, level: "success", message: "物理系统启动完成", time: "10:23:50", icon: CheckCircle },
  ];

  const systemInfo = [
    { label: "操作系统", value: "Windows 11 Pro" },
    { label: "CPU", value: "Intel Core i7-12700K @ 3.6GHz" },
    { label: "GPU", value: "NVIDIA RTX 4070 Ti (12GB)" },
    { label: "内存", value: "32 GB DDR5" },
    { label: "引擎版本", value: "1.0.0-alpha" },
    { label: "渲染API", value: "Vulkan 1.3" },
  ];

  const profilerData = [
    { name: "渲染", time: 8.2, percentage: 49 },
    { name: "物理", time: 3.1, percentage: 18 },
    { name: "脚本", time: 2.5, percentage: 15 },
    { name: "动画", time: 1.8, percentage: 11 },
    { name: "音频", time: 0.7, percentage: 4 },
    { name: "其他", time: 0.4, percentage: 3 },
  ];

  return (
    <div className="h-full flex flex-col">
      <div className="flex-1 overflow-hidden">
        <Tabs defaultValue="performance" className="h-full flex flex-col">
          <div className="border-b border-border bg-card px-4">
            <TabsList className="h-12 bg-transparent">
              <TabsTrigger value="performance">性能监控</TabsTrigger>
              <TabsTrigger value="logs">日志</TabsTrigger>
              <TabsTrigger value="profiler">性能分析</TabsTrigger>
              <TabsTrigger value="system">系统信息</TabsTrigger>
            </TabsList>
          </div>

          <div className="flex-1 overflow-hidden">
            <TabsContent value="performance" className="h-full m-0">
              <ScrollArea className="h-full">
                <div className="p-6 space-y-6">
                  {/* 性能指标卡片 */}
                  <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                    {performanceMetrics.map((metric) => {
                      const Icon = metric.icon;
                      return (
                        <Card key={metric.label} className="p-4">
                          <div className="flex items-start justify-between">
                            <div>
                              <div className="text-sm text-muted-foreground mb-1">
                                {metric.label}
                              </div>
                              <div className="text-2xl font-bold">
                                {metric.value}
                                <span className="text-sm font-normal text-muted-foreground ml-1">
                                  {metric.unit}
                                </span>
                              </div>
                            </div>
                            <Icon className={`w-5 h-5 ${metric.color}`} />
                          </div>
                        </Card>
                      );
                    })}
                  </div>

                  {/* 性能图表占位 */}
                  <Card className="p-6">
                    <h3 className="text-sm font-semibold mb-4">帧率历史</h3>
                    <div className="h-48 bg-muted/30 rounded flex items-center justify-center">
                      <div className="text-center text-muted-foreground">
                        <Activity className="w-12 h-12 mx-auto mb-2" />
                        <p className="text-sm">实时性能图表</p>
                        <p className="text-xs mt-1">
                          将显示FPS、帧时间等实时数据
                        </p>
                      </div>
                    </div>
                  </Card>

                  {/* 内存使用 */}
                  <Card className="p-6">
                    <h3 className="text-sm font-semibold mb-4">内存分配</h3>
                    <div className="space-y-3">
                      <div>
                        <div className="flex justify-between text-sm mb-1">
                          <span>纹理</span>
                          <span className="text-muted-foreground">256 MB</span>
                        </div>
                        <div className="h-2 bg-muted rounded-full overflow-hidden">
                          <div className="h-full bg-primary w-[50%]"></div>
                        </div>
                      </div>
                      <div>
                        <div className="flex justify-between text-sm mb-1">
                          <span>网格</span>
                          <span className="text-muted-foreground">128 MB</span>
                        </div>
                        <div className="h-2 bg-muted rounded-full overflow-hidden">
                          <div className="h-full bg-primary w-[25%]"></div>
                        </div>
                      </div>
                      <div>
                        <div className="flex justify-between text-sm mb-1">
                          <span>音频</span>
                          <span className="text-muted-foreground">64 MB</span>
                        </div>
                        <div className="h-2 bg-muted rounded-full overflow-hidden">
                          <div className="h-full bg-primary w-[12.5%]"></div>
                        </div>
                      </div>
                      <div>
                        <div className="flex justify-between text-sm mb-1">
                          <span>其他</span>
                          <span className="text-muted-foreground">64 MB</span>
                        </div>
                        <div className="h-2 bg-muted rounded-full overflow-hidden">
                          <div className="h-full bg-primary w-[12.5%]"></div>
                        </div>
                      </div>
                    </div>
                  </Card>
                </div>
              </ScrollArea>
            </TabsContent>

            <TabsContent value="logs" className="h-full m-0">
              <div className="h-full flex flex-col">
                <div className="border-b border-border bg-card px-4 py-2 flex items-center justify-between">
                  <div className="flex gap-2">
                    <Button size="sm" variant="outline" className="h-7 text-xs">
                      全部
                    </Button>
                    <Button size="sm" variant="ghost" className="h-7 text-xs">
                      信息
                    </Button>
                    <Button size="sm" variant="ghost" className="h-7 text-xs">
                      警告
                    </Button>
                    <Button size="sm" variant="ghost" className="h-7 text-xs">
                      错误
                    </Button>
                  </div>
                  <Button size="sm" variant="ghost" className="h-7 gap-2">
                    <Trash2 className="w-3 h-3" />
                    <span className="text-xs">清空</span>
                  </Button>
                </div>
                <ScrollArea className="flex-1">
                  <div className="p-4 space-y-1 font-mono text-xs">
                    {logs.map((log) => {
                      const Icon = log.icon;
                      return (
                        <div
                          key={log.id}
                          className={`flex items-start gap-2 px-2 py-1.5 rounded ${
                            log.level === "error"
                              ? "bg-destructive/10 text-destructive"
                              : log.level === "warning"
                              ? "bg-yellow-500/10 text-yellow-500"
                              : log.level === "success"
                              ? "bg-green-500/10 text-green-500"
                              : "bg-muted/30"
                          }`}
                        >
                          <Icon className="w-4 h-4 mt-0.5 shrink-0" />
                          <span className="text-muted-foreground shrink-0">
                            [{log.time}]
                          </span>
                          <span className="flex-1">{log.message}</span>
                        </div>
                      );
                    })}
                  </div>
                </ScrollArea>
              </div>
            </TabsContent>

            <TabsContent value="profiler" className="h-full m-0">
              <ScrollArea className="h-full">
                <div className="p-6 space-y-6">
                  <Card className="p-6">
                    <h3 className="text-sm font-semibold mb-4">CPU时间分配</h3>
                    <div className="space-y-3">
                      {profilerData.map((item) => (
                        <div key={item.name}>
                          <div className="flex justify-between text-sm mb-1">
                            <span>{item.name}</span>
                            <span className="text-muted-foreground">
                              {item.time} ms ({item.percentage}%)
                            </span>
                          </div>
                          <div className="h-2 bg-muted rounded-full overflow-hidden">
                            <div
                              className="h-full bg-primary"
                              style={{ width: `${item.percentage}%` }}
                            ></div>
                          </div>
                        </div>
                      ))}
                    </div>
                  </Card>

                  <Card className="p-6">
                    <h3 className="text-sm font-semibold mb-4">调用堆栈</h3>
                    <div className="bg-muted/30 rounded p-4 font-mono text-xs space-y-1">
                      <div>update_systems() - 8.2ms</div>
                      <div className="pl-4">├─ render_system() - 5.1ms</div>
                      <div className="pl-8">│  ├─ draw_meshes() - 3.2ms</div>
                      <div className="pl-8">│  └─ post_processing() - 1.9ms</div>
                      <div className="pl-4">├─ physics_system() - 2.1ms</div>
                      <div className="pl-4">└─ script_system() - 1.0ms</div>
                    </div>
                  </Card>
                </div>
              </ScrollArea>
            </TabsContent>

            <TabsContent value="system" className="h-full m-0">
              <ScrollArea className="h-full">
                <div className="p-6">
                  <Card className="p-6">
                    <h3 className="text-sm font-semibold mb-4">系统信息</h3>
                    <div className="space-y-3">
                      {systemInfo.map((info) => (
                        <div key={info.label} className="flex justify-between">
                          <span className="text-sm text-muted-foreground">
                            {info.label}
                          </span>
                          <span className="text-sm font-medium">{info.value}</span>
                        </div>
                      ))}
                    </div>
                    <Separator className="my-4" />
                    <div className="space-y-2">
                      <h4 className="text-sm font-semibold">硬件加速</h4>
                      <div className="space-y-1 text-sm">
                        <div className="flex items-center gap-2">
                          <CheckCircle className="w-4 h-4 text-green-500" />
                          <span>SIMD优化 (AVX2)</span>
                        </div>
                        <div className="flex items-center gap-2">
                          <CheckCircle className="w-4 h-4 text-green-500" />
                          <span>GPU加速 (CUDA)</span>
                        </div>
                        <div className="flex items-center gap-2">
                          <CheckCircle className="w-4 h-4 text-green-500" />
                          <span>NPU加速 (OpenVINO)</span>
                        </div>
                        <div className="flex items-center gap-2">
                          <CheckCircle className="w-4 h-4 text-green-500" />
                          <span>FSR超分辨率</span>
                        </div>
                      </div>
                    </div>
                  </Card>
                </div>
              </ScrollArea>
            </TabsContent>
          </div>
        </Tabs>
      </div>
    </div>
  );
}
