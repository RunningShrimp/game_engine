# P1-4 Unwrap/Expect Statistics

**Generated**: 2025-12-29
**Scope**: Complete codebase analysis

---

## Overall Statistics

### Total Counts

```
unwrap() calls:  1,541
expect() calls:    177
────────────────────────
Total:           1,718
```

### Distribution by Location

| Location | unwrap | expect | Total | Percentage |
|----------|--------|--------|-------|------------|
| **src/** | 1,374 | 155 | **1,529** | 89.0% |
| **tests/** | 167 | 22 | **189** | 11.0% |
| **Total** | **1,541** | **177** | **1,718** | 100% |

---

## Module Breakdown

### Top 20 Modules by Total Count

| Rank | Module | unwrap | expect | Total | % of Total |
|------|--------|--------|--------|-------|------------|
| 1 | **domain** | 725 | 0 | 725 | 42.2% |
| 2 | **profiling** | 109 | 17 | 126 | 7.3% |
| 3 | **physics** | 103 | 3 | 106 | 6.2% |
| 4 | **performance** | 79 | 2 | 81 | 4.7% |
| 5 | **network** | 51 | 14 | 65 | 3.8% |
| 6 | **resources** | 73 | 5 | 78 | 4.5% |
| 7 | **ecs** | 56 | 0 | 56 | 3.3% |
| 8 | **render** | 55 | 7 | 62 | 3.6% |
| 9 | **error** | 30 | 7 | 37 | 2.2% |
| 10 | **editor** | 19 | 6 | 25 | 1.5% |
| 11 | **services** | 18 | 0 | 18 | 1.0% |
| 12 | **core** | 31 | 9 | 40 | 2.3% |
| 13 | **xr** | 23 | 0 | 23 | 1.3% |
| 14 | **ai** | 6 | 1 | 7 | 0.4% |
| 15 | **animation** | 2 | 0 | 2 | 0.1% |
| 16 | **audio** | 0 | 0 | 0 | 0.0% |
| 17 | **platform** | 5 | 3 | 8 | 0.5% |
| 18 | **scene** | 7 | 0 | 7 | 0.4% |
| 19 | **scripting** | 3 | 6 | 9 | 0.5% |
| 20 | **world** | 4 | 0 | 4 | 0.2% |

**Note**: Domain module at 42.2% significantly skews the distribution.

---

## File-Level Analysis

### Files with >50 unwrap/expect

| File | Module | unwrap | expect | Total | Type |
|------|--------|--------|--------|-------|------|
| `domain/scene.rs` | domain | 105 | 0 | 105 | Production |
| `domain/tests/services_tests.rs` | tests | 118 | 0 | 118 | Test |
| `domain/tests/actor_tests.rs` | tests | 91 | 0 | 91 | Test |
| `domain/tests/scene_tests.rs` | tests | 92 | 0 | 92 | Test |
| `domain/tests/cqrs_tests.rs` | tests | 33 | 0 | 33 | Test |
| `domain/tests/ecs_integration_tests.rs` | tests | 34 | 0 | 34 | Test |
| `domain/tests/event_sourcing_tests.rs` | tests | 30 | 0 | 30 | Test |
| `domain/tests/value_objects_tests.rs` | tests | 49 | 0 | 49 | Test |
| `domain/tests/entity_tests.rs` | tests | 17 | 0 | 17 | Test |
| `physics/physics_core_tests.rs` | tests | 46 | 0 | 46 | Test |
| `physics/extended_tests.rs` | tests | 35 | 0 | 35 | Test |
| `ecs/soa_layout_tests.rs` | tests | 32 | 0 | 32 | Test |
| `ecs/tests.rs` | tests | 20 | 0 | 20 | Test |
| `core/core_module_tests.rs` | tests | 17 | 0 | 17 | Test |

### Production Files with >20 unwrap/expect

| File | Module | unwrap | expect | Total | Priority |
|------|--------|--------|--------|-------|----------|
| `domain/scene.rs` | domain | 105 | 0 | 105 | **HIGH** |
| `render/domain_objects.rs` | render | 41 | 0 | 41 | **HIGH** |
| `profiling/frame_analyzer.rs` | profiling | 15 | 0 | 15 | MEDIUM |
| `domain/services.rs` | domain | 15 | 0 | 15 | MEDIUM |
| `performance/memory/arena.rs` | performance | 17 | 0 | 17 | MEDIUM |
| `domain/entity.rs` | domain | 13 | 0 | 13 | MEDIUM |
| `domain/actor.rs` | domain | 12 | 0 | 12 | MEDIUM |
| `domain/tests/actor_tests.rs` | tests | 91 | 0 | 91 | LOW (test) |
| `domain/tests/scene_tests.rs` | tests | 92 | 0 | 92 | LOW (test) |
| `domain/tests/services_tests.rs` | tests | 118 | 0 | 118 | LOW (test) |

---

## Production vs Test Code

### Production Code (Non-Test Files)

**Total**: ~220 unwrap/expect

**Top 10 Production Files**:

| Rank | File | unwrap | expect | Total |
|------|------|--------|--------|-------|
| 1 | `domain/scene.rs` | 105 | 0 | 105 |
| 2 | `render/domain_objects.rs` | 41 | 0 | 41 |
| 3 | `profiling/frame_analyzer.rs` | 15 | 0 | 15 |
| 4 | `domain/services.rs` | 15 | 0 | 15 |
| 5 | `performance/memory/arena.rs` | 17 | 0 | 17 |
| 6 | `domain/entity.rs` | 13 | 0 | 13 |
| 7 | `domain/actor.rs` | 12 | 0 | 12 |
| 8 | `domain/audio.rs` | 19 | 0 | 19 |
| 9 | `profiling/storage.rs` | 11 | 8 | 19 |
| 10 | `profiling/service.rs` | 10 | 0 | 10 |

### Test Code (Test Files and Functions)

**Total**: ~1,498 unwrap/expect

**Top 10 Test Files**:

| Rank | File | unwrap | expect | Total |
|------|------|--------|--------|-------|
| 1 | `domain/tests/services_tests.rs` | 118 | 0 | 118 |
| 2 | `domain/tests/scene_tests.rs` | 92 | 0 | 92 |
| 3 | `domain/tests/actor_tests.rs` | 91 | 0 | 91 |
| 4 | `domain/tests/cqrs_tests.rs` | 33 | 0 | 33 |
| 5 | `domain/tests/ecs_integration_tests.rs` | 34 | 0 | 34 |
| 6 | `domain/tests/event_sourcing_tests.rs` | 30 | 0 | 30 |
| 7 | `domain/tests/value_objects_tests.rs` | 49 | 0 | 49 |
| 8 | `physics/physics_core_tests.rs` | 46 | 0 | 46 |
| 9 | `physics/extended_tests.rs` | 35 | 0 | 35 |
| 10 | `ecs/soa_layout_tests.rs` | 32 | 0 | 32 |

---

## Critical Path Analysis

### High-Priority Production Files

**Rendering Hot Path**:
- `render/domain_objects.rs`: 41 unwrap
  - Used every frame
  - Failures should never panic
  - **Priority**: CRITICAL

**Core Domain Logic**:
- `domain/scene.rs`: 105 unwrap
  - Scene management
  - Critical for game loop
  - **Priority**: HIGH

**Network Security**:
- `network/key_exchange.rs`: 12 total
- `network/security.rs`: 12 total
  - Crypto operations
  - Security-critical
  - **Priority**: HIGH

**Resource Management**:
- `resources/dependency_manager.rs`: 9 unwrap
  - Asset loading
  - Core infrastructure
  - **Priority**: MEDIUM

---

## unwrap vs expect

### Usage Patterns

| Pattern | Count | Percentage | Typical Use |
|---------|-------|------------|-------------|
| `.unwrap()` | 1,541 | 89.7% | General unwrapping |
| `.expect("msg")` | 177 | 10.3% | Documented panics |

### Distribution

**In production code**:
- `.unwrap()`: ~210 (95%)
- `.expect()`: ~10 (5%)

**In test code**:
- `.unwrap()`: ~1,330 (89%)
- `.expect()`: ~168 (11%)

**Recommendation**: Convert test `.unwrap()` to `.expect()` for better diagnostics.

---

## Conversion Priorities

### Priority 1: Critical Production (Days 1-2)

| File | Count | Effort | Impact |
|------|-------|--------|--------|
| `render/domain_objects.rs` | 41 | 0.5 day | Eliminates render panics |
| `domain/scene.rs` | 105 | 1 day | Core domain stability |
| `network/key_exchange.rs` | 12 | 0.25 day | Security robustness |
| `network/security.rs` | 12 | 0.25 day | Security robustness |
| **Total** | **170** | **2 days** | **High impact** |

### Priority 2: Medium Production (Days 3-4)

| File | Count | Effort | Impact |
|------|-------|--------|--------|
| `domain/entity.rs` | 13 | 0.25 day | Domain model safety |
| `domain/actor.rs` | 12 | 0.25 day | Actor model safety |
| `domain/services.rs` | 15 | 0.25 day | Service layer |
| `profiling/storage.rs` | 19 | 0.5 day | Observability |
| `profiling/frame_analyzer.rs` | 15 | 0.5 day | Profiling safety |
| `resources/dependency_manager.rs` | 9 | 0.25 day | Asset loading |
| **Total** | **83** | **2 days** | **Medium impact** |

### Priority 3: Test Code (Automated)

| Category | Count | Effort | Impact |
|----------|-------|--------|--------|
| High-value tests | ~500 | 1 day (automated) | Better test diagnostics |
| All tests | ~1,400 | 2 days (automated) | Comprehensive coverage |

---

## Progress Tracking

### Target Metrics

| Metric | Current | Target | Reduction |
|--------|---------|--------|------------|
| **Production unwrap** | ~220 | <50 | -170 (-77%) |
| **Test unwrap** | ~1,400 | <100 | -1,300 (-93%) |
| **Total unwrap/expect** | 1,718 | <1,050 | -668 (-39%) |

### Milestones

- [ ] **Milestone 1**: Critical production fixed (170 reductions)
- [ ] **Milestone 2**: All production fixed (250 reductions)
- [ ] **Milestone 3**: High-value tests converted (750 reductions)
- [ ] **Milestone 4**: All tests converted (1,618 reductions)

---

## Code Quality Metrics

### Current State

- **Panic risk in production**: HIGH (220 potential panics)
- **Test diagnostics quality**: LOW (unwrap with no context)
- **Error handling coverage**: MEDIUM (good infrastructure, not fully used)
- **Code maintainability**: MEDIUM (mixed patterns)

### Target State (After Option A)

- **Panic risk in production**: LOW (<50 potential panics)
- **Test diagnostics quality**: HIGH (expect with clear messages)
- **Error handling coverage**: HIGH (consistent patterns)
- **Code maintainability**: HIGH (standardized approach)

---

## Recommendations Summary

1. **Immediate Actions** (Week 1):
   - Fix 4 critical production files (170 unwrap)
   - Focus on render, domain, network

2. **Short-term** (Week 2):
   - Fix 6 medium-priority production files (83 unwrap)
   - Run automated test conversion

3. **Long-term** (Ongoing):
   - Enforce error handling in code review
   - Add clippy lint for unwrap in production
   - Document safe unwrap patterns

---

**Report End**

For detailed analysis, see:
- **P1-4-unwrap-replacement-report.md** - Comprehensive analysis
- **P1-4-SUMMARY.md** - Executive summary
- **P1-4-STATISTICS.md** - This document
