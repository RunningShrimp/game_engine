use glam::{Vec3, Quat};

/// 插值模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterpolationMode {
    /// 线性插值
    Linear,
    /// 阶梯插值 (无插值)
    Step,
    /// 三次贝塞尔插值
    CubicBezier,
}

/// 关键帧
#[derive(Debug, Clone)]
pub struct Keyframe<T> {
    /// 时间 (秒)
    pub time: f32,
    /// 值
    pub value: T,
}

/// 关键帧轨道
#[derive(Debug, Clone)]
pub struct KeyframeTrack<T> {
    /// 关键帧列表
    pub keyframes: Vec<Keyframe<T>>,
    /// 插值模式
    pub interpolation: InterpolationMode,
}

impl<T> KeyframeTrack<T>
where
    T: Clone,
{
    pub fn new(interpolation: InterpolationMode) -> Self {
        Self {
            keyframes: Vec::new(),
            interpolation,
        }
    }
    
    /// 添加关键帧
    pub fn add_keyframe(&mut self, time: f32, value: T) {
        let keyframe = Keyframe { time, value };
        
        // 按时间排序插入
        let index = self.keyframes.binary_search_by(|k| k.time.partial_cmp(&time).unwrap()).unwrap_or_else(|i| i);
        self.keyframes.insert(index, keyframe);
    }
    
    /// 获取指定时间的值
    pub fn sample(&self, time: f32) -> Option<T> {
        if self.keyframes.is_empty() {
            return None;
        }
        
        // 如果时间在第一个关键帧之前
        if time <= self.keyframes[0].time {
            return Some(self.keyframes[0].value.clone());
        }
        
        // 如果时间在最后一个关键帧之后
        if time >= self.keyframes.last().unwrap().time {
            return Some(self.keyframes.last().unwrap().value.clone());
        }
        
        // 查找相邻的两个关键帧
        for i in 0..self.keyframes.len() - 1 {
            let k0 = &self.keyframes[i];
            let k1 = &self.keyframes[i + 1];
            
            if time >= k0.time && time <= k1.time {
                match self.interpolation {
                    InterpolationMode::Step => {
                        return Some(k0.value.clone());
                    }
                    _ => {
                        // 对于不支持插值的类型,返回第一个关键帧的值
                        return Some(k0.value.clone());
                    }
                }
            }
        }
        
        None
    }
}

/// Vec3关键帧轨道的特化实现,支持线性插值
impl KeyframeTrack<Vec3> {
    pub fn sample_vec3(&self, time: f32) -> Option<Vec3> {
        if self.keyframes.is_empty() {
            return None;
        }
        
        if time <= self.keyframes[0].time {
            return Some(self.keyframes[0].value);
        }
        
        if time >= self.keyframes.last().unwrap().time {
            return Some(self.keyframes.last().unwrap().value);
        }
        
        for i in 0..self.keyframes.len() - 1 {
            let k0 = &self.keyframes[i];
            let k1 = &self.keyframes[i + 1];
            
            if time >= k0.time && time <= k1.time {
                match self.interpolation {
                    InterpolationMode::Step => {
                        return Some(k0.value);
                    }
                    InterpolationMode::Linear => {
                        let t = (time - k0.time) / (k1.time - k0.time);
                        return Some(k0.value.lerp(k1.value, t));
                    }
                    InterpolationMode::CubicBezier => {
                        // 简化版本,使用线性插值
                        let t = (time - k0.time) / (k1.time - k0.time);
                        return Some(k0.value.lerp(k1.value, t));
                    }
                }
            }
        }
        
        None
    }
}

/// Quat关键帧轨道的特化实现,支持球面线性插值
impl KeyframeTrack<Quat> {
    pub fn sample_quat(&self, time: f32) -> Option<Quat> {
        if self.keyframes.is_empty() {
            return None;
        }
        
        if time <= self.keyframes[0].time {
            return Some(self.keyframes[0].value);
        }
        
        if time >= self.keyframes.last().unwrap().time {
            return Some(self.keyframes.last().unwrap().value);
        }
        
        for i in 0..self.keyframes.len() - 1 {
            let k0 = &self.keyframes[i];
            let k1 = &self.keyframes[i + 1];
            
            if time >= k0.time && time <= k1.time {
                match self.interpolation {
                    InterpolationMode::Step => {
                        return Some(k0.value);
                    }
                    InterpolationMode::Linear => {
                        let t = (time - k0.time) / (k1.time - k0.time);
                        return Some(k0.value.slerp(k1.value, t));
                    }
                    InterpolationMode::CubicBezier => {
                        // 简化版本,使用球面线性插值
                        let t = (time - k0.time) / (k1.time - k0.time);
                        return Some(k0.value.slerp(k1.value, t));
                    }
                }
            }
        }
        
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_keyframe_track() {
        let mut track = KeyframeTrack::<Vec3>::new(InterpolationMode::Linear);
        
        track.add_keyframe(0.0, Vec3::new(0.0, 0.0, 0.0));
        track.add_keyframe(1.0, Vec3::new(1.0, 1.0, 1.0));
        
        // 测试插值
        let value = track.sample_vec3(0.5).unwrap();
        assert!((value.x - 0.5).abs() < 0.001);
        assert!((value.y - 0.5).abs() < 0.001);
        assert!((value.z - 0.5).abs() < 0.001);
    }
}
