# 游戏引擎中间语言（IL）架构设计

**设计日期**: 2025年1月1日
**设计人**: Game Engine Team
**版本**: v1.0
**状态**: 架构设计阶段

---

## 1. 概述

### 1.1 目标

设计一个专用的中间语言（IL）和编译器架构，用于游戏引擎脚本系统，实现：

1. ✅ **跨语言支持** - 统一Lua、TypeScript、C#等脚本语言
2. ✅ **高性能** - 接近原生代码的性能
3. ✅ **安全** - 内存安全，类型安全
4. ✅ **可扩展** - 易于添加新语言支持和优化

### 1.2 架构概览

```
脚本源代码
    ↓
编译器前端 (Parser + Semantic Analysis)
    ↓
中间语言 (IL) - 统一表示
    ├── IR (Intermediate Representation)
    ├── 类型系统
    └── 内存模型
    ↓
编译器后端 (Backend)
    ├── WebAssembly (WASM)
    ├── 原生代码 (Native)
    └── 字节码 (Bytecode)
    ↓
运行时 (Runtime)
```

---

## 2. 中间语言设计

### 2.1 指令集架构 (ISA)

#### 基本指令类型

```rust
/// IL指令类型
pub enum ILInstruction {
    // ========== 控制流 ==========
    /// 无条件跳转
    Jump(Label),
    /// 条件跳转
    Branch { cond: Operand, then_label: Label, else_label: Label },
    /// 函数调用
    Call { func: FunctionId, args: Vec<Operand> },
    /// 函数返回
    Return(Option<Operand>),

    // ========== 算术运算 ==========
    /// 整数加法
    AddI { dst: Register, src1: Operand, src2: Operand },
    /// 整数减法
    SubI { dst: Register, src1: Operand, src2: Operand },
    /// 整数乘法
    MulI { dst: Register, src1: Operand, src2: Operand },
    /// 整数除法
    DivI { dst: Register, src1: Operand, src2: Operand },
    /// 浮点加法
    AddF { dst: Register, src1: Operand, src2: Operand },
    /// 浮点减法
    SubF { dst: Register, src1: Operand, src2: Operand },

    // ========== 位运算 ==========
    /// 按位与
    And { dst: Register, src1: Operand, src2: Operand },
    /// 按位或
    Or { dst: Register, src1: Operand, src2: Operand },
    /// 按位异或
    Xor { dst: Register, src1: Operand, src2: Operand },
    /// 左移
    Shl { dst: Register, src: Operand, amount: u8 },
    /// 右移
    Shr { dst: Register, src: Operand, amount: u8 },

    // ========== 内存操作 ==========
    /// 加载常量
    LoadConst { dst: Register, value: Constant },
    /// 加载全局变量
    LoadGlobal { dst: Register, addr: Address },
    /// 存储全局变量
    StoreGlobal { addr: Address, src: Operand },
    /// 加载本地变量
    LoadLocal { dst: Register, offset: u32 },
    /// 存储本地变量
    StoreLocal { offset: u32, src: Operand },

    // ========== 数组/结构体 ==========
    /// 数组加载
    LoadElem { dst: Register, base: Operand, index: Operand },
    /// 数组存储
    StoreElem { base: Operand, index: Operand, src: Operand },
    /// 结构体加载
    LoadField { dst: Register, base: Operand, field: FieldId },
    /// 结构体存储
    StoreField { base: Operand, field: FieldId, src: Operand },

    // ========== 比较运算 ==========
    /// 相等比较
    Eq { dst: Register, src1: Operand, src2: Operand },
    /// 不等比较
    Ne { dst: Register, src1: Operand, src2: Operand },
    /// 小于比较
    Lt { dst: Register, src1: Operand, src2: Operand },
    /// 大于比较
    Gt { dst: Register, src1: Operand, src2: Operand },

    // ========== 类型转换 ==========
    /// 整数转浮点
    I2F { dst: Register, src: Operand },
    /// 浮点转整数
    F2I { dst: Register, src: Operand },
    /// 截断
    Trunc { dst: Register, src: Operand, bits: u8 },

    // ========== 特殊指令 ==========
    /// 空操作
    Nop,
    /// 调试标记
    DebugMarker(String),
}
```

### 2.2 类型系统

#### 基本类型

```rust
/// IL类型系统
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ILType {
    /// 空类型
    Void,
    /// 底类型 (布尔值)
    Bool,
    /// 整数类型 (位数)
    Integer(u8),
    /// 浮点类型 (位数)
    Float(u8),
    /// 指针类型
    Pointer(Box<ILType>),
    /// 数组类型
    Array {
        elem: Box<ILType>,
        size: Option<u64>,
    },
    /// 结构体类型
    Struct {
        name: String,
        fields: Vec<(String, ILType)>,
    },
    /// 函数类型
    Function {
        params: Vec<ILType>,
        return_type: Box<ILType>,
    },
}
```

#### 类型别名

```rust
// 便捷类型别名
pub type I32 = ILType::Integer(32);
pub type I64 = ILType::Integer(64);
pub type F32 = ILType::Float(32);
pub type F64 = ILType::Float(64);
pub type Bool = ILType::Bool;
pub type Void = ILType::Void;
```

### 2.3 内存模型

#### 栈帧布局

```
栈帧
┌─────────────────────────┐
│  参数区 (Arguments)     │ ← 高地址
├─────────────────────────┤
│  返回地址 (Return Addr) │
├─────────────────────────┤
│  保存寄存器 (Saved Regs)│
├─────────────────────────┤
│  本地变量 (Locals)      │
│                         │
│    [空余空间]           │
│                         │ ← SP (Stack Pointer)
├─────────────────────────┤
│  调用者栈帧             │
└─────────────────────────┘
```

#### 堆分配

```rust
/// 堆分配指令
pub enum HeapInstr {
    /// 分配内存
    Alloc {
        dst: Register,
        size: Operand,
        align: Option<u8>,
    },
    /// 释放内存
    Free {
        addr: Operand,
    },
}
```

---

## 3. 编译器前端

### 3.1 解析器 (Parser)

#### 语言前端

```rust
/// 语言前端类型
pub enum Frontend {
    Lua,
    TypeScript,
    CSharp,
    Python,
}

/// 解析器接口
pub trait Parser {
    /// 解析源代码为AST
    fn parse(&mut self, source: &str) -> Result<AST, ParseError>;

    /// 获取语言特性
    fn language(&self) -> Frontend;
}
```

#### AST节点

```rust
/// 抽象语法树节点
#[derive(Debug, Clone)]
pub enum ASTNode {
    /// 函数定义
    Function {
        name: String,
        params: Vec<Parameter>,
        return_type: ILType,
        body: Box<ASTNode>,
    },
    /// 语句块
    Block(Vec<ASTNode>),
    /// 变量声明
    VarDecl {
        name: String,
        var_type: ILType,
        init: Option<Box<ASTNode>>,
    },
    /// If语句
    If {
        condition: Box<ASTNode>,
        then_block: Box<ASTNode>,
        else_block: Option<Box<ASTNode>>,
    },
    /// 循环语句
    While {
        condition: Box<ASTNode>,
        body: Box<ASTNode>,
    },
    /// For循环
    For {
        var: String,
        start: Box<ASTNode>,
        end: Box<ASTNode>,
        body: Box<ASTNode>,
    },
    /// 表达式语句
    ExprStmt(Box<ASTNode>),
    /// 表达式
    Expr(Expr),
}

/// 表达式节点
#[derive(Debug, Clone)]
pub enum Expr {
    /// 字面量
    Literal(Literal),
    /// 变量引用
    Variable(String),
    /// 二元运算
    BinOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    /// 一元运算
    UnOp {
        op: UnOp,
        operand: Box<Expr>,
    },
    /// 函数调用
    Call {
        func: String,
        args: Vec<Expr>,
    },
    /// 成员访问
    Member {
        object: Box<Expr>,
        field: String,
    },
    /// 索引访问
    Index {
        array: Box<Expr>,
        index: Box<Expr>,
    },
}
```

### 3.2 语义分析

#### 类型检查

```rust
/// 类型检查器
pub struct TypeChecker {
    /// 符号表
    symbol_table: SymbolTable,
    /// 当前函数
    current_function: Option<FunctionId>,
}

impl TypeChecker {
    /// 检查AST节点类型
    pub fn check(&mut self, node: &ASTNode) -> Result<ILType, TypeError>;

    /// 推断表达式类型
    pub fn infer_type(&mut self, expr: &Expr) -> Result<ILType, TypeError>;

    /// 类型强制转换
    pub fn coerce(
        &mut self,
        expr: &Expr,
        target: ILType
    ) -> Result<Expr, TypeError>;
}
```

---

## 4. IL生成

### 4.1 IR生成器

```rust
/// IR生成器
pub struct IRGenerator {
    /// 当前函数
    current_function: Option<Function>,
    /// 基本块列表
    basic_blocks: Vec<BasicBlock>,
    /// 当前基本块
    current_bb: Option<BasicBlockId>,
    /// 寄存器分配
    reg_allocator: RegisterAllocator,
    /// 符号表
    symbols: SymbolTable,
}

impl IRGenerator {
    /// 生成函数IR
    pub fn generate_function(
        &mut self,
        func: &ASTNode
    ) -> Result<Function, CodeGenError>;

    /// 生成语句块IR
    pub fn generate_block(
        &mut self,
        block: &[ASTNode]
    ) -> Result<BasicBlock, CodeGenError>;

    /// 生成表达式IR
    pub fn generate_expr(
        &mut self,
        expr: &Expr
    ) -> Result<Operand, CodeGenError>;
}
```

### 4.2 SSA形式

```rust
/// 静态单一赋值 (SSA) 转换
pub struct SSATransformer;

impl SSATransformer {
    /// 将IR转换为SSA形式
    pub fn transform(&mut self, func: &mut Function) {
        // 1. 计算支配边界
        // 2. 插入φ函数
        // 3. 重命名变量
    }

    /// 计算支配树
    fn compute_dominators(&self, func: &Function) -> DominatorTree;

    /// 插入φ函数
    fn insert_phi_functions(&mut self, func: &mut Function, dom: &DominatorTree);
}
```

---

## 5. 编译器后端

### 5.1 WebAssembly后端

```rust
/// WebAssembly后端
pub struct WASMBackend;

impl WASMBackend {
    /// 编译为WASM模块
    pub fn compile(&mut self, prog: Program) -> Result<WASMModule, CompileError>;

    /// 生成WASM二进制
    pub fn emit_binary(&self, module: &WASMModule) -> Vec<u8>;
}
```

#### WASM特性映射

| IL指令 | WASM指令 | 说明 |
|---------|----------|------|
| `AddI` | `i32.add` | 32位整数加法 |
| `AddF` | `f32.add` | 32位浮点加法 |
| `LoadLocal` | `local.get` | 加载本地变量 |
| `StoreLocal` | `local.set` | 存储本地变量 |
| `Call` | `call` | 函数调用 |

### 5.2 原生代码后端 (Cranelift)

```rust
/// Cranelift后端
pub struct CraneliftBackend;

impl CraneliftBackend {
    /// 编译为机器码
    pub fn compile(
        &mut self,
        prog: Program,
        target: TargetISA
    ) -> Result<CompiledCode, CompileError>;

    /// JIT编译
    pub fn compile_jit(
        &mut self,
        prog: Program
    ) -> Result<JITModule, CompileError>;
}
```

#### 支持的目标架构

```rust
/// 目标架构
pub enum TargetISA {
    x86_64,
    AArch64,
    RISCV64,
}
```

---

## 6. 运行时系统

### 6.1 虚拟机架构

```rust
/// IL虚拟机
pub struct ILVM {
    /// 指令指针
    ip: usize,
    /// 栈指针
    sp: usize,
    /// 基址指针
    bp: usize,
    /// 栈内存
    stack: Vec<u8>,
    /// 全局内存
    globals: Vec<u8>,
    /// 寄存器
    registers: [Value; 32],
    /// 函数表
    functions: Vec<Function>,
}

impl ILVM {
    /// 创建新虚拟机
    pub fn new() -> Self;

    /// 加载程序
    pub fn load(&mut self, prog: Program) -> Result<(), VMError>;

    /// 执行程序
    pub fn run(&mut self) -> Result<Value, VMError>;

    /// 单步执行
    pub fn step(&mut self) -> Result<(), VMError>;
}
```

### 6.2 垃圾回收

```rust
/// 垃圾回收器
pub struct GarbageCollector {
    /// GC策略
    strategy: GCStrategy,
    /// 根集合
    roots: Vec<Value>,
}

/// GC策略
pub enum GCStrategy {
    /// 标记-清除
    MarkSweep,
    /// 复制
    Copying,
    /// 分代
    Generational {
        young_size: usize,
        old_size: usize,
    },
}

impl GarbageCollector {
    /// 执行垃圾回收
    pub fn collect(&mut self, vm: &mut ILVM) -> GCGStats;
}
```

---

## 7. 优化Pass

### 7.1 常量传播

```rust
/// 常量传播优化
pub struct ConstantPropagation;

impl OptimizationPass for ConstantPropagation {
    fn run(&mut self, func: &mut Function) {
        // 1. 识别常量
        // 2. 传播常量
        // 3. 折叠常量表达式
    }
}
```

### 7.2 死代码消除

```rust
/// 死代码消除
pub struct DeadCodeElimination;

impl OptimizationPass for DeadCodeElimination {
    fn run(&mut self, func: &mut Function) {
        // 1. 构建控制流图
        // 2. 标记活跃代码
        // 3. 删除死代码
    }
}
```

### 7.3 内联优化

```rust
/// 函数内联
pub struct Inliner;

impl Inliner {
    /// 内联小函数
    pub fn inline_functions(
        &mut self,
        func: &mut Function,
        threshold: usize,
    );
}
```

---

## 8. 调试和错误处理

### 8.1 调试信息

```rust
/// 调试信息
pub struct DebugInfo {
    /// 源代码映射
    source_map: SourceMap,
    /// 变量信息
    variables: Vec<VarDebugInfo>,
    /// 行号表
    line_table: LineTable,
}

/// 源代码映射
#[derive(Debug, Clone)]
pub struct SourceMap {
    /// IL指令 → 源代码位置
    mappings: HashMap<InstrId, SourceLocation>,
}

#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
}
```

### 8.2 错误处理

```rust
/// 编译错误类型
pub enum CompileError {
    /// 解析错误
    Parse(ParseError),
    /// 类型错误
    Type(TypeError),
    /// 代码生成错误
    CodeGen(CodeGenError),
    /// 优化错误
    Optimization(OptError),
}

/// 错误位置
#[derive(Debug, Clone)]
pub struct ErrorLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub context: String,
}
```

---

## 9. 实施计划

### 9.1 第一阶段 (3个月)

**目标**: 基础IL设计和编译器前端

- ✅ 设计IL指令集
- ✅ 实现类型系统
- ✅ 实现Lua解析器
- ✅ 实现基本代码生成

**里程碑**: 可以编译简单的Lua脚本为IL

### 9.2 第二阶段 (3个月)

**目标**: WebAssembly后端

- ✅ 实现WASM后端
- ✅ 实现WASM运行时集成
- ✅ 实现优化Pass
- ✅ 测试和调试

**里程碑**: Lua脚本可以编译为WASM并运行

### 9.3 第三阶段 (6个月)

**目标**: 完整编译器支持

- ✅ TypeScript解析器
- ✅ C#解析器
- ✅ Cranelift后端
- ✅ JIT编译
- ✅ 性能优化

**里程碑**: 所有脚本语言支持完整，性能提升>20%

---

## 10. 性能目标

### 10.1 编译速度

| 指标 | 目标 |
|-----|------|
| 小型脚本 (100行) | <100ms |
| 中型脚本 (1000行) | <1s |
| 大型脚本 (10000行) | <10s |

### 10.2 运行时性能

| 指标 | 目标 |
|-----|------|
| 解释执行 | 基线 |
| 编译为WASM | >5x 提升 |
| JIT编译 | >20x 提升 |

---

## 11. 风险和缓解

### 11.1 技术风险

| 风险 | 概率 | 影响 | 缓解策略 |
|-----|------|------|----------|
| **设计复杂度高** | 高 | 高 | 分阶段实施，先实现MVP |
| **性能不达标** | 中 | 高 | 早期性能测试，持续优化 |
| **多语言支持困难** | 高 | 中 | 先支持一种语言，逐步扩展 |
| **Cranelift集成** | 中 | 中 | 准备备选方案 |

### 11.2 实施风险

| 风险 | 缓解策略 |
|-----|----------|
| **开发周期长** | 分阶段交付，优先级排序 |
| **人才需求** | 培训现有团队，招聘编译器专家 |
| **维护成本** | 文档完善，代码规范 |

---

## 12. 结论

中间语言（IL）架构是游戏引擎脚本系统的重要基础设施。

### 12.1 优势

✅ **统一抽象** - 所有脚本语言编译为统一IL
✅ **高性能** - 原生代码级别的性能
✅ **可扩展** - 易于添加新语言支持
✅ **未来导向** - 为后续优化奠定基础

### 12.2 建议

**建议**: 分阶段实施，优先级排序

**Phase 1** (3个月):
1. 基础IL设计
2. Lua编译器前端
3. WASM后端

**Phase 2** (3个月):
1. TypeScript编译器前端
2. 优化Pass
3. 完整测试

**Phase 3** (6个月):
1. C#编译器前端
2. Cranelift后端
3. JIT编译

---

## 附录

### A. 参考资料

- [Cranelift](https://github.com/bytecodealliance/cranelift)
- [WebAssembly](https://webassembly.org/)
- [LLVM IR](https://llvm.org/docs/LangRef.html)
- [Dragon Book](https://www.amazon.com/dp/0321486811/)

### B. 相关项目

- [LuaJIT](https://luajit.org/)
- [V8 (JavaScript Engine)](https://v8.dev/)
- [Roslyn (C# Compiler)](https://github.com/dotnet/roslyn)
