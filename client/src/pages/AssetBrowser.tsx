import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import {
  FileImage,
  FileText,
  FileVideo,
  Folder,
  Grid3x3,
  Image,
  List,
  Music,
  Search,
  Upload,
} from "lucide-react";
import { useState } from "react";

type ViewMode = "grid" | "list";

export default function AssetBrowser() {
  const [viewMode, setViewMode] = useState<ViewMode>("grid");
  const [searchQuery, setSearchQuery] = useState("");

  const folders = [
    { id: 1, name: "模型", icon: Grid3x3, count: 24 },
    { id: 2, name: "纹理", icon: Image, count: 156 },
    { id: 3, name: "材质", icon: FileImage, count: 48 },
    { id: 4, name: "音频", icon: Music, count: 32 },
    { id: 5, name: "视频", icon: FileVideo, count: 8 },
    { id: 6, name: "脚本", icon: FileText, count: 67 },
  ];

  const assets = [
    { id: 1, name: "character_model.fbx", type: "model", size: "2.4 MB" },
    { id: 2, name: "ground_texture.png", type: "texture", size: "1.2 MB" },
    { id: 3, name: "metal_material.mat", type: "material", size: "128 KB" },
    { id: 4, name: "background_music.mp3", type: "audio", size: "3.8 MB" },
    { id: 5, name: "intro_video.mp4", type: "video", size: "12.5 MB" },
    { id: 6, name: "player_controller.js", type: "script", size: "4 KB" },
    { id: 7, name: "enemy_ai.lua", type: "script", size: "6 KB" },
    { id: 8, name: "wood_texture.png", type: "texture", size: "980 KB" },
  ];

  const filteredAssets = assets.filter((asset) =>
    asset.name.toLowerCase().includes(searchQuery.toLowerCase())
  );

  return (
    <div className="h-full flex">
      {/* 左侧文件夹树 */}
      <div className="w-64 border-r border-border bg-card">
        <div className="h-12 border-b border-border flex items-center px-3 justify-between">
          <span className="text-sm font-medium">资产文件夹</span>
          <Button size="sm" variant="ghost" className="h-7 w-7 p-0">
            <Upload className="w-4 h-4" />
          </Button>
        </div>
        <ScrollArea className="h-[calc(100%-3rem)]">
          <div className="p-2">
            {folders.map((folder) => {
              const Icon = folder.icon;
              return (
                <button
                  key={folder.id}
                  className="w-full flex items-center gap-2 px-2 py-2 rounded hover:bg-accent text-sm text-left group"
                >
                  <Icon className="w-4 h-4 text-muted-foreground" />
                  <span className="flex-1">{folder.name}</span>
                  <span className="text-xs text-muted-foreground">
                    {folder.count}
                  </span>
                </button>
              );
            })}
          </div>
        </ScrollArea>
      </div>

      {/* 右侧资产列表 */}
      <div className="flex-1 flex flex-col">
        {/* 工具栏 */}
        <div className="h-12 border-b border-border bg-card flex items-center px-4 gap-3">
          <div className="flex-1 relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
            <Input
              type="text"
              placeholder="搜索资产..."
              className="pl-9 h-8 bg-input"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            />
          </div>
          <Separator orientation="vertical" className="h-6" />
          <div className="flex gap-1">
            <Button
              size="sm"
              variant={viewMode === "grid" ? "secondary" : "ghost"}
              className="h-8 w-8 p-0"
              onClick={() => setViewMode("grid")}
            >
              <Grid3x3 className="w-4 h-4" />
            </Button>
            <Button
              size="sm"
              variant={viewMode === "list" ? "secondary" : "ghost"}
              className="h-8 w-8 p-0"
              onClick={() => setViewMode("list")}
            >
              <List className="w-4 h-4" />
            </Button>
          </div>
        </div>

        {/* 资产网格/列表 */}
        <ScrollArea className="flex-1">
          <div className="p-4">
            {viewMode === "grid" ? (
              <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4">
                {filteredAssets.map((asset) => (
                  <Card
                    key={asset.id}
                    className="p-3 hover:bg-accent/50 cursor-pointer transition-colors"
                  >
                    <div className="aspect-square bg-muted rounded mb-2 flex items-center justify-center">
                      {asset.type === "model" && (
                        <Grid3x3 className="w-8 h-8 text-muted-foreground" />
                      )}
                      {asset.type === "texture" && (
                        <Image className="w-8 h-8 text-muted-foreground" />
                      )}
                      {asset.type === "material" && (
                        <FileImage className="w-8 h-8 text-muted-foreground" />
                      )}
                      {asset.type === "audio" && (
                        <Music className="w-8 h-8 text-muted-foreground" />
                      )}
                      {asset.type === "video" && (
                        <FileVideo className="w-8 h-8 text-muted-foreground" />
                      )}
                      {asset.type === "script" && (
                        <FileText className="w-8 h-8 text-muted-foreground" />
                      )}
                    </div>
                    <div className="text-xs font-medium truncate" title={asset.name}>
                      {asset.name}
                    </div>
                    <div className="text-xs text-muted-foreground mt-1">
                      {asset.size}
                    </div>
                  </Card>
                ))}
              </div>
            ) : (
              <div className="space-y-1">
                {filteredAssets.map((asset) => (
                  <div
                    key={asset.id}
                    className="flex items-center gap-3 px-3 py-2 rounded hover:bg-accent cursor-pointer"
                  >
                    <div className="w-8 h-8 bg-muted rounded flex items-center justify-center">
                      {asset.type === "model" && (
                        <Grid3x3 className="w-4 h-4 text-muted-foreground" />
                      )}
                      {asset.type === "texture" && (
                        <Image className="w-4 h-4 text-muted-foreground" />
                      )}
                      {asset.type === "material" && (
                        <FileImage className="w-4 h-4 text-muted-foreground" />
                      )}
                      {asset.type === "audio" && (
                        <Music className="w-4 h-4 text-muted-foreground" />
                      )}
                      {asset.type === "video" && (
                        <FileVideo className="w-4 h-4 text-muted-foreground" />
                      )}
                      {asset.type === "script" && (
                        <FileText className="w-4 h-4 text-muted-foreground" />
                      )}
                    </div>
                    <div className="flex-1">
                      <div className="text-sm font-medium">{asset.name}</div>
                      <div className="text-xs text-muted-foreground">
                        {asset.type}
                      </div>
                    </div>
                    <div className="text-xs text-muted-foreground">
                      {asset.size}
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </ScrollArea>
      </div>

      {/* 右侧预览面板 */}
      <div className="w-80 border-l border-border bg-card">
        <div className="h-12 border-b border-border flex items-center px-3">
          <span className="text-sm font-medium">资产预览</span>
        </div>
        <ScrollArea className="h-[calc(100%-3rem)]">
          <div className="p-4">
            <div className="aspect-square bg-muted rounded mb-4 flex items-center justify-center">
              <Folder className="w-12 h-12 text-muted-foreground" />
            </div>
            <div className="space-y-3">
              <div>
                <label className="text-xs font-medium text-muted-foreground">
                  文件名
                </label>
                <div className="text-sm mt-1">未选择</div>
              </div>
              <Separator />
              <div>
                <label className="text-xs font-medium text-muted-foreground">
                  类型
                </label>
                <div className="text-sm mt-1">-</div>
              </div>
              <div>
                <label className="text-xs font-medium text-muted-foreground">
                  大小
                </label>
                <div className="text-sm mt-1">-</div>
              </div>
              <div>
                <label className="text-xs font-medium text-muted-foreground">
                  创建时间
                </label>
                <div className="text-sm mt-1">-</div>
              </div>
              <Separator />
              <Button className="w-full" size="sm" variant="outline">
                在场景中使用
              </Button>
            </div>
          </div>
        </ScrollArea>
      </div>
    </div>
  );
}
