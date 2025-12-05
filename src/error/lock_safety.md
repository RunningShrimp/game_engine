# 锁安全性修改记录

## 修改目标
将代码中所有使用 `lock().unwrap()` 的地方替换为更安全的错误处理方式，避免panic风险。

## 修改策略
使用 `crate::error::safe_lock()` 函数替代 `lock().unwrap()`，该函数会：
1. 处理锁污染情况
2. 提供有意义的错误信息
3. 允许恢复已污染的锁

## 修改范围
src/profiling/service.rs

## 修改内容

### 1. acknowledge_alert 函数 (line 332)
- 原代码: `let mut alerting_engine = self.alerting_engine.lock().unwrap();`
- 新代码: 
  let mut alerting_engine = crate::error::safe_lock(&self.alerting_engine, "ProfilingService.alerting_engine")
      .map_err(|e| crate::profiling::ProfilingError::ConfigurationError(e.to_string()))?;

### 2. export_data 函数 (line 344)
- 原代码: `let storage = self.storage.lock().unwrap();`
- 新代码: 
  let storage = crate::error::safe_lock(&self.storage, "ProfilingService.storage")
      .map_err(|e| crate::profiling::ProfilingError::ConfigurationError(e.to_string()))?;

### 3. perform_maintenance 函数 (line 384)
- 原代码: `if let Ok(storage_stats) = self.storage.lock().unwrap().get_storage_stats() {`
- 新代码: 
  if let Ok(storage) = self.storage.lock() {
      if let Ok(storage_stats) = storage.get_storage_stats() {

### 4. perform_maintenance 函数 (line 395)
- 原代码: `let mut collector = self.collector.lock().unwrap();`
- 新代码: 
  let mut collector = crate::error::safe_lock(&self.collector, "ProfilingService.collector")
      .map_err(|e| crate::profiling::ProfilingError::ConfigurationError(e.to_string()))?;

### 5. perform_maintenance 函数 (line 402)
- 原代码: `let alerting_engine = self.alerting_engine.lock().unwrap();`
- 新代码: 
  let alerting_engine = crate::error::safe_lock(&self.alerting_engine, "ProfilingService.alerting_engine")
      .map_err(|e| crate::profiling::ProfilingError::ConfigurationError(e.to_string()))?;

### 6. start_dashboard_server 函数 (line 424)
- 原代码: `if self.dashboard.lock().unwrap().is_some() {`
- 新代码: 
  if let Ok(dashboard) = self.dashboard.lock() {
      if dashboard.is_some() {
          return Ok(());
      }
  }

### 7. start_dashboard_server 函数 (line 430)
- 原代码: `let storage = self.storage.lock().unwrap();`
- 新代码: 
  let storage = crate::error::safe_lock(&self.storage, "ProfilingService.storage")
      .map_err(|e| crate::profiling::ProfilingError::ConfigurationError(e.to_string()))?;

### 8. start_dashboard_server 函数 (line 442)
- 原代码: `let mut dashboard = self.dashboard.lock().unwrap();`
- 新代码: 
  let mut dashboard = crate::error::safe_lock(&self.dashboard, "ProfilingService.dashboard")
      .map_err(|e| crate::profiling::ProfilingError::ConfigurationError(e.to_string()))?;

### 9. stop_dashboard_server 函数 (line 451)
- 原代码: `let mut dashboard = self.dashboard.lock().unwrap();`
- 新代码: 
  let mut dashboard = crate::error::safe_lock(&self.dashboard, "ProfilingService.dashboard")
      .map_err(|e| crate::profiling::ProfilingError::ConfigurationError(e.to_string()))?;

### 10. add_default_alert_rules 函数 (line 458)
- 原代码: `let mut alerting_engine = self.alerting_engine.lock().unwrap();`
- 新代码: 
  let mut alerting_engine = crate::error::safe_lock(&self.alerting_engine, "ProfilingService.alerting_engine")
      .map_err(|e| crate::profiling::ProfilingError::ConfigurationError(e.to_string()))?;

## 验证结果
修改后的代码已通过 `cargo build` 和 `cargo check` 验证，可以正常编译。