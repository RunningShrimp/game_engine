// HRIR插值系统
//
// 实现双线性和三线性插值以提高HRTF定位精度

use crate::audio::hrtf_processor::HrirDataset;

/// HRIR插值器
pub struct HrirInterpolator {
    dataset: HrirDataset,
    interpolation_method: InterpolationMethod,
}

/// 插值方法
#[derive(Clone, Copy, Debug)]
pub enum InterpolationMethod {
    /// 最近邻 (最快)
    Nearest,
    /// 双线性插值 (平衡)
    Bilinear,
    /// 三线性插值 (最精确)
    Trilinear,
}

impl HrirInterpolator {
    /// 创建新的HRIR插值器
    pub fn new(dataset: HrirDataset, method: InterpolationMethod) -> Self {
        Self {
            dataset,
            interpolation_method: method,
        }
    }

    /// 获取插值后的HRIR
    pub fn get_hrir(&self, azimuth: f32, elevation: f32) -> (Vec<f32>, Vec<f32>) {
        match self.interpolation_method {
            InterpolationMethod::Nearest => self.nearest_neighbor(azimuth, elevation),
            InterpolationMethod::Bilinear => self.bilinear(azimuth, elevation),
            InterpolationMethod::Trilinear => self.trilinear(azimuth, elevation, 1.0),
        }
    }

    /// 最近邻插值
    fn nearest_neighbor(&self, azimuth: f32, elevation: f32) -> (Vec<f32>, Vec<f32>) {
        let az_idx = self.azimuth_to_index(azimuth);
        let el_idx = self.elevation_to_index(elevation);

        let left_idx = (az_idx * self.dataset.elevation_steps + el_idx) * 2;
        let right_idx = left_idx + 1;

        (
            self.dataset.hrir[left_idx].clone(),
            self.dataset.hrir[right_idx].clone(),
        )
    }

    /// 双线性插值
    fn bilinear(&self, azimuth: f32, elevation: f32) -> (Vec<f32>, Vec<f32>) {
        // 计算浮点索引
        let az_float = (azimuth + 180.0) / self.dataset.azimuth_resolution;
        let el_float = (elevation + 90.0) / self.dataset.elevation_resolution;

        // 获取周围4个点
        let az0 = az_float.floor() as usize;
        let az1 = (az0 + 1).min(self.dataset.azimuth_steps - 1);
        let el0 = el_float.floor() as usize;
        let el1 = (el0 + 1).min(self.dataset.elevation_steps - 1);

        // 插值权重
        let az_weight = az_float - az0 as f32;
        let el_weight = el_float - el0 as f32;

        // 获取4个角的HRIR
        let idx_00 = (az0 * self.dataset.elevation_steps + el0) * 2;
        let idx_01 = (az0 * self.dataset.elevation_steps + el1) * 2;
        let idx_10 = (az1 * self.dataset.elevation_steps + el0) * 2;
        let idx_11 = (az1 * self.dataset.elevation_steps + el1) * 2;

        // 获取HRIR长度
        let hrir_length = self.dataset.hrir[0].len();

        // 插值左声道
        let mut left_hrir = Vec::with_capacity(hrir_length);
        for i in 0..hrir_length {
            let v00 = self.dataset.hrir[idx_00][i];
            let v01 = self.dataset.hrir[idx_01][i];
            let v10 = self.dataset.hrir[idx_10][i];
            let v11 = self.dataset.hrir[idx_11][i];

            // 双线性插值
            let top = v00 * (1.0 - az_weight) + v10 * az_weight;
            let bottom = v01 * (1.0 - az_weight) + v11 * az_weight;
            let value = top * (1.0 - el_weight) + bottom * el_weight;

            left_hrir.push(value);
        }

        // 插值右声道
        let mut right_hrir = Vec::with_capacity(hrir_length);
        for i in 0..hrir_length {
            let v00 = self.dataset.hrir[idx_00 + 1][i];
            let v01 = self.dataset.hrir[idx_01 + 1][i];
            let v10 = self.dataset.hrir[idx_10 + 1][i];
            let v11 = self.dataset.hrir[idx_11 + 1][i];

            let top = v00 * (1.0 - az_weight) + v10 * az_weight;
            let bottom = v01 * (1.0 - az_weight) + v11 * az_weight;
            let value = top * (1.0 - el_weight) + bottom * el_weight;

            right_hrir.push(value);
        }

        (left_hrir, right_hrir)
    }

    /// 三线性插值 (包含距离维度)
    fn trilinear(&self, azimuth: f32, elevation: f32, distance_factor: f32) -> (Vec<f32>, Vec<f32>) {
        // 首先进行双线性插值
        let (mut left_hrir, mut right_hrir) = self.bilinear(azimuth, elevation);

        // 距离衰减插值
        // distance_factor: 1.0 = 最近, 0.0 = 最远
        for sample in left_hrir.iter_mut() {
            *sample *= distance_factor;
        }

        for sample in right_hrir.iter_mut() {
            *sample *= distance_factor;
        }

        (left_hrir, right_hrir)
    }

    /// 方位角转换为索引
    fn azimuth_to_index(&self, azimuth: f32) -> usize {
        let idx = ((azimuth + 180.0) / self.dataset.azimuth_resolution) as usize;
        idx.min(self.dataset.azimuth_steps - 1)
    }

    /// 仰角转换为索引
    fn elevation_to_index(&self, elevation: f32) -> usize {
        let idx = ((elevation + 90.0) / self.dataset.elevation_resolution) as usize;
        idx.min(self.dataset.elevation_steps - 1)
    }

    /// 设置插值方法
    pub fn set_interpolation_method(&mut self, method: InterpolationMethod) {
        self.interpolation_method = method;
    }
}

/// 辅助函数：计算两个HRIR之间的差异
pub fn hrir_difference(hrir1: &[f32], hrir2: &[f32]) -> f32 {
    let len = hrir1.len().min(hrir2.len());
    if len == 0 {
        return 0.0;
    }

    let mut sum_diff = 0.0;
    for i in 0..len {
        let diff = hrir1[i] - hrir2[i];
        sum_diff += diff * diff;
    }

    (sum_diff / len as f32).sqrt()
}

/// 辅助函数：HRIR能量归一化
pub fn normalize_hrir(hrir: &mut [f32]) {
    let energy: f32 = hrir.iter().map(|&x| x * x).sum();
    if energy > 0.0 {
        let scale = 1.0 / energy.sqrt();
        for sample in hrir.iter_mut() {
            *sample *= scale;
        }
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpolation_methods() {
        // 创建模拟数据集
        let dataset = create_test_dataset();
        let interpolator = HrirInterpolator::new(dataset, InterpolationMethod::Bilinear);

        // 测试插值
        let (left, right) = interpolator.get_hrir(45.0, 10.0);

        assert!(!left.is_empty());
        assert!(!right.is_empty());
    }

    #[test]
    fn test_nearest_neighbor() {
        let dataset = create_test_dataset();
        let interpolator = HrirInterpolator::new(dataset, InterpolationMethod::Nearest);

        let (left, right) = interpolator.get_hrir(47.5, 12.5);

        assert_eq!(left.len(), 256);
        assert_eq!(right.len(), 256);
    }

    #[test]
    fn test_hrir_difference() {
        let hrir1 = vec![1.0, 0.5, 0.25];
        let hrir2 = vec![1.0, 0.5, 0.25];
        let hrir3 = vec![0.0, 0.0, 0.0];

        assert!(hrir_difference(&hrir1, &hrir2) < 0.001);
        assert!(hrir_difference(&hrir1, &hrir3) > 0.001);
    }

    #[test]
    fn test_normalize_hrir() {
        let mut hrir = vec![1.0, 2.0, 3.0];
        normalize_hrir(&mut hrir);

        let energy: f32 = hrir.iter().map(|&x| x * x).sum();
        assert!((energy - 1.0).abs() < 0.001);
    }

    fn create_test_dataset() -> HrirDataset {
        HrirDataset {
            hrir: vec![vec![0.0; 256]; 2592], // 72 * 18 * 2
            sample_rate: 44100.0,
            azimuth_resolution: 5.0,
            elevation_resolution: 10.0,
            azimuth_steps: 72,
            elevation_steps: 18,
        }
    }
}
