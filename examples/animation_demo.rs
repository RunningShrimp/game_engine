//! 动画系统使用示例
//!
//! 演示游戏引擎的完整动画系统功能

use game_engine::animation::{
    AnimationClip, AnimationStateMachine, AnimationCompressor, CompressionConfig,
    BlendSpace1D, BlendSpace2D, BlendTreeNode, AnimationBlendTree, AnimationState,
    StateTransition, TransitionCondition, Parameter, ParameterValue, ParameterType,
    AnimationLayer, AvatarMask, HumanoidBones, LayerBlendingMode, Keyframe, KeyframeTrack,
    InterpolationMode,
};
use glam::{Quat, Vec3};
use std::sync::Arc;

fn main() {
    println!("=== 游戏引擎动画系统示例 ===\n");

    // 示例1: 基础动画剪辑
    example_1_basic_clip();

    // 示例2: 动画混合树
    example_2_blend_tree();

    // 示例3: 1D混合空间
    example_3_blend_space_1d();

    // 示例4: 2D混合空间
    example_4_blend_space_2d();

    // 示例5: 动画状态机
    example_5_state_machine();

    // 示例6: 状态转换
    example_6_state_transitions();

    // 示例7: 混合参数
    example_7_blend_parameters();

    // 示例8: 动画层
    example_8_animation_layers();

    // 示例9: Avatar遮罩
    example_9_avatar_mask();

    // 示例10: 动画压缩
    example_10_animation_compression();

    // 示例11: 完整角色动画系统
    example_11_full_character_system();
}

/// 示例1: 基础动画剪辑
fn example_1_basic_clip() {
    println!("=== 示例1: 基础动画剪辑 ===\n");

    // 创建行走动画
    let mut walk_clip = AnimationClip::new("walk".to_string(), 1.0);
    walk_clip.looping = true;

    // 添加位置关键帧
    let mut position_track = KeyframeTrack::new(InterpolationMode::Linear);
    position_track.add_keyframe(Keyframe::new(0.0, Vec3::ZERO));
    position_track.add_keyframe(Keyframe::new(0.5, Vec3::new(0.5, 0.0, 0.0)));
    position_track.add_keyframe(Keyframe::new(1.0, Vec3::new(1.0, 0.0, 0.0)));
    walk_clip.add_position_track(0, position_track);

    // 添加旋转关键帧
    let mut rotation_track = KeyframeTrack::new(InterpolationMode::Linear);
    rotation_track.add_keyframe(Keyframe::new(0.0, Quat::IDENTITY));
    rotation_track.add_keyframe(Keyframe::new(0.5, Quat::from_rotation_y(0.1)));
    rotation_track.add_keyframe(Keyframe::new(1.0, Quat::IDENTITY));
    walk_clip.add_rotation_track(0, rotation_track);

    println!("✓ 动画剪辑: '{}', 持续时间: {}秒", walk_clip.name, walk_clip.duration);
    println!("  - 位置轨道: {} 个关键帧", walk_clip.position_tracks[&0].keyframes.len());
    println!("  - 旋转轨道: {} 个关键帧", walk_clip.rotation_tracks[&0].keyframes.len());
    println!("  - 循环播放: {}\n", walk_clip.looping);
}

/// 示例2: 动画混合树
fn example_2_blend_tree() {
    println!("=== 示例2: 动画混合树 ===\n");

    // 创建动画剪辑
    let idle_clip = Arc::new(AnimationClip::new("idle".to_string(), 1.0));
    let walk_clip = Arc::new(AnimationClip::new("walk".to_string(), 1.0));
    let run_clip = Arc::new(AnimationClip::new("run".to_string(), 1.0));

    // 创建混合节点
    let idle_node = BlendTreeNode::Clip {
        clip: idle_clip.clone(),
        speed: 1.0,
    };

    let walk_node = BlendTreeNode::Clip {
        clip: walk_clip.clone(),
        speed: 1.0,
    };

    let run_node = BlendTreeNode::Clip {
        clip: run_clip.clone(),
        speed: 1.0,
    };

    // 创建混合树：根据速度混合idle/walk/run
    let root = BlendTreeNode::Mix {
        weight: 0.5,
        children: vec![idle_node, walk_node, run_node],
    };

    let blend_tree = AnimationBlendTree::new(root);
    println!("✓ 动画混合树创建成功");
    println!("  - 混合节点: Mix (线性混合)");
    println!("  - 子节点: 3个 (idle, walk, run)");
    println!("  - 权重: 0.5\n");
}

/// 示例3: 1D混合空间
fn example_3_blend_space_1d() {
    println!("=== 示例3: 1D混合空间 ===\n");

    // 创建基于速度的1D混合空间
    let mut blend_space_1d = BlendSpace1D::new("speed".to_string(), 0.0, 10.0);

    // 添加动画剪辑
    blend_space_1d.add_clip(Arc::new(AnimationClip::new("idle".to_string(), 1.0)));
    blend_space_1d.add_clip(Arc::new(AnimationClip::new("walk".to_string(), 1.0)));
    blend_space_1d.add_clip(Arc::new(AnimationClip::new("run".to_string(), 1.0)));
    blend_space_1d.add_clip(Arc::new(AnimationClip::new("sprint".to_string(), 1.0)));

    println!("✓ 1D混合空间创建成功");
    println!("  - 参数: speed");
    println!("  - 范围: 0.0 - 10.0");
    println!("  - 动画数量: {}", blend_space_1d.clips.len());

    // 评估混合
    let blend_weight = blend_space_1d.evaluate(5.0); // 中等速度
    println!("  - 速度=5.0时的混合权重: {:.2}\n", blend_weight);
}

/// 示例4: 2D混合空间
fn example_4_blend_space_2d() {
    println!("=== 示例4: 2D混合空间 ===\n");

    // 创建基于方向和速度的2D混合空间
    let mut blend_space_2d = BlendSpace2D::new(
        "horizontal".to_string(),
        "vertical".to_string(),
        (-1.0, 1.0),
        (-1.0, 1.0),
        (3, 3),
    );

    // 设置网格位置的动画
    blend_space_2d.set_clip(0, 0, Arc::new(AnimationClip::new("walk_back_left".to_string(), 1.0)));
    blend_space_2d.set_clip(1, 0, Arc::new(AnimationClip::new("walk_back".to_string(), 1.0)));
    blend_space_2d.set_clip(2, 0, Arc::new(AnimationClip::new("walk_back_right".to_string(), 1.0)));
    blend_space_2d.set_clip(0, 1, Arc::new(AnimationClip::new("walk_left".to_string(), 1.0)));
    blend_space_2d.set_clip(1, 1, Arc::new(AnimationClip::new("idle".to_string(), 1.0)));
    blend_space_2d.set_clip(2, 1, Arc::new(AnimationClip::new("walk_right".to_string(), 1.0)));
    blend_space_2d.set_clip(0, 2, Arc::new(AnimationClip::new("walk_front_left".to_string(), 1.0)));
    blend_space_2d.set_clip(1, 2, Arc::new(AnimationClip::new("walk_front".to_string(), 1.0)));
    blend_space_2d.set_clip(2, 2, Arc::new(AnimationClip::new("walk_front_right".to_string(), 1.0)));

    println!("✓ 2D混合空间创建成功");
    println!("  - X轴: horizontal (-1.0 - 1.0)");
    println!("  - Y轴: vertical (-1.0 - 1.0)");
    println!("  - 网格大小: 3x3");

    // 评估混合
    let results = blend_space_2d.evaluate(0.5, 0.3); // 向右前方移动
    println!("  - 方向=(0.5, 0.3)时的混合结果:");
    for (x, y, weight) in results.iter() {
        println!("    网格[{}, {}] 权重: {:.2}", x, y, weight);
    }
    println!();
}

/// 示例5: 动画状态机
fn example_5_state_machine() {
    println!("=== 示例5: 动画状态机 ===\n");

    // 创建状态机
    let mut state_machine = AnimationStateMachine::new("player_locomotion".to_string());

    // 添加状态
    let idle_state = AnimationState::new("idle".to_string());
    let walk_state = AnimationState::new("walk".to_string());
    let run_state = AnimationState::new("run".to_string());

    state_machine.add_state(idle_state);
    state_machine.add_state(walk_state);
    state_machine.add_state(run_state);

    // 添加参数
    state_machine.add_parameter("speed".to_string(), Parameter {
        name: "speed".to_string(),
        value: ParameterValue::Float(0.0),
        param_type: ParameterType::Float,
    });

    state_machine.add_parameter("is_grounded".to_string(), Parameter {
        name: "is_grounded".to_string(),
        value: ParameterValue::Bool(true),
        param_type: ParameterType::Bool,
    });

    println!("✓ 动画状态机创建成功");
    println!("  - ID: player_locomotion");
    println!("  - 当前状态: {}", state_machine.current_state);
    println!("  - 状态数量: {}", state_machine.states.len());
    println!("  - 参数数量: {}\n", state_machine.parameters.len());
}

/// 示例6: 状态转换
fn example_6_state_transitions() {
    println!("=== 示例6: 状态转换 ===\n");

    let transition1 = StateTransition::new(
        "idle".to_string(),
        "walk".to_string(),
        TransitionCondition::Parameter {
            name: "speed".to_string(),
            operator: game_engine::animation::ParameterOperator::Greater,
            value: ParameterValue::Float(0.1),
        },
    )
    .with_duration(0.2);

    let transition2 = StateTransition::new(
        "walk".to_string(),
        "run".to_string(),
        TransitionCondition::Parameter {
            name: "speed".to_string(),
            operator: game_engine::animation::ParameterOperator::Greater,
            value: ParameterValue::Float(5.0),
        },
    )
    .with_duration(0.15)
    .with_interrupt(true);

    let transition3 = StateTransition::new(
        "walk".to_string(),
        "idle".to_string(),
        TransitionCondition::Parameter {
            name: "speed".to_string(),
            operator: game_engine::animation::ParameterOperator::Less,
            value: ParameterValue::Float(0.1),
        },
    )
    .with_duration(0.2);

    println!("✓ 状态转换创建成功");
    println!("  - 转换1: idle → walk (speed > 0.1)");
    println!("    持续时间: {}秒, 可中断: {}", transition1.duration, transition1.can_interrupt);
    println!("  - 转换2: walk → run (speed > 5.0)");
    println!("    持续时间: {}秒, 可中断: {}", transition2.duration, transition2.can_interrupt);
    println!("  - 转换3: walk → idle (speed < 0.1)");
    println!("    持续时间: {}秒, 可中断: {}\n", transition3.duration, transition3.can_interrupt);
}

/// 示例7: 混合参数
fn example_7_blend_parameters() {
    println!("=== 示例7: 混合参数 ===\n");

    // Float参数
    let speed_param = Parameter {
        name: "speed".to_string(),
        value: ParameterValue::Float(3.5),
        param_type: ParameterType::Float,
    };

    // Int参数
    let health_param = Parameter {
        name: "health".to_string(),
        value: ParameterValue::Int(100),
        param_type: ParameterType::Int,
    };

    // Bool参数
    let grounded_param = Parameter {
        name: "is_grounded".to_string(),
        value: ParameterValue::Bool(true),
        param_type: ParameterType::Bool,
    };

    // Trigger参数
    let jump_trigger = Parameter {
        name: "jump".to_string(),
        value: ParameterValue::Trigger,
        param_type: ParameterType::Trigger,
    };

    println!("✓ 混合参数创建成功");
    println!("  - Float参数: {} = {:.1}", speed_param.name,
        match speed_param.value { ParameterValue::Float(v) => v, _ => 0.0 });
    println!("  - Int参数: {} = {}", health_param.name,
        match health_param.value { ParameterValue::Int(v) => v, _ => 0 });
    println!("  - Bool参数: {} = {}", grounded_param.name,
        match grounded_param.value { ParameterValue::Bool(v) => v, _ => false });
    println!("  - Trigger参数: {}\n", jump_trigger.name);
}

/// 示例8: 动画层
fn example_8_animation_layers() {
    println!("=== 示例8: 动画层 ===\n");

    // 基础层（下半身）
    let base_layer = AnimationLayer {
        name: "Base Layer".to_string(),
        weight: 1.0,
        blending_mode: LayerBlendingMode::Override,
        state_machine: None,
        avatar_mask: None,
    };

    // 上半身层
    let upper_body_layer = AnimationLayer {
        name: "Upper Body".to_string(),
        weight: 1.0,
        blending_mode: LayerBlendingMode::Override,
        state_machine: None,
        avatar_mask: Some(AvatarMask::new("upper_body".to_string())),
    };

    // 叠加层（如瞄准动画）
    let additive_layer = AnimationLayer {
        name: "Aiming".to_string(),
        weight: 0.5,
        blending_mode: LayerBlendingMode::Additive,
        state_machine: None,
        avatar_mask: Some(AvatarMask::new("aim".to_string())),
    };

    println!("✓ 动画层创建成功");
    println!("  - 基础层: {} (权重: {:.1}, 模式: Override)",
        base_layer.name, base_layer.weight);
    println!("  - 上半身层: {} (权重: {:.1}, 模式: Override)",
        upper_body_layer.name, upper_body_layer.weight);
    println!("  - 瞄准层: {} (权重: {:.1}, 模式: Additive)",
        additive_layer.name, additive_layer.weight);
    println!();
}

/// 示例9: Avatar遮罩
fn example_9_avatar_mask() {
    println!("=== 示例9: Avatar遮罩 ===\n");

    // 创建上半身遮罩
    let mut upper_mask = AvatarMask::new("upper_body".to_string());

    // 设置骨骼权重
    upper_mask.set_bone_weight("spine".to_string(), 1.0);
    upper_mask.set_bone_weight("chest".to_string(), 1.0);
    upper_mask.set_bone_weight("head".to_string(), 1.0);
    upper_mask.set_bone_weight("left_arm".to_string(), 1.0);
    upper_mask.set_bone_weight("right_arm".to_string(), 1.0);
    upper_mask.set_bone_weight("left_leg".to_string(), 0.0);
    upper_mask.set_bone_weight("right_leg".to_string(), 0.0);

    println!("✓ Avatar遮罩创建成功");
    println!("  - 名称: {}", upper_mask.name);
    println!("  - 骨骼权重:");
    println!("    脊部: {:.0}", upper_mask.get_bone_weight("spine"));
    println!("    胸部: {:.0}", upper_mask.get_bone_weight("chest"));
    println!("    头部: {:.0}", upper_mask.get_bone_weight("head"));
    println!("    左臂: {:.0}", upper_mask.get_bone_weight("left_arm"));
    println!("    右臂: {:.0}", upper_mask.get_bone_weight("right_arm"));
    println!("    左腿: {:.0}", upper_mask.get_bone_weight("left_leg"));
    println!("    右腿: {:.0}\n", upper_mask.get_bone_weight("right_leg"));

    // 创建Humanoid骨骼遮罩
    let humanoid_bones = HumanoidBones {
        head: 1.0,
        left_arm: 1.0,
        right_arm: 1.0,
        left_leg: 0.0,
        right_leg: 0.0,
        body: 1.0,
    };

    println!("✓ Humanoid骨骼遮罩:");
    println!("  - 头部: {:.0}", humanoid_bones.head);
    println!("  - 左臂: {:.0}", humanoid_bones.left_arm);
    println!("  - 右臂: {:.0}", humanoid_bones.right_arm);
    println!("  - 左腿: {:.0}", humanoid_bones.left_leg);
    println!("  - 右腿: {:.0}", humanoid_bones.right_leg);
    println!("  - 躯干: {:.0}\n", humanoid_bones.body);
}

/// 示例10: 动画压缩
fn example_10_animation_compression() {
    println!("=== 示例10: 动画压缩 ===\n");

    // 创建测试动画
    let mut clip = AnimationClip::new("test_animation".to_string(), 2.0);

    let mut position_track = KeyframeTrack::new(InterpolationMode::Linear);
    for i in 0..100 {
        let time = i as f32 * 0.02;
        let pos = Vec3::new(time * 0.5, 0.0, 0.0);
        position_track.add_keyframe(Keyframe::new(time, pos));
    }
    clip.add_position_track(0, position_track);

    println!("原始动画:");
    println!("  - 关键帧数量: {}", clip.position_tracks[&0].keyframes.len());

    // 使用默认配置压缩
    let compressor = AnimationCompressor::with_default_config();
    let compressed_clip = compressor.compress_clip(&clip);

    let stats = compressor.get_compression_stats(&clip, &compressed_clip);

    println!("\n✓ 动画压缩完成 (默认配置):");
    println!("  - 压缩后关键帧: {}", compressed_clip.position_tracks[&0].keyframes.len());
    println!("  - 压缩率: {:.1}%", stats.compression_percentage());
    println!("  - 关键帧减少率: {:.1}%", stats.keyframe_reduction_percentage());

    // 使用高质量配置
    let high_quality_compressor = AnimationCompressor::new(CompressionConfig::high_quality());
    let high_quality_clip = high_quality_compressor.compress_clip(&clip);
    let high_quality_stats = high_quality_compressor.get_compression_stats(&clip, &high_quality_clip);

    println!("\n✓ 动画压缩完成 (高质量配置):");
    println!("  - 压缩后关键帧: {}", high_quality_clip.position_tracks[&0].keyframes.len());
    println!("  - 压缩率: {:.1}%", high_quality_stats.compression_percentage());
    println!("  - 关键帧减少率: {:.1}%", high_quality_stats.keyframe_reduction_percentage());

    // 使用最大压缩配置
    let max_compressor = AnimationCompressor::new(CompressionConfig::maximum_compression());
    let max_compressed_clip = max_compressor.compress_clip(&clip);
    let max_stats = max_compressor.get_compression_stats(&clip, &max_compressed_clip);

    println!("\n✓ 动画压缩完成 (最大压缩配置):");
    println!("  - 压缩后关键帧: {}", max_compressed_clip.position_tracks[&0].keyframes.len());
    println!("  - 压缩率: {:.1}%", max_stats.compression_percentage());
    println!("  - 关键帧减少率: {:.1}%", max_stats.keyframe_reduction_percentage());
    println!();
}

/// 示例11: 完整角色动画系统
fn example_11_full_character_system() {
    println!("=== 示例11: 完整角色动画系统 ===\n");

    // 创建角色状态机
    let mut character_fsm = AnimationStateMachine::new("character_fsm".to_string());

    // 添加参数
    character_fsm.add_parameter("speed".to_string(), Parameter {
        name: "speed".to_string(),
        value: ParameterValue::Float(0.0),
        param_type: ParameterType::Float,
    });

    character_fsm.add_parameter("direction_x".to_string(), Parameter {
        name: "direction_x".to_string(),
        value: ParameterValue::Float(0.0),
        param_type: ParameterType::Float,
    });

    character_fsm.add_parameter("direction_y".to_string(), Parameter {
        name: "direction_y".to_string(),
        value: ParameterValue::Float(0.0),
        param_type: ParameterType::Float,
    });

    character_fsm.add_parameter("is_attacking".to_string(), Parameter {
        name: "is_attacking".to_string(),
        value: ParameterValue::Bool(false),
        param_type: ParameterType::Bool,
    });

    character_fsm.add_parameter("jump_trigger".to_string(), Parameter {
        name: "jump_trigger".to_string(),
        value: ParameterValue::Trigger,
        param_type: ParameterType::Trigger,
    });

    // 创建动画层
    let base_layer = AnimationLayer {
        name: "Base Layer".to_string(),
        weight: 1.0,
        blending_mode: LayerBlendingMode::Override,
        state_machine: None,
        avatar_mask: None,
    };

    let upper_body_layer = AnimationLayer {
        name: "Upper Body".to_string(),
        weight: 1.0,
        blending_mode: LayerBlendingMode::Override,
        state_machine: None,
        avatar_mask: Some(AvatarMask::new("upper_body".to_string())),
    };

    character_fsm.layers = vec![base_layer, upper_body_layer];

    println!("✓ 完整角色动画系统创建成功");
    println!("  - 状态机ID: character_fsm");
    println!("  - 参数数量: {}", character_fsm.parameters.len());
    println!("  - 层数量: {}", character_fsm.layers.len());
    println!("\n动画系统架构:");
    println!("  ├── Base Layer (下半身动画)");
    println!("  │   ├── Idle (站立)");
    println!("  │   ├── Walk (行走)");
    println!("  │   ├── Run (奔跑)");
    println!("  │   └── Jump (跳跃)");
    println!("  └── Upper Body Layer (上半身动画)");
    println!("      ├── Attack (攻击)");
    println!("      ├── Block (格挡)");
    println!("      └── Wave (挥手)");
    println!();
}
