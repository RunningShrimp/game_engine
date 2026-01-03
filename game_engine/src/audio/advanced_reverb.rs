// 高级混响系统
//
// 实现基于房间几何的实时混响，支持高阶反射和混响预设

use std::collections::HashMap;
use std::f32::consts::PI;

/// 声音速度 (m/s)
const SPEED_OF_SOUND: f32 = 343.0;

/// 混响预设
#[derive(Debug, Clone, Copy)]
pub enum ReverbPreset {
    /// 小房间（卧室）
    SmallRoom,
    /// 中型房间（客厅）
    MediumRoom,
    /// 大厅
    LargeHall,
    /// 教堂
    Cathedral,
    /// 音乐厅
    ConcertHall,
    /// 录音棚
    Studio,
    /// 森林（户外）
    Forest,
    /// 山洞
    Cave,
    /// 自定义
    Custom { rt60: f32, density: f32 },
}

impl ReverbPreset {
    /// 获取预设参数
    pub fn get_parameters(&self) -> (f32, f32, f32, f32) {
        match self {
            ReverbPreset::SmallRoom => (0.5, 0.3, 0.7, 0.5), // RT60=0.5s, 密度, 早期反射, 后期混响
            ReverbPreset::MediumRoom => (1.0, 0.5, 0.6, 0.6), // RT60=1.0s
            ReverbPreset::LargeHall => (2.5, 0.7, 0.5, 0.7), // RT60=2.5s
            ReverbPreset::Cathedral => (4.0, 0.8, 0.4, 0.8), // RT60=4.0s
            ReverbPreset::ConcertHall => (2.0, 0.7, 0.5, 0.7), // RT60=2.0s
            ReverbPreset::Studio => (0.3, 0.2, 0.8, 0.4),    // RT60=0.3s
            ReverbPreset::Forest => (1.5, 0.4, 0.3, 0.5),    // RT60=1.5s
            ReverbPreset::Cave => (3.0, 0.6, 0.3, 0.8),      // RT60=3.0s
            ReverbPreset::Custom { rt60, density } => (*rt60, *density, 0.5, 0.6),
        }
    }
}

/// 墙壁位置
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Wall {
    Left,
    Right,
    Floor,
    Ceiling,
    Front,
    Back,
}

/// 房间混响
pub struct RoomReverb {
    dimensions: (f32, f32, f32),         // 房间尺寸 (宽, 高, 深)
    reflection_order: usize,             // 反射阶数
    wall_absorption: HashMap<Wall, f32>, // 墙壁吸音系数
    preset: ReverbPreset,                // 混响预设
}

impl RoomReverb {
    /// 创建新的房间混响
    pub fn new(dimensions: (f32, f32, f32)) -> Self {
        let mut wall_absorption = HashMap::new();

        // 默认材质（混凝土）
        wall_absorption.insert(Wall::Left, 0.02);
        wall_absorption.insert(Wall::Right, 0.02);
        wall_absorption.insert(Wall::Floor, 0.05);
        wall_absorption.insert(Wall::Ceiling, 0.05);
        wall_absorption.insert(Wall::Front, 0.02);
        wall_absorption.insert(Wall::Back, 0.02);

        Self {
            dimensions,
            reflection_order: 3, // 计算3阶反射
            wall_absorption,
            preset: ReverbPreset::MediumRoom,
        }
    }

    /// 从预设创建房间混响
    pub fn from_preset(dimensions: (f32, f32, f32), preset: ReverbPreset) -> Self {
        let mut reverb = Self::new(dimensions);
        reverb.set_preset(preset);
        reverb
    }

    /// 设置混响预设
    pub fn set_preset(&mut self, preset: ReverbPreset) {
        self.preset = preset;

        // 根据预设调整墙壁吸音系数
        match preset {
            ReverbPreset::SmallRoom => {
                // 较高的吸音系数
                self.wall_absorption.insert(Wall::Floor, 0.15);
                self.wall_absorption.insert(Wall::Ceiling, 0.10);
            }
            ReverbPreset::Cathedral => {
                // 较低的吸音系数（石墙）
                self.wall_absorption.insert(Wall::Left, 0.01);
                self.wall_absorption.insert(Wall::Right, 0.01);
                self.wall_absorption.insert(Wall::Floor, 0.02);
                self.wall_absorption.insert(Wall::Ceiling, 0.02);
            }
            _ => {}
        }
    }

    /// 获取当前预设
    pub fn get_preset(&self) -> ReverbPreset {
        self.preset
    }

    /// 设置墙壁材质
    pub fn set_wall_material(&mut self, wall: Wall, absorption: f32) {
        self.wall_absorption.insert(wall, absorption.min(1.0).max(0.0));
    }

    /// 计算脉冲响应
    pub fn compute_impulse_response(
        &self,
        source_pos: (f32, f32, f32),
        listener_pos: (f32, f32, f32),
        sample_rate: f32,
    ) -> Vec<f32> {
        let ir_length = (sample_rate * 2.0) as usize; // 2秒脉冲响应
        let mut ir = vec![0.0f32; ir_length];

        // 1. 直达声
        let direct_distance = Self::distance(source_pos, listener_pos);
        let direct_delay = (direct_distance / SPEED_OF_SOUND * sample_rate) as usize;
        let direct_gain = 1.0 / (direct_distance + 1.0);

        if direct_delay < ir_length {
            ir[direct_delay] += direct_gain;
        }

        // 2. 反射声（Image Source Method）
        for order in 1..=self.reflection_order {
            for image_source in self.generate_image_sources(source_pos, order) {
                let distance = Self::distance(image_source, listener_pos);
                let delay = (distance / SPEED_OF_SOUND * sample_rate) as usize;

                if delay >= ir_length {
                    continue;
                }

                // 计算反射增益（考虑墙壁吸音）
                let mut gain = 1.0 / (distance + 1.0);

                // 简化的反射损失计算
                gain *= 0.8_f32.powi(order as i32);

                ir[delay] += gain;
            }
        }

        // 3. 后期混响（扩散尾音）
        let rt60 = self.compute_rt60();
        let decay_factor = (-2.0 / rt60).exp();

        for i in (ir_length / 2)..ir_length {
            let time = i as f32 / sample_rate;
            ir[i] *= decay_factor.powf(time);
        }

        ir
    }

    /// 计算RT60（混响时间）
    pub fn compute_rt60(&self) -> f32 {
        // Sabine公式: RT60 = 0.161 * V / A
        let volume = self.dimensions.0 * self.dimensions.1 * self.dimensions.2;

        let total_absorption: f32 = self.wall_absorption.values().cloned().sum();

        if total_absorption > 0.0 {
            0.161 * volume / total_absorption
        } else {
            2.0 // 默认2秒
        }
    }

    /// 生成镜像源（支持高阶反射）
    fn generate_image_sources(
        &self,
        source_pos: (f32, f32, f32),
        order: usize,
    ) -> Vec<(f32, f32, f32)> {
        let mut images = Vec::new();

        // 生成高阶反射
        // 使用递归方式生成镜像源
        self.generate_image_sources_recursive(source_pos, order, 0, &mut images);

        images
    }

    /// 递归生成镜像源
    fn generate_image_sources_recursive(
        &self,
        pos: (f32, f32, f32),
        max_order: usize,
        current_order: usize,
        images: &mut Vec<(f32, f32, f32)>,
    ) {
        if current_order >= max_order {
            return;
        }

        // 对每个墙面生成镜像
        let walls = [
            (Wall::Left, -1, 0, 0, 0.0), // x = 0
            (Wall::Right, 1, 0, 0, self.dimensions.0),
            (Wall::Floor, 0, -1, 0, 0.0), // y = 0
            (Wall::Ceiling, 0, 1, 0, self.dimensions.1),
            (Wall::Front, 0, 0, -1, 0.0), // z = 0
            (Wall::Back, 0, 0, 1, self.dimensions.2),
        ];

        for (wall, sign_x, sign_y, sign_z, offset) in walls {
            // 计算镜像位置
            let mirrored = match (sign_x, sign_y, sign_z) {
                (-1, 0, 0) => (-pos.0, pos.1, pos.2),
                (1, 0, 0) => (2.0 * self.dimensions.0 - pos.0, pos.1, pos.2),
                (0, -1, 0) => (pos.0, -pos.1, pos.2),
                (0, 1, 0) => (pos.0, 2.0 * self.dimensions.1 - pos.1, pos.2),
                (0, 0, -1) => (pos.0, pos.1, -pos.2),
                (0, 0, 1) => (pos.0, pos.1, 2.0 * self.dimensions.2 - pos.2),
                _ => pos,
            };

            // 添加镜像源
            images.push(mirrored);

            // 递归生成更高阶的反射
            self.generate_image_sources_recursive(mirrored, max_order, current_order + 1, images);
        }
    }

    /// 计算两点距离
    fn distance(p1: (f32, f32, f32), p2: (f32, f32, f32)) -> f32 {
        let dx = p1.0 - p2.0;
        let dy = p1.1 - p2.1;
        let dz = p1.2 - p2.2;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

/// FDN混响 (Feedback Delay Network)
pub struct FdnReverb {
    delay_lines: Vec<DelayLine>,
    feedback_matrix: [[f32; 8]; 8], // 8x8反馈矩阵
    damping: Vec<f32>,
    sample_rate: f32,
}

impl FdnReverb {
    /// 创建新的FDN混响
    pub fn new(sample_rate: f32, rt60: f32) -> Self {
        let delay_lengths = [503, 521, 541, 577, 613, 641, 673, 701]; // 质数长度
        let mut delay_lines = Vec::with_capacity(8);

        for &length in &delay_lengths {
            delay_lines.push(DelayLine::new(length));
        }

        // Hadamard反馈矩阵（正交，保证稳定性）
        let feedback_matrix = Self::hadamard_matrix();

        // 计算阻尼系数
        let damping = (-3.0 * delay_lengths[0] as f32 / (sample_rate * rt60)).exp();

        Self {
            delay_lines,
            feedback_matrix,
            damping: vec![damping; 8],
            sample_rate,
        }
    }

    /// 处理音频
    pub fn process(&mut self, input: f32) -> (f32, f32) {
        let mut outputs = [0.0f32; 8];

        // 处理每个延迟线
        for i in 0..8 {
            // 读取延迟线
            let delayed = self.delay_lines[i].read();

            // 应用反馈
            let mut feedback = 0.0;
            for j in 0..8 {
                feedback += self.feedback_matrix[i][j] * outputs[j];
            }

            // 应用阻尼
            feedback *= self.damping[i];

            // 写入延迟线
            self.delay_lines[i].write(input + feedback);

            outputs[i] = delayed;
        }

        // 输出（混合所有通道）
        let left = (outputs[0] + outputs[2] + outputs[4] + outputs[6]) * 0.25;
        let right = (outputs[1] + outputs[3] + outputs[5] + outputs[7]) * 0.25;

        (left, right)
    }

    /// 生成Hadamard矩阵
    fn hadamard_matrix() -> [[f32; 8]; 8] {
        [
            [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
            [1.0, -1.0, 1.0, -1.0, 1.0, -1.0, 1.0, -1.0],
            [1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, -1.0],
            [1.0, -1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0],
            [1.0, 1.0, 1.0, 1.0, -1.0, -1.0, -1.0, -1.0],
            [1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0],
            [1.0, 1.0, -1.0, -1.0, -1.0, -1.0, 1.0, 1.0],
            [1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0],
        ]
    }
}

/// 延迟线
struct DelayLine {
    buffer: Vec<f32>,
    write_index: usize,
    length: usize,
}

impl DelayLine {
    fn new(length: usize) -> Self {
        Self {
            buffer: vec![0.0; length],
            write_index: 0,
            length,
        }
    }

    fn write(&mut self, sample: f32) {
        self.buffer[self.write_index] = sample;
        self.write_index = (self.write_index + 1) % self.length;
    }

    fn read(&self) -> f32 {
        self.buffer[self.write_index]
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_room_reverb_creation() {
        let reverb = RoomReverb::new((10.0, 5.0, 8.0));
        assert_eq!(reverb.dimensions, (10.0, 5.0, 8.0));
    }

    #[test]
    fn test_compute_rt60() {
        let mut reverb = RoomReverb::new((10.0, 5.0, 8.0));

        // 设置一些默认吸收系数以获得合理的RT60值
        // 木材吸收系数 ~0.15，混凝土吸收系数 ~0.02
        reverb.set_wall_material(Wall::Floor, 0.15);
        reverb.set_wall_material(Wall::Ceiling, 0.02);

        let rt60 = reverb.compute_rt60();

        // RT60应该在合理范围内（0.1到10秒）
        assert!(rt60 > 0.1 && rt60 < 10.0);
    }

    #[test]
    fn test_impulse_response() {
        let reverb = RoomReverb::new((10.0, 5.0, 8.0));
        let ir = reverb.compute_impulse_response((1.0, 2.0, 3.0), (5.0, 2.5, 4.0), 44100.0);

        // 脉冲响应应该非空
        assert!(!ir.is_empty());
        assert!(ir.len() <= 44100 * 2);

        // 应该有直达声
        let has_direct_sound = ir.iter().any(|&v| v > 0.0);
        assert!(has_direct_sound);
    }

    #[test]
    fn test_fdn_reverb() {
        let mut reverb = FdnReverb::new(44100.0, 2.0);

        // 处理一些音频
        for i in 0..1000 {
            let input = (i as f32 / 1000.0 * 2.0 * PI).sin();
            let (left, right) = reverb.process(input);

            // 验证输出有效
            assert!(left.is_finite());
            assert!(right.is_finite());
        }
    }

    #[test]
    fn test_wall_material() {
        let mut reverb = RoomReverb::new((10.0, 5.0, 8.0));
        reverb.set_wall_material(Wall::Floor, 0.5);

        assert_eq!(reverb.wall_absorption[&Wall::Floor], 0.5);
    }
}
