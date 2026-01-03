# Console Platform Certification System

## 概述

控制台平台认证系统是一个全面的认证要求检查工具,用于确保游戏符合各个游戏机平台的认证标准。该系统支持自动化测试、报告生成和平台特定要求检查。

## 支持的平台

- **Nintendo Switch** - 任天堂Switch平台认证要求
- **PlayStation 5** - 索尼PlayStation 5平台认证要求
- **PlayStation 4** - 索尼PlayStation 4平台认证要求
- **Xbox Series X/S** - 微软Xbox Series X/S平台认证要求
- **Xbox One** - 微软Xbox One平台认证要求

## 功能特性

### 1. 平台认证要求检查

系统提供全面的认证要求检查,包括:

#### 通用要求 (所有平台)
- 成就/奖杯系统集成
- 云存档支持
- 手柄震动反馈
- 错误处理
- 加载屏幕与进度显示
- 暂停菜单
- 网络断线处理
- 存档损坏处理
- 帧率稳定性
- 内存使用限制
- 按键提示
- 无障碍选项

#### Nintendo Switch 特定要求
- 存档大小限制 (通常 < 100MB)
- Joy-Con 配对/取消配对支持
- 底座/掌机模式切换
- 截图功能
- 视频捕获 (可选)
- Nintendo Ages 家长控制集成
- 手柄发现与分配
- 睡眠模式处理

#### PlayStation 特定要求
- 奖杯图标
- PSN 集成
- PlayStation UI 指南遵循
- PS 按钮行为
- Share 按钮功能
- Party Chat 集成
- PSN 头像显示 (可选)
- 活动卡片 Activity Cards (PS5, 可选)
- DualSense 功能 (PS5, 推荐)
- Tempest 3D 音频 (PS5, 可选)

#### Xbox 特定要求
- 成就图标
- Xbox Live 集成
- Gamerscore 分配规则
- Xbox UI 指南遵循
- Xbox 按钮行为
- Smart Delivery (可选)
- Quick Resume 支持
- 云游戏兼容性 (可选)
- 语言支持
- Party Chat 集成

### 2. 认证要求分类

系统将认证要求分为以下类别:

- **UserInterface** - 用户界面和用户体验要求
- **Online** - 在线功能和网络要求
- **Storage** - 存档数据和存储要求
- **Input** - 输入和手柄要求
- **Performance** - 性能和技术要求
- **PlatformIntegration** - 平台集成要求
- **Security** - 安全和隐私要求
- **Accessibility** - 无障碍要求
- **Legal** - 法律和合规要求

### 3. 严重性级别

每个认证要求都有相应的严重性级别:

- **Critical** - 关键失败,将阻止认证通过
- **Major** - 主要问题,必须修复
- **Minor** - 次要问题,应该修复
- **Info** - 信息性提示

### 4. 报告生成

系统支持多种格式的认证报告:

#### 文本报告
```rust
let report = checker.check_all();
println!("{}", report.generate_report());
```

#### JSON 报告
```rust
let report = checker.check_all();
report.save_json("certification_report.json")?;
```

#### HTML 报告
```rust
let report = checker.check_all();
report.save_html("certification_report.html")?;
```

### 5. 自动化测试

系统提供完整的测试套件,可以自动检查所有认证要求:

```rust
let checker = CertificationChecker::new(ConsolePlatform::PlayStation5);
let report = checker.check_all();

if report.all_passed() {
    println!("所有认证要求通过!");
} else {
    println!("认证未通过:");
    for req in report.failed_requirements() {
        println!("  - {}: {}", req.id, req.name);
    }
}
```

## 使用指南

### 基本使用

```rust
use game_engine::platform::console::certification::*;
use game_engine::platform::console::ConsolePlatform;

// 创建认证检查器
let checker = CertificationChecker::new(ConsolePlatform::PlayStation5);

// 运行所有认证检查
let report = checker.check_all();

// 生成报告
println!("{}", report.generate_report());

// 检查是否通过
if report.all_passed() {
    println!("认证通过!");
} else {
    println!("认证未通过,通过率: {:.1}%", report.pass_rate());
}
```

### 添加自定义要求

```rust
let mut checker = CertificationChecker::new(ConsolePlatform::XboxSeries);

// 创建自定义认证要求
let custom_req = CertRequirement::new(
    "custom_60fps",
    CertCategory::Performance,
    "60 FPS Requirement",
    "Game must maintain 60 FPS during gameplay",
    true,  // required
    Severity::Critical,
)
.with_notes("Critical for this game")
.with_reference("https://example.com/framerate")
.mark_passed();

// 添加自定义要求
checker.add_custom_rule(custom_req);

// 运行检查
let report = checker.check_all();
```

### 按类别查看要求

```rust
let checker = CertificationChecker::new(ConsolePlatform::NintendoSwitch);
let report = checker.check_all();

// 获取所有用户界面要求
let ui_reqs = report.requirements_by_category(CertCategory::UserInterface);
for req in ui_reqs {
    println!("{}: {} - {}", req.id, req.name, req.description);
}
```

### 查看关键失败项

```rust
let checker = CertificationChecker::new(ConsolePlatform::PlayStation5);
let report = checker.check_all();

// 获取所有关键失败
let critical = report.critical_failures();
if !critical.is_empty() {
    println!("关键失败项:");
    for req in critical {
        println!("  ✗ {}: {}", req.id, req.name);
        println!("    {}", req.description);
        if let Some(ref url) = req.reference_url {
            println!("    参考: {}", url);
        }
    }
}
```

### 保存多种格式报告

```rust
use std::path::Path;

let checker = CertificationChecker::new(ConsolePlatform::XboxSeries);
let report = checker.check_all();

// 保存 JSON 报告
report.save_json("reports/xbox_certification.json")?;

// 保存 HTML 报告
report.save_html("reports/xbox_certification.html")?;

// 或者在内存中获取
let json_string = report.to_json()?;
let html_string = report.to_html();
```

### 配置认证检查器

```rust
use game_engine::platform::console::certification::*;

let config = CertCheckerConfig {
    include_optional: true,
    severity_threshold: Severity::Major,
    verbose: true,
    save_reports: true,
    output_dir: Some("reports".to_string()),
};

let checker = CertificationChecker::with_config(ConsolePlatform::PlayStation5, config);
let report = checker.check_all();
```

## 认证检查流程

1. **初始化** - 创建特定平台的认证检查器
2. **配置** - 设置检查器配置(可选)
3. **添加自定义规则** - 添加项目特定的认证要求(可选)
4. **运行检查** - 执行所有认证要求检查
5. **生成报告** - 生成认证报告
6. **分析结果** - 查看失败项并修复
7. **重新检查** - 重复检查直到所有要求通过

## 报告内容

### 摘要信息
- 总要求数量
- 通过数量和百分比
- 失败数量
- 总体状态(通过/失败)

### 关键失败
- 列出所有阻止认证的关键问题

### 分类要求
- 按类别分组的所有要求
- 每个要求的详细状态
- 描述、严重性、是否必需

### 元数据
- 引擎版本
- 测试运行器版本
- 平台SDK版本(如有)
- 测试环境
- 测试持续时间

## 最佳实践

### 1. 早期集成
在游戏开发早期就开始使用认证系统,定期运行检查:
```rust
#[cfg(test)]
mod certification_tests {
    use super::*;

    #[test]
    fn regular_certification_check() {
        let checker = CertificationChecker::new(target_platform());
        let report = checker.check_all();
        assert!(report.critical_failures().is_empty(),
            "Critical certification failures detected");
    }
}
```

### 2. 持续集成
将认证检查集成到CI/CD流程:
```yaml
# .github/workflows/certification.yml
name: Console Certification Check
on: [push, pull_request]

jobs:
  certification:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Run Certification Checks
        run: cargo test --test certification_tests
```

### 3. 自定义项目特定要求
为项目添加特定的认证要求:
```rust
let game_specific_reqs = vec![
    CertRequirement::new(
        "loading_time_max",
        CertCategory::Performance,
        "Loading Time Limit",
        "Initial load must complete within 30 seconds",
        true,
        Severity::Critical,
    ),
    CertRequirement::new(
        "save_data_size",
        CertCategory::Storage,
        "Save Data Size",
        "Single save must not exceed 50MB",
        true,
        Severity::Major,
    ),
];

for req in game_specific_reqs {
    checker.add_custom_rule(req);
}
```

### 4. 追踪认证进度
使用版本控制追踪认证进度:
```rust
// 定期保存认证报告
let report_path = format!(
    "reports/certification_{}.json",
    chrono::Utc::now().format("%Y%m%d_%H%M%S")
);
report.save_json(report_path)?;
```

## 测试覆盖

系统提供全面的单元测试和集成测试:

```bash
# 运行所有认证测试
cargo test --test certification_tests

# 运行特定平台测试
cargo test test_nintendo_switch_requirements
cargo test test_playstation_requirements
cargo test test_xbox_requirements
```

## 故障排除

### 认证检查失败

1. **查看详细报告**
   ```rust
   let report = checker.check_all();
   println!("{}", report.generate_report());
   ```

2. **检查关键失败**
   ```rust
   for req in report.critical_failures() {
       println!("Critical: {} - {}", req.id, req.name);
       println!("  Description: {}", req.description);
       println!("  Reference: {:?}", req.reference_url);
   }
   ```

3. **按类别审查**
   ```rust
   for category in &[CertCategory::UserInterface,
                     CertCategory::Performance,
                     CertCategory::Input] {
       let reqs = report.requirements_by_category(*category);
       println!("{}: {}/{} passed",
           category,
           reqs.iter().filter(|r| r.passed).count(),
           reqs.len()
       );
   }
   ```

### 常见问题

**Q: 为什么大多数要求都显示未通过?**
A: 当前实现使用模拟检查。在实际游戏中,需要实现真实的检查逻辑来验证每个要求。

**Q: 如何添加平台特定要求?**
A: 使用 `add_custom_rule()` 方法添加自定义要求,或修改 `get_*_requirements()` 方法。

**Q: 报告可以自定义吗?**
A: 可以通过修改报告生成方法或创建自定义报告器来自定义报告格式。

**Q: 支持哪些平台?**
A: 当前支持 Nintendo Switch、PlayStation 4/5 和 Xbox One/Series X/S。

## API 参考

### 主要类型

- `CertificationChecker` - 认证检查器
- `CertificationReport` - 认证报告
- `CertRequirement` - 认证要求
- `CertCategory` - 认证类别枚举
- `Severity` - 严重性级别枚举
- `CertCheckerConfig` - 检查器配置
- `ReportMetadata` - 报告元数据

### 主要方法

#### CertificationChecker
- `new(platform)` - 创建检查器
- `with_config(platform, config)` - 使用配置创建
- `add_custom_rule(requirement)` - 添加自定义规则
- `check_all()` - 运行所有检查

#### CertificationReport
- `all_passed()` - 检查是否全部通过
- `failed_requirements()` - 获取失败项
- `passed_requirements()` - 获取通过项
- `requirements_by_category(category)` - 按类别获取
- `critical_failures()` - 获取关键失败
- `pass_rate()` - 计算通过率
- `generate_report()` - 生成文本报告
- `to_json()` - 转换为JSON
- `save_json(path)` - 保存JSON报告
- `to_html()` - 生成HTML
- `save_html(path)` - 保存HTML报告

## 参考资料

- [Nintendo Switch 开发者门户](https://developer.nintendo.com/)
- [PlayStation Partners](https://partners.playstation.com/)
- [Xbox 开发者门户](https://partner.microsoft.com/)
- [游戏认证最佳实践](https://www.gamedeveloper.com/)

## 许可证

MIT License OR Apache-2.0

## 贡献

欢迎提交问题报告和拉取请求!

---

**版本**: 0.1.0
**最后更新**: 2026-01-02
