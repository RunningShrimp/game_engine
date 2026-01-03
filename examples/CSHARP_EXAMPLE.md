# C# 脚本示例

本示例展示了如何在游戏引擎中使用C#脚本。

## 前置要求

1. **安装 .NET SDK 8.0 或更高版本**

   **macOS:**
   ```bash
   brew install --cask dotnet-sdk
   ```

   **Linux (Ubuntu/Debian):**
   ```bash
   wget https://packages.microsoft.com/config/ubuntu/$(lsb_release -rs)/packages-microsoft-prod.deb -O packages-microsoft-prod.deb
   sudo dpkg -i packages-microsoft-prod.deb
   sudo apt-get update
   sudo apt-get install -y dotnet-sdk-8.0
   ```

   **Windows:**
   - 下载并安装 [.NET SDK](https://dotnet.microsoft.com/download)

2. **验证安装:**
   ```bash
   dotnet --version
   # 应该显示: 8.0.x 或更高
   ```

## 运行示例

```bash
# 在项目根目录执行
cargo run --example csharp_example --features csharp
```

## 示例内容

### 1. Hello World
基础的C#脚本执行，演示如何定义和调用静态方法。

```csharp
public class HelloWorld
{
    public static string Greet(string name)
    {
        return $"Hello, {name}! Welcome to C# scripting!";
    }
}
```

### 2. 数学计算
使用LINQ和C#标准库进行数学运算。

```csharp
public class Calculator
{
    public static int Sum(int n)
    {
        return Enumerable.Range(1, n).Sum();
    }

    public static bool IsPrime(int n)
    {
        // 质数检测逻辑
    }
}
```

### 3. 对象和集合
展示如何使用C#类、字典和列表。

```csharp
public class Player
{
    public string Name { get; set; }
    public int Level { get; set; }
    public double Health { get; set; }

    public Dictionary<string, object> GetStatus()
    {
        return new Dictionary<string, object>
        {
            { "name", Name },
            { "level", Level },
            { "health", Health }
        };
    }
}
```

### 4. 编译缓存性能
演示编译缓存的性能提升效果。

**预期输出:**
```
首次执行（需要编译）...
⏱️  首次执行耗时: 500.23ms

重复执行（使用缓存）...
⚡ 缓存命中耗时: 0.85ms

📈 性能提升: 588.6x
```

### 5. 缓存统计
查看缓存命中率和统计信息。

**预期输出:**
```
缓存统计:
  命中次数: 15
  未命中次数: 3
  编译次数: 3
  淘汰次数: 0
  命中率: 83.3%
```

## 性能特性

### 编译缓存
- **首次编译:** ~500ms
- **缓存命中:** <1ms
- **性能提升:** 500x

### 跨平台支持
- ✅ Windows (使用 .NET SDK)
- ✅ Linux (使用 .NET SDK)
- ✅ macOS (使用 .NET SDK)

### 数据类型转换
引擎自动处理Rust和C#之间的类型转换：

| C# 类型 | Rust 类型 |
|---------|-----------|
| `string` | `String` |
| `int` | `i64` |
| `double` | `f64` |
| `bool` | `bool` |
| `T[]` | `Vec` |
| `Dictionary<K,V>` | `HashMap` |

## 高级用法

### 游戏逻辑脚本

```csharp
using System;

public class PlayerController
{
    private static float speed = 5.0f;

    public static Vector3 Move(Vector3 position, Vector3 direction, float deltaTime)
    {
        return position + direction * speed * deltaTime;
    }

    public static bool CanJump(bool isGrounded)
    {
        return isGrounded;
    }
}
```

### 数据处理

```csharp
using System;
using System.Linq;

public class DataProcessor
{
    public static double[] Filter(double[] values, double threshold)
    {
        return values.Where(v => v > threshold).ToArray();
    }

    public static Dictionary<string, int> GroupBy(string[] items)
    {
        return items.GroupBy(x => x)
                   .ToDictionary(g => g.Key, g => g.Count());
    }
}
```

## 故障排除

### 错误: ".NET SDK not found"
**解决方案:** 安装 .NET SDK 8.0+

### 错误: "Compilation failed"
**解决方案:** 检查C#代码语法错误，查看详细错误信息

### 错误: "Cache miss every time"
**解决方案:** 检查缓存目录权限，确保脚本名称一致

## 相关文档

- [C# 实现指南](../docs/csharp_implementation_guide.md)
- [C# 性能优化总结](../docs/csharp_optimization_summary.md)
- [C# 运行时评估](../docs/csharp_runtime_evaluation.md)

## 未来计划

- ⏳ **P2-CSHARP-004.3**: 持久化.NET进程池（<5ms执行时间）
- ⏳ **P2-CSHARP-004.4**: 热重载支持

## 贡献

欢迎提交问题报告和改进建议！
