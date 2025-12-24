//! 服务调度器
//!
//! 管理服务的调度和优先级执行。

use std::collections::{BinaryHeap, HashMap};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::{Mutex, RwLock, Notify};

use super::{ServiceId};
use super::service::ServiceError;

#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    pub tick_interval: Duration,
    pub max_concurrent_updates: usize,
    pub enable_time_budget: bool,
    pub time_budget_per_frame: Duration,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            tick_interval: Duration::from_millis(16),
            max_concurrent_updates: 32,
            enable_time_budget: true,
            time_budget_per_frame: Duration::from_millis(10),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScheduledService {
    pub service_id: ServiceId,
    pub priority: i8,
    pub last_update: Instant,
    pub update_interval: Duration,
}

impl ScheduledService {
    pub fn new(service_id: ServiceId, priority: i8, update_interval: Duration) -> Self {
        Self {
            service_id,
            priority,
            last_update: Instant::now(),
            update_interval,
        }
    }

    pub fn should_update(&self, now: Instant) -> bool {
        now.duration_since(self.last_update) >= self.update_interval
    }

    pub fn update_time_remaining(&self, now: Instant) -> Duration {
        let elapsed = now.duration_since(self.last_update);
        if elapsed >= self.update_interval {
            Duration::ZERO
        } else {
            self.update_interval - elapsed
        }
    }
}

impl Eq for ScheduledService {}

impl PartialEq for ScheduledService {
    fn eq(&self, other: &Self) -> bool {
        self.service_id == other.service_id
    }
}

impl Ord for ScheduledService {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.priority.cmp(&other.priority) {
            std::cmp::Ordering::Equal => {
                self.last_update.cmp(&other.last_update)
            }
            other => other.reverse(),
        }
    }
}

impl PartialOrd for ScheduledService {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone)]
pub struct SchedulerStats {
    pub total_updates: u64,
    pub skipped_updates: u64,
    pub average_update_time: Duration,
    pub max_update_time: Duration,
    pub active_services: usize,
    pub last_tick_time: Duration,
}

impl Default for SchedulerStats {
    fn default() -> Self {
        Self {
            total_updates: 0,
            skipped_updates: 0,
            average_update_time: Duration::ZERO,
            max_update_time: Duration::ZERO,
            active_services: 0,
            last_tick_time: Duration::ZERO,
        }
    }
}

pub struct ServiceScheduler {
    config: SchedulerConfig,
    scheduled_services: Arc<Mutex<BinaryHeap<ScheduledService>>>,
    service_map: Arc<RwLock<HashMap<ServiceId, ScheduledService>>>,
    stats: Arc<Mutex<SchedulerStats>>,
    notify: Arc<Notify>,
}

impl ServiceScheduler {
    pub fn new(config: SchedulerConfig) -> Self {
        Self {
            config,
            scheduled_services: Arc::new(Mutex::new(BinaryHeap::new())),
            service_map: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(Mutex::new(SchedulerStats::default())),
            notify: Arc::new(Notify::new()),
        }
    }

    pub async fn schedule(&self, service_id: ServiceId, priority: i8, update_interval: Duration) {
        let scheduled = ScheduledService::new(service_id.clone(), priority, update_interval);

        {
            let mut map = self.service_map.write().await;
            map.insert(service_id.clone(), scheduled.clone());
        }

        {
            let mut heap = self.scheduled_services.lock().await;
            heap.push(scheduled);
        }

        self.notify.notify_one();
    }

    pub async fn unschedule(&self, service_id: &ServiceId) {
        {
            let mut map = self.service_map.write().await;
            map.remove(service_id);
        }

        let mut heap = self.scheduled_services.lock().await;
        heap.retain(|s| &s.service_id != service_id);
    }

    pub async fn set_priority(&self, service_id: &ServiceId, priority: i8) -> Result<(), ServiceError> {
        let mut map = self.service_map.write().await;
        if let Some(scheduled) = map.get_mut(service_id) {
            scheduled.priority = priority;

            let mut heap = self.scheduled_services.lock().await;
            heap.retain(|s| &s.service_id != service_id);
            heap.push(scheduled.clone());

            Ok(())
        } else {
            Err(ServiceError::NotFound(service_id.as_str().to_string()))
        }
    }

    pub async fn set_update_interval(
        &self,
        service_id: &ServiceId,
        interval: Duration,
    ) -> Result<(), ServiceError> {
        let mut map = self.service_map.write().await;
        if let Some(scheduled) = map.get_mut(service_id) {
            scheduled.update_interval = interval;

            let mut heap = self.scheduled_services.lock().await;
            heap.retain(|s| &s.service_id != service_id);
            heap.push(scheduled.clone());

            Ok(())
        } else {
            Err(ServiceError::NotFound(service_id.as_str().to_string()))
        }
    }

    pub async fn update(&self) {
        let start = Instant::now();
        let mut updates = Vec::new();
        let now = Instant::now();

        {
            let mut heap = self.scheduled_services.lock().await;
            let map = self.service_map.read().await;

            let mut remaining_services = Vec::new();

            while let Some(scheduled) = heap.pop() {
                if scheduled.should_update(now) {
                    if let Some(s) = map.get(&scheduled.service_id) {
                        updates.push(s.service_id.clone());
                    }
                    remaining_services.push(scheduled);
                } else {
                    remaining_services.push(scheduled);
                }

                if updates.len() >= self.config.max_concurrent_updates {
                    break;
                }
            }

            for s in remaining_services {
                heap.push(s);
            }
        }

        let update_count = updates.len();
        let mut total_update_time = Duration::ZERO;
        let mut max_update_time = Duration::ZERO;

        for service_id in updates {
            {
                let mut map = self.service_map.write().await;
                if let Some(scheduled) = map.get_mut(&service_id) {
                    scheduled.last_update = now;
                }
            }

            total_update_time += self.config.tick_interval;
            max_update_time = max_update_time.max(self.config.tick_interval);
        }

        let elapsed = start.elapsed();

        {
            let mut stats = self.stats.lock().await;
            stats.total_updates += update_count as u64;
            stats.last_tick_time = elapsed;
            stats.active_services = self.service_map.read().await.len();

            if stats.total_updates > 0 {
                let new_average = (stats.average_update_time * (stats.total_updates - 1) as u32 + total_update_time)
                    / stats.total_updates as u32;
                stats.average_update_time = new_average;
            }

            stats.max_update_time = stats.max_update_time.max(max_update_time);
        }

        let remaining_time = self.config.tick_interval.saturating_sub(elapsed);
        if !remaining_time.is_zero() {
            tokio::time::sleep(remaining_time).await;
        }
    }

    pub async fn stats(&self) -> SchedulerStats {
        self.stats.lock().await.clone()
    }

    pub async fn next_update_time(&self, service_id: &ServiceId) -> Option<Duration> {
        let map = self.service_map.read().await;
        map.get(service_id)
            .map(|s| s.update_time_remaining(Instant::now()))
    }

    pub async fn is_scheduled(&self, service_id: &ServiceId) -> bool {
        let map = self.service_map.read().await;
        map.contains_key(service_id)
    }

    pub async fn scheduled_count(&self) -> usize {
        let map = self.service_map.read().await;
        map.len()
    }
}

impl Default for ServiceScheduler {
    fn default() -> Self {
        Self::new(SchedulerConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scheduler_creation() {
        let scheduler = ServiceScheduler::new(SchedulerConfig::default());
        assert_eq!(scheduler.scheduled_count().await, 0);
    }

    #[tokio::test]
    async fn test_schedule_service() {
        let scheduler = ServiceScheduler::new(SchedulerConfig::default());
        let service_id = ServiceId::new("test_service");

        scheduler.schedule(service_id.clone(), 0, Duration::from_millis(100)).await;

        assert!(scheduler.is_scheduled(&service_id).await);
        assert_eq!(scheduler.scheduled_count().await, 1);
    }

    #[tokio::test]
    async fn test_unschedule_service() {
        let scheduler = ServiceScheduler::new(SchedulerConfig::default());
        let service_id = ServiceId::new("test_service");

        scheduler.schedule(service_id.clone(), 0, Duration::from_millis(100)).await;
        scheduler.unschedule(&service_id).await;

        assert!(!scheduler.is_scheduled(&service_id).await);
    }
}
