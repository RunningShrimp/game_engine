# WebAssembly构建指南

本文档说明如何构建和运行游戏引擎的WebAssembly版本。

## 概述

游戏引擎支持编译为WebAssembly，可以在现代浏览器中运行。WASM版本提供了：

- 高性能的Web游戏运行
- 跨平台兼容性
- 无需插件，直接在浏览器中运行
- 接近原生性能

## 前置要求

### 1. 安装Rust工具链

```bash
# 安装Rust（如果还没有）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装wasm32 target
rustup target add wasm32-unknown-unknown
```

### 2. 安装wasm-pack

```bash
cargo install wasm-pack
```

### 3. 验证安装

```bash
# 检查Rust版本
rustc --version

# 检查wasm32 target
rustup target list --installed | grep wasm32

# 检查wasm-pack
wasm-pack --version
```

## 构建WASM版本

### 快速开始

使用提供的构建脚本：

```bash
# 构建发布版本（推荐）
./scripts/build_wasm.sh --release

# 构建开发版本（包含调试信息）
./scripts/build_wasm.sh --dev

# 指定输出目录
./scripts/build_wasm.sh --release --output my_dist

# 构建特定示例
./scripts/build_wasm.sh --release --example wasm_example
```

### 手动构建

如果需要手动构建：

```bash
cd game_engine

# 构建WASM模块
wasm-pack build --target web --release --out-dir ../dist

# 或者开发模式
wasm-pack build --target web --dev --out-dir ../dist
```

## 运行WASM版本

### 本地服务器

WASM文件需要通过HTTP服务器提供，不能直接打开HTML文件。

#### 使用Python

```bash
cd dist
python3 -m http.server 8000
# 打开 http://localhost:8000
```

#### 使用Node.js

```bash
# 安装http-server（如果还没有）
npm install -g http-server

cd dist
http-server -p 8000
# 打开 http://localhost:8000
```

#### 使用PHP

```bash
cd dist
php -S localhost:8000
# 打开 http://localhost:8000
```

### 部署到生产环境

#### 静态托管

可以将`dist`目录部署到任何静态托管服务：

- **GitHub Pages**: 推送到gh-pages分支
- **Netlify**: 拖拽dist目录到Netlify
- **Vercel**: 使用Vercel CLI部署
- **Cloudflare Pages**: 连接GitHub仓库

#### 配置建议

1. **启用压缩**: 配置服务器压缩`.wasm`和`.js`文件
2. **缓存策略**: 为`.wasm`文件设置长期缓存
3. **MIME类型**: 确保服务器正确设置`.wasm`文件的MIME类型为`application/wasm`

## 性能优化

### 1. 使用发布构建

发布构建经过优化，性能更好：

```bash
./scripts/build_wasm.sh --release
```

### 2. 启用SIMD（如果支持）

在`Cargo.toml`中启用SIMD特性：

```toml
[target.'cfg(target_arch = "wasm32")'.dependencies]
# ... 其他依赖
```

构建时使用SIMD target：

```bash
RUSTFLAGS='-C target-feature=+simd128' wasm-pack build --target web --release
```

### 3. 优化二进制大小

使用`wasm-opt`进一步优化：

```bash
# 安装wasm-opt
npm install -g wasm-opt

# 优化WASM文件
wasm-opt -O3 dist/game_engine_bg.wasm -o dist/game_engine_bg_optimized.wasm
```

### 4. 使用内存池

WASM版本自动使用内存池优化，减少分配开销。

## 调试

### 浏览器开发者工具

1. 打开浏览器开发者工具（F12）
2. 查看Console标签页查看日志
3. 使用Sources标签页设置断点
4. 使用Performance标签页分析性能

### 使用开发构建

开发构建包含更多调试信息：

```bash
./scripts/build_wasm.sh --dev
```

### 启用详细日志

在代码中启用详细日志：

```rust
#[cfg(target_arch = "wasm32")]
use web_sys::console;

console::log_1(&"Debug message".into());
```

## 常见问题

### 1. 构建失败：找不到wasm-pack

**解决方案**: 安装wasm-pack
```bash
cargo install wasm-pack
```

### 2. 运行时错误：无法加载WASM模块

**可能原因**:
- 文件路径不正确
- 服务器MIME类型配置错误
- CORS问题

**解决方案**:
- 检查文件路径
- 确保使用HTTP服务器（不是file://）
- 检查浏览器控制台的错误信息

### 3. 性能问题

**可能原因**:
- 使用开发构建
- 浏览器硬件加速未启用
- 内存使用过高

**解决方案**:
- 使用发布构建
- 启用浏览器硬件加速
- 优化资源使用

### 4. 内存不足

**解决方案**:
- 减少同时加载的资源
- 使用资源压缩
- 实现资源卸载机制

## 浏览器兼容性

| 浏览器 | 最低版本 | 备注 |
|--------|---------|------|
| Chrome | 57+ | 完全支持 |
| Firefox | 52+ | 完全支持 |
| Safari | 11+ | 需要WebGL2 |
| Edge | 16+ | 完全支持 |

### 移动浏览器

移动浏览器支持有限，性能可能较低：

- iOS Safari: 需要iOS 11+
- Chrome Android: 需要Android 5+
- Firefox Android: 需要Android 5+

## 示例代码

### 基本使用

```javascript
import init, { start } from './game_engine.js';

async function run() {
    // 初始化引擎
    await init();
    
    // 启动游戏
    start();
}

run();
```

### 自定义配置

```javascript
import init from './game_engine.js';

async function run() {
    // 初始化引擎
    await init();
    
    // 获取canvas元素
    const canvas = document.getElementById('game-canvas');
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;
    
    // 启动游戏循环
    start();
}

run();
```

## 性能监控

WASM版本包含性能监控功能：

```javascript
// 在HTML中查看FPS和内存使用
// 性能信息会自动显示在页面上
```

## 最佳实践

1. **使用发布构建**: 生产环境始终使用发布构建
2. **启用压缩**: 配置服务器压缩WASM文件
3. **监控性能**: 使用浏览器性能工具监控性能
4. **优化资源**: 压缩纹理和模型资源
5. **渐进式加载**: 实现资源渐进式加载
6. **错误处理**: 添加适当的错误处理和用户提示

## 相关资源

- [WebAssembly官方文档](https://webassembly.org/)
- [wasm-pack文档](https://rustwasm.github.io/wasm-pack/)
- [Rust和WebAssembly](https://rustwasm.github.io/book/)
- [WebGL优化指南](https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/WebGL_best_practices)

## 故障排除

如果遇到问题：

1. 检查浏览器控制台的错误信息
2. 验证所有依赖已正确安装
3. 确保使用HTTP服务器（不是file://）
4. 检查浏览器兼容性
5. 查看构建日志中的警告和错误

## 支持

如有问题，请查看：
- 项目文档
- GitHub Issues
- 社区论坛

