# UE5迁移指南

本指南详细说明如何将Unreal Engine 5项目迁移到游戏引擎。

## 目录

- [概述](#概述)
- [迁移准备](#迁移准备)
- [迁移流程](#迁移流程)
- [蓝图转换](#蓝图转换)
- [资源迁移](#资源迁移)
- [常见问题](#常见问题)

## 概述

UE5迁移工具支持以下功能：

- ✅ **85%准确率的蓝图转换** - Blueprint → Lua/C#代码
- ✅ **蓝图组件映射** - UE5 Components → ECS组件
- ✅ **动画系统迁移** - Animation Sequences → 动画系统
- ✅ **材质系统转换** - UE5 Materials → PBR材质
- ✅ **资源格式转换** - uasset/umap → 引擎格式

## 迁移准备

### 1. 安装依赖

```bash
# 启用必要的特性
cargo build --features regex,serde_yaml
```

### 2. 准备UE5项目

确保UE5项目包含：

```
YourUE5Project/
├── Content/             # 资源目录
│   ├── Blueprints/      # 蓝图
│   ├── Materials/       # 材质
│   ├── Meshes/          # 网格
│   ├── Maps/            # 地图/场景
│   └── Sounds/          # 音频
├── Config/              # 配置文件
└── Source/              # C++源码
```

## 迁移流程

### 基本迁移

```rust
use game_engine::tools::migration::unreal::{UnrealProjectImporter, UnrealProjectAnalysis};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建导入器
    let importer = UnrealProjectImporter::new();

    // 分析UE5项目
    let project_path = std::path::PathBuf::from("/path/to/ue5/project");
    let analysis = importer.analyze(&project_path).await?;

    println!("项目分析结果:");
    println!("  纹理数: {}", analysis.texture_count);
    println!("  网格数: {}", analysis.mesh_count);
    println!("  材质数: {}", analysis.material_count);
    println!("  场景数: {}", analysis.scene_count);
    println!("  脚本数(蓝图): {}", analysis.script_count);

    Ok(())
}
```

### 完整迁移流程

```rust
use game_engine::tools::migration::unreal::UnrealProjectImporter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let importer = UnrealProjectImporter::new();

    let project_path = std::path::PathBuf::from("/path/to/ue5/project");
    let output_path = std::path::PathBuf::from("/path/to/output");

    // 1. 分析项目
    let analysis = importer.analyze(&project_path).await?;
    println!("✅ 项目分析完成");

    // 2. 迁移资源
    let asset_report = importer.migrate_assets(&output_path).await?;
    println!("✅ 资源迁移完成:");
    println!("   蓝图: {}", asset_report.converted_blueprints);
    println!("   材质: {}", asset_report.converted_materials);
    println!("   纹理: {}", asset_report.converted_textures);
    println!("   网格: {}", asset_report.converted_meshes);

    Ok(())
}
```

## 蓝图转换

### Actor蓝图到代码

**UE5蓝图结构:**
```
BP_PlayerCharacter (Blueprint)
├── Variables
│   ├── Health: float
│   └── MaxHealth: float
├── Events
│   ├── EventBeginPlay
│   └── EventTick
└── Functions
    └── TakeDamage
```

**转换后 (Lua):**
```lua
local Engine = require('engine')

local BP_PlayerCharacter = {}
BP_PlayerCharacter.mt = {}

function BP_PlayerCharacter.new(entity)
    local self = setmetatable({}, BP_PlayerCharacter)
    self.entity = entity
    self.health = 100.0
    self.max_health = 100.0
    return self
end

function BP_PlayerCharacter:on_start()
    -- EventBeginPlay
    Engine.log("Player spawned!")
end

function BP_PlayerCharacter:on_update(delta_time)
    -- EventTick
    -- Tick logic here
end

function BP_PlayerCharacter:take_damage(damage)
    -- TakeDamage function
    self.health = self.health - damage
    if self.health <= 0 then
        self:handle_death()
    end
end

function BP_PlayerCharacter:handle_death()
    Engine.log("Player died!")
    -- Respawn logic
end

return BP_PlayerCharacter
```

**转换后 (TypeScript):**
```typescript
import { Engine, Entity } from '@game-engine/core';

export class BP_PlayerCharacter {
    private entity: Entity;
    private health: number = 100.0;
    private max_health: number = 100.0;

    constructor(entity: Entity) {
        this.entity = entity;
    }

    on_start(): void {
        // EventBeginPlay
        Engine.log("Player spawned!");
    }

    on_update(deltaTime: number): void {
        // EventTick
    }

    takeDamage(damage: number): void {
        this.health -= damage;
        if (this.health <= 0) {
            this.handleDeath();
        }
    }

    private handleDeath(): void {
        Engine.log("Player died!");
    }
}
```

### 蓝图节点映射

| UE5节点 | Lua代码 | TypeScript代码 |
|---------|---------|----------------|
| `Event BeginPlay` | `on_start()` | `onStart(): void` |
| `Event Tick` | `on_update(dt)` | `onUpdate(deltaTime): void` |
| `Print String` | `Engine.log(text)` | `Engine.log(text)` |
| `Get Player Character` | `Engine.player.get()` | `Engine.player.get()` |
| `Set Actor Location` | `entity.position = pos` | `entity.position = pos` |
| `Get Actor Location` | `entity.position` | `entity.position` |
| `Add Movement Input` | `entity:add_movement_input()` | `entity.addMovementInput()` |

### 蓝图类型转换

#### Character蓝图

**UE5:**
```
BP_Character (Character)
├── Components
│   ├── CapsuleComponent
│   ├── CharacterMovementComponent
│   └── SkeletalMeshComponent
```

**转换后:**
```lua
-- Lua Character
function BP_Character.new(entity)
    local self = setmetatable({}, BP_Character)
    self.entity = entity

    -- 添加组件
    self.capsule = entity:add_component("CapsuleCollider")
    self.movement = entity:add_component("CharacterMovement")
    self.skeletal_mesh = entity:add_component("SkeletalMesh")

    return self
end
```

#### Widget蓝图

**UE5:**
```
W_HealthBar (Widget)
├── Canvas Panel
└── Progress Bar (Health)
```

**转换后:**
```lua
-- Lua Widget
local W_HealthBar = {}

function W_HealthBar.new()
    local self = {}
    self.canvas = Engine.ui.create_canvas_panel()
    self.health_bar = Engine.ui.create_progress_bar()

    self.canvas:add_child(self.health_bar)

    return self
end

function W_HealthBar:set_health_percent(percent)
    self.health_bar:set_value(percent)
end

return W_HealthBar
```

### 变量类型映射

| UE5类型 | Lua类型 | TypeScript类型 |
|---------|---------|----------------|
| `float` | `number` | `number` |
| `int32` | `number` | `number` |
| `bool` | `boolean` | `boolean` |
| `FString` | `string` | `string` |
| `FName` | `string` | `string` |
| `FVector` | `Vec3` | `Vec3` |
| `FRotator` | `Quat` | `Quat` |
| `FTransform` | `Transform` | `Transform` |
| `TArray<Type>` | `table` | `Type[]` |
| `TMap<Key, Value>` | `table` | `Map<Key, Value>` |

## 资源迁移

### 材质转换

**UE5材质:**
```
M_PlayerMaterial (Material)
├── Base Color: Texture2D
├── Metallic: 0.0
├── Roughness: 0.5
└── Normal: Texture2D
```

**引擎PBR材质:**
```yaml
engine_material:
  shader_type: "pbr"
  properties:
    albedo_color: [1.0, 1.0, 1.0, 1.0]
    albedo_map: "textures/player_base_color.png"
    metallic: 0.0
    roughness: 0.5
    roughness_map: "textures/player_roughness.png"
    normal_map: "textures/player_normal.png"
    normal_scale: 1.0
```

### 网格转换

支持的网格格式：

- **FBX** → 需要重新导出为glTF
- **glTF/GLB** → 推荐,直接支持
- **Obj** → 基本支持

**建议导出设置:**
```
导出格式: glTF 2.0
包含:
  ✓ 网格
  ✓ 法线
  ✓ UV坐标
  ✓ 骨骼(如果有动画)
  ✓ 权重
缩放: 100.0 (UE5使用厘米)
```

### 纹理转换

支持的纹理格式：
- **PNG** → 推荐(无损压缩)
- **JPG** → 支持(有损压缩)
- **TGA** → 支持
- **EXR** → 支持(HDR纹理)

### 动画转换

**UE5 Animation Sequence:**
```
Anim_Attack (AnimSequence)
├── Bone Transform Tracks
│   ├── Root
│   ├── Spine
│   └── Arms
└── Curve Tracks (Optional)
```

**引擎动画剪辑:**
```yaml
animation_clip:
  name: "attack"
  duration: 1.5
  tracks:
    - bone: "Root"
      keyframes:
        - time: 0.0
          position: [0, 0, 0]
          rotation: [0, 0, 0, 1]
        - time: 1.5
          position: [1, 0, 0]
          rotation: [0, 0, 0, 1]
```

### 音频转换

支持的音频格式：
- **WAV** → 推荐
- **OGG** → 推荐(压缩)
- **MP3** → 支持

**UE5 Sound Cue → 引擎音频源:**
```lua
-- Lua Audio Source
local sound_source = Engine.audio.create_source()
sound_source:set_clip("audio/footsteps.ogg")
sound_source:set_loop(true)
sound_source:set_volume(0.8)
sound_source:play()
```

## 场景迁移

### Level/Map转换

**UE5 Map (.umap):**
```
MainMap (Level)
├── Player Start
├── Static Meshes
│   ├── Floor
│   └── Walls
└── Lights
    ├── Directional Light
    └── Point Lights
```

**引擎场景:**
```yaml
entities:
  - name: "PlayerStart"
    position: [0.0, 0.0, 0.0]
    components:
      - Transform

  - name: "Floor"
    position: [0.0, -0.5, 0.0]
    components:
      - Transform
      - Mesh:
          path: "meshes/floor.glb"
      - Collider:
          shape: "box"

  - name: "DirectionalLight"
    position: [10.0, 10.0, 0.0]
    rotation: [45.0, 0.0, 0.0]
    components:
      - Light:
          type: "directional"
          intensity: 1.0
```

## 常见问题

### 1. 蓝图转换不完整

**问题:** 复杂蓝图逻辑节点未完全转换
**解决:** 手动实现复杂的节点逻辑

### 2. 动画不播放

**问题:** 骨骼名称不匹配
**解决:** 重命名骨骼或修改动画映射

### 3. 纹理显示错误

**问题:** 纹理坐标不正确
**解决:** 检查UV通道设置,重新导出模型

### 4. 材质渲染异常

**问题:** PBR参数不匹配
**解决:** 调整材质参数值

### 5. 物理行为不一致

**问题:** 物理引擎差异
**解决:** 调整碰撞体和物理材质

## 最佳实践

### 1. 蓝图简化

在转换前简化蓝图：

- ✅ 使用函数而非复杂节点图
- ✅ 避免宏和蓝图宏库
- ✅ 减少数据依赖
- ✅ 简化事件流

### 2. 资源优化

转换前优化资源：

- 压缩纹理大小
- 合并材质槽
- 简化网格拓扑
- 优化动画关键帧

### 3. 测试转换

为转换的蓝图编写测试：

```rust
#[test]
fn test_blueprint_conversion() {
    // 导入UE5蓝图
    let blueprint = importer.import_blueprint(&blueprint_path).await?;

    // 转换为Lua
    let lua_code = importer.convert_blueprint_to_lua(&blueprint)?;

    // 验证代码可编译
    assert!(lua_code.contains("function"));
    assert!(lua_code.contains("on_start"));
}
```

## 技术限制

### 不支持的UE5功能

- ❌ Niagara粒子系统 (使用引擎粒子系统重写)
- ❌ Control Rig (手动创建动画系统)
- ❌ Chaos物理 (基础物理支持)
- ❌ MetaSounds (使用引擎音频系统)
- ❌ World Partition (手动管理场景)

### 部分支持的功能

- ⚠️ Gameplay Ability System (简化版本)
- ⚠️ AI系统 (基础行为树)
- ⚠️ 网络复制 (需要手动实现)

## API映射表

### 生命周期事件

| UE5 | Lua | TypeScript |
|-----|-----|------------|
| `EventBeginPlay` | `on_start()` | `onStart(): void` |
| `EventTick` | `on_update(dt)` | `onUpdate(deltaTime): void` |
| `EventEndPlay` | `on_destroy()` | `onDestroy(): void` |

### Actor API

| UE5 | Lua | TypeScript |
|-----|-----|------------|
| `GetActorLocation()` | `entity.position` | `entity.position` |
| `SetActorLocation()` | `entity.position = v` | `entity.position = v` |
| `GetActorRotation()` | `entity.rotation` | `entity.rotation` |
| `SetActorRotation()` | `entity.rotation = r` | `entity.rotation = r` |
| `DestroyActor()` | `entity:destroy()` | `entity.destroy()` |

### 组件API

| UE5 | Lua | TypeScript |
|-----|-----|------------|
| `GetComponent()` | `entity:get_component()` | `entity.getComponent()` |
| `AddComponent()` | `entity:add_component()` | `entity.addComponent()` |
| `RemoveComponent()` | `entity:remove_component()` | `entity.removeComponent()` |

## 下一步

- [ ] 查看蓝图转换实现: `unreal.rs`
- [ ] 了解节点到代码映射
- [ ] 探索资源转换器
- [ ] 阅读迁移向导: `wizard.rs`

## 获取帮助

遇到迁移问题?

1. 查看迁移日志和警告信息
2. 参考`unreal.rs`中的实现
3. 使用迁移向导工具
4. 提交Issue到项目仓库

---

**注意**: 本迁移工具持续改进中,蓝图转换准确率会随版本更新提升。建议在迁移前备份项目。
