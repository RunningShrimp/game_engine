# AI系统增强计划 (AI System Enhancement Plan)

**版本**: v0.2.0 → v0.3.0
**创建日期**: 2025-12-30
**优先级**: P2
**预计时间**: 3-4周

---

## 📋 目录

1. [当前状态](#当前状态)
2. [增强目标](#增强目标)
3. [行为树系统](#行为树系统)
4. [覆盖图系统](#覆盖图系统)
5. [其他AI增强](#其他ai增强)
6. [实施计划](#实施计划)
7. [验证标准](#验证标准)

---

## 当前状态

### ✅ 已实现功能

**基础AI系统**:
- ✅ 状态机 (Finite State Machine)
- ✅ 基础寻路 (A*)
- ✅ 简单决策系统
- ✅ 感知系统 (视觉、听觉)
- ✅ 导航网格 (NavMesh)
- ✅ 群体行为 (flocking)

**代码模块**:
```rust
// game_engine/src/ai/
mod state_machine;    // 状态机
mod pathfinding;      // 寻路
mod perception;       // 感知
mod navigation;       // 导航
mod behavior;         // 行为
mod decision;         // 决策
```

**性能指标**:
- 状态机更新: <0.1ms per agent ✅
- A*寻路: <10ms (1000个节点) ✅
- 感知更新: <0.5ms per agent ✅
- 支持AI数量: ~100个并发

### ⚠️ 功能缺口

1. **行为树 (Behavior Trees)** - 缺失
2. **覆盖图 (Influence Maps)** - 缺失
3. **GOAP (Goal-Oriented Action Planning)** - 缺失
4. **HTN (Hierarchical Task Network)** - 缺失
5. **效用系统 (Utility AI)** - 缺失
6. **机器学习集成** - 缺失

---

## 增强目标

### P1 核心功能 (必须实现)

1. **行为树系统**
   - 可视化编辑器支持
   - 复合节点（Sequence、Selector、Parallel）
   - 装饰器节点
   - 条件节点
   - 行为节点
   - 黑板系统

2. **覆盖图系统**
   - 2D/3D覆盖图
   - 动态更新
   - 多层叠加
   - 可视化调试
   - 战术分析

### P2 增强功能 (推荐实现)

3. **GOAP系统**
   - 目标导向规划
   - 原子动作系统
   - 世界状态管理
   - 动态规划重算

4. **效用系统**
   - 效用曲线
   - 多准则决策
   - 权重系统

---

## 行为树系统

### 概述

行为树是一种用于AI决策的层次化结构，通过组合简单行为构建复杂AI逻辑。

### 架构设计

```rust
// game_engine/src/ai/behavior_tree/mod.rs

pub use self::node::*;
pub use self::composite::*;
pub use self::decorator::*;
pub use self::context::*;
pub use self::blackboard::*;

mod node;
mod composite;
mod decorator;
mod context;
mod blackboard;
```

### 核心类型

#### 1. 节点Trait

```rust
// game_engine/src/ai/behavior_tree/node.rs

use bevy_ecs::prelude::*;

/// 行为树节点trait
pub trait BehaviorNode: Send + Sync {
    /// 执行节点
    fn tick(&mut self, context: &mut BehaviorContext) -> BehaviorStatus;

    /// 重置节点状态
    fn reset(&mut self);

    /// 克隆节点（用于共享行为树）
    fn clone_node(&self) -> Box<dyn BehaviorNode>;

    /// 获取节点名称（调试用）
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}

/// 节点执行状态
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BehaviorStatus {
    Success,   // 成功
    Failure,   // 失败
    Running,   // 运行中
}
```

#### 2. 复合节点

```rust
// game_engine/src/ai/behavior_tree/composite.rs

/// Sequence节点：顺序执行所有子节点，全部成功才成功
pub struct SequenceNode {
    children: Vec<Box<dyn BehaviorNode>>,
    current_index: usize,
}

impl BehaviorNode for SequenceNode {
    fn tick(&mut self, context: &mut BehaviorContext) -> BehaviorStatus {
        while self.current_index < self.children.len() {
            let status = self.children[self.current_index].tick(context);

            match status {
                BehaviorStatus::Running => return BehaviorStatus::Running,
                BehaviorStatus::Failure => {
                    self.reset();
                    return BehaviorStatus::Failure;
                }
                BehaviorStatus::Success => {
                    self.current_index += 1;
                }
            }
        }

        self.reset();
        BehaviorStatus::Success
    }

    fn reset(&mut self) {
        self.current_index = 0;
        for child in &mut self.children {
            child.reset();
        }
    }
}

/// Selector节点：顺序执行子节点，任一成功即成功
pub struct SelectorNode {
    children: Vec<Box<dyn BehaviorNode>>,
    current_index: usize,
}

impl BehaviorNode for SelectorNode {
    fn tick(&mut self, context: &mut BehaviorContext) -> BehaviorStatus {
        while self.current_index < self.children.len() {
            let status = self.children[self.current_index].tick(context);

            match status {
                BehaviorStatus::Running => return BehaviorStatus::Running,
                BehaviorStatus::Success => {
                    self.reset();
                    return BehaviorStatus::Success;
                }
                BehaviorStatus::Failure => {
                    self.current_index += 1;
                }
            }
        }

        self.reset();
        BehaviorStatus::Failure
    }

    fn reset(&mut self) {
        self.current_index = 0;
        for child in &mut self.children {
            child.reset();
        }
    }
}

/// Parallel节点：并行执行所有子节点
pub struct ParallelNode {
    children: Vec<Box<dyn BehaviorNode>>,
    policy: ParallelPolicy,
}

#[derive(Clone, Copy)]
pub enum ParallelPolicy {
    /// 任一成功即成功
    SucceedOnOne,
    /// 全部成功才成功
    SucceedOnAll,
    /// 任一失败即失败
    FailOnOne,
    /// 全部失败才失败
    FailOnAll,
}

impl BehaviorNode for ParallelNode {
    fn tick(&mut self, context: &mut BehaviorContext) -> BehaviorStatus {
        let mut success_count = 0;
        let mut failure_count = 0;
        let mut running_count = 0;

        for child in &mut self.children {
            let status = child.tick(context);

            match status {
                BehaviorStatus::Success => success_count += 1,
                BehaviorStatus::Failure => failure_count += 1,
                BehaviorStatus::Running => running_count += 1,
            }
        }

        if running_count > 0 {
            return BehaviorStatus::Running;
        }

        match self.policy {
            ParallelPolicy::SucceedOnOne if success_count > 0 => BehaviorStatus::Success,
            ParallelPolicy::FailOnOne if failure_count > 0 => BehaviorStatus::Failure,
            ParallelPolicy::SucceedOnAll if success_count == self.children.len() => BehaviorStatus::Success,
            ParallelPolicy::FailOnAll if failure_count == self.children.len() => BehaviorStatus::Failure,
            _ => BehaviorStatus::Failure,
        }
    }
}
```

#### 3. 装饰器节点

```rust
// game_engine/src/ai/behavior_tree/decorator.rs

/// 反转节点结果
pub struct InverterDecorator {
    child: Box<dyn BehaviorNode>,
}

impl BehaviorNode for InverterDecorator {
    fn tick(&mut self, context: &mut BehaviorContext) -> BehaviorStatus {
        match self.child.tick(context) {
            BehaviorStatus::Success => BehaviorStatus::Failure,
            BehaviorStatus::Failure => BehaviorStatus::Success,
            BehaviorStatus::Running => BehaviorStatus::Running,
        }
    }

    fn reset(&mut self) {
        self.child.reset();
    }
}

/// 重复执行子节点N次
pub struct RepeatDecorator {
    child: Box<dyn BehaviorNode>,
    count: usize,
    current: usize,
}

impl BehaviorNode for RepeatDecorator {
    fn tick(&mut self, context: &mut BehaviorContext) -> BehaviorStatus {
        while self.current < self.count {
            let status = self.child.tick(context);

            match status {
                BehaviorStatus::Running => return BehaviorStatus::Running,
                BehaviorStatus::Success => {
                    self.current += 1;
                    self.child.reset();
                }
                BehaviorStatus::Failure => {
                    self.reset();
                    return BehaviorStatus::Failure;
                }
            }
        }

        self.reset();
        BehaviorStatus::Success
    }
}

/// 冷却装饰器
pub struct CooldownDecorator {
    child: Box<dyn BehaviorNode>,
    duration: Duration,
    last_run: Option<Instant>,
}

impl BehaviorNode for CooldownDecorator {
    fn tick(&mut self, context: &mut BehaviorContext) -> BehaviorStatus {
        if let Some(last_run) = self.last_run {
            if last_run.elapsed() < self.duration {
                return BehaviorStatus::Failure;
            }
        }

        let status = self.child.tick(context);

        if status == BehaviorStatus::Success {
            self.last_run = Some(Instant::now());
        }

        status
    }
}
```

#### 4. 条件和行为节点

```rust
// game_engine/src/ai/behavior_tree/leaf.rs

/// 条件节点：检查条件是否满足
pub struct ConditionNode<F>
where
    F: Fn(&BehaviorContext) -> bool + Send + Sync,
{
    name: String,
    condition: F,
}

impl<F> BehaviorNode for ConditionNode<F>
where
    F: Fn(&BehaviorContext) -> bool + Send + Sync,
{
    fn tick(&mut self, context: &mut BehaviorContext) -> BehaviorStatus {
        if (self.condition)(context) {
            BehaviorStatus::Success
        } else {
            BehaviorStatus::Failure
        }
    }
}

/// 行为节点：执行具体动作
pub struct ActionNode<F>
where
    F: FnMut(&mut BehaviorContext) -> BehaviorStatus + Send + Sync,
{
    name: String,
    action: F,
}

impl<F> BehaviorNode for ActionNode<F>
where
    F: FnMut(&mut BehaviorContext) -> BehaviorStatus + Send + Sync,
{
    fn tick(&mut self, context: &mut BehaviorContext) -> BehaviorStatus {
        (self.action)(context)
    }
}
```

#### 5. 黑板系统

```rust
// game_engine/src/ai/behavior_tree/blackboard.rs

use std::any::{Any, TypeId};
use std::collections::HashMap;

/// 黑板：节点间共享数据存储
pub struct Blackboard {
    data: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Blackboard {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// 写入数据
    pub fn write<T: 'static + Send + Sync>(&mut self, value: T) {
        self.data.insert(TypeId::of::<T>(), Box::new(value));
    }

    /// 读取数据
    pub fn read<T: 'static>(&self) -> Option<&T> {
        self.data
            .get(&TypeId::of::<T>())
            .and_then(|any| any.downcast_ref::<T>())
    }

    /// 读取并修改数据
    pub fn read_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.data
            .get_mut(&TypeId::of::<T>())
            .and_then(|any| any.downcast_mut::<T>())
    }

    /// 移除数据
    pub fn remove<T: 'static>(&mut self) -> Option<T> {
        self.data
            .remove(&TypeId::of::<T>())
            .and_then(|any| any.downcast::<T>().ok().map(|boxed| *boxed))
    }
}

/// 行为树上下文
pub struct BehaviorContext {
    /// 实体ID
    pub entity: Entity,

    /// 黑板
    pub blackboard: Blackboard,

    /// 世界引用（用于访问ECS）
    pub world: &World,

    /// Delta时间
    pub dt: Duration,
}
```

### 构建器模式

```rust
// game_engine/src/ai/behavior_tree/builder.rs

pub struct BehaviorTreeBuilder {
    root: Option<Box<dyn BehaviorNode>>,
}

impl BehaviorTreeBuilder {
    pub fn new() -> Self {
        Self { root: None }
    }

    /// Sequence节点
    pub fn sequence(self) -> CompositeBuilder<SequenceNode> {
        CompositeBuilder::new(SequenceNode::new())
    }

    /// Selector节点
    pub fn selector(self) -> CompositeBuilder<SelectorNode> {
        CompositeBuilder::new(SelectorNode::new())
    }
}

pub struct CompositeBuilder<N> {
    node: N,
}

impl<N: BehaviorNode> CompositeBuilder<N> {
    fn new(node: N) -> Self {
        Self { node }
    }

    /// 添加子节点
    pub fn push(mut self, child: Box<dyn BehaviorNode>) -> Self {
        self.node.push_child(child);
        self
    }

    /// 构建完成
    pub fn build(self) -> Box<dyn BehaviorNode> {
        Box::new(self.node)
    }
}

// 使用示例
let behavior_tree = BehaviorTreeBuilder::new()
    .sequence()
        .push(Box::new(ConditionNode::new("Has Target", |ctx| {
            ctx.blackboard.read::<TargetEntity>().is_some()
        })))
        .push(Box::new(ActionNode::new("Attack", |ctx| {
            // 攻击逻辑
            BehaviorStatus::Success
        })))
    .build();
```

### 序列化/反序列化

```rust
// 支持从JSON加载行为树
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct BehaviorTreeJson {
    tree: String,
    nodes: Vec<NodeJson>,
}

#[derive(Serialize, Deserialize)]
struct NodeJson {
    id: String,
    type_: String,  // "sequence", "selector", "condition", "action"
    name: String,
    children: Vec<String>,  // 子节点ID
    config: serde_json::Value,  // 节点特定配置
}

impl BehaviorTree {
    pub fn from_json(json: &str) -> Result<Self, ParseError> {
        // 解析JSON并构建行为树
    }

    pub fn to_json(&self) -> String {
        // 序列化为JSON
    }
}
```

---

## 覆盖图系统

### 概述

覆盖图(Influence Maps)是一种空间表示技术，用于表示环境中不同位置的"影响"或"价值"，常用于战术AI和空间推理。

### 核心实现

```rust
// game_engine/src/ai/influence_map/mod.rs

pub use self::grid::*;
pub use self::layer::*;
pub use self::propagation::*;

mod grid;
mod layer;
mod propagation;

use std::collections::HashMap;

/// 2D覆盖图网格
pub struct InfluenceGrid {
    width: usize,
    height: usize,
    cell_size: f32,
    values: Vec<f32>,
}

impl InfluenceGrid {
    pub fn new(width: usize, height: usize, cell_size: f32) -> Self {
        Self {
            width,
            height,
            cell_size,
            values: vec![0.0; width * height],
        }
    }

    /// 获取位置的影响值
    pub fn get(&self, x: usize, y: usize) -> f32 {
        assert!(x < self.width && y < self.height);
        self.values[y * self.width + x]
    }

    /// 设置位置的影响值
    pub fn set(&mut self, x: usize, y: usize, value: f32) {
        assert!(x < self.width && y < self.height);
        self.values[y * self.width + x] = value;
    }

    /// 世界坐标转网格坐标
    pub fn world_to_grid(&self, world_pos: Vec2) -> (usize, usize) {
        let x = (world_pos.x / self.cell_size).floor() as usize;
        let y = (world_pos.y / self.cell_size).floor() as usize;
        (x.min(self.width - 1), y.min(self.height - 1))
    }

    /// 网格坐标转世界坐标
    pub fn grid_to_world(&self, grid_x: usize, grid_y: usize) -> Vec2 {
        Vec2::new(
            grid_x as f32 * self.cell_size + self.cell_size / 2.0,
            grid_y as f32 * self.cell_size + self.cell_size / 2.0,
        )
    }

    /// 添加影响源（点源）
    pub fn add_influence(&mut self, position: Vec2, strength: f32, radius: f32) {
        let (center_x, center_y) = self.world_to_grid(position);
        let radius_cells = (radius / self.cell_size).ceil() as usize;

        for dy in -radius_cells as i32..=radius_cells as i32 {
            for dx in -radius_cells as i32..=radius_cells as i32 {
                let gx = center_x as i32 + dx;
                let gy = center_y as i32 + dy;

                if gx < 0 || gy < 0 || gx >= self.width as i32 || gy >= self.height as i32 {
                    continue;
                }

                let distance = ((dx * dx + dy * dy) as f32).sqrt() * self.cell_size;

                if distance <= radius {
                    let falloff = 1.0 - (distance / radius);
                    let gx = gx as usize;
                    let gy = gy as usize;

                    self.values[gy * self.width + gx] += strength * falloff;
                }
            }
        }
    }

    /// 应用衰减
    pub fn decay(&mut self, factor: f32) {
        for value in &mut self.values {
            *value *= factor;
        }
    }

    /// 归一化到[0, 1]
    pub fn normalize(&mut self) {
        let max = self.values.iter().cloned().fold(0.0_f32, f32::max);
        if max > 0.0 {
            for value in &mut self.values {
                *value /= max;
            }
        }
    }
}
```

### 多层覆盖图

```rust
/// 多层覆盖图系统
pub struct InfluenceMapSystem {
    layers: HashMap<String, InfluenceGrid>,
    resolution: f32,  // 网格分辨率
}

impl InfluenceMapSystem {
    pub fn new(resolution: f32) -> Self {
        Self {
            layers: HashMap::new(),
            resolution,
        }
    }

    /// 添加覆盖图层
    pub fn add_layer(&mut self, name: String, width: usize, height: usize) {
        self.layers.insert(
            name,
            InfluenceGrid::new(width, height, self.resolution),
        );
    }

    /// 获取层
    pub fn get_layer(&self, name: &str) -> Option<&InfluenceGrid> {
        self.layers.get(name)
    }

    /// 获取可变层
    pub fn get_layer_mut(&mut self, name: &str) -> Option<&mut InfluenceGrid> {
        self.layers.get_mut(name)
    }

    /// 合并多个层
    pub fn merge_layers(&self, layer_names: &[&str], weights: &[f32]) -> InfluenceGrid {
        assert_eq!(layer_names.len(), weights.len());

        let first_layer = self.get_layer(layer_names[0]).unwrap();
        let mut result = InfluenceGrid::new(
            first_layer.width,
            first_layer.height,
            first_layer.cell_size,
        );

        for (layer_name, weight) in layer_names.iter().zip(weights.iter()) {
            let layer = self.get_layer(layer_name).unwrap();

            for i in 0..layer.values.len() {
                result.values[i] += layer.values[i] * weight;
            }
        }

        result
    }

    /// 查找最高值位置
    pub fn find_max_value_position(&self, layer_name: &str) -> Option<Vec2> {
        let layer = self.get_layer(layer_name)?;

        let mut max_value = f32::MIN;
        let mut max_index = 0;

        for (i, &value) in layer.values.iter().enumerate() {
            if value > max_value {
                max_value = value;
                max_index = i;
            }
        }

        let y = max_index / layer.width;
        let x = max_index % layer.width;

        Some(layer.grid_to_world(x, y))
    }
}
```

### 影响传播

```rust
/// 影响传播算法
pub struct InfluencePropagation {
    iterations: usize,
    decay: f32,
}

impl InfluencePropagation {
    pub fn new(iterations: usize, decay: f32) -> Self {
        Self { iterations, decay }
    }

    /// 迭代传播
    pub fn propagate(&self, grid: &mut InfluenceGrid) {
        let mut temp = grid.values.clone();

        for _ in 0..self.iterations {
            for y in 1..grid.height - 1 {
                for x in 1..grid.width - 1 {
                    let idx = y * grid.width + x;

                    // 平均邻近单元格
                    let average = (
                        grid.values[idx - 1] +           // 左
                        grid.values[idx + 1] +           // 右
                        grid.values[idx - grid.width] + // 上
                        grid.values[idx + grid.width]   // 下
                    ) / 4.0;

                    temp[idx] = grid.values[idx] * (1.0 - self.decay) + average * self.decay;
                }
            }

            // 交换缓冲区
            std::mem::swap(&mut grid.values, &mut temp);
        }
    }
}
```

### 战术应用

```rust
/// 战术覆盖图管理器
pub struct TacticalInfluenceMap {
    territory: InfluenceMapSystem,      // 领土控制
    danger: InfluenceMapSystem,          // 危险区域
    opportunity: InfluenceMapSystem,    // 机会区域
    visibility: InfluenceMapSystem,     // 可见性
}

impl TacticalInfluenceMap {
    pub fn new(map_size: Vec2, resolution: f32) -> Self {
        let width = (map_size.x / resolution) as usize;
        let height = (map_size.y / resolution) as usize;

        let mut territory = InfluenceMapSystem::new(resolution);
        territory.add_layer("friendly".to_string(), width, height);
        territory.add_layer("enemy".to_string(), width, height);

        let mut danger = InfluenceMapSystem::new(resolution);
        danger.add_layer("enemy_fire".to_string(), width, height);
        danger.add_layer("hazards".to_string(), width, height);

        let mut opportunity = InfluenceMapSystem::new(resolution);
        opportunity.add_layer("cover".to_string(), width, height);
        opportunity.add_layer("high_ground".to_string(), width, height);

        let mut visibility = InfluenceMapSystem::new(resolution);
        visibility.add_layer("line_of_sight".to_string(), width, height);

        Self {
            territory,
            danger,
            opportunity,
            visibility,
        }
    }

    /// 更新所有覆盖图
    pub fn update(&mut self, entities: &[Entity], dt: Duration) {
        // 1. 更新领土控制
        self.update_territory(entities);

        // 2. 更新危险区域
        self.update_danger(entities);

        // 3. 更新机会区域
        self.update_opportunity(entities);

        // 4. 传播影响
        let propagation = InfluencePropagation::new(3, 0.5);

        for layer in self.territory.layers.values_mut() {
            propagation.propagate(layer);
        }

        for layer in self.danger.layers.values_mut() {
            propagation.propagate(layer);
        }
    }

    /// 计算位置的战术得分
    pub fn evaluate_position(&self, position: Vec2) -> TacticalScore {
        let friendly_control = self.territory.get_layer("friendly")
            .map(|l| l.get_from_world(position))
            .unwrap_or(0.0);

        let enemy_control = self.territory.get_layer("enemy")
            .map(|l| l.get_from_world(position))
            .unwrap_or(0.0);

        let danger = self.danger.merge_layers(
            &["enemy_fire", "hazards"],
            &[1.0, 0.5]
        ).get_from_world(position);

        let opportunity = self.opportunity.merge_layers(
            &["cover", "high_ground"],
            &[0.7, 0.3]
        ).get_from_world(position);

        // 综合评分
        let score = (friendly_control - enemy_control) * 2.0
                  - danger
                  + opportunity;

        TacticalScore {
            position,
            overall: score,
            friendly_control,
            enemy_control,
            danger,
            opportunity,
        }
    }

    /// 查找最佳位置
    pub fn find_best_position(&self, center: Vec2, radius: f32) -> Vec2 {
        let mut best_pos = center;
        let mut best_score = f32::MIN;

        // 采样候选位置
        for angle in 0..360 {
            let rad = angle as f32 * std::f32::consts::PI / 180.0;
            for r in 0..10 {
                let dist = (r as f32 / 10.0) * radius;
                let pos = center + Vec2::new(rad.cos(), rad.sin()) * dist;

                let score = self.evaluate_position(pos);
                if score.overall > best_score {
                    best_score = score.overall;
                    best_pos = pos;
                }
            }
        }

        best_pos
    }
}

#[derive(Clone, Debug)]
pub struct TacticalScore {
    pub position: Vec2,
    pub overall: f32,
    pub friendly_control: f32,
    pub enemy_control: f32,
    pub danger: f32,
    pub opportunity: f32,
}
```

---

## 其他AI增强

### 1. GOAP系统

```rust
// Goal-Oriented Action Planning

pub struct GoapSystem {
    actions: Vec<Box<dyn Action>>,
    goals: Vec<Box<dyn Goal>>,
}

pub trait Action: Send + Sync {
    fn preconditions(&self, world_state: &WorldState) -> bool;
    fn effects(&self, world_state: &mut WorldState);
    fn cost(&self, world_state: &WorldState) -> f32;
    fn execute(&self, entity: Entity, world: &mut World);
}

pub trait Goal: Send + Sync {
    fn is_satisfied(&self, world_state: &WorldState) -> bool;
    fn priority(&self, world_state: &WorldState) -> f32;
}

impl GoapSystem {
    pub fn plan(&self, entity: Entity, world_state: &WorldState) -> Option<Vec<Action>> {
        // A*搜索最佳动作序列
    }
}
```

### 2. 效用系统

```rust
// Utility AI

pub struct UtilitySystem {
    curves: Vec<Box<dyn UtilityCurve>>,
    actions: Vec<Box<dyn UtilityAction>>,
}

pub trait UtilityCurve: Send + Sync {
    fn evaluate(&self, value: f32) -> f32;
}

pub trait UtilityAction: Send + Sync {
    fn utility(&self, context: &AIContext) -> f32;
    fn execute(&self, context: &mut AIContext);
}

// 预定义曲线
pub struct LinearCurve {
    slope: f32,
    intercept: f32,
}

pub struct ExponentialCurve {
    base: f32,
    exponent: f32,
}

pub struct SigmoidCurve {
    steepness: f32,
    midpoint: f32,
}
```

---

## 实施计划

### Phase 1: 行为树基础 (Week 1-2)

- [ ] 核心节点系统（node, composite, decorator）
- [ ] 黑板系统
- [ ] 构建器API
- [ ] 基础测试

**验收标准**:
- ✅ 可以创建和执行简单行为树
- ✅ 所有复合节点工作正常
- ✅ 单元测试覆盖率>80%

### Phase 2: 行为树高级特性 (Week 3)

- [ ] 条件和行为节点库
- [ ] JSON序列化
- [ ] 可视化调试工具
- [ ] 集成到ECS

**验收标准**:
- ✅ 支持常见行为模式
- ✅ 可以从JSON加载行为树
- ✅ 可视化工作正常

### Phase 3: 覆盖图系统 (Week 4)

- [ ] InfluenceGrid实现
- [ ] 多层系统
- [ ] 影响传播
- [ ] 战术应用

**验收标准**:
- ✅ 覆盖图更新<5ms
- ✅ 传播算法正确
- ✅ 战术决策合理

### Phase 4: 集成和优化 (Week 5-6)

- [ ] 与现有AI系统集成
- [ ] 性能优化
- [ ] 文档和示例
- [ ] 发布准备

**验收标准**:
- ✅ AI更新<1ms per agent
- ✅ 支持500+并发AI
- ✅ 完整文档

---

## 验证标准

### 行为树测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequence_node() {
        let mut sequence = SequenceNode::new();
        sequence.push_child(Box::new(AlwaysSuccessNode));
        sequence.push_child(Box::new(AlwaysSuccessNode));

        let mut context = BehaviorContext::test();
        assert_eq!(sequence.tick(&mut context), BehaviorStatus::Success);
    }

    #[test]
    fn test_selector_node() {
        let mut selector = SelectorNode::new();
        selector.push_child(Box::new(AlwaysFailureNode));
        selector.push_child(Box::new(AlwaysSuccessNode));

        let mut context = BehaviorContext::test();
        assert_eq!(selector.tick(&mut context), BehaviorStatus::Success);
    }
}
```

### 覆盖图测试

```rust
#[test]
fn test_influence_propagation() {
    let mut grid = InfluenceGrid::new(10, 10, 1.0);

    // 添加强影响源
    grid.add_influence(Vec2::new(5.0, 5.0), 10.0, 3.0);

    let propagation = InfluencePropagation::new(5, 0.5);
    propagation.propagate(&mut grid);

    // 验证传播范围
    assert!(grid.get(5, 5) > 0.0);
    assert!(grid.get(0, 0) > 0.0);  // 应该传播到角落
}
```

### 性能基准

```rust
#[bench]
fn bench_behavior_tree_tick(b: &mut Bencher) {
    let mut tree = create_complex_behavior_tree();
    let mut context = BehaviorContext::test();

    b.iter(|| {
        tree.tick(&mut context)
    });
}

// 目标: <0.01ms per tick

#[bench]
fn bench_influence_map_update(b: &mut Bencher) {
    let mut map = TacticalInfluenceMap::new(Vec2::new(100.0, 100.0), 1.0);

    b.iter(|| {
        map.update(&entities, Duration::from_millis(16));
    });
}

// 目标: <5ms per update (100 entities)
```

---

## 相关资源

### 学术资源
- [Behavior Trees in AI Games](https://www.cs.virginia.edu/~robins/lops/BehaviorTrees.pdf)
- [Influence Maps for Strategic Reasoning](https://www.aaai.org/Papers/AIIDE/2005/AIIDE05-032.pdf)
- "Programming Game AI by Example" - Mat Buckland

### 开源库
- [behavior-tree-rs](https://github.com/logico/behavior-tree) - Rust行为树实现
- [Flame (an Eclipse plugin for Behavior Trees)](https://github.com/flame/flame)

### 游戏AI参考
- [The AI Blog](http://www.ai-blog.net/)
- [Game AI Pro](https://www.crcpress.com/Game-AI-Pro-Expert-Briefings/Congdon/p/book/9781466565419)
- [GDC Vault (AI sessions)](https://www.gdcvault.com/)

---

**维护者**: 游戏引擎AI团队
**最后更新**: 2025-12-30
**版本**: v1.0
