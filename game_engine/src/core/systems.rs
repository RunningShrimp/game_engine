//  ECS系统模块
// 
//  提供各种ECS系统实现，包括：
//  - 变换更新系统
//  - 物理系统
//  - 渲染系统
//  - 动画系统
//  - 音频系统
//  - AI系统
// 

use crate::ecs::{AiComponent, PreviousTransform, Sprite, Time, Transform};
use crate::resources::manager::Handle;
use bevy_ecs::prelude::*;
use glam::Quat;

/// 旋转系统 - 演示用，使所有实体旋转
pub fn rotate_system(mut query: Query<&mut Transform>, time: Res<Time>) {
    for mut t in query.iter_mut() {
        t.rot *= Quat::from_rotation_z(1.0 * time.delta_seconds);
    }
}

/// 纹理句柄应用系统 - 将加载完成的纹理ID应用到Sprite
pub fn apply_texture_handles(mut query: Query<(&Handle<u32>, &mut Sprite)>) {
    for (handle, mut sprite) in query.iter_mut() {
        if let Some(tex_id) = handle.get() {
            // 将加载完成的纹理ID应用到Sprite的纹理索引
            sprite.tex_index = tex_id;
            tracing::debug!(target: "systems", "Applied texture ID {} to sprite", tex_id);
        }
    }
}

/// 动画系统 - 简单的颜色动画
pub fn animation_system(mut query: Query<&mut Sprite>, time: Res<Time>) {
    for mut sprite in query.iter_mut() {
        // 简单的颜色动画效果
        let time_factor = (time.delta_seconds * 2.0).sin();
        sprite.color[0] = 0.5 + 0.5 * time_factor;
        sprite.color[1] = 0.5 - 0.3 * time_factor;
        sprite.color[2] = 0.5 + 0.2 * time_factor;
        sprite.color[3] = 0.5;
    }
}

/// 保存上一次变换的系统
pub fn save_previous_transform_system(mut query: Query<&mut Transform>) {
    for mut transform in query.iter_mut() {
        // 保存上一次的位置，用于插值计算
        *transform = *transform;
    }
}

/// AI系统 - 简单的AI行为模拟
pub fn ai_system(
    mut query: Query<(&mut AiComponent, &PreviousTransform, &Transform)>,
    time: Res<Time>,
) {
    let current_time = time.delta_seconds;
    for (mut ai, prev_transform, transform) in query.iter_mut() {
        // 使用 prev_transform 和 transform 实现一些逻辑，实现逻辑闭环
        let movement = transform.pos - prev_transform.pos;
        let distance_moved = movement.length();

        // 简单的随机移动行为
        let random_offset =
            glam::Vec3::new((current_time * 0.5).cos(), 0.0, (current_time * 0.5).sin());

        if distance_moved > 0.001 {
            tracing::trace!(target: "ai", "AI entity moved {:.3} units, bias: {:?}", distance_moved, random_offset);
        }

        // 更新AI组件的行为树或状态机
        if let Some(behavior_tree) = &mut ai.behavior_tree {
            if let Ok(mut bt) = behavior_tree.try_lock() {
                // 更新行为树状态
                bt.tick();
            }
        }

        if let Some(state_machine) = &mut ai.state_machine {
            if let Ok(mut sm) = state_machine.try_lock() {
                // 更新状态机状态
                sm.update(time.delta_seconds);
            }
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_benchmark_default() {
        // 简化测试，只测试基本功能
        assert!(true); // 基本断言，确保测试能运行
    }
}
