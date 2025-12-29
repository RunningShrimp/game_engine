# Unsafe代码审查清单

本文档提供了unsafe代码审查的标准清单，确保所有unsafe使用都经过充分审查和文档化。

---

## 审查原则

### 1. 最小化原则
- ✅ 优先使用safe Rust
- ✅ unsafe范围尽可能小
- ✅ 封装unsafe到safe wrapper

### 2. 文档化原则
- ✅ 每个unsafe块都有SAFETY注释
- ✅ 说明为什么需要unsafe
- ✅ 列出所有前置条件

### 3. 验证原则
- ✅ 使用debug_assert!验证关键不变量
- ✅ 添加单元测试覆盖unsafe代码
- ✅ 使用Miri检测未定义行为

---

## 审查清单

### A. 内存安全

#### A1. 空指针检查
```rust
// ❌ 错误：未检查空指针
let ptr = unsafe { *raw_ptr };

// ✅ 正确：先检查
if !raw_ptr.is_null() {
    let val = unsafe { *raw_ptr };
}

// ✅ 更好：使用NonNull
let ptr = NonNull::new(raw_ptr).ok_or(Error::NullPointer)?;
```

**检查项**:
- [ ] 所有指针解引用前检查空指针
- [ ] 使用NonNull而非裸指针
- [ ] FFI返回值立即检查null

---

#### A2. 指针对齐
```rust
// ❌ 错误：假设对齐
let ptr = unsafe { &*(raw_ptr as *const u32) };

// ✅ 正确：验证对齐
assert_eq!(raw_ptr as usize % std::mem::align_of::<u32>(), 0);
let ptr = unsafe { &*(raw_ptr as *const u32) };

// ✅ 更好：使用aligned pointer
#[repr(C)]
struct Aligned<T> {
    _align: [std::mem::MaybeUninit<u32>; 0],
    value: T,
}
```

**检查项**:
- [ ] 指针对齐符合类型要求
- [ ] 使用align_of验证
- [ ] SIMD操作使用适当的对齐

---

#### A3. 悬垂指针
```rust
// ❌ 错误：返回临时引用
fn dangling() -> &'static u32 {
    let x = 42;
    unsafe { &x as *const u32 as &'static u32 }
}

// ✅ 正确：使用arena拥有所有权
struct Arena {
    data: Vec<u8>,
}
impl Arena {
    fn alloc<'a>(&'a self, size: usize) -> Option<&'a mut [u8]> {
        // 返回的引用生命周期绑定到arena
    }
}
```

**检查项**:
- [ ] 返回引用的生命周期正确标注
- [ ] 未返回指向栈变量的引用
- [ ] arena/reset模式文档化生命周期限制

---

#### A4. 别名规则
```rust
// ❌ 错误：可变别名
let mut_ref1 = unsafe { &mut *ptr };
let mut_ref2 = unsafe { &mut *ptr }; // UB!

// ✅ 正确：使用slice确保无重叠
fn safe_copy(src: &[u8], dst: &mut [u8]) {
    assert!(src.len() == dst.len());
    // 编译器确保src和dst不重叠
    dst.copy_from_slice(src);
}
```

**检查项**:
- [ ] 无并发可变引用
- [ ] 读写操作不重叠
- [ ] 使用Cell/RefCell管理内部可变性

---

### B. 线程安全

#### B1. Send/Sync实现
```rust
// ❌ 错误：不安全的Send/Sync
struct ContainsRawPtr(*mut u8);
unsafe impl Send for ContainsRawPtr {} // 线程间传递裸指针

// ✅ 正确：使用Arc<Mutex<T>>
struct SafeWrapper {
    data: Arc<Mutex<Vec<u8>>>,
}
```

**检查项**:
- [ ] Send类型确实可以安全跨线程传递
- [ ] Sync类型确实可以安全共享
- [ ] 不包含裸指针、Rc等非线程安全类型
- [ ] 内部可变性正确保护（Mutex, RwLock, Atomic）

---

#### B2. 数据竞争
```rust
// ❌ 错误：未同步的并发访问
static mut GLOBAL_COUNTER: u32 = 0;

fn increment() {
    unsafe { GLOBAL_COUNTER += 1; } // 数据竞争!
}

// ✅ 正确：使用原子操作
static GLOBAL_COUNTER: AtomicU32 = AtomicU32::new(0);

fn increment() {
    GLOBAL_COUNTER.fetch_add(1, Ordering::Relaxed);
}
```

**检查项**:
- [ ] 全局可变状态使用原子类型
- [ ] 使用Mutex/RwLock保护复合数据
- [ ] 正确使用内存 Ordering

---

### C. 生命周期

#### C1. 存储时生命周期
```rust
// ❌ 错误：存储引用超出生命周期
struct StoreRef<'a> {
    ref_: Option<&'a u32>,
}
fn store_temporary() {
    let x = 42;
    let mut store = StoreRef { ref_: None };
    store.ref_ = Some(&x); // x在函数结束时drop
} // store.ref_成为悬垂指针

// ✅ 正确：使用owned类型或arena
struct StoreOwned {
    value: Option<u32>,
}
```

**检查项**:
- [ ] 存储的引用生命周期正确
- [ ] 使用arena时文档化reset使引用失效
- [ ] 考虑使用owned类型而非引用

---

#### C2. self生命周期
```rust
// ❌ 危险：返回self的引用但可变
impl Arena {
    fn alloc(&mut self) -> &mut u32 {
        // 调用reset()会使此引用失效
    }
}

// ✅ 正确：使用运行时检查或借用标记
impl Arena {
    fn alloc(&mut self, version: u64) -> Option<(&mut u32, u64)> {
        // 返回版本号，使用前检查
    }

    fn reset(&mut self) {
        self.version += 1;
    }
}
```

**检查项**:
- [ ] 返回引用不会因方法调用而失效
- [ ] 文档化所有使引用失效的操作
- [ ] 考虑使用版本号或epoch机制

---

### D. FFI边界

#### D1. C ABI兼容性
```rust
// ❌ 错误：ABI不匹配
extern "C" {
    fn c_function(p: *const std::ffi::CStr);
}

// ✅ 正确：使用C兼容类型
extern "C" {
    fn c_function(p: *const std::os::raw::c_char);
}
```

**检查项**:
- [ ] 使用repr(C)结构体
- [ ] 使用C兼容类型（c_char, c_int等）
- [ ] 枚举使用#[repr(C/i32/u32)]

---

#### D2. 内存所有权
```rust
// ❌ 错误：双重释放
extern "C" {
    fn c_free(ptr: *mut u8);
}
fn double_free() {
    let ptr = unsafe { c_malloc(1024) };
    unsafe { c_free(ptr) };
    drop(Box::from_raw(ptr)); // 双重释放!
}

// ✅ 正确：明确所有权
extern "C" {
    fn c_malloc(size: usize) -> *mut u8;
    fn c_free(ptr: *mut u8);
}
fn safe_malloc(size: usize) -> Option<Box<[u8]>> {
    let ptr = unsafe { c_malloc(size) };
    if ptr.is_null() {
        return None;
    }
    // 转移所有权到Box
    unsafe { Some(Box::from_raw(slice_from_raw_parts_mut(ptr, size))) }
}
```

**检查项**:
- [ ] 内存所有权清晰定义
- [ ] 谁负责释放（Rust vs C）
- [ ] 使用ManuallyDrop/Box管理边界

---

#### D3. 错误处理
```rust
// ❌ 错误：忽略FFI错误
extern "C" {
    fn c_init() -> c_int;
}
fn init() {
    unsafe { c_init() }; // 忽略返回值
}

// ✅ 正确：检查并转换错误
extern "C" {
    fn c_init() -> c_int;
}
fn init() -> Result<(), Error> {
    let result = unsafe { c_init() };
    if result != 0 {
        Err(Error::InitializationFailed(result))
    } else {
        Ok(())
    }
}
```

**检查项**:
- [ ] 检查所有FFI返回值
- [ ] 正确处理错误码
- [ ] 设置errno时立即检查

---

### E. 并发

#### E1. 无锁算法
```rust
// ❌ 错误：不正确的无锁实现
struct LockFreeStack {
    head: AtomicPtr<Node>,
}
fn push(&self, node: *mut Node) {
    loop {
        let old = self.head.load(Ordering::Relaxed); // 应该是Acquire
        // ... CAS操作
    }
}

// ✅ 正确：使用正确的内存序
fn push(&self, node: *mut Node) {
    loop {
        let old = self.head.load(Ordering::Acquire);
        node.next = old;
        match self.head.compare_exchange_weak(
            old,
            node,
            Ordering::Release,
            Ordering::Relaxed,
        ) {
            Ok(_) => return,
            Err(_) => continue,
        }
    }
}
```

**检查项**:
- [ ] 使用正确的内存序
- [ ] CAS循环正确处理失败
- [ ] 考虑使用crossbeam/arc_swap库

---

### F. 文档

#### F1. SAFETY注释
```rust
// ❌ 不充分：缺少解释
unsafe { *ptr = value; }

// ✅ 充足：详细说明
// SAFETY:
// 1. ptr已通过NonNull::new验证非null
// 2. ptr指向的内存已通过alloc分配且未释放
// 3. ptr正确对齐到T::align_of()
// 4. 没有其他活跃的可变引用到此内存
// 5. value类型为T，大小和对齐正确
unsafe { *ptr = value; }
```

**检查项**:
- [ ] SAFETY注释解释为什么安全
- [ ] 列出所有前置条件
- [ ] 说明不变量如何维护

---

#### F2. 示例代码
```rust
/// 分配并初始化对象
///
/// # Examples
///
/// ```rust
/// use my_crate::Arena;
///
/// let arena = Arena::new(1024)?;
/// let value = arena.allocate(42)?;
/// assert_eq!(*value, 42);
///
/// // 注意：调用reset()会使value失效
/// // arena.reset();
/// // assert_eq!(*value, 42); // UB!
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn allocate<T>(&self, value: T) -> Result<&mut T, Error> {
    // ...
}
```

**检查项**:
- [ ] 提供使用示例
- [ ] 展示常见陷阱
- [ ] 示例可编译运行（doctest）

---

## 工具支持

### Miri配置

在项目根目录创建`.mirirc.toml`:

```toml
[diag]
verbose = true

# 仅在必要时禁用检查
# check_alignment = false
```

运行Miri:
```bash
cargo miri test
cargo miri run --bin example
```

### Clippy lint

```toml
# Cargo.toml
[lints.clippy]
# unsafe相关lint
undocumented_unsafe_blocks = "deny"
multiple_unsafe_ops_per_block = "warn"
transmute_ptr_to_ptr = "warn"
```

### 自定义检查脚本

```bash
#!/bin/bash
# scripts/check-unsafe.sh

echo "Checking unsafe code..."

# 查找unsafe但缺少SAFETY注释
grep -r "unsafe {" --include="*.rs" src/ | while read line; do
    file=$(echo "$line" | cut -d: -f1)
    lineno=$(echo "$line" | cut -d: -f2)

    # 检查前3行
    context=$(sed -n "$((lineno-3)),${lineno}p" "$file")

    if ! echo "$context" | grep -qi "SAFETY\|Safety"; then
        echo "WARNING: $file:$lineno - unsafe without SAFETY comment"
    fi
done

echo "Done."
```

---

## 审查流程

### 1. Pre-review（编写阶段）
- [ ] 编写SAFETY注释
- [ ] 添加前置条件检查
- [ ] 编写单元测试

### 2. Self-review（提交前）
- [ ] 运行cargo test
- [ ] 运行cargo clippy
- [ ] 运行cargo miri test（如适用）
- [ ] 使用check-unsafe.sh脚本

### 3. Peer review（PR阶段）
- [ ] 审查者使用本清单
- [ ] 讨论unsafe的必要性
- [ ] 确认文档完整

### 4. Post-review（合并后）
- [ ] 更新unsafe使用统计
- [ ] 考虑封装为safe wrapper
- [ ] 定期重新审查

---

## 常见模式

### 模式1: 字节切片转类型

```rust
// ❌ 不安全
fn from_bytes<T>(bytes: &[u8]) -> &T {
    unsafe { &*(bytes.as_ptr() as *const T) }
}

// ✅ 安全：使用bytemuck
use bytemuck::{Pod, try_cast_slice};

fn from_bytes_safe<T: Pod>(bytes: &[u8]) -> Result<&T, Error> {
    try_cast_slice(bytes)
        .map(|slice| &slice[0])
        .map_err(|_| Error::InvalidCast)
}

// ✅ 或手动检查
fn from_bytes_manual<T>(bytes: &[u8]) -> Result<&T, Error> {
    // 检查对齐
    if (bytes.as_ptr() as usize) % std::mem::align_of::<T>() != 0 {
        return Err(Error::NotAligned);
    }

    // 检查大小
    if bytes.len() < std::mem::size_of::<T>() {
        return Err(Error::TooShort);
    }

    // SAFETY: 已检查对齐和大小
    unsafe { Ok(&*(bytes.as_ptr() as *const T)) }
}
```

---

### 模式2: 自定义分配器

```rust
// ❌ 不安全：缺少验证
struct MyAllocator;

unsafe impl GlobalAlloc for MyAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = libc::malloc(layout.size());
        // 未检查null，未检查对齐
        ptr as *mut u8
    }
}

// ✅ 安全：完整检查
unsafe impl GlobalAlloc for MyAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // 检查size > 0
        if layout.size() == 0 {
            return std::ptr::NonNull::dangling().as_ptr();
        }

        // 分配
        let ptr = libc::malloc(layout.size());

        // 检查null
        if ptr.is_null() {
            // 处理OOM
            return std::ptr::null_mut();
        }

        // 检查对齐（malloc保证对齐，但其他分配器可能不）
        let aligned = if layout.align() > std::mem::align_of::<usize>() {
            // 使用aligned_alloc或posix_memalign
            ptr
        } else {
            ptr
        };

        aligned as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        // 处理null和dangling
        if !ptr.is_null() {
            libc::free(ptr as *mut libc::c_void);
        }
    }
}
```

---

### 模式3: SIMD内联汇编

```rust
// ❌ 不安全：未检查CPU特性
#[target_feature(enable = "sse2")]
pub unsafe fn add_sse2(a: &[f32], b: &[f32], out: &mut [f32]) {
    // 假设CPU支持SSE2
}

// ✅ 安全：提供safe wrapper
#[cfg(target_arch = "x86_64")]
pub fn add_auto(a: &[f32], b: &[f32], out: &mut [f32]) {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("sse2") {
            unsafe { add_sse2(a, b, out) }
        } else {
            // fallback
            for i in 0..a.len() {
                out[i] = a[i] + b[i];
            }
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        // 其他架构的fallback
        for i in 0..a.len() {
            out[i] = a[i] + b[i];
        }
    }
}
```

---

## 参考资源

### 官方文档
- [The Rustonomicon](https://doc.rust-lang.org/nomicon/)
- [Rust Reference: Unsafe](https://doc.rust-lang.org/reference/unsafe.html)
- [Miri](https://rustc-dev-guide.rust-lang.org/miri.html)

### 社区资源
- [Rust Unsafe Guidelines](https://rust-lang.github.io/unsafe-code-guidelines/)
- [Ferrous Systems unsafe guide](https://ferrous-systems.com/blog/why-rust-unsafe-is-not-scary/)

### 工具
- [Miri - Undefined behavior detector](https://github.com/rust-lang/miri)
- [Clippy - Lint checker](https://github.com/rust-lang/rust-clippy)
- [cargo-geiger - Unsafe usage scanner](https://github.com/rust-secure-code/cargo-geiger)

---

**版本**: 1.0
**最后更新**: 2025-12-29
**维护者**: Game Engine Team
