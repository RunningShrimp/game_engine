// C# 脚本示例
//
// 展示游戏引擎中C#脚本的使用方法，包括：
// - 基础脚本执行
// - 类型转换
// - 编译缓存
// - 性能优化
//
// 运行：
// ```bash
// cargo run --example csharp_example --features csharp
// ```

#[cfg(feature = "csharp")]
use game_engine::scripting::csharp_dotnet::DotNetCliHost;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter("scripting=debug,game_engine=info")
        .init();

    println!("🎮 C# 脚本示例 - 游戏引擎");
    println!("{}", "=".repeat(50));

    // 检查.NET SDK是否可用
    let host = match DotNetCliHost::initialize() {
        Ok(h) => {
            println!("✅ .NET SDK {} 已就绪", h.dotnet_version);
            h
        }
        Err(e) => {
            eprintln!("❌ 初始化失败: {}", e);
            eprintln!("请安装 .NET SDK 8.0 或更高版本:");
            eprintln!("  macOS:   brew install --cask dotnet-sdk");
            eprintln!("  Linux:   参考微软文档");
            eprintln!("  Windows: 下载安装程序");
            eprintln!("  https://dotnet.microsoft.com/download");
            std::process::exit(1);
        }
    };

    // 示例1：Hello World
    println!("\n📝 示例1: Hello World");
    println!("{}", "-".repeat(30));
    example_hello_world(&host)?;

    // 示例2：数据计算
    println!("\n📝 示例2: 数学计算");
    println!("{}", "-".repeat(30));
    example_calculations(&host)?;

    // 示例3：对象和集合
    println!("\n📝 示例3: 对象和集合");
    println!("{}", "-".repeat(30));
    example_objects(&host)?;

    // 示例4：编译缓存演示
    println!("\n📝 示例4: 编译缓存性能");
    println!("{}", "-".repeat(30));
    example_compile_cache(&host)?;

    // 示例5：缓存统计
    println!("\n📝 示例5: 缓存统计");
    println!("{}", "-".repeat(30));
    example_cache_statistics(&host)?;

    println!("\n✅ 所有示例运行完成！");
    Ok(())
}

/// 示例1：Hello World
#[cfg(feature = "csharp")]
fn example_hello_world(host: &DotNetCliHost) -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
using System;

public class HelloWorld
{
    public static string Greet(string name)
    {
        return $"Hello, {name}! Welcome to C# scripting!";
    }

    public static int GetAnswer()
    {
        return 42;
    }
}
"#;

    // 调用静态方法
    let result1 = host.compile_and_execute(code, "hello_world")?;
    println!(" greeted: {:?}", result1);

    Ok(())
}

/// 示例2：数学计算
#[cfg(feature = "csharp")]
fn example_calculations(host: &DotNetCliHost) -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
using System;
using System.Linq;

public class Calculator
{
    public static int Sum(int n)
    {
        return Enumerable.Range(1, n).Sum();
    }

    public static double Average(int[] numbers)
    {
        if (numbers == null || numbers.Length == 0)
            return 0.0;
        return numbers.Average();
    }

    public static bool IsPrime(int n)
    {
        if (n < 2) return false;
        for (int i = 2; i * i <= n; i++)
            if (n % i == 0) return false;
        return true;
    }
}
"#;

    // 计算总和
    let result = host.compile_and_execute(code, "calculator")?;
    println!("计算结果: {:?}", result);

    Ok(())
}

/// 示例3：对象和集合
#[cfg(feature = "csharp")]
fn example_objects(host: &DotNetCliHost) -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
using System;
using System.Collections.Generic;

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
            { "health", Health },
            { "isAlive", Health > 0 }
        };
    }
}

public class GameEngine
{
    public static Dictionary<string, object> CreatePlayer(string name, int level)
    {
        var player = new Player
        {
            Name = name,
            Level = level,
            Health = 100.0
        };

        return player.GetStatus();
    }

    public static List<Dictionary<string, object>> CreateParty()
    {
        var party = new List<Dictionary<string, object>>();

        string[] names = { "Warrior", "Mage", "Archer" };
        for (int i = 0; i < names.Length; i++)
        {
            party.Add(new Dictionary<string, object>
            {
                { "slot", i },
                { "name", names[i] },
                { "level", 10 + i * 5 }
            });
        }

        return party;
    }
}
"#;

    // 创建玩家
    let result = host.compile_and_execute(code, "player_system")?;
    println!("玩家系统: {:?}", result);

    Ok(())
}

/// 示例4：编译缓存性能演示
#[cfg(feature = "csharp")]
fn example_compile_cache(host: &DotNetCliHost) -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
using System;

public class CachedScript
{
    public static string Process(string input)
    {
        return $"Processed: {input}";
    }
}
"#;

    println!("首次执行（需要编译）...");
    let start = std::time::Instant::now();
    host.compile_and_execute(code, "cached_script")?;
    let first_time = start.elapsed();
    println!("⏱️  首次执行耗时: {:?}", first_time);

    println!("\n重复执行（使用缓存）...");
    let start = std::time::Instant::now();
    host.compile_and_execute(code, "cached_script")?;
    let cached_time = start.elapsed();
    println!("⚡ 缓存命中耗时: {:?}", cached_time);

    let speedup = first_time.as_secs_f64() / cached_time.as_secs_f64().max(0.0001);
    println!("📈 性能提升: {:.1}x", speedup);

    Ok(())
}

/// 示例5：缓存统计
#[cfg(feature = "csharp")]
fn example_cache_statistics(host: &DotNetCliHost) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(stats) = host.get_cache_stats() {
        println!("缓存统计:");
        println!("  命中次数: {}", stats.hits);
        println!("  未命中次数: {}", stats.misses);
        println!("  编译次数: {}", stats.compiles);
        println!("  淘汰次数: {}", stats.evictions);

        let hit_rate = host.get_cache_hit_rate();
        println!("  命中率: {:.1}%", hit_rate * 100.0);
    } else {
        println!("缓存统计不可用");
    }

    Ok(())
}

// 非 csharp feature 的空实现
#[cfg(not(feature = "csharp"))]
fn main() {
    println!("⚠️  此示例需要启用 'csharp' feature");
    println!("运行: cargo run --example csharp_example --features csharp");
}
