//! 值对象模块
//!
//! 封装领域概念为值对象，提高领域模型质量。
//!
//! ## 设计原则
//!
//! - 值对象是不可变的
//! - 值对象通过值相等性进行比较
//! - 值对象包含验证逻辑
//! - 值对象封装领域概念

use crate::impl_default;
use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};
use std::fmt;

/// 位置值对象
///
/// 封装3D位置信息，包含验证逻辑。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position {
    x: f32,
    y: f32,
    z: f32,
}

impl Position {
    /// 创建新位置
    ///
    /// # 参数
    /// - `x`, `y`, `z`: 坐标值
    ///
    /// # 返回
    /// 如果坐标有效则返回`Some(Position)`，否则返回`None`
    pub fn new(x: f32, y: f32, z: f32) -> Option<Self> {
        // 验证：位置不能包含NaN或无穷大
        if x.is_finite() && y.is_finite() && z.is_finite() {
            Some(Self { x, y, z })
        } else {
            None
        }
    }

    /// 创建位置（不验证）
    ///
    /// # 警告
    /// 仅在确定坐标有效时使用
    pub fn new_unchecked(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// 从Vec3创建
    pub fn from_vec3(vec: Vec3) -> Option<Self> {
        Self::new(vec.x, vec.y, vec.z)
    }

    /// 转换为Vec3
    pub fn to_vec3(self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }

    /// 获取X坐标
    pub fn x(self) -> f32 {
        self.x
    }

    /// 获取Y坐标
    pub fn y(self) -> f32 {
        self.y
    }

    /// 获取Z坐标
    pub fn z(self) -> f32 {
        self.z
    }

    /// 计算到另一个位置的距离
    pub fn distance_to(self, other: Position) -> f32 {
        self.to_vec3().distance(other.to_vec3())
    }

    /// 计算到另一个位置的平方距离（性能优化）
    pub fn distance_squared_to(self, other: Position) -> f32 {
        self.to_vec3().distance_squared(other.to_vec3())
    }

    /// 偏移位置
    pub fn offset(self, delta: Vec3) -> Option<Self> {
        Self::new(self.x + delta.x, self.y + delta.y, self.z + delta.z)
    }
}

impl_default!(Position {
    x: 0.0,
    y: 0.0,
    z: 0.0,
});

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Position({}, {}, {})", self.x, self.y, self.z)
    }
}

/// 旋转值对象
///
/// 封装四元数旋转信息。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rotation {
    quaternion: Quat,
}

impl Rotation {
    /// 创建新旋转（从四元数）
    pub fn from_quat(quat: Quat) -> Self {
        Self {
            quaternion: quat.normalize(), // 确保归一化
        }
    }

    /// 创建单位旋转
    pub fn identity() -> Self {
        Self {
            quaternion: Quat::IDENTITY,
        }
    }

    /// 从欧拉角创建（弧度）
    pub fn from_euler(x: f32, y: f32, z: f32) -> Self {
        Self {
            quaternion: Quat::from_euler(glam::EulerRot::XYZ, x, y, z),
        }
    }

    /// 转换为四元数
    pub fn to_quat(self) -> Quat {
        self.quaternion
    }

    /// 组合旋转（先应用self，再应用other）
    pub fn combine(self, other: Rotation) -> Self {
        Self {
            quaternion: (self.quaternion * other.quaternion).normalize(),
        }
    }

    /// 旋转向量
    pub fn rotate_vec3(self, vec: Vec3) -> Vec3 {
        self.quaternion * vec
    }

    /// 获取逆旋转
    pub fn inverse(self) -> Self {
        Self {
            quaternion: self.quaternion.inverse(),
        }
    }

    /// 球面线性插值
    pub fn slerp(self, other: Rotation, t: f32) -> Self {
        Self {
            quaternion: self.quaternion.slerp(other.quaternion, t),
        }
    }
}

impl Default for Rotation {
    fn default() -> Self {
        Self::identity()
    }
}

impl fmt::Display for Rotation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (x, y, z, w) = self.quaternion.into();
        write!(f, "Rotation({}, {}, {}, {})", x, y, z, w)
    }
}

/// 缩放值对象
///
/// 封装3D缩放信息，包含验证逻辑。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Scale {
    x: f32,
    y: f32,
    z: f32,
}

impl Scale {
    /// 创建新缩放
    ///
    /// # 参数
    /// - `x`, `y`, `z`: 缩放值（必须为正数）
    ///
    /// # 返回
    /// 如果缩放值有效则返回`Some(Scale)`，否则返回`None`
    pub fn new(x: f32, y: f32, z: f32) -> Option<Self> {
        // 验证：缩放值必须为正数且有限
        if x > 0.0 && y > 0.0 && z > 0.0 && x.is_finite() && y.is_finite() && z.is_finite() {
            Some(Self { x, y, z })
        } else {
            None
        }
    }

    /// 创建统一缩放
    pub fn uniform(value: f32) -> Option<Self> {
        Self::new(value, value, value)
    }

    /// 创建缩放（不验证）
    ///
    /// # 警告
    /// 仅在确定缩放值有效时使用
    pub fn new_unchecked(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// 从Vec3创建
    pub fn from_vec3(vec: Vec3) -> Option<Self> {
        Self::new(vec.x, vec.y, vec.z)
    }

    /// 转换为Vec3
    pub fn to_vec3(self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }

    /// 获取X缩放
    pub fn x(self) -> f32 {
        self.x
    }

    /// 获取Y缩放
    pub fn y(self) -> f32 {
        self.y
    }

    /// 获取Z缩放
    pub fn z(self) -> f32 {
        self.z
    }

    /// 组合缩放（相乘）
    pub fn combine(self, other: Scale) -> Self {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
        }
    }
}

impl_default!(Scale {
    x: 1.0,
    y: 1.0,
    z: 1.0,
});

impl fmt::Display for Scale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Scale({}, {}, {})", self.x, self.y, self.z)
    }
}

/// 变换值对象
///
/// 封装位置、旋转、缩放的组合。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    position: Position,
    rotation: Rotation,
    scale: Scale,
}

impl Transform {
    /// 创建新变换
    pub fn new(position: Position, rotation: Rotation, scale: Scale) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    /// 创建单位变换
    pub fn identity() -> Self {
        Self {
            position: Position::default(),
            rotation: Rotation::identity(),
            scale: Scale::default(),
        }
    }

    /// 获取位置
    pub fn position(self) -> Position {
        self.position
    }

    /// 获取旋转
    pub fn rotation(self) -> Rotation {
        self.rotation
    }

    /// 获取缩放
    pub fn scale(self) -> Scale {
        self.scale
    }

    /// 设置位置
    pub fn with_position(self, position: Position) -> Self {
        Self {
            position,
            rotation: self.rotation,
            scale: self.scale,
        }
    }

    /// 设置旋转
    pub fn with_rotation(self, rotation: Rotation) -> Self {
        Self {
            position: self.position,
            rotation,
            scale: self.scale,
        }
    }

    /// 设置缩放
    pub fn with_scale(self, scale: Scale) -> Self {
        Self {
            position: self.position,
            rotation: self.rotation,
            scale,
        }
    }

    /// 组合变换（先应用self，再应用other）
    pub fn combine(self, other: Transform) -> Self {
        // 先缩放，再旋转，最后平移
        let scaled_pos = other.scale.to_vec3() * self.position.to_vec3();
        let rotated_pos = other.rotation.rotate_vec3(scaled_pos);
        let final_pos =
            Position::from_vec3(rotated_pos + other.position.to_vec3()).unwrap_or(self.position);

        Self {
            position: final_pos,
            rotation: self.rotation.combine(other.rotation),
            scale: self.scale.combine(other.scale),
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

impl fmt::Display for Transform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Transform({}, {}, {})",
            self.position, self.rotation, self.scale
        )
    }
}

/// 音量值对象
///
/// 封装音频音量信息，范围0.0-1.0。
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Volume {
    value: f32,
}

impl Volume {
    /// 创建新音量
    ///
    /// # 参数
    /// - `value`: 音量值（0.0-1.0）
    ///
    /// # 返回
    /// 如果音量值有效则返回`Some(Volume)`，否则返回`None`
    pub fn new(value: f32) -> Option<Self> {
        // 验证：音量必须在0.0-1.0范围内且有限
        if value >= 0.0 && value <= 1.0 && value.is_finite() {
            Some(Self { value })
        } else {
            None
        }
    }

    /// 创建音量（不验证）
    ///
    /// # 警告
    /// 仅在确定音量值有效时使用
    pub fn new_unchecked(value: f32) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
        }
    }

    /// 获取音量值
    pub fn value(self) -> f32 {
        self.value
    }

    /// 静音
    pub fn muted() -> Self {
        Self { value: 0.0 }
    }

    /// 最大音量
    pub fn max() -> Self {
        Self { value: 1.0 }
    }

    /// 是否静音
    pub fn is_muted(self) -> bool {
        self.value == 0.0
    }

    /// 线性插值
    pub fn lerp(self, other: Volume, t: f32) -> Self {
        Self {
            value: self.value + (other.value - self.value) * t.clamp(0.0, 1.0),
        }
    }
}

impl_default!(Volume { value: 1.0 });

impl fmt::Display for Volume {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Volume({:.2})", self.value)
    }
}

/// 质量值对象
///
/// 封装物理质量信息，必须为正数。
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Mass {
    value: f32,
}

impl Mass {
    /// 创建新质量
    ///
    /// # 参数
    /// - `value`: 质量值（必须为正数）
    ///
    /// # 返回
    /// 如果质量值有效则返回`Some(Mass)`，否则返回`None`
    pub fn new(value: f32) -> Option<Self> {
        // 验证：质量必须为正数且有限
        if value > 0.0 && value.is_finite() {
            Some(Self { value })
        } else {
            None
        }
    }

    /// 创建质量（不验证）
    ///
    /// # 警告
    /// 仅在确定质量值有效时使用
    pub fn new_unchecked(value: f32) -> Self {
        Self {
            value: value.max(0.0),
        }
    }

    /// 获取质量值
    pub fn value(self) -> f32 {
        self.value
    }

    /// 零质量（用于静态物体）
    pub fn zero() -> Self {
        Self { value: 0.0 }
    }

    /// 是否为零质量
    pub fn is_zero(self) -> bool {
        self.value == 0.0
    }
}

impl_default!(Mass { value: 1.0 });

impl fmt::Display for Mass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Mass({:.2})", self.value)
    }
}

/// 速度值对象
///
/// 封装3D速度信息，包含验证逻辑。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Velocity {
    x: f32,
    y: f32,
    z: f32,
}

impl Velocity {
    /// 创建新速度
    ///
    /// # 参数
    /// - `x`, `y`, `z`: 速度分量
    ///
    /// # 返回
    /// 如果速度值有效则返回`Some(Velocity)`，否则返回`None`
    pub fn new(x: f32, y: f32, z: f32) -> Option<Self> {
        // 验证：速度不能包含NaN或无穷大
        if x.is_finite() && y.is_finite() && z.is_finite() {
            Some(Self { x, y, z })
        } else {
            None
        }
    }

    /// 创建速度（不验证）
    ///
    /// # 警告
    /// 仅在确定速度值有效时使用
    pub fn new_unchecked(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// 从Vec3创建
    pub fn from_vec3(vec: Vec3) -> Option<Self> {
        Self::new(vec.x, vec.y, vec.z)
    }

    /// 转换为Vec3
    pub fn to_vec3(self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }

    /// 获取X速度
    pub fn x(self) -> f32 {
        self.x
    }

    /// 获取Y速度
    pub fn y(self) -> f32 {
        self.y
    }

    /// 获取Z速度
    pub fn z(self) -> f32 {
        self.z
    }

    /// 获取速度大小
    pub fn magnitude(self) -> f32 {
        self.to_vec3().length()
    }

    /// 获取速度平方大小（性能优化）
    pub fn magnitude_squared(self) -> f32 {
        self.to_vec3().length_squared()
    }

    /// 归一化速度
    pub fn normalized(self) -> Option<Self> {
        let vec = self.to_vec3();
        let len = vec.length();
        if len > 0.0 {
            Self::from_vec3(vec / len)
        } else {
            None
        }
    }

    /// 零速度
    pub fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    /// 是否为零速度
    pub fn is_zero(self) -> bool {
        self.x == 0.0 && self.y == 0.0 && self.z == 0.0
    }
}

impl Default for Velocity {
    fn default() -> Self {
        Self::zero()
    }
}

impl fmt::Display for Velocity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Velocity({}, {}, {})", self.x, self.y, self.z)
    }
}

/// 时长值对象
///
/// 封装时间长度信息（秒），必须为非负数。
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Duration {
    seconds: f32,
}

impl Duration {
    /// 创建新时长
    ///
    /// # 参数
    /// - `seconds`: 秒数（必须为非负数）
    ///
    /// # 返回
    /// 如果时长值有效则返回`Some(Duration)`，否则返回`None`
    pub fn new(seconds: f32) -> Option<Self> {
        // 验证：时长必须为非负数且有限
        if seconds >= 0.0 && seconds.is_finite() {
            Some(Self { seconds })
        } else {
            None
        }
    }

    /// 创建时长（不验证）
    ///
    /// # 警告
    /// 仅在确定时长值有效时使用
    pub fn new_unchecked(seconds: f32) -> Self {
        Self {
            seconds: seconds.max(0.0),
        }
    }

    /// 从秒数创建
    pub fn from_seconds(seconds: f32) -> Option<Self> {
        Self::new(seconds)
    }

    /// 从毫秒创建
    pub fn from_millis(millis: f32) -> Option<Self> {
        Self::new(millis / 1000.0)
    }

    /// 获取秒数
    pub fn seconds(self) -> f32 {
        self.seconds
    }

    /// 获取毫秒数
    pub fn millis(self) -> f32 {
        self.seconds * 1000.0
    }

    /// 零时长
    pub fn zero() -> Self {
        Self { seconds: 0.0 }
    }

    /// 是否为零时长
    pub fn is_zero(self) -> bool {
        self.seconds == 0.0
    }
}

impl Default for Duration {
    fn default() -> Self {
        Self::zero()
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Duration({:.2}s)", self.seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_creation() {
        let pos = Position::new(1.0, 2.0, 3.0).unwrap();
        assert_eq!(pos.x(), 1.0);
        assert_eq!(pos.y(), 2.0);
        assert_eq!(pos.z(), 3.0);
    }

    #[test]
    fn test_position_validation() {
        assert!(Position::new(f32::NAN, 0.0, 0.0).is_none());
        assert!(Position::new(0.0, f32::INFINITY, 0.0).is_none());
    }

    #[test]
    fn test_position_distance() {
        let pos1 = Position::new(0.0, 0.0, 0.0).unwrap();
        let pos2 = Position::new(3.0, 4.0, 0.0).unwrap();
        assert_eq!(pos1.distance_to(pos2), 5.0);
    }

    #[test]
    fn test_rotation_creation() {
        let rot = Rotation::identity();
        assert_eq!(rot.to_quat(), Quat::IDENTITY);
    }

    #[test]
    fn test_rotation_combine() {
        let rot1 = Rotation::identity();
        let rot2 = Rotation::from_euler(0.0, 0.0, 1.0);
        let combined = rot1.combine(rot2);
        assert_ne!(combined.to_quat(), Quat::IDENTITY);
    }

    #[test]
    fn test_scale_creation() {
        let scale = Scale::new(2.0, 3.0, 4.0).unwrap();
        assert_eq!(scale.x(), 2.0);
        assert_eq!(scale.y(), 3.0);
        assert_eq!(scale.z(), 4.0);
    }

    #[test]
    fn test_scale_validation() {
        assert!(Scale::new(-1.0, 1.0, 1.0).is_none());
        assert!(Scale::new(0.0, 1.0, 1.0).is_none());
        assert!(Scale::new(f32::NAN, 1.0, 1.0).is_none());
    }

    #[test]
    fn test_transform_creation() {
        let pos = Position::new(1.0, 2.0, 3.0).unwrap();
        let rot = Rotation::identity();
        let scale = Scale::uniform(2.0).unwrap();
        let transform = Transform::new(pos, rot, scale);
        assert_eq!(transform.position(), pos);
    }

    #[test]
    fn test_volume_creation() {
        let volume = Volume::new(0.5).unwrap();
        assert_eq!(volume.value(), 0.5);
    }

    #[test]
    fn test_volume_validation() {
        assert!(Volume::new(-0.1).is_none());
        assert!(Volume::new(1.1).is_none());
        assert!(Volume::new(f32::NAN).is_none());
    }

    #[test]
    fn test_mass_creation() {
        let mass = Mass::new(10.0).unwrap();
        assert_eq!(mass.value(), 10.0);
    }

    #[test]
    fn test_mass_validation() {
        assert!(Mass::new(-1.0).is_none());
        assert!(Mass::new(0.0).is_none());
        assert!(Mass::new(f32::NAN).is_none());
    }

    #[test]
    fn test_velocity_creation() {
        let vel = Velocity::new(1.0, 2.0, 3.0).unwrap();
        assert_eq!(vel.x(), 1.0);
        assert_eq!(vel.y(), 2.0);
        assert_eq!(vel.z(), 3.0);
    }

    #[test]
    fn test_velocity_magnitude() {
        let vel = Velocity::new(3.0, 4.0, 0.0).unwrap();
        assert_eq!(vel.magnitude(), 5.0);
    }

    #[test]
    fn test_duration_creation() {
        let duration = Duration::new(5.0).unwrap();
        assert_eq!(duration.seconds(), 5.0);
        assert_eq!(duration.millis(), 5000.0);
    }

    #[test]
    fn test_duration_validation() {
        assert!(Duration::new(-1.0).is_none());
        assert!(Duration::new(f32::NAN).is_none());
    }

    #[test]
    fn test_position_offset() {
        let pos = Position::new(1.0, 2.0, 3.0).unwrap();
        let offset = pos.offset(Vec3::new(1.0, 1.0, 1.0));
        assert!(offset.is_some());
        let new_pos = offset.unwrap();
        assert_eq!(new_pos.x(), 2.0);
        assert_eq!(new_pos.y(), 3.0);
        assert_eq!(new_pos.z(), 4.0);
    }

    #[test]
    fn test_position_distance_squared() {
        let pos1 = Position::new(0.0, 0.0, 0.0).unwrap();
        let pos2 = Position::new(3.0, 4.0, 0.0).unwrap();
        assert_eq!(pos1.distance_squared_to(pos2), 25.0);
    }

    #[test]
    fn test_position_from_vec3() {
        let vec = Vec3::new(1.0, 2.0, 3.0);
        let pos = Position::from_vec3(vec);
        assert!(pos.is_some());
        assert_eq!(pos.unwrap().to_vec3(), vec);
    }

    #[test]
    fn test_rotation_inverse() {
        let rot = Rotation::from_euler(1.0, 0.0, 0.0);
        let inv = rot.inverse();
        let combined = rot.combine(inv);
        // 旋转和逆旋转的组合应该接近单位旋转
        let quat = combined.to_quat();
        assert!((quat.length() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_rotation_slerp() {
        let rot1 = Rotation::identity();
        let rot2 = Rotation::from_euler(0.0, 1.0, 0.0);
        let slerped = rot1.slerp(rot2, 0.5);
        // 插值结果应该是有效的旋转
        assert!((slerped.to_quat().length() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_rotation_rotate_vec3() {
        let rot = Rotation::from_euler(0.0, std::f32::consts::PI / 2.0, 0.0); // 绕Y轴旋转90度
        let vec = Vec3::new(1.0, 0.0, 0.0);
        let rotated = rot.rotate_vec3(vec);
        // 旋转后的向量应该接近(0, 0, -1)
        assert!(rotated.x.abs() < 0.001);
        assert!(rotated.y.abs() < 0.001);
        assert!(rotated.z < -0.9);
    }

    #[test]
    fn test_scale_combine() {
        let scale1 = Scale::new(2.0, 3.0, 4.0).unwrap();
        let scale2 = Scale::new(1.0, 2.0, 0.5).unwrap();
        let combined = scale1.combine(scale2);
        assert_eq!(combined.x(), 2.0);
        assert_eq!(combined.y(), 6.0);
        assert_eq!(combined.z(), 2.0);
    }

    #[test]
    fn test_scale_uniform() {
        let scale = Scale::uniform(2.0).unwrap();
        assert_eq!(scale.x(), 2.0);
        assert_eq!(scale.y(), 2.0);
        assert_eq!(scale.z(), 2.0);
    }

    #[test]
    fn test_scale_from_vec3() {
        let vec = Vec3::new(2.0, 3.0, 4.0);
        let scale = Scale::from_vec3(vec);
        assert!(scale.is_some());
        assert_eq!(scale.unwrap().to_vec3(), vec);
    }

    #[test]
    fn test_transform_with_position() {
        let transform = Transform::identity();
        let pos = Position::new(1.0, 2.0, 3.0).unwrap();
        let new_transform = transform.with_position(pos);
        assert_eq!(new_transform.position(), pos);
    }

    #[test]
    fn test_transform_with_rotation() {
        let transform = Transform::identity();
        let rot = Rotation::from_euler(1.0, 0.0, 0.0);
        let new_transform = transform.with_rotation(rot);
        assert_eq!(new_transform.rotation().to_quat(), rot.to_quat());
    }

    #[test]
    fn test_transform_with_scale() {
        let transform = Transform::identity();
        let scale = Scale::new(2.0, 3.0, 4.0).unwrap();
        let new_transform = transform.with_scale(scale);
        assert_eq!(new_transform.scale(), scale);
    }

    #[test]
    fn test_transform_combine() {
        let transform1 = Transform::identity();
        let pos = Position::new(1.0, 0.0, 0.0).unwrap();
        let transform2 = Transform::identity().with_position(pos);
        let combined = transform1.combine(transform2);
        // 组合变换的位置应该接近(1, 0, 0)
        let combined_pos = combined.position().to_vec3();
        assert!((combined_pos.x - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_volume_muted() {
        let muted = Volume::muted();
        assert_eq!(muted.value(), 0.0);
        assert!(muted.is_muted());
    }

    #[test]
    fn test_volume_max() {
        let max = Volume::max();
        assert_eq!(max.value(), 1.0);
        assert!(!max.is_muted());
    }

    #[test]
    fn test_volume_lerp() {
        let vol1 = Volume::muted();
        let vol2 = Volume::max();
        let lerped = vol1.lerp(vol2, 0.5);
        assert_eq!(lerped.value(), 0.5);
    }

    #[test]
    fn test_mass_zero() {
        let zero = Mass::zero();
        assert_eq!(zero.value(), 0.0);
        assert!(zero.is_zero());
    }

    #[test]
    fn test_mass_is_zero() {
        let mass = Mass::new(0.1).unwrap();
        assert!(!mass.is_zero());
        
        let zero = Mass::zero();
        assert!(zero.is_zero());
    }

    #[test]
    fn test_velocity_magnitude_squared() {
        let vel = Velocity::new(3.0, 4.0, 0.0).unwrap();
        assert_eq!(vel.magnitude_squared(), 25.0);
    }

    #[test]
    fn test_velocity_normalized() {
        let vel = Velocity::new(3.0, 4.0, 0.0).unwrap();
        let normalized = vel.normalized();
        assert!(normalized.is_some());
        let norm = normalized.unwrap();
        assert!((norm.magnitude() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_velocity_zero() {
        let zero = Velocity::zero();
        assert_eq!(zero.x(), 0.0);
        assert_eq!(zero.y(), 0.0);
        assert_eq!(zero.z(), 0.0);
        assert_eq!(zero.magnitude(), 0.0);
    }

    #[test]
    fn test_velocity_from_vec3() {
        let vec = Vec3::new(1.0, 2.0, 3.0);
        let vel = Velocity::from_vec3(vec);
        assert!(vel.is_some());
        assert_eq!(vel.unwrap().to_vec3(), vec);
    }

    #[test]
    fn test_duration_from_millis() {
        let duration = Duration::from_millis(5000.0).unwrap();
        assert_eq!(duration.millis(), 5000.0);
        assert_eq!(duration.seconds(), 5.0);
    }

    #[test]
    fn test_duration_from_seconds() {
        let duration = Duration::from_seconds(5.0).unwrap();
        assert_eq!(duration.seconds(), 5.0);
        assert_eq!(duration.millis(), 5000.0);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // 值对象属性测试策略
    fn finite_f32() -> impl Strategy<Value = f32> {
        (-1000.0f32..1000.0).prop_filter("must be finite", |&x| x.is_finite())
    }

    fn positive_finite_f32() -> impl Strategy<Value = f32> {
        (0.0001f32..1000.0)
            .prop_filter("must be finite and positive", |&x| x.is_finite() && x > 0.0)
    }

    fn volume_f32() -> impl Strategy<Value = f32> {
        (0.0f32..=1.0).prop_filter("must be finite", |&x| x.is_finite())
    }

    fn duration_f32() -> impl Strategy<Value = f32> {
        (0.0f32..10000.0).prop_filter("must be finite and non-negative", |&x| {
            x.is_finite() && x >= 0.0
        })
    }

    // Position属性测试
    proptest! {
        #[test]
        fn position_always_valid_when_finite(
            x in finite_f32(),
            y in finite_f32(),
            z in finite_f32()
        ) {
            let pos = Position::new(x, y, z);
            prop_assert!(pos.is_some());
            let pos = pos.unwrap();
            prop_assert_eq!(pos.x(), x);
            prop_assert_eq!(pos.y(), y);
            prop_assert_eq!(pos.z(), z);
        }

        #[test]
        fn position_distance_symmetric(
            x1 in finite_f32(),
            y1 in finite_f32(),
            z1 in finite_f32(),
            x2 in finite_f32(),
            y2 in finite_f32(),
            z2 in finite_f32()
        ) {
            if let (Some(pos1), Some(pos2)) = (Position::new(x1, y1, z1), Position::new(x2, y2, z2)) {
                let dist1 = pos1.distance_to(pos2);
                let dist2 = pos2.distance_to(pos1);
                prop_assert!((dist1 - dist2).abs() < 0.0001);
            }
        }

        #[test]
        fn position_distance_triangle_inequality(
            x1 in finite_f32(),
            y1 in finite_f32(),
            z1 in finite_f32(),
            x2 in finite_f32(),
            y2 in finite_f32(),
            z2 in finite_f32(),
            x3 in finite_f32(),
            y3 in finite_f32(),
            z3 in finite_f32()
        ) {
            if let (Some(pos1), Some(pos2), Some(pos3)) = (
                Position::new(x1, y1, z1),
                Position::new(x2, y2, z2),
                Position::new(x3, y3, z3)
            ) {
                let dist12 = pos1.distance_to(pos2);
                let dist23 = pos2.distance_to(pos3);
                let dist13 = pos1.distance_to(pos3);
                // 三角不等式：dist13 <= dist12 + dist23
                prop_assert!(dist13 <= dist12 + dist23 + 0.0001);
            }
        }

        #[test]
        fn position_offset_preserves_validity(
            x in finite_f32(),
            y in finite_f32(),
            z in finite_f32(),
            dx in finite_f32(),
            dy in finite_f32(),
            dz in finite_f32()
        ) {
            if let Some(pos) = Position::new(x, y, z) {
                let delta = Vec3::new(dx, dy, dz);
                let offset = pos.offset(delta);
                if (x + dx).is_finite() && (y + dy).is_finite() && (z + dz).is_finite() {
                    prop_assert!(offset.is_some());
                }
            }
        }
    }

    // Rotation属性测试
    proptest! {
        #[test]
        fn rotation_combine_associative(
            x1 in -3.14f32..3.14,
            y1 in -3.14f32..3.14,
            z1 in -3.14f32..3.14,
            x2 in -3.14f32..3.14,
            y2 in -3.14f32..3.14,
            z2 in -3.14f32..3.14,
            x3 in -3.14f32..3.14,
            y3 in -3.14f32..3.14,
            z3 in -3.14f32..3.14
        ) {
            let rot1 = Rotation::from_euler(x1, y1, z1);
            let rot2 = Rotation::from_euler(x2, y2, z2);
            let rot3 = Rotation::from_euler(x3, y3, z3);

            // 测试结合律：(rot1 * rot2) * rot3 ≈ rot1 * (rot2 * rot3)
            let left = rot1.combine(rot2).combine(rot3);
            let right = rot1.combine(rot2.combine(rot3));

            // 四元数乘法满足结合律（允许小的浮点误差）
            let q1 = left.to_quat();
            let q2 = right.to_quat();
            let diff = (q1.x - q2.x).abs() + (q1.y - q2.y).abs() + (q1.z - q2.z).abs() + (q1.w - q2.w).abs();
            prop_assert!(diff < 0.01);
        }

        #[test]
        fn rotation_inverse_cancels(
            x in -3.14f32..3.14,
            y in -3.14f32..3.14,
            z in -3.14f32..3.14
        ) {
            let rot = Rotation::from_euler(x, y, z);
            let inv = rot.inverse();
            let combined = rot.combine(inv);
            let identity = Rotation::identity();

            // rot * rot^-1 ≈ identity
            let q1 = combined.to_quat();
            let q2 = identity.to_quat();
            let diff = (q1.x - q2.x).abs() + (q1.y - q2.y).abs() + (q1.z - q2.z).abs() + (q1.w - q2.w).abs();
            prop_assert!(diff < 0.01);
        }

        #[test]
        fn rotation_always_normalized(
            x in -3.14f32..3.14,
            y in -3.14f32..3.14,
            z in -3.14f32..3.14
        ) {
            let rot = Rotation::from_euler(x, y, z);
            let quat = rot.to_quat();
            let length = (quat.x * quat.x + quat.y * quat.y + quat.z * quat.z + quat.w * quat.w).sqrt();
            prop_assert!((length - 1.0).abs() < 0.0001);
        }
    }

    // Scale属性测试
    proptest! {
        #[test]
        fn scale_always_positive_when_valid(
            x in positive_finite_f32(),
            y in positive_finite_f32(),
            z in positive_finite_f32()
        ) {
            let scale = Scale::new(x, y, z);
            prop_assert!(scale.is_some());
            let scale = scale.unwrap();
            prop_assert!(scale.x() > 0.0);
            prop_assert!(scale.y() > 0.0);
            prop_assert!(scale.z() > 0.0);
        }

        #[test]
        fn scale_combine_commutative(
            x1 in positive_finite_f32(),
            y1 in positive_finite_f32(),
            z1 in positive_finite_f32(),
            x2 in positive_finite_f32(),
            y2 in positive_finite_f32(),
            z2 in positive_finite_f32()
        ) {
            if let (Some(scale1), Some(scale2)) = (
                Scale::new(x1, y1, z1),
                Scale::new(x2, y2, z2)
            ) {
                // 缩放组合满足交换律：scale1 * scale2 = scale2 * scale1
                let combined1 = scale1.combine(scale2);
                let combined2 = scale2.combine(scale1);
                prop_assert_eq!(combined1.x(), combined2.x());
                prop_assert_eq!(combined1.y(), combined2.y());
                prop_assert_eq!(combined1.z(), combined2.z());
            }
        }
    }

    // Volume属性测试
    proptest! {
        #[test]
        fn volume_always_in_range_when_valid(
            value in volume_f32()
        ) {
            let volume = Volume::new(value);
            prop_assert!(volume.is_some());
            let volume = volume.unwrap();
            prop_assert!(volume.value() >= 0.0);
            prop_assert!(volume.value() <= 1.0);
        }

        #[test]
        fn volume_lerp_bounded(
            v1 in volume_f32(),
            v2 in volume_f32(),
            t in 0.0f32..=1.0
        ) {
            if let (Some(vol1), Some(vol2)) = (Volume::new(v1), Volume::new(v2)) {
                let lerped = vol1.lerp(vol2, t);
                prop_assert!(lerped.value() >= 0.0);
                prop_assert!(lerped.value() <= 1.0);
            }
        }
    }

    // Mass属性测试
    proptest! {
        #[test]
        fn mass_always_positive_when_valid(
            value in positive_finite_f32()
        ) {
            let mass = Mass::new(value);
            prop_assert!(mass.is_some());
            let mass = mass.unwrap();
            prop_assert!(mass.value() > 0.0);
        }
    }

    // Velocity属性测试
    proptest! {
        #[test]
        fn velocity_always_valid_when_finite(
            x in finite_f32(),
            y in finite_f32(),
            z in finite_f32()
        ) {
            let vel = Velocity::new(x, y, z);
            prop_assert!(vel.is_some());
            let vel = vel.unwrap();
            prop_assert_eq!(vel.x(), x);
            prop_assert_eq!(vel.y(), y);
            prop_assert_eq!(vel.z(), z);
        }

        #[test]
        fn velocity_magnitude_non_negative(
            x in finite_f32(),
            y in finite_f32(),
            z in finite_f32()
        ) {
            if let Some(vel) = Velocity::new(x, y, z) {
                prop_assert!(vel.magnitude() >= 0.0);
                prop_assert!(vel.magnitude_squared() >= 0.0);
            }
        }

        #[test]
        fn velocity_normalized_has_unit_length(
            x in finite_f32(),
            y in finite_f32(),
            z in finite_f32()
        ) {
            if let Some(vel) = Velocity::new(x, y, z) {
                if vel.magnitude() > 0.0001 {
                    if let Some(normalized) = vel.normalized() {
                        let mag = normalized.magnitude();
                        prop_assert!((mag - 1.0).abs() < 0.0001);
                    }
                }
            }
        }
    }

    // Duration属性测试
    proptest! {
        #[test]
        fn duration_always_non_negative_when_valid(
            seconds in duration_f32()
        ) {
            let duration = Duration::new(seconds);
            prop_assert!(duration.is_some());
            let duration = duration.unwrap();
            prop_assert!(duration.seconds() >= 0.0);
            prop_assert!(duration.millis() >= 0.0);
        }

        #[test]
        fn duration_conversion_consistent(
            seconds in duration_f32()
        ) {
            if let Some(duration) = Duration::new(seconds) {
                let millis = duration.millis();
                let back_to_seconds = millis / 1000.0;
                prop_assert!((back_to_seconds - seconds).abs() < 0.001);
            }
        }
    }

    // Transform属性测试
    proptest! {
        #[test]
        fn transform_identity_preserves(
            x in finite_f32(),
            y in finite_f32(),
            z in finite_f32()
        ) {
            if let Some(pos) = Position::new(x, y, z) {
                let rot = Rotation::identity();
                let scale = Scale::default();
                let transform = Transform::new(pos, rot, scale);

                let transformed_pos = transform.position();
                prop_assert!((transformed_pos.x() - pos.x()).abs() < 0.0001);
                prop_assert!((transformed_pos.y() - pos.y()).abs() < 0.0001);
                prop_assert!((transformed_pos.z() - pos.z()).abs() < 0.0001);
            }
        }
    }
}
