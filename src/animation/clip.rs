use super::keyframe::{KeyframeTrack, InterpolationMode};
use glam::{Vec3, Quat};
use std::collections::HashMap;

/// 动画片段
#[derive(Debug, Clone)]
pub struct AnimationClip {
    /// 动画名称
    pub name: String,
    /// 持续时间 (秒)
    pub duration: f32,
    /// 位置轨道 (实体ID -> 轨道)
    pub position_tracks: HashMap<u64, KeyframeTrack<Vec3>>,
    /// 旋转轨道 (实体ID -> 轨道)
    pub rotation_tracks: HashMap<u64, KeyframeTrack<Quat>>,
    /// 缩放轨道 (实体ID -> 轨道)
    pub scale_tracks: HashMap<u64, KeyframeTrack<Vec3>>,
    /// 是否循环
    pub looping: bool,
}

impl AnimationClip {
    pub fn new(name: impl Into<String>, duration: f32) -> Self {
        Self {
            name: name.into(),
            duration,
            position_tracks: HashMap::new(),
            rotation_tracks: HashMap::new(),
            scale_tracks: HashMap::new(),
            looping: false,
        }
    }
    
    /// 添加位置轨道
    pub fn add_position_track(&mut self, entity_id: u64, track: KeyframeTrack<Vec3>) {
        self.position_tracks.insert(entity_id, track);
    }
    
    /// 添加旋转轨道
    pub fn add_rotation_track(&mut self, entity_id: u64, track: KeyframeTrack<Quat>) {
        self.rotation_tracks.insert(entity_id, track);
    }
    
    /// 添加缩放轨道
    pub fn add_scale_track(&mut self, entity_id: u64, track: KeyframeTrack<Vec3>) {
        self.scale_tracks.insert(entity_id, track);
    }
    
    /// 采样指定时间的位置
    pub fn sample_position(&self, entity_id: u64, time: f32) -> Option<Vec3> {
        self.position_tracks.get(&entity_id).and_then(|track| track.sample_vec3(time))
    }
    
    /// 采样指定时间的旋转
    pub fn sample_rotation(&self, entity_id: u64, time: f32) -> Option<Quat> {
        self.rotation_tracks.get(&entity_id).and_then(|track| track.sample_quat(time))
    }
    
    /// 采样指定时间的缩放
    pub fn sample_scale(&self, entity_id: u64, time: f32) -> Option<Vec3> {
        self.scale_tracks.get(&entity_id).and_then(|track| track.sample_vec3(time))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_animation_clip() {
        let mut clip = AnimationClip::new("test_animation", 2.0);
        
        let mut position_track = KeyframeTrack::<Vec3>::new(InterpolationMode::Linear);
        position_track.add_keyframe(0.0, Vec3::new(0.0, 0.0, 0.0));
        position_track.add_keyframe(2.0, Vec3::new(2.0, 2.0, 2.0));
        
        clip.add_position_track(1, position_track);
        
        // 测试采样
        let pos = clip.sample_position(1, 1.0).unwrap();
        assert!((pos.x - 1.0).abs() < 0.001);
        assert!((pos.y - 1.0).abs() < 0.001);
        assert!((pos.z - 1.0).abs() < 0.001);
    }
}
