//! Velocity组件包装器
//!
//! 为glam::Vec3创建Component包装，解决ECS trait bound问题

use bevy_ecs::prelude::*;
use glam::Vec3;

/// 速度组件
#[derive(Component, Debug, Clone, PartialEq)]
pub struct Velocity(pub Vec3);

impl From<Vec3> for Velocity {
    fn from(v: Vec3) -> Self {
        Self(v)
    }
}

impl From<Velocity> for Vec3 {
    fn from(v: Velocity) -> Self {
        v.0
    }
}

impl AsRef<Vec3> for Velocity {
    fn as_ref(&self) -> &Vec3 {
        &self.0
    }
}

impl AsMut<Vec3> for Velocity {
    fn as_mut(&mut self) -> &mut Vec3 {
        &mut self.0
    }
}

/// 位置组件
#[derive(Component, Debug, Clone, PartialEq)]
pub struct Position(pub Vec3);

impl From<Vec3> for Position {
    fn from(v: Vec3) -> Self {
        Self(v)
    }
}

impl From<Position> for Vec3 {
    fn from(p: Position) -> Self {
        p.0
    }
}

impl AsRef<Vec3> for Position {
    fn as_ref(&self) -> &Vec3 {
        &self.0
    }
}

impl AsMut<Vec3> for Position {
    fn as_mut(&mut self) -> &mut Vec3 {
        &mut self.0
    }
}

/// 逆质量组件（1/mass）
#[derive(Component, Debug, Clone, PartialEq)]
pub struct InverseMass(pub f32);

impl From<f32> for InverseMass {
    fn from(m: f32) -> Self {
        Self(m)
    }
}

impl From<InverseMass> for f32 {
    fn from(im: InverseMass) -> Self {
        im.0
    }
}

/// 全局变换组件包装器
#[derive(Component, Debug, Clone, PartialEq)]
pub struct GlobalTransform(pub glam::Mat4);

impl From<glam::Mat4> for GlobalTransform {
    fn from(m: glam::Mat4) -> Self {
        Self(m)
    }
}

impl From<GlobalTransform> for glam::Mat4 {
    fn from(t: GlobalTransform) -> Self {
        t.0
    }
}

impl AsRef<glam::Mat4> for GlobalTransform {
    fn as_ref(&self) -> &glam::Mat4 {
        &self.0
    }
}

impl AsMut<glam::Mat4> for GlobalTransform {
    fn as_mut(&mut self) -> &mut glam::Mat4 {
        &mut self.0
    }
}
