# 快速上手指南

本指南将帮助您快速设置并运行游戏引擎。

## 1. 环境要求

- Rust 1.70 或更高版本
- Cargo
- Git

## 2. 克隆项目

```bash
gh repo clone RunningShrimp/game_engine
cd game_engine
```

## 3. 编译和运行

### 本地运行

```bash
cargo run --release
```

### Web 平台运行

首先,安装 `wasm-pack`:

```bash
cargo install wasm-pack
```

然后,编译为 WebAssembly:

```bash
wasm-pack build --target web
```

最后,在 `pkg` 目录下启动一个简单的 Web 服务器,例如:

```bash
python3 -m http.server --directory pkg
```

然后在浏览器中访问 `http://localhost:8000`。

## 4. API 文档

API 文档位于 `target/doc/game_engine/index.html`。您可以在浏览器中打开此文件以查看完整的 API 参考。

## 5. 示例代码

`src/main.rs` 中包含一个简单的示例,展示了如何初始化引擎、创建实体和组件,以及运行主循环。

```rust
// src/main.rs

use game_engine::core::Engine;

fn main() {
    // 创建引擎实例
    let mut engine = Engine::new();

    // 创建一个实体,并添加 Transform 和 Sprite 组件
    engine.world.spawn((game_engine::ecs::Transform::default(), game_engine::ecs::Sprite::default()));

    // 运行引擎主循环
    engine.run();
}
```

## 6. 后续步骤

- 查阅 API 文档,了解更多关于引擎核心模块的信息。
- 修改 `src/main.rs` 以创建您自己的场景和游戏逻辑。
- 探索 `src/ecs` 和 `src/render` 目录,了解组件系统和渲染管线的实现。
