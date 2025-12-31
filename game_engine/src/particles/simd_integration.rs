/// 粒子系统SIMD批量处理集成
///
/// 集成game_engine_simd的粒子批量处理功能。
use bevy_ecs::prelude::*;

// 条件性导入SIMD支持
use game_engine_simd::batch::BatchConfig;
#[cfg(feature = "simd")]
use game_engine_simd::batch::particle::{BatchParticle, Particle};

/// 粒子组件
#[derive(Component, Debug, Clone)]
pub struct SimdParticle {
    /// 粒子位置
    pub position: [f32; 3],
    /// 粒子速度
    pub velocity: [f32; 3],
    /// 粒子加速度
    pub acceleration: [f32; 3],
    /// 粒子生命值
    pub life: f32,
    /// 粒子大小
    pub size: f32,
    /// 粒子旋转角度
    pub rotation: f32,
    /// 粒子颜色 [r, g, b, a]
    pub color: [f32; 4],
}

impl Default for SimdParticle {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            velocity: [0.0; 3],
            acceleration: [0.0; 3],
            life: 1.0,
            size: 1.0,
            rotation: 0.0,
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

impl From<Particle> for SimdParticle {
    fn from(p: Particle) -> Self {
        Self {
            position: p.position,
            velocity: p.velocity,
            acceleration: p.acceleration,
            life: p.life,
            size: p.size,
            rotation: p.rotation,
            color: p.color,
        }
    }
}

impl From<SimdParticle> for Particle {
    fn from(p: SimdParticle) -> Self {
        Self {
            position: p.position,
            velocity: p.velocity,
            acceleration: p.acceleration,
            life: p.life,
            size: p.size,
            rotation: p.rotation,
            color: p.color,
        }
    }
}

/// SIMD粒子批量处理器资源
#[derive(Resource)]
pub struct SimdParticleProcessor {
    /// 批量处理器
    #[cfg(feature = "simd")]
    processor: BatchParticle,
    /// 配置
    config: BatchConfig,
}

impl std::fmt::Debug for SimdParticleProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimdParticleProcessor").field("config", &self.config).finish()
    }
}

impl Default for SimdParticleProcessor {
    fn default() -> Self {
        let config = BatchConfig::default();
        #[cfg(feature = "simd")]
        let processor = BatchParticle::new(config.clone());

        Self {
            #[cfg(feature = "simd")]
            processor,
            config,
        }
    }
}

/// SIMD粒子批量更新系统
///
/// 使用SIMD加速批量更新粒子状态
pub fn simd_particle_update_system(
    mut query: Query<&mut SimdParticle>,
    mut processor: ResMut<SimdParticleProcessor>,
) {
    // 收集粒子数据
    let mut particles: Vec<Particle> = query
        .iter_mut()
        .filter(|p| p.life > 0.0)
        .map(|p| Particle::from(p.clone()))
        .collect();

    if particles.is_empty() {
        return;
    }

    let delta_time = 0.016; // 固定时间步

    // 使用SIMD批量更新粒子
    #[cfg(feature = "simd")]
    {
        let stats = processor.processor.update_particles(&mut particles, delta_time);
        tracing::trace!(target: "particles", "SIMD updated {} particles in {}μs", stats.elements_processed, stats.processing_time_us);
    }

    #[cfg(not(feature = "simd"))]
    {
        // 标量回退实现
        for particle in &mut particles {
            if particle.life <= 0.0 {
                continue;
            }

            // 更新速度
            particle.velocity[0] += particle.acceleration[0] * delta_time;
            particle.velocity[1] += particle.acceleration[1] * delta_time;
            particle.velocity[2] += particle.acceleration[2] * delta_time;

            // 更新位置
            particle.position[0] += particle.velocity[0] * delta_time;
            particle.position[1] += particle.velocity[1] * delta_time;
            particle.position[2] += particle.velocity[2] * delta_time;

            // 更新生命值
            particle.life -= delta_time;
        }
    }

    // 将结果写回ECS组件
    for (mut particle_component, particle_data) in query.iter_mut().zip(particles.iter()) {
        particle_component.position = particle_data.position;
        particle_component.velocity = particle_data.velocity;
        particle_component.acceleration = particle_data.acceleration;
        particle_component.life = particle_data.life;
        particle_component.size = particle_data.size;
        particle_component.rotation = particle_data.rotation;
        particle_component.color = particle_data.color;
    }
}

/// SIMD粒子力场应用系统
///
/// 使用SIMD加速批量应用力场效果
pub fn simd_particle_force_field_system(
    mut query: Query<&mut SimdParticle>,
    mut processor: ResMut<SimdParticleProcessor>,
) {
    let mut particles: Vec<Particle> = query
        .iter_mut()
        .filter(|p| p.life > 0.0)
        .map(|p| Particle::from(p.clone()))
        .collect();

    if particles.is_empty() {
        return;
    }

    // 应用示例力场（中心点，强度，半径）
    #[cfg(feature = "simd")]
    {
        let _stats =
            processor
                .processor
                .apply_force_field(&mut particles, [0.0, 0.0, 0.0], 100.0, 20.0);
    }

    #[cfg(not(feature = "simd"))]
    {
        // 标量回退：简单的中心引力
        let field_position = [0.0, 0.0, 0.0];
        let field_strength = 100.0;
        let field_radius_sq = 20.0 * 20.0;

        for particle in &mut particles {
            if particle.life <= 0.0 {
                continue;
            }

            let dx = field_position[0] - particle.position[0];
            let dy = field_position[1] - particle.position[1];
            let dz = field_position[2] - particle.position[2];

            let dist_sq = dx * dx + dy * dy + dz * dz;

            if dist_sq < field_radius_sq && dist_sq > 1e-6 {
                let dist = dist_sq.sqrt();
                let force = field_strength / dist_sq;
                let inv_dist = 1.0 / dist;

                particle.acceleration[0] += dx * inv_dist * force;
                particle.acceleration[1] += dy * inv_dist * force;
                particle.acceleration[2] += dz * inv_dist * force;
            }
        }
    }

    // 将结果写回ECS组件
    for (mut particle_component, particle_data) in query.iter_mut().zip(particles.iter()) {
        particle_component.acceleration = particle_data.acceleration;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simd_particle_conversion() {
        let particle = Particle::default();
        let simd_particle = SimdParticle::from(particle.clone());
        let converted_back = Particle::from(simd_particle);

        assert_eq!(particle.position, converted_back.position);
        assert_eq!(particle.life, converted_back.life);
    }

    #[test]
    fn test_simd_particle_processor_default() {
        let processor = SimdParticleProcessor::default();
        // 配置应该有默认值
        assert!(processor.config.batch_size > 0);
    }
}
