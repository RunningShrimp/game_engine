# 网络系统 API 参考

## NetworkService

网络服务，处理多人游戏的网络同步。

### 示例

```rust
use game_engine::network::{NetworkService, NetworkState};

let mut network_service = NetworkService::new();
let network_state = NetworkState::default();

// 连接到服务器
network_service.connect("127.0.0.1:8080")?;

// 提交输入
network_service.submit_input(input_command)?;

// 应用客户端预测
network_service.apply_prediction(&mut world)?;

// 同步服务器状态
network_service.sync_server_state(&mut world)?;
```

## 客户端预测

### PredictionSystem

客户端预测系统，减少输入延迟。

### 示例

```rust
use game_engine::network::prediction::PredictionSystem;

let mut prediction = PredictionSystem::new();

// 记录输入
prediction.record_input(input_command, current_tick);

// 应用预测
prediction.apply_prediction(&mut world, current_tick)?;

// 回滚（如果服务器状态不一致）
prediction.rollback(&mut world, server_tick)?;
```

## 服务器权威

### AuthoritySystem

服务器权威系统，防止作弊。

### 示例

```rust
use game_engine::network::authority::AuthoritySystem;

let mut authority = AuthoritySystem::new();

// 验证客户端输入
if authority.validate_input(&input_command)? {
    authority.apply_input(&mut world, input_command)?;
} else {
    // 输入无效，拒绝
}
```

## 网络插值

### InterpolationSystem

网络插值系统，平滑网络状态。

### 示例

```rust
use game_engine::network::interpolation::InterpolationSystem;

let mut interpolation = InterpolationSystem::new();

// 添加状态快照
interpolation.add_snapshot(tick, state)?;

// 获取插值后的状态
let interpolated_state = interpolation.get_interpolated_state(current_tick)?;
```

