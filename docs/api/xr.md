# XR 系统 API 参考

## OpenXrBackend

OpenXR 后端实现，支持 VR/AR 设备。

### 示例

```rust
use game_engine::xr::{XrConfig, OpenXrBackend};

let config = XrConfig {
    application_name: "My VR Game".to_string(),
    blend_mode: BlendMode::Opaque,
    reference_space: ReferenceSpaceType::Stage,
};

let mut xr_backend = OpenXrBackend::new(config)?;
```

## XrRenderer

XR 渲染器，支持立体渲染、ATW 和注视点渲染。

### 示例

```rust
use game_engine::xr::XrRenderer;

let mut xr_renderer = XrRenderer::new(device, queue);
xr_renderer.initialize()?;

// 渲染立体视图
xr_renderer.render_stereo(
    &mut encoder,
    &views,
    &render_targets,
    &depth_targets,
    |view, target, depth| {
        // 渲染逻辑
    },
)?;

// 应用 ATW
xr_renderer.apply_atw(
    &mut encoder,
    &rendered_texture,
    &depth_texture,
    &output_texture,
    &rendered_pose,
    &current_pose,
    &projection,
    &inv_projection,
    (1920, 1080),
)?;
```

## XrInputManager

XR 输入管理器，处理控制器和手部追踪。

### 示例

```rust
use game_engine::xr::{XrInputManager, Hand, ControllerButton};

let mut input_manager = XrInputManager::new();

// 检查按钮
if input_manager.is_button_pressed(Hand::Right, ControllerButton::TriggerClick) {
    println!("Trigger pressed!");
}

// 获取触发器值
let trigger = input_manager.get_trigger_value(Hand::Right);

// 触发震动
input_manager.vibrate(Hand::Right, 0.5, 100_000_000);
```

## 手部追踪

### HandTrackingData

手部追踪数据，包含所有关节的姿态。

### 示例

```rust
use game_engine::xr::HandTrackingData;

if let Some(hand_data) = input_manager.get_hand_tracking() {
    // 获取手掌姿态
    if let Some(palm) = hand_data.left_hand_joints.iter()
        .find(|j| j.joint_type == HandJointType::Palm) {
        let pose = palm.pose;
        // 使用手掌姿态
    }
}
```

