/// 粒子系统批量处理优化

use super::{BatchConfig, BatchStats};
use std::time::Instant;

/// 粒子数据
#[derive(Debug, Clone, Copy)]
pub struct Particle {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub acceleration: [f32; 3],
    pub life: f32,
    pub size: f32,
    pub rotation: f32,
    pub color: [f32; 4],
}

impl Default for Particle {
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

/// 批量粒子处理器
pub struct BatchParticle {
    config: BatchConfig,
}

impl BatchParticle {
    pub fn new(config: BatchConfig) -> Self {
        Self { config }
    }
    
    /// 批量更新粒子
    pub fn update_particles(
        &self,
        particles: &mut [Particle],
        delta_time: f32,
    ) -> BatchStats {
        let start = Instant::now();
        let count = particles.len();
        
        for particle in particles.iter_mut() {
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
        
        BatchStats {
            elements_processed: count,
            processing_time_us: start.elapsed().as_micros() as u64,
            backend_used: Some(self.config.backend),
        }
    }
    
    /// 批量应用力场
    pub fn apply_force_field(
        &self,
        particles: &mut [Particle],
        field_position: [f32; 3],
        field_strength: f32,
        field_radius: f32,
    ) -> BatchStats {
        let start = Instant::now();
        let count = particles.len();
        
        let radius_sq = field_radius * field_radius;
        
        for particle in particles.iter_mut() {
            if particle.life <= 0.0 {
                continue;
            }
            
            // 计算到力场中心的向量
            let dx = field_position[0] - particle.position[0];
            let dy = field_position[1] - particle.position[1];
            let dz = field_position[2] - particle.position[2];
            
            let dist_sq = dx * dx + dy * dy + dz * dz;
            
            // 如果在力场范围内
            if dist_sq < radius_sq && dist_sq > 1e-6 {
                let dist = dist_sq.sqrt();
                let force = field_strength / dist_sq;
                
                // 归一化方向并应用力
                let inv_dist = 1.0 / dist;
                particle.acceleration[0] += dx * inv_dist * force;
                particle.acceleration[1] += dy * inv_dist * force;
                particle.acceleration[2] += dz * inv_dist * force;
            }
        }
        
        BatchStats {
            elements_processed: count,
            processing_time_us: start.elapsed().as_micros() as u64,
            backend_used: Some(self.config.backend),
        }
    }
    
    /// 批量应用涡流场
    pub fn apply_vortex_field(
        &self,
        particles: &mut [Particle],
        vortex_position: [f32; 3],
        vortex_axis: [f32; 3],
        vortex_strength: f32,
        vortex_radius: f32,
    ) -> BatchStats {
        let start = Instant::now();
        let count = particles.len();
        
        let radius_sq = vortex_radius * vortex_radius;
        
        // 归一化轴向量
        let axis_len = (vortex_axis[0] * vortex_axis[0] 
                      + vortex_axis[1] * vortex_axis[1] 
                      + vortex_axis[2] * vortex_axis[2]).sqrt();
        
        if axis_len < 1e-6 {
            return BatchStats {
                elements_processed: 0,
                processing_time_us: start.elapsed().as_micros() as u64,
                backend_used: Some(self.config.backend),
            };
        }
        
        let axis = [
            vortex_axis[0] / axis_len,
            vortex_axis[1] / axis_len,
            vortex_axis[2] / axis_len,
        ];
        
        for particle in particles.iter_mut() {
            if particle.life <= 0.0 {
                continue;
            }
            
            // 计算到涡流中心的向量
            let to_particle = [
                particle.position[0] - vortex_position[0],
                particle.position[1] - vortex_position[1],
                particle.position[2] - vortex_position[2],
            ];
            
            // 投影到轴上
            let proj_len = to_particle[0] * axis[0] 
                         + to_particle[1] * axis[1] 
                         + to_particle[2] * axis[2];
            
            // 计算径向向量
            let radial = [
                to_particle[0] - proj_len * axis[0],
                to_particle[1] - proj_len * axis[1],
                to_particle[2] - proj_len * axis[2],
            ];
            
            let radial_dist_sq = radial[0] * radial[0] 
                               + radial[1] * radial[1] 
                               + radial[2] * radial[2];
            
            // 如果在涡流范围内
            if radial_dist_sq < radius_sq && radial_dist_sq > 1e-6 {
                let radial_dist = radial_dist_sq.sqrt();
                
                // 计算切向力（叉积：axis × radial）
                let tangent = [
                    axis[1] * radial[2] - axis[2] * radial[1],
                    axis[2] * radial[0] - axis[0] * radial[2],
                    axis[0] * radial[1] - axis[1] * radial[0],
                ];
                
                // 力的大小随距离衰减
                let force = vortex_strength * (1.0 - radial_dist / vortex_radius);
                
                particle.acceleration[0] += tangent[0] * force;
                particle.acceleration[1] += tangent[1] * force;
                particle.acceleration[2] += tangent[2] * force;
            }
        }
        
        BatchStats {
            elements_processed: count,
            processing_time_us: start.elapsed().as_micros() as u64,
            backend_used: Some(self.config.backend),
        }
    }
    
    /// 批量碰撞检测（与平面）
    pub fn collide_with_plane(
        &self,
        particles: &mut [Particle],
        plane_normal: [f32; 3],
        plane_distance: f32,
        restitution: f32,
    ) -> BatchStats {
        let start = Instant::now();
        let count = particles.len();
        
        // 归一化平面法线
        let normal_len = (plane_normal[0] * plane_normal[0] 
                        + plane_normal[1] * plane_normal[1] 
                        + plane_normal[2] * plane_normal[2]).sqrt();
        
        if normal_len < 1e-6 {
            return BatchStats {
                elements_processed: 0,
                processing_time_us: start.elapsed().as_micros() as u64,
                backend_used: Some(self.config.backend),
            };
        }
        
        let normal = [
            plane_normal[0] / normal_len,
            plane_normal[1] / normal_len,
            plane_normal[2] / normal_len,
        ];
        
        for particle in particles.iter_mut() {
            if particle.life <= 0.0 {
                continue;
            }
            
            // 计算粒子到平面的距离
            let dist = particle.position[0] * normal[0]
                     + particle.position[1] * normal[1]
                     + particle.position[2] * normal[2]
                     - plane_distance;
            
            // 如果穿过平面
            if dist < 0.0 {
                // 将粒子移回平面上方
                particle.position[0] -= dist * normal[0];
                particle.position[1] -= dist * normal[1];
                particle.position[2] -= dist * normal[2];
                
                // 反弹速度
                let vel_dot_normal = particle.velocity[0] * normal[0]
                                   + particle.velocity[1] * normal[1]
                                   + particle.velocity[2] * normal[2];
                
                particle.velocity[0] -= (1.0 + restitution) * vel_dot_normal * normal[0];
                particle.velocity[1] -= (1.0 + restitution) * vel_dot_normal * normal[1];
                particle.velocity[2] -= (1.0 + restitution) * vel_dot_normal * normal[2];
            }
        }
        
        BatchStats {
            elements_processed: count,
            processing_time_us: start.elapsed().as_micros() as u64,
            backend_used: Some(self.config.backend),
        }
    }
    
    /// 移除死亡粒子
    pub fn remove_dead_particles(&self, particles: &mut Vec<Particle>) -> usize {
        let initial_count = particles.len();
        particles.retain(|p| p.life > 0.0);
        initial_count - particles.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_particle_update() {
        let config = BatchConfig::default();
        let processor = BatchParticle::new(config);
        
        let mut particles = vec![Particle {
            position: [0.0, 0.0, 0.0],
            velocity: [1.0, 0.0, 0.0],
            acceleration: [0.0, -9.8, 0.0],
            life: 1.0,
            ..Default::default()
        }];
        
        let stats = processor.update_particles(&mut particles, 0.1);
        assert_eq!(stats.elements_processed, 1);
        assert!(particles[0].position[0] > 0.0);
        assert!(particles[0].life < 1.0);
    }

    #[test]
    fn test_force_field() {
        let config = BatchConfig::default();
        let processor = BatchParticle::new(config);
        
        let mut particles = vec![Particle {
            position: [0.0, 0.0, 0.0],
            velocity: [0.0; 3],
            acceleration: [0.0; 3],
            life: 1.0,
            ..Default::default()
        }];
        
        let stats = processor.apply_force_field(
            &mut particles,
            [10.0, 0.0, 0.0],
            100.0,
            20.0,
        );
        
        assert_eq!(stats.elements_processed, 1);
        assert!(particles[0].acceleration[0] > 0.0);
    }
}
