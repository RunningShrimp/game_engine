# AI功能增强指南

## 概述

本文档介绍游戏引擎的AI功能增强，包括完整的导航网格生成器、增强的群体智能系统和决策树编辑器。

## 增强的导航网格生成器

### 功能特性

- **体素化场景**: 将场景转换为体素表示，提高生成精度
- **网格简化**: 自动简化导航网格，减少多边形数量
- **区域合并**: 智能合并小区域，优化网格结构
- **动态更新**: 支持动态场景的增量更新

### 使用方法

```rust
use game_engine::ai::{EnhancedNavMeshGenerator, EnhancedNavMeshConfig};

// 创建配置
let config = EnhancedNavMeshConfig {
    base_config: NavMeshConfig::default(),
    voxel_size: 0.1,
    enable_voxelization: true,
    enable_simplification: true,
    simplification_threshold: 0.1,
    enable_region_merging: true,
    region_merge_threshold: 0.5,
};

// 创建生成器
let mut generator = EnhancedNavMeshGenerator::new(config);

// 体素化场景
let vertices = vec![/* ... */];
let indices = vec![/* ... */];
generator.voxelize_scene(&vertices, &indices, true)?;

// 生成导航网格
let navmesh = generator.generate_from_voxels()?;
```

### 配置选项

- `voxel_size`: 体素大小（影响精度和性能）
- `enable_voxelization`: 是否启用体素化
- `enable_simplification`: 是否启用网格简化
- `simplification_threshold`: 简化阈值（角度）
- `enable_region_merging`: 是否启用区域合并
- `region_merge_threshold`: 区域合并阈值

## 增强的群体智能系统

### 功能特性

- **分层群体**: 支持多个子群体，每个子群体可以有独立的领导者
- **领导者跟随**: 代理可以跟随子群体的领导者
- **路径跟随**: 群体可以沿着预定义路径移动
- **群体目标**: 群体可以朝向共同目标移动
- **动态行为权重**: 可以动态调整各种行为的权重

### 使用方法

```rust
use game_engine::ai::{EnhancedFlockManager, EnhancedFlockConfig};

// 创建配置
let config = EnhancedFlockConfig {
    base_config: FlockConfig::default(),
    enable_leader_following: true,
    leader_follow_weight: 1.5,
    enable_path_following: true,
    path_follow_weight: 1.0,
    enable_group_goal: true,
    group_goal_weight: 1.0,
    sub_flock_count: 3,
};

// 创建管理器
let mut manager = EnhancedFlockManager::new(config);

// 添加代理到不同子群体
let agent1 = manager.add_agent_to_flock(Vec3::new(0.0, 0.0, 0.0), 0);
let agent2 = manager.add_agent_to_flock(Vec3::new(1.0, 0.0, 0.0), 0);
let agent3 = manager.add_agent_to_flock(Vec3::new(5.0, 0.0, 0.0), 1);

// 设置子群体领导者
manager.set_leader(0, agent1)?;

// 设置路径
manager.set_path(vec![
    Vec3::new(0.0, 0.0, 0.0),
    Vec3::new(10.0, 0.0, 0.0),
    Vec3::new(10.0, 0.0, 10.0),
]);

// 设置群体目标
manager.set_group_goal(Some(Vec3::new(20.0, 0.0, 20.0)));

// 更新群体行为
manager.update(delta_time);
```

### 配置选项

- `enable_leader_following`: 是否启用领导者跟随
- `leader_follow_weight`: 领导者跟随权重
- `enable_path_following`: 是否启用路径跟随
- `path_follow_weight`: 路径跟随权重
- `enable_group_goal`: 是否启用群体目标
- `group_goal_weight`: 群体目标权重
- `sub_flock_count`: 子群体数量

## 决策树编辑器

### 功能特性

- **节点创建和编辑**: 创建和编辑决策树节点
- **树结构管理**: 管理节点之间的父子关系
- **树验证**: 验证决策树的完整性和正确性
- **序列化支持**: 支持决策树的保存和加载
- **可视化支持**: 提供节点位置信息用于可视化

### 使用方法

```rust
use game_engine::ai::{
    DecisionTreeEditor, DecisionNodeType, DecisionTreeError,
};

// 创建编辑器
let mut editor = DecisionTreeEditor::new();

// 创建新决策树
let tree = editor.create_tree("My Decision Tree".to_string());

// 添加节点
let root_id = tree.add_node(
    DecisionNodeType::Selector,
    "Root".to_string(),
    (0.0, 0.0),
);

let condition_id = tree.add_node(
    DecisionNodeType::Condition,
    "Check Health".to_string(),
    (-100.0, 100.0),
);

let action_id = tree.add_node(
    DecisionNodeType::Action,
    "Heal".to_string(),
    (100.0, 100.0),
);

// 连接节点
tree.add_child(root_id, condition_id)?;
tree.add_child(root_id, action_id)?;

// 更新节点
tree.update_node(condition_id, NodeUpdates {
    name: Some("Check Health Low".to_string()),
    description: Some("Check if health is below 50%".to_string()),
    position: None,
    data: Some(DecisionNodeData::Condition {
        expression: "health < 0.5".to_string(),
        parameters: HashMap::new(),
    }),
})?;

// 验证决策树
tree.validate()?;

// 保存决策树
editor.save_current_tree()?;
```

### 节点类型

- **Condition**: 条件节点（叶子节点），用于检查条件
- **Action**: 动作节点（叶子节点），用于执行动作
- **Selector**: 选择器节点（内部节点），尝试子节点直到一个成功
- **Sequence**: 序列节点（内部节点），按顺序执行所有子节点
- **Decorator**: 装饰器节点，修改子节点的行为

### 决策树结构

决策树是一个有向无环图（DAG），具有以下特点：
- 有一个根节点
- 所有节点都从根节点可达
- 叶子节点（Condition/Action）不能有子节点
- 内部节点（Selector/Sequence/Decorator）可以有子节点

## 性能优化建议

### 导航网格生成

1. **体素大小**: 较小的体素大小提高精度但增加计算时间
2. **简化阈值**: 较大的阈值减少多边形数量但可能降低精度
3. **区域合并**: 启用区域合并可以减少区域数量，提高寻路效率

### 群体智能

1. **子群体数量**: 根据场景需求设置合适的子群体数量
2. **感知半径**: 较大的感知半径增加计算量但提供更好的群体行为
3. **行为权重**: 根据游戏需求调整各种行为的权重

### 决策树

1. **树深度**: 保持合理的树深度以提高执行效率
2. **节点数量**: 避免创建过多的节点
3. **验证**: 定期验证决策树以确保正确性

## 硬件要求

### 导航网格生成

- **CPU**: 多核CPU推荐（可并行处理）
- **内存**: 根据场景大小，可能需要几GB内存
- **时间**: 大型场景的生成可能需要几分钟

### 群体智能

- **CPU**: 单核性能重要（实时更新）
- **内存**: 每个代理约100字节
- **性能**: 1000个代理约需1-2ms更新

### 决策树

- **CPU**: 决策树执行非常快（微秒级）
- **内存**: 每个节点约1KB
- **存储**: 决策树文件通常很小（几KB到几MB）

## 限制和注意事项

1. **导航网格生成限制**:
   - 体素化需要大量内存
   - 复杂场景的生成时间可能较长
   - 动态更新需要重新生成部分网格

2. **群体智能限制**:
   - 大量代理会影响性能
   - 子群体数量过多会增加计算复杂度
   - 路径跟随需要预定义路径

3. **决策树限制**:
   - 不支持循环引用
   - 树结构必须是有效的DAG
   - 可视化需要外部工具

## 未来计划

- [ ] 导航网格的增量更新
- [ ] 群体智能的GPU加速
- [ ] 决策树的可视化编辑器UI
- [ ] 决策树的运行时调试工具
- [ ] 更复杂的群体行为（如捕食-被捕食）

## 更多信息

- [AI系统API参考](../api_reference.md)
- [寻路系统](./pathfinding.md)
- [行为树系统](./behavior_tree.md)

