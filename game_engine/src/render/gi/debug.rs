//! # DDGI调试可视化
//!
//! 提供DDGI系统的调试和可视化功能。

use crate::render::gi::ddgi::DDGIVolume;
use crate::render::gi::probe::DDGIProbe;
use glam::{Mat4, Vec3};
use wgpu::{Buffer, CommandEncoder, Device, Queue, RenderPass, RenderPipeline, util::DeviceExt};

/// 探针可视化模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeVisualization {
    /// 不显示
    None,
    /// 球体
    Spheres,
    /// 线条
    Lines,
    /// 热力图
    Heatmap,
    /// 辐照度颜色
    Irradiance,
    /// 深度信息
    Depth,
}

/// GI调试可视化器
pub struct GIDebugVisualizer {
    /// 是否显示探针
    show_probes: bool,
    /// 是否显示辐照度
    show_irradiance: bool,
    /// 探针可视化模式
    probe_visualization: ProbeVisualization,
    /// 探针网格渲染管线
    probe_pipeline: Option<RenderPipeline>,
    /// 探针球体网格
    sphere_mesh: Option<DebugMesh>,
    /// 探针线条网格
    line_mesh: Option<DebugMesh>,
    /// Uniform缓冲区
    uniform_buffer: Option<Buffer>,
}

/// 调试网格数据
struct DebugMesh {
    /// 顶点缓冲区
    vertex_buffer: Buffer,
    /// 索引缓冲区
    index_buffer: Buffer,
    /// 索引数量
    index_count: u32,
}

impl GIDebugVisualizer {
    /// 创建新的调试可视化器
    pub fn new() -> Self {
        Self {
            show_probes: true,
            show_irradiance: false,
            probe_visualization: ProbeVisualization::Spheres,
            probe_pipeline: None,
            sphere_mesh: None,
            line_mesh: None,
            uniform_buffer: None,
        }
    }

    /// 初始化渲染资源
    pub fn initialize(&mut self, device: &Device) {
        // 创建球体网格
        self.sphere_mesh = Some(self.create_sphere_mesh(device));
        self.line_mesh = Some(self.create_line_mesh(device));

        // 创建uniform缓冲区
        self.uniform_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GI Debug Uniform Buffer"),
            size: 64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));
    }

    /// 创建球体网格
    fn create_sphere_mesh(&self, device: &Device) -> DebugMesh {
        // 简化球体（12段）
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let segments = 12;
        let rings = 8;

        // 生成顶点
        for ring in 0..=rings {
            let theta = (ring as f32 / rings as f32) * std::f32::consts::PI;
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();

            for segment in 0..=segments {
                let phi = (segment as f32 / segments as f32) * 2.0 * std::f32::consts::PI;
                let sin_phi = phi.sin();
                let cos_phi = phi.cos();

                let x = cos_phi * sin_theta;
                let y = cos_theta;
                let z = sin_phi * sin_theta;

                vertices.push([x, y, z]);
            }
        }

        // 生成索引
        for ring in 0..rings {
            for segment in 0..segments {
                let current = ring * (segments + 1) + segment;
                let next = current + segments + 1;

                indices.push(current as u32);
                indices.push(next as u32);
                indices.push((current + 1) as u32);

                indices.push((current + 1) as u32);
                indices.push(next as u32);
                indices.push((next + 1) as u32);
            }
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sphere Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sphere Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        DebugMesh {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
        }
    }

    /// 创建线条网格
    fn create_line_mesh(&self, device: &Device) -> DebugMesh {
        let vertices = vec![
            [0.0, 0.0, 0.0], // 原点
            [1.0, 0.0, 0.0], // X轴
            [0.0, 1.0, 0.0], // Y轴
            [0.0, 0.0, 1.0], // Z轴
        ];

        let indices = vec![0, 1, 0, 2, 0, 3];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Line Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Line Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        DebugMesh {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
        }
    }

    /// 设置是否显示探针
    pub fn set_show_probes(&mut self, show: bool) {
        self.show_probes = show;
    }

    /// 设置是否显示辐照度
    pub fn set_show_irradiance(&mut self, show: bool) {
        self.show_irradiance = show;
    }

    /// 设置探针可视化模式
    pub fn set_probe_visualization(&mut self, mode: ProbeVisualization) {
        self.probe_visualization = mode;
    }

    /// 渲染调试信息
    pub fn render(&self, encoder: &mut CommandEncoder, volume: &DDGIVolume) {
        if !self.show_probes {
            return;
        }

        match self.probe_visualization {
            ProbeVisualization::None => {}
            ProbeVisualization::Spheres => {
                self.render_probe_spheres(encoder, volume);
            }
            ProbeVisualization::Lines => {
                self.render_probe_connections(encoder, volume);
            }
            ProbeVisualization::Heatmap => {
                self.render_irradiance_heatmap(encoder, volume);
            }
            ProbeVisualization::Irradiance => {
                self.render_irradiance_colors(encoder, volume);
            }
            ProbeVisualization::Depth => {
                self.render_depth_info(encoder, volume);
            }
        }
    }

    /// 渲染探针球体
    fn render_probe_spheres(&self, _encoder: &mut CommandEncoder, volume: &DDGIVolume) {
        let _sphere_mesh = match &self.sphere_mesh {
            Some(mesh) => mesh,
            None => return,
        };

        // 渲染每个探针为小球
        for idx in 0..volume.probe_count() {
            if let Some(probe) = volume.get_probe(idx) {
                let transform =
                    Mat4::from_translation(probe.position) * Mat4::from_scale(Vec3::splat(0.1));

                // 这里应该使用渲染管线绘制球体
                // 简化实现：当前只记录
                let _ = transform;
            }
        }
    }

    /// 渲染探针连接
    fn render_probe_connections(&self, _encoder: &mut CommandEncoder, volume: &DDGIVolume) {
        // 渲染探针之间的连接线
        let config = volume.config();
        let probe_counts = config.probe_counts;

        for z in 0..probe_counts.z {
            for y in 0..probe_counts.y {
                for x in 0..probe_counts.x {
                    let idx = ((z * probe_counts.y + y) * probe_counts.x + x) as usize;

                    if let Some(probe) = volume.get_probe(idx) {
                        // 绘制到X方向邻居的线
                        if x < probe_counts.x - 1 {
                            let neighbor_idx =
                                ((z * probe_counts.y + y) * probe_counts.x + x + 1) as usize;
                            if let Some(neighbor) = volume.get_probe(neighbor_idx) {
                                self.draw_line(_encoder, probe.position, neighbor.position);
                            }
                        }

                        // 绘制到Y方向邻居的线
                        if y < probe_counts.y - 1 {
                            let neighbor_idx =
                                ((z * probe_counts.y + y + 1) * probe_counts.x + x) as usize;
                            if let Some(neighbor) = volume.get_probe(neighbor_idx) {
                                self.draw_line(_encoder, probe.position, neighbor.position);
                            }
                        }

                        // 绘制到Z方向邻居的线
                        if z < probe_counts.z - 1 {
                            let neighbor_idx =
                                (((z + 1) * probe_counts.y + y) * probe_counts.x + x) as usize;
                            if let Some(neighbor) = volume.get_probe(neighbor_idx) {
                                self.draw_line(_encoder, probe.position, neighbor.position);
                            }
                        }
                    }
                }
            }
        }
    }

    /// 渲染辐照度热力图
    fn render_irradiance_heatmap(&self, _encoder: &mut CommandEncoder, volume: &DDGIVolume) {
        for idx in 0..volume.probe_count() {
            if let Some(probe) = volume.get_probe(idx) {
                // 计算辐照度强度
                let intensity = probe.irradiance.length();

                // 根据强度选择颜色（从蓝到红）
                let color = if intensity < 0.33 {
                    Vec3::new(0.0, 0.0, 1.0) // 蓝色
                } else if intensity < 0.66 {
                    Vec3::new(0.0, 1.0, 0.0) // 绿色
                } else {
                    Vec3::new(1.0, 0.0, 0.0) // 红色
                };

                // 渲染彩色球体
                let _ = color;
            }
        }
    }

    /// 渲染辐照度颜色
    fn render_irradiance_colors(&self, _encoder: &mut CommandEncoder, volume: &DDGIVolume) {
        for idx in 0..volume.probe_count() {
            if let Some(probe) = volume.get_probe(idx) {
                // 使用辐照度颜色渲染探针
                let _ = probe.irradiance;
            }
        }
    }

    /// 渲染深度信息
    fn render_depth_info(&self, _encoder: &mut CommandEncoder, volume: &DDGIVolume) {
        for idx in 0..volume.probe_count() {
            if let Some(probe) = volume.get_probe(idx) {
                // 使用深度值着色
                let depth_color = probe.depth / volume.config().max_depth;
                let _ = Vec3::splat(depth_color);
            }
        }
    }

    /// 绘制线条
    fn draw_line(&self, _encoder: &mut CommandEncoder, _start: Vec3, _end: Vec3) {
        // 简化实现：当前只记录
    }

    /// 更新uniform数据
    pub fn update_uniforms(&self, _queue: &Queue, _camera_view_proj: Mat4) {
        if let Some(buffer) = &self.uniform_buffer {
            // 更新uniform缓冲区
            let _ = buffer;
        }
    }

    /// 获取探针统计信息
    pub fn get_probe_stats(&self, volume: &DDGIVolume) -> ProbeStats {
        let mut total_irradiance = Vec3::ZERO;
        let mut min_depth = f32::MAX;
        let mut max_depth = f32::MIN;
        let mut active_probes = 0;

        let total_count = volume.probe_count();
        for idx in 0..total_count {
            if let Some(probe) = volume.get_probe(idx) {
                total_irradiance += probe.irradiance;
                min_depth = min_depth.min(probe.depth);
                max_depth = max_depth.max(probe.depth);
                active_probes += 1;
            }
        }

        let avg_irradiance = if active_probes > 0 {
            total_irradiance / active_probes as f32
        } else {
            Vec3::ZERO
        };

        ProbeStats {
            total_probes: total_count,
            active_probes,
            avg_irradiance,
            min_depth: if min_depth != f32::MAX {
                min_depth
            } else {
                0.0
            },
            max_depth: if max_depth != f32::MIN {
                max_depth
            } else {
                0.0
            },
        }
    }
}

impl Default for GIDebugVisualizer {
    fn default() -> Self {
        Self::new()
    }
}

/// 探针统计信息
#[derive(Debug, Clone)]
pub struct ProbeStats {
    pub total_probes: usize,
    pub active_probes: usize,
    pub avg_irradiance: Vec3,
    pub min_depth: f32,
    pub max_depth: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_visualizer_creation() {
        let visualizer = GIDebugVisualizer::new();
        assert!(visualizer.show_probes);
        assert!(!visualizer.show_irradiance);
    }

    #[test]
    fn test_probe_visualization_modes() {
        let modes = [
            ProbeVisualization::None,
            ProbeVisualization::Spheres,
            ProbeVisualization::Lines,
            ProbeVisualization::Heatmap,
            ProbeVisualization::Irradiance,
            ProbeVisualization::Depth,
        ];

        for mode in modes {
            let mut visualizer = GIDebugVisualizer::new();
            visualizer.set_probe_visualization(mode);
            assert_eq!(visualizer.probe_visualization, mode);
        }
    }
}
