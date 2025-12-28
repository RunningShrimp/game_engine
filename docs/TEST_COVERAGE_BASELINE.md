# 代码覆盖率基线报告

**生成时间**: 2024年  
**工具**: cargo-tarpaulin / grcov  
**目标**: 建立代码覆盖率基线，用于跟踪测试覆盖率的改进

## 建立基线步骤

### 1. 安装覆盖率工具

#### 使用 cargo-tarpaulin (推荐)

```bash
cargo install cargo-tarpaulin
```

#### 使用 grcov

```bash
cargo install grcov
```

### 2. 运行覆盖率测试

使用提供的脚本：

```bash
# 使用 tarpaulin
./scripts/run_coverage_report.sh

# 或使用 Makefile
make coverage-html
```

### 3. 生成基线报告

基线报告将包含以下信息：
- 总体覆盖率百分比
- 各模块覆盖率
- 未覆盖的文件和函数
- 覆盖率趋势

## 基线配置

### 排除文件

以下文件/目录应从覆盖率计算中排除：

- `*/tests/*` - 测试文件本身
- `*/benches/*` - 基准测试文件
- `*/examples/*` - 示例代码
- `*/target/*` - 构建产物
- `*/src/bindings/*` - 绑定代码（如果适用）

### 目标覆盖率

根据模块重要性设定目标覆盖率：

| 模块类型 | 目标覆盖率 | 优先级 |
|---------|----------|--------|
| 核心模块 (core, domain) | 80%+ | P0 |
| 关键模块 (ecs, render, physics) | 75%+ | P0 |
| 服务模块 (audio, network) | 70%+ | P1 |
| 工具模块 (editor, profiling) | 60%+ | P2 |
| 可选模块 (xr, scripting) | 50%+ | P2 |

## 基线数据

### 当前覆盖率状态

**待运行基线测试后填写**

- **总体覆盖率**: TBD%
- **行覆盖率**: TBD%
- **函数覆盖率**: TBD%
- **分支覆盖率**: TBD%

### 模块覆盖率详情

| 模块 | 覆盖率 | 状态 | 备注 |
|------|--------|------|------|
| core | TBD% | ⏳ | 待测量 |
| domain | TBD% | ⏳ | 待测量 |
| ecs | TBD% | ⏳ | 待测量 |
| render | TBD% | ⏳ | 待测量 |
| physics | TBD% | ⏳ | 待测量 |
| audio | TBD% | ⏳ | 待测量 |
| network | TBD% | ⏳ | 待测量 |
| resources | TBD% | ⏳ | 待测量 |
| ai | TBD% | ⏳ | 待测量 |
| animation | TBD% | ⏳ | 待测量 |
| editor | TBD% | ⏳ | 待测量 |
| scripting | TBD% | ⏳ | 待测量 |
| xr | TBD% | ⏳ | 待测量 |

### 未覆盖的关键文件

**待运行基线测试后填写**

1. TBD
2. TBD
3. TBD

## 覆盖率改进计划

### 短期目标 (1-2周)

- [ ] 核心模块 (core, domain) 达到 70%+
- [ ] 关键模块 (ecs, render, physics) 达到 65%+
- [ ] 识别并测试关键路径

### 中期目标 (1个月)

- [ ] 核心模块达到 80%+
- [ ] 关键模块达到 75%+
- [ ] 服务模块达到 70%+

### 长期目标 (3个月)

- [ ] 所有模块达到目标覆盖率
- [ ] 建立覆盖率监控CI
- [ ] 实现覆盖率趋势跟踪

## 运行基线测试

### 快速运行

```bash
# 使用脚本（推荐）
./scripts/run_coverage_report.sh

# 查看报告
open target/coverage/index.html
```

### 手动运行

```bash
# 使用 cargo-tarpaulin
cargo tarpaulin \
    --workspace \
    --out Html \
    --output-dir target/coverage \
    --exclude-files "*/tests/*" \
    --exclude-files "*/benches/*" \
    --exclude-files "*/examples/*" \
    --timeout 300 \
    --all-features

# 查看报告
open target/coverage/index.html
```

## 更新基线

基线应定期更新（建议每月一次）：

1. 运行覆盖率测试
2. 更新本文档中的覆盖率数据
3. 分析覆盖率变化趋势
4. 调整改进计划

## CI集成

覆盖率基线应集成到CI流程中：

```yaml
# .github/workflows/coverage.yml
- name: Generate coverage report
  run: ./scripts/run_coverage_report.sh

- name: Upload coverage to Codecov
  uses: codecov/codecov-action@v3
  with:
    files: ./target/coverage/lcov.info
```

## 相关文档

- [覆盖率测试指南](./coverage_report_guide.md)
- [测试策略](./best_practices.md#测试策略)
- [CI/CD优化](./cicd_optimization.md)

