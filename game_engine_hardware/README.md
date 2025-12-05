# game_engine_hardware

硬件检测和优化模块，提供GPU、NPU、SoC等硬件的自动检测和优化建议。

## 功能特性

- GPU检测和优化建议
- NPU检测和加速支持
- SoC检测和功耗管理
- 硬件能力评估
- 自动配置生成
- 自适应性能调整

## 使用示例

```rust
use game_engine_hardware::{get_hardware_info, HardwareInfo};

// 获取硬件信息
let info = get_hardware_info();
println!("GPU: {}", info.gpu.name);
println!("性能等级: {:?}", info.capability.tier);
```

