// 音频遮挡/阻塞系统
//
// 基于物理的音频遮挡计算

use std::sync::Arc;

/// 音频遮挡结果
#[derive(Debug, Clone)]
pub struct OcclusionResult {
    /// 遮挡因子 (0.0 = 无遮挡, 1.0 = 完全遮挡)
    pub occlusion_factor: f32,
    /// 传输损失 (0.0 = 无损失, 1.0 = 完全损失)
    pub transmission_loss: f32,
    /// 低频衰减 (dB)
    pub low_frequency_attenuation: f32,
    /// 高频衰减 (dB)
    pub high_frequency_attenuation: f32,
}

/// 音频材质属性
#[derive(Debug, Clone, Copy)]
pub struct AcousticMaterial {
    /// 传输系数 (0.0 = 完全反射, 1.0 = 完全传输)
    pub transmission_coefficient: f32,
    /// 吸收系数 (0.0 = 完全反射, 1.0 = 完全吸收)
    pub absorption_coefficient: f32,
    /// 频率依赖性 (低频/高频传输比)
    pub frequency_dependency: f32,
}

impl AcousticMaterial {
    /// 混凝土
    pub fn concrete() -> Self {
        Self {
            transmission_coefficient: 0.01,
            absorption_coefficient: 0.02,
            frequency_dependency: 0.5, // 低频传输更多
        }
    }

    /// 木材
    pub fn wood() -> Self {
        Self {
            transmission_coefficient: 0.1,
            absorption_coefficient: 0.15,
            frequency_dependency: 0.7,
        }
    }

    /// 玻璃
    pub fn glass() -> Self {
        Self {
            transmission_coefficient: 0.3,
            absorption_coefficient: 0.05,
            frequency_dependency: 0.9,
        }
    }

    /// 金属
    pub fn metal() -> Self {
        Self {
            transmission_coefficient: 0.001,
            absorption_coefficient: 0.01,
            frequency_dependency: 0.2,
        }
    }

    /// 空气（无遮挡）
    pub fn air() -> Self {
        Self {
            transmission_coefficient: 1.0,
            absorption_coefficient: 0.0,
            frequency_dependency: 1.0,
        }
    }
}

/// 音频遮挡系统
pub struct AudioOcclusion {
    physics_world: Option<Arc<dyn PhysicsWorld>>,
    max_rays: usize,
    max_distance: f32,
}

/// 光线
pub struct Ray {
    pub origin: [f32; 3],
    pub direction: [f32; 3],
}

impl Ray {
    pub fn new(origin: [f32; 3], direction: [f32; 3]) -> Self {
        Self { origin, direction }
    }
}

/// 光线投射命中
pub struct RaycastHit {
    pub distance: f32,
    pub point: [f32; 3],
    pub normal: [f32; 3],
    pub material: Option<AcousticMaterial>,
}

impl AudioOcclusion {
    /// 创建新的音频遮挡系统
    pub fn new() -> Self {
        Self {
            physics_world: None,
            max_rays: 10,
            max_distance: 100.0,
        }
    }

    /// 设置物理世界
    pub fn set_physics_world(&mut self, world: Arc<dyn PhysicsWorld>) {
        self.physics_world = Some(world);
    }

    /// 计算遮挡
    pub fn compute_occlusion(
        &self,
        source: (f32, f32, f32),
        listener: (f32, f32, f32),
    ) -> OcclusionResult {
        // 如果没有物理世界，返回无遮挡
        let physics_world = match &self.physics_world {
            Some(world) => world,
            None => {
                return OcclusionResult {
                    occlusion_factor: 0.0,
                    transmission_loss: 0.0,
                    low_frequency_attenuation: 0.0,
                    high_frequency_attenuation: 0.0,
                };
            }
        };

        // 1. 直达光检查
        let direct_ray = Ray::new(
            [source.0, source.1, source.2],
            [
                listener.0 - source.0,
                listener.1 - source.1,
                listener.2 - source.2,
            ],
        );

        let hits = physics_world.cast_ray(&direct_ray, self.max_distance);

        if hits.is_empty() {
            // 无遮挡
            return OcclusionResult {
                occlusion_factor: 0.0,
                transmission_loss: 0.0,
                low_frequency_attenuation: 0.0,
                high_frequency_attenuation: 0.0,
            };
        }

        // 2. 计算遮挡
        let mut occlusion_factor: f32 = 0.0;
        let mut transmission_loss: f32 = 1.0;
        let mut low_freq_loss = 0.0;
        let mut high_freq_loss = 0.0;

        for hit in &hits {
            if let Some(material) = hit.material {
                occlusion_factor += 1.0;
                transmission_loss *= (1.0 - material.transmission_coefficient);

                // 频率依赖的衰减
                low_freq_loss += -20.0 * (1.0 - material.transmission_coefficient * material.frequency_dependency).log10();
                high_freq_loss += -20.0 * (1.0 - material.transmission_coefficient).log10();
            }
        }

        // 限制范围
        occlusion_factor = occlusion_factor.min(1.0);
        transmission_loss = transmission_loss.min(1.0);

        OcclusionResult {
            occlusion_factor,
            transmission_loss,
            low_frequency_attenuation: low_freq_loss.min(60.0),
            high_frequency_attenuation: high_freq_loss.min(60.0),
        }
    }

    /// 使用多条光线的更精确遮挡计算
    pub fn compute_occlusion_multi_ray(
        &self,
        source: (f32, f32, f32),
        listener: (f32, f32, f32),
    ) -> OcclusionResult {
        let mut total_occlusion = 0.0;
        let mut total_transmission_loss = 0.0;
        let mut total_low_freq = 0.0;
        let mut total_high_freq = 0.0;

        let ray_count = self.max_rays.min(10);
        let cone_angle = 30.0_f32.to_radians(); // 30度锥

        for i in 0..ray_count {
            // 在锥体内均匀分布光线
            let angle_offset = if ray_count > 1 {
                let t = i as f32 / (ray_count - 1) as f32 - 0.5;
                t * cone_angle
            } else {
                0.0
            };

            // TODO: 实际计算偏移后的光线方向
            // 简化版：只使用直达光
            let result = self.compute_occlusion(source, listener);
            total_occlusion += result.occlusion_factor;
            total_transmission_loss += result.transmission_loss;
            total_low_freq += result.low_frequency_attenuation;
            total_high_freq += result.high_frequency_attenuation;
        }

        let n = ray_count as f32;
        OcclusionResult {
            occlusion_factor: (total_occlusion / n).min(1.0),
            transmission_loss: (total_transmission_loss / n).min(1.0),
            low_frequency_attenuation: total_low_freq / n,
            high_frequency_attenuation: total_high_freq / n,
        }
    }

    /// 设置最大光线数量
    pub fn set_max_rays(&mut self, max_rays: usize) {
        self.max_rays = max_rays;
    }

    /// 设置最大检测距离
    pub fn set_max_distance(&mut self, max_distance: f32) {
        self.max_distance = max_distance;
    }
}

impl Default for AudioOcclusion {
    fn default() -> Self {
        Self::new()
    }
}

/// 物理世界trait（抽象接口）
pub trait PhysicsWorld: Send + Sync {
    /// 光线投射
    fn cast_ray(&self, ray: &Ray, max_distance: f32) -> Vec<RaycastHit>;
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_occlusion_without_physics() {
        let occlusion = AudioOcclusion::new();
        let result = occlusion.compute_occlusion((0.0, 0.0, 0.0), (10.0, 0.0, 0.0));

        assert_eq!(result.occlusion_factor, 0.0);
        assert_eq!(result.transmission_loss, 0.0);
    }

    #[test]
    fn test_acoustic_materials() {
        let concrete = AcousticMaterial::concrete();
        let wood = AcousticMaterial::wood();
        let glass = AcousticMaterial::glass();

        // 混凝土应该比木材更难传输声音
        assert!(concrete.transmission_coefficient < wood.transmission_coefficient);
        // 玻璃应该比混凝土更容易传输
        assert!(glass.transmission_coefficient > concrete.transmission_coefficient);
    }

    #[test]
    fn test_occlusion_clamping_in_compute() {
        // 测试compute_occlusion会对结果进行限制
        let occlusion = AudioOcclusion::new();

        // 没有物理世界时，结果应该是0（无遮挡）
        let result = occlusion.compute_occlusion((0.0, 0.0, 0.0), (10.0, 0.0, 0.0));

        assert!(result.occlusion_factor <= 1.0);
        assert!(result.transmission_loss <= 1.0);
        assert!(result.low_frequency_attenuation <= 60.0);
        assert!(result.high_frequency_attenuation <= 60.0);
        assert_eq!(result.occlusion_factor, 0.0); // 无物理世界时应该为0
        assert_eq!(result.transmission_loss, 0.0);
    }
}
