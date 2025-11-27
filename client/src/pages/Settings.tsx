import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Separator } from "@/components/ui/separator";
import { Switch } from "@/components/ui/switch";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

export default function Settings() {
  return (
    <div className="h-full flex flex-col">
      <div className="flex-1 overflow-hidden">
        <Tabs defaultValue="general" className="h-full flex flex-col">
          <div className="border-b border-border bg-card px-4">
            <TabsList className="h-12 bg-transparent">
              <TabsTrigger value="general">通用</TabsTrigger>
              <TabsTrigger value="rendering">渲染</TabsTrigger>
              <TabsTrigger value="physics">物理</TabsTrigger>
              <TabsTrigger value="editor">编辑器</TabsTrigger>
            </TabsList>
          </div>

          <div className="flex-1 overflow-hidden">
            <TabsContent value="general" className="h-full m-0">
              <ScrollArea className="h-full">
                <div className="p-6 max-w-3xl space-y-6">
                  <Card className="p-6">
                    <h3 className="text-sm font-semibold mb-4">项目设置</h3>
                    <div className="space-y-4">
                      <div className="space-y-2">
                        <Label htmlFor="project-name">项目名称</Label>
                        <Input
                          id="project-name"
                          defaultValue="我的游戏项目"
                          className="bg-input"
                        />
                      </div>
                      <div className="space-y-2">
                        <Label htmlFor="company-name">公司名称</Label>
                        <Input
                          id="company-name"
                          placeholder="输入公司名称"
                          className="bg-input"
                        />
                      </div>
                      <div className="space-y-2">
                        <Label htmlFor="version">版本号</Label>
                        <Input
                          id="version"
                          defaultValue="1.0.0"
                          className="bg-input"
                        />
                      </div>
                    </div>
                  </Card>

                  <Card className="p-6">
                    <h3 className="text-sm font-semibold mb-4">语言与区域</h3>
                    <div className="space-y-4">
                      <div className="space-y-2">
                        <Label htmlFor="language">界面语言</Label>
                        <Select defaultValue="zh-CN">
                          <SelectTrigger id="language" className="bg-input">
                            <SelectValue />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="zh-CN">简体中文</SelectItem>
                            <SelectItem value="en-US">English</SelectItem>
                            <SelectItem value="ja-JP">日本語</SelectItem>
                          </SelectContent>
                        </Select>
                      </div>
                      <div className="space-y-2">
                        <Label htmlFor="timezone">时区</Label>
                        <Select defaultValue="asia-shanghai">
                          <SelectTrigger id="timezone" className="bg-input">
                            <SelectValue />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="asia-shanghai">
                              亚洲/上海 (UTC+8)
                            </SelectItem>
                            <SelectItem value="america-new-york">
                              美洲/纽约 (UTC-5)
                            </SelectItem>
                            <SelectItem value="europe-london">
                              欧洲/伦敦 (UTC+0)
                            </SelectItem>
                          </SelectContent>
                        </Select>
                      </div>
                    </div>
                  </Card>

                  <Card className="p-6">
                    <h3 className="text-sm font-semibold mb-4">自动保存</h3>
                    <div className="space-y-4">
                      <div className="flex items-center justify-between">
                        <div>
                          <Label>启用自动保存</Label>
                          <p className="text-sm text-muted-foreground">
                            定期自动保存项目更改
                          </p>
                        </div>
                        <Switch defaultChecked />
                      </div>
                      <div className="space-y-2">
                        <Label htmlFor="autosave-interval">保存间隔（分钟）</Label>
                        <Input
                          id="autosave-interval"
                          type="number"
                          defaultValue="5"
                          className="bg-input"
                        />
                      </div>
                    </div>
                  </Card>
                </div>
              </ScrollArea>
            </TabsContent>

            <TabsContent value="rendering" className="h-full m-0">
              <ScrollArea className="h-full">
                <div className="p-6 max-w-3xl space-y-6">
                  <Card className="p-6">
                    <h3 className="text-sm font-semibold mb-4">图形API</h3>
                    <div className="space-y-4">
                      <div className="space-y-2">
                        <Label htmlFor="graphics-api">渲染后端</Label>
                        <Select defaultValue="vulkan">
                          <SelectTrigger id="graphics-api" className="bg-input">
                            <SelectValue />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="vulkan">Vulkan</SelectItem>
                            <SelectItem value="dx12">DirectX 12</SelectItem>
                            <SelectItem value="metal">Metal</SelectItem>
                            <SelectItem value="opengl">OpenGL</SelectItem>
                          </SelectContent>
                        </Select>
                      </div>
                    </div>
                  </Card>

                  <Card className="p-6">
                    <h3 className="text-sm font-semibold mb-4">质量设置</h3>
                    <div className="space-y-4">
                      <div className="space-y-2">
                        <Label htmlFor="quality-preset">质量预设</Label>
                        <Select defaultValue="high">
                          <SelectTrigger id="quality-preset" className="bg-input">
                            <SelectValue />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="low">低</SelectItem>
                            <SelectItem value="medium">中</SelectItem>
                            <SelectItem value="high">高</SelectItem>
                            <SelectItem value="ultra">超高</SelectItem>
                          </SelectContent>
                        </Select>
                      </div>
                      <Separator />
                      <div className="flex items-center justify-between">
                        <div>
                          <Label>启用全局光照</Label>
                          <p className="text-sm text-muted-foreground">
                            动态GI系统
                          </p>
                        </div>
                        <Switch defaultChecked />
                      </div>
                      <div className="flex items-center justify-between">
                        <div>
                          <Label>启用屏幕空间反射</Label>
                          <p className="text-sm text-muted-foreground">SSR</p>
                        </div>
                        <Switch defaultChecked />
                      </div>
                      <div className="flex items-center justify-between">
                        <div>
                          <Label>启用体积云</Label>
                          <p className="text-sm text-muted-foreground">
                            真实感云渲染
                          </p>
                        </div>
                        <Switch defaultChecked />
                      </div>
                      <div className="flex items-center justify-between">
                        <div>
                          <Label>启用FSR超分辨率</Label>
                          <p className="text-sm text-muted-foreground">
                            AMD FidelityFX Super Resolution
                          </p>
                        </div>
                        <Switch defaultChecked />
                      </div>
                    </div>
                  </Card>

                  <Card className="p-6">
                    <h3 className="text-sm font-semibold mb-4">阴影设置</h3>
                    <div className="space-y-4">
                      <div className="space-y-2">
                        <Label htmlFor="shadow-quality">阴影质量</Label>
                        <Select defaultValue="high">
                          <SelectTrigger id="shadow-quality" className="bg-input">
                            <SelectValue />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="low">低 (512x512)</SelectItem>
                            <SelectItem value="medium">中 (1024x1024)</SelectItem>
                            <SelectItem value="high">高 (2048x2048)</SelectItem>
                            <SelectItem value="ultra">超高 (4096x4096)</SelectItem>
                          </SelectContent>
                        </Select>
                      </div>
                      <div className="space-y-2">
                        <Label htmlFor="shadow-distance">阴影距离</Label>
                        <Input
                          id="shadow-distance"
                          type="number"
                          defaultValue="100"
                          className="bg-input"
                        />
                      </div>
                    </div>
                  </Card>
                </div>
              </ScrollArea>
            </TabsContent>

            <TabsContent value="physics" className="h-full m-0">
              <ScrollArea className="h-full">
                <div className="p-6 max-w-3xl space-y-6">
                  <Card className="p-6">
                    <h3 className="text-sm font-semibold mb-4">物理引擎</h3>
                    <div className="space-y-4">
                      <div className="space-y-2">
                        <Label htmlFor="physics-fps">物理更新频率 (Hz)</Label>
                        <Input
                          id="physics-fps"
                          type="number"
                          defaultValue="60"
                          className="bg-input"
                        />
                      </div>
                      <div className="space-y-2">
                        <Label htmlFor="gravity">重力加速度</Label>
                        <Input
                          id="gravity"
                          type="number"
                          defaultValue="-9.81"
                          step="0.01"
                          className="bg-input"
                        />
                      </div>
                    </div>
                  </Card>

                  <Card className="p-6">
                    <h3 className="text-sm font-semibold mb-4">高级物理</h3>
                    <div className="space-y-4">
                      <div className="flex items-center justify-between">
                        <div>
                          <Label>启用布料模拟</Label>
                          <p className="text-sm text-muted-foreground">
                            GPU加速布料物理
                          </p>
                        </div>
                        <Switch defaultChecked />
                      </div>
                      <div className="flex items-center justify-between">
                        <div>
                          <Label>启用流体模拟</Label>
                          <p className="text-sm text-muted-foreground">
                            SPH流体系统
                          </p>
                        </div>
                        <Switch />
                      </div>
                      <div className="flex items-center justify-between">
                        <div>
                          <Label>启用软体模拟</Label>
                          <p className="text-sm text-muted-foreground">
                            可变形物体
                          </p>
                        </div>
                        <Switch />
                      </div>
                    </div>
                  </Card>
                </div>
              </ScrollArea>
            </TabsContent>

            <TabsContent value="editor" className="h-full m-0">
              <ScrollArea className="h-full">
                <div className="p-6 max-w-3xl space-y-6">
                  <Card className="p-6">
                    <h3 className="text-sm font-semibold mb-4">编辑器外观</h3>
                    <div className="space-y-4">
                      <div className="space-y-2">
                        <Label htmlFor="theme">主题</Label>
                        <Select defaultValue="dark">
                          <SelectTrigger id="theme" className="bg-input">
                            <SelectValue />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="light">浅色</SelectItem>
                            <SelectItem value="dark">深色</SelectItem>
                            <SelectItem value="auto">跟随系统</SelectItem>
                          </SelectContent>
                        </Select>
                      </div>
                      <div className="space-y-2">
                        <Label htmlFor="font-size">字体大小</Label>
                        <Select defaultValue="medium">
                          <SelectTrigger id="font-size" className="bg-input">
                            <SelectValue />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="small">小</SelectItem>
                            <SelectItem value="medium">中</SelectItem>
                            <SelectItem value="large">大</SelectItem>
                          </SelectContent>
                        </Select>
                      </div>
                    </div>
                  </Card>

                  <Card className="p-6">
                    <h3 className="text-sm font-semibold mb-4">编辑器行为</h3>
                    <div className="space-y-4">
                      <div className="flex items-center justify-between">
                        <div>
                          <Label>显示网格</Label>
                          <p className="text-sm text-muted-foreground">
                            在3D视口中显示网格
                          </p>
                        </div>
                        <Switch defaultChecked />
                      </div>
                      <div className="flex items-center justify-between">
                        <div>
                          <Label>显示坐标轴</Label>
                          <p className="text-sm text-muted-foreground">
                            显示世界坐标轴
                          </p>
                        </div>
                        <Switch defaultChecked />
                      </div>
                      <div className="flex items-center justify-between">
                        <div>
                          <Label>吸附到网格</Label>
                          <p className="text-sm text-muted-foreground">
                            移动对象时吸附到网格
                          </p>
                        </div>
                        <Switch />
                      </div>
                    </div>
                  </Card>

                  <Card className="p-6">
                    <h3 className="text-sm font-semibold mb-4">快捷键</h3>
                    <div className="space-y-3 text-sm">
                      <div className="flex justify-between">
                        <span className="text-muted-foreground">保存</span>
                        <code className="px-2 py-1 bg-muted rounded">Ctrl+S</code>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-muted-foreground">运行</span>
                        <code className="px-2 py-1 bg-muted rounded">F5</code>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-muted-foreground">停止</span>
                        <code className="px-2 py-1 bg-muted rounded">Shift+F5</code>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-muted-foreground">撤销</span>
                        <code className="px-2 py-1 bg-muted rounded">Ctrl+Z</code>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-muted-foreground">重做</span>
                        <code className="px-2 py-1 bg-muted rounded">Ctrl+Y</code>
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
