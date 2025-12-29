//  Metrics存储系统
//
//  提供统一的metrics存储和查询接口。
//  支持时间序列数据存储和聚合统计。

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// 单个metric数据点
#[derive(Debug, Clone)]
pub struct MetricDataPoint {
    /// 时间戳
    pub timestamp: Instant,
    /// 值
    pub value: f64,
    /// 标签
    pub tags: HashMap<String, String>,
}

/// Metrics聚合统计
#[derive(Debug, Clone)]
pub struct MetricAggregate {
    /// 最小值
    pub min: f64,
    /// 最大值
    pub max: f64,
    /// 平均值
    pub avg: f64,
    /// 样本数量
    pub count: usize,
}

/// Metrics存储
#[derive(Debug)]
pub struct MetricsStorage {
    /// 存储所有metric数据
    metrics: Arc<Mutex<HashMap<String, VecDeque<MetricDataPoint>>>>,
    /// 每个metric的最大保留数据点数
    max_samples: usize,
}

impl MetricsStorage {
    /// 创建新的metrics存储
    ///
    /// # 参数
    ///
    /// * `max_samples` - 每个metric保留的最大数据点数
    pub fn new(max_samples: usize) -> Self {
        Self {
            metrics: Arc::new(Mutex::new(HashMap::new())),
            max_samples,
        }
    }

    /// 记录metric
    ///
    /// # 参数
    ///
    /// * `name` - metric名称
    /// * `value` - metric值
    /// * `tags` - 可选的标签
    pub fn record(&self, name: &str, value: f64, tags: Option<HashMap<String, String>>) {
        let mut metrics = self.metrics.lock().unwrap();
        let entry = metrics.entry(name.to_string()).or_insert_with(VecDeque::new);

        let data_point = MetricDataPoint {
            timestamp: Instant::now(),
            value,
            tags: tags.unwrap_or_default(),
        };

        entry.push_back(data_point);

        // 限制数据点数量
        while entry.len() > self.max_samples {
            entry.pop_front();
        }
    }

    /// 获取metric的所有数据点
    ///
    /// # 参数
    ///
    /// * `name` - metric名称
    ///
    /// # 返回
    ///
    /// metric数据点的克隆，如果不存在则返回空
    pub fn get_metrics(&self, name: &str) -> Vec<MetricDataPoint> {
        let metrics = self.metrics.lock().unwrap();
        metrics.get(name).map(|v| v.clone().into_iter().collect()).unwrap_or_default()
    }

    /// 获取metric在指定时间范围内的数据点
    ///
    /// # 参数
    ///
    /// * `name` - metric名称
    /// * `duration` - 时间范围（从现在往前）
    ///
    /// # 返回
    ///
    /// 指定时间范围内的数据点
    pub fn get_metrics_in_window(&self, name: &str, duration: Duration) -> Vec<MetricDataPoint> {
        let metrics = self.metrics.lock().unwrap();
        let now = Instant::now();

        metrics
            .get(name)
            .map(|v| {
                v.iter()
                    .filter(|dp| now.duration_since(dp.timestamp) <= duration)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 计算metric的聚合统计
    ///
    /// # 参数
    ///
    /// * `name` - metric名称
    /// * `duration` - 可选的时间范围，None表示全部数据
    ///
    /// # 返回
    ///
    /// 聚合统计，如果metric不存在则返回None
    pub fn aggregate(&self, name: &str, duration: Option<Duration>) -> Option<MetricAggregate> {
        let data = if let Some(dur) = duration {
            self.get_metrics_in_window(name, dur)
        } else {
            self.get_metrics(name)
        };

        if data.is_empty() {
            return None;
        }

        let values: Vec<f64> = data.iter().map(|dp| dp.value).collect();
        let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let sum: f64 = values.iter().sum();
        let count = values.len();

        Some(MetricAggregate {
            min,
            max,
            avg: sum / count as f64,
            count,
        })
    }

    /// 获取所有metric名称
    ///
    /// # 返回
    ///
    /// 当前存储的所有metric名称
    pub fn get_all_metric_names(&self) -> Vec<String> {
        let metrics = self.metrics.lock().unwrap();
        metrics.keys().cloned().collect()
    }

    /// 清除指定metric的所有数据
    ///
    /// # 参数
    ///
    /// * `name` - metric名称
    pub fn clear(&self, name: &str) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.remove(name);
    }

    /// 清除所有metrics数据
    pub fn clear_all(&self) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.clear();
    }
}

impl Default for MetricsStorage {
    fn default() -> Self {
        Self::new(1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_storage_record() {
        let storage = MetricsStorage::new(10);

        storage.record("test_metric", 10.0, None);
        storage.record("test_metric", 20.0, None);
        storage.record("test_metric", 30.0, None);

        let metrics = storage.get_metrics("test_metric");
        assert_eq!(metrics.len(), 3);
        assert_eq!(metrics[0].value, 10.0);
        assert_eq!(metrics[1].value, 20.0);
        assert_eq!(metrics[2].value, 30.0);
    }

    #[test]
    fn test_metrics_storage_max_samples() {
        let storage = MetricsStorage::new(3);

        for i in 0..5 {
            storage.record("test_metric", i as f64, None);
        }

        let metrics = storage.get_metrics("test_metric");
        assert_eq!(metrics.len(), 3);
        // 应该保留最新的3个
        assert_eq!(metrics[0].value, 2.0);
        assert_eq!(metrics[1].value, 3.0);
        assert_eq!(metrics[2].value, 4.0);
    }

    #[test]
    fn test_metrics_storage_aggregate() {
        let storage = MetricsStorage::new(10);

        storage.record("test_metric", 10.0, None);
        storage.record("test_metric", 20.0, None);
        storage.record("test_metric", 30.0, None);

        let agg = storage.aggregate("test_metric", None);
        assert!(agg.is_some());
        let agg = agg.unwrap();
        assert_eq!(agg.min, 10.0);
        assert_eq!(agg.max, 30.0);
        assert_eq!(agg.avg, 20.0);
        assert_eq!(agg.count, 3);
    }

    #[test]
    fn test_metrics_storage_clear() {
        let storage = MetricsStorage::new(10);

        storage.record("test_metric", 10.0, None);
        assert_eq!(storage.get_metrics("test_metric").len(), 1);

        storage.clear("test_metric");
        assert_eq!(storage.get_metrics("test_metric").len(), 0);
    }
}
