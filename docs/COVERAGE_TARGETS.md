# 测试覆盖率目标

**创建日期**: 2025-01-XX  
**目的**: 定义各层的测试覆盖率目标

---

## 覆盖率目标

### 领域层 (`src/domain/`)

**目标**: 90%+ 覆盖率

**理由**: 领域层包含核心业务逻辑，需要高覆盖率确保正确性。

**当前状态**: 待测量

**关键模块**:
- `src/domain/scene.rs` - 场景管理
- `src/domain/entity.rs` - 实体管理
- `src/domain/render.rs` - 渲染领域对象
- `src/domain/physics.rs` - 物理领域对象
- `src/domain/audio.rs` - 音频领域对象
- `src/domain/value_objects.rs` - 值对象

### 服务层 (`src/services/`)

**目标**: 80%+ 覆盖率

**理由**: 服务层协调领域对象，需要良好测试覆盖。

**当前状态**: 待测量

**关键模块**:
- `src/services/render.rs` - 渲染服务
- `src/services/audio.rs` - 音频服务
- `src/services/scripting.rs` - 脚本服务

### 基础设施层 (`src/render/`, `src/physics/`, `src/platform/`)

**目标**: 70%+ 覆盖率

**理由**: 基础设施层包含硬件相关代码，难以完全测试。

**当前状态**: 待测量

**关键模块**:
- `src/render/wgpu.rs` - wgpu渲染器
- `src/physics/` - 物理引擎集成
- `src/platform/` - 平台抽象

---

## 覆盖率测量

### 运行覆盖率测试

```bash
# 使用脚本运行
./scripts/run_coverage.sh

# 或直接运行
cargo tarpaulin --out Html --output-dir coverage \
  --exclude-files '*/tests/*' \
  --exclude-files '*/examples/*' \
  --exclude-files '*/benches/*'
```

### 查看覆盖率报告

覆盖率报告生成在 `coverage/tarpaulin-report.html`。

### CI/CD集成

覆盖率测试已集成到CI/CD流程（`.github/workflows/coverage.yml`）。

---

## 覆盖率提升策略

1. **优先领域层**: 先提升领域层覆盖率到90%+
2. **服务层跟进**: 然后提升服务层覆盖率到80%+
3. **基础设施层**: 最后提升基础设施层覆盖率到70%+

---

## 更新记录

- 2025-01-XX: 创建覆盖率目标文档


