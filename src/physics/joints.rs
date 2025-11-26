use bevy_ecs::prelude::*;
use rapier3d::prelude::*;
use rapier3d::na::Unit;
use glam::Vec3;

/// 关节类型
#[derive(Component, Clone)]
pub enum JointDesc {
    /// 固定关节
    Fixed {
        entity_a: Entity,
        entity_b: Entity,
        anchor_a: Vec3,
        anchor_b: Vec3,
    },
    /// 铰链关节
    Revolute {
        entity_a: Entity,
        entity_b: Entity,
        anchor_a: Vec3,
        anchor_b: Vec3,
        axis: Vec3,
        limits: Option<(f32, f32)>,
    },
    /// 棱柱关节
    Prismatic {
        entity_a: Entity,
        entity_b: Entity,
        anchor_a: Vec3,
        anchor_b: Vec3,
        axis: Vec3,
        limits: Option<(f32, f32)>,
    },
    /// 球形关节
    Spherical {
        entity_a: Entity,
        entity_b: Entity,
        anchor_a: Vec3,
        anchor_b: Vec3,
    },
}

impl JointDesc {
    /// 转换为Rapier关节
    pub fn to_rapier_joint(&self) -> GenericJoint {
        match self {
            JointDesc::Fixed { anchor_a, anchor_b, .. } => {
                GenericJointBuilder::new(JointAxesMask::LOCKED_FIXED_AXES)
                    .local_anchor1(point![anchor_a.x, anchor_a.y, anchor_a.z])
                    .local_anchor2(point![anchor_b.x, anchor_b.y, anchor_b.z])
                    .build()
            }
            JointDesc::Revolute { anchor_a, anchor_b, axis, limits, .. } => {
                let mut joint = GenericJointBuilder::new(JointAxesMask::LOCKED_REVOLUTE_AXES)
                    .local_anchor1(point![anchor_a.x, anchor_a.y, anchor_a.z])
                    .local_anchor2(point![anchor_b.x, anchor_b.y, anchor_b.z])
                    .local_axis1(Unit::new_normalize(vector![axis.x, axis.y, axis.z]))
                    .local_axis2(Unit::new_normalize(vector![axis.x, axis.y, axis.z]));
                
                if let Some((min, max)) = limits {
                    joint = joint.limits(JointAxis::AngX, [*min, *max]);
                }
                
                joint.build()
            }
            JointDesc::Prismatic { anchor_a, anchor_b, axis, limits, .. } => {
                let mut joint = GenericJointBuilder::new(JointAxesMask::LOCKED_PRISMATIC_AXES)
                    .local_anchor1(point![anchor_a.x, anchor_a.y, anchor_a.z])
                    .local_anchor2(point![anchor_b.x, anchor_b.y, anchor_b.z])
                    .local_axis1(Unit::new_normalize(vector![axis.x, axis.y, axis.z]))
                    .local_axis2(Unit::new_normalize(vector![axis.x, axis.y, axis.z]));
                
                if let Some((min, max)) = limits {
                    joint = joint.limits(JointAxis::X, [*min, *max]);
                }
                
                joint.build()
            }
            JointDesc::Spherical { anchor_a, anchor_b, .. } => {
                GenericJointBuilder::new(JointAxesMask::LOCKED_SPHERICAL_AXES)
                    .local_anchor1(point![anchor_a.x, anchor_a.y, anchor_a.z])
                    .local_anchor2(point![anchor_b.x, anchor_b.y, anchor_b.z])
                    .build()
            }
        }
    }
}

/// 关节组件
#[derive(Component)]
pub struct Joint {
    pub handle: ImpulseJointHandle,
}

/// 初始化关节系统
pub fn init_joints_system(
    mut commands: Commands,
    mut physics: ResMut<super::physics3d::PhysicsWorld3D>,
    joint_query: Query<(Entity, &JointDesc), Without<Joint>>,
    rb_query: Query<&super::physics3d::RigidBody3D>,
) {
    for (entity, joint_desc) in joint_query.iter() {
        // 获取两个刚体的句柄
        let (entity_a, entity_b) = match joint_desc {
            JointDesc::Fixed { entity_a, entity_b, .. } => (entity_a, entity_b),
            JointDesc::Revolute { entity_a, entity_b, .. } => (entity_a, entity_b),
            JointDesc::Prismatic { entity_a, entity_b, .. } => (entity_a, entity_b),
            JointDesc::Spherical { entity_a, entity_b, .. } => (entity_a, entity_b),
        };
        
        if let (Ok(rb_a), Ok(rb_b)) = (rb_query.get(*entity_a), rb_query.get(*entity_b)) {
            let joint = joint_desc.to_rapier_joint();
            let handle = physics.impulse_joint_set.insert(
                rb_a.handle,
                rb_b.handle,
                joint,
                true,
            );
            
            commands.entity(entity).insert(Joint { handle });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_joint_desc() {
        let joint_desc = JointDesc::Fixed {
            entity_a: Entity::from_raw(0),
            entity_b: Entity::from_raw(1),
            anchor_a: Vec3::ZERO,
            anchor_b: Vec3::new(1.0, 0.0, 0.0),
        };
        
        let _joint = joint_desc.to_rapier_joint();
        // 验证关节创建成功
    }
}
