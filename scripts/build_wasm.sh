#!/bin/bash
# WebAssembly构建脚本
#
# 用于构建游戏引擎的WebAssembly版本
# 支持开发和生产构建

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 默认配置
TARGET="wasm32-unknown-unknown"
PROFILE="release"
OUTPUT_DIR="dist"
EXAMPLE="wasm_example"

# 解析命令行参数
while [[ $# -gt 0 ]]; do
    case $1 in
        --dev)
            PROFILE="dev"
            shift
            ;;
        --release)
            PROFILE="release"
            shift
            ;;
        --output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        --example)
            EXAMPLE="$2"
            shift 2
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --dev          Build in development mode (default: release)"
            echo "  --release      Build in release mode"
            echo "  --output DIR   Output directory (default: dist)"
            echo "  --example NAME Example to build (default: wasm_example)"
            echo "  --help         Show this help message"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

echo -e "${GREEN}=== Building WebAssembly Example ===${NC}"
echo "Target: $TARGET"
echo "Profile: $PROFILE"
echo "Output: $OUTPUT_DIR"
echo "Example: $EXAMPLE"
echo ""

# 检查wasm-pack是否安装
if ! command -v wasm-pack &> /dev/null; then
    echo -e "${YELLOW}wasm-pack not found. Installing...${NC}"
    cargo install wasm-pack
fi

# 检查wasm32 target是否安装
if ! rustup target list --installed | grep -q "$TARGET"; then
    echo -e "${YELLOW}Installing wasm32 target...${NC}"
    rustup target add $TARGET
fi

# 创建输出目录
mkdir -p "$OUTPUT_DIR"

# 构建参数
BUILD_ARGS=(
    "build"
    "--target" "web"
    "--out-dir" "$OUTPUT_DIR"
)

if [ "$PROFILE" = "release" ]; then
    BUILD_ARGS+=("--release")
else
    BUILD_ARGS+=("--dev")
fi

# 运行wasm-pack构建
echo -e "${GREEN}Running wasm-pack build...${NC}"
cd game_engine

if wasm-pack "${BUILD_ARGS[@]}"; then
    echo -e "${GREEN}Build successful!${NC}"
else
    echo -e "${RED}Build failed!${NC}"
    exit 1
fi

cd ..

# 生成HTML文件
echo -e "${GREEN}Generating HTML file...${NC}"
cat > "$OUTPUT_DIR/index.html" << 'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Game Engine - WASM Example</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 100vh;
            overflow: hidden;
        }
        
        #container {
            position: relative;
            width: 100vw;
            height: 100vh;
        }
        
        #game-canvas {
            display: block;
            width: 100%;
            height: 100%;
            background: #000;
        }
        
        #loading {
            position: absolute;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
            color: white;
            font-size: 24px;
            text-align: center;
        }
        
        #loading::after {
            content: '...';
            animation: dots 1.5s steps(4, end) infinite;
        }
        
        @keyframes dots {
            0%, 20% { content: '.'; }
            40% { content: '..'; }
            60%, 100% { content: '...'; }
        }
        
        #info {
            position: absolute;
            top: 10px;
            left: 10px;
            color: white;
            background: rgba(0, 0, 0, 0.5);
            padding: 10px;
            border-radius: 5px;
            font-size: 12px;
            display: none;
        }
        
        #info.show {
            display: block;
        }
    </style>
</head>
<body>
    <div id="container">
        <canvas id="game-canvas"></canvas>
        <div id="loading">Loading Game Engine</div>
        <div id="info">
            <div>FPS: <span id="fps">0</span></div>
            <div>Memory: <span id="memory">0</span> MB</div>
        </div>
    </div>
    
    <script type="module">
        import init, { start } from './game_engine.js';
        
        const canvas = document.getElementById('game-canvas');
        const loading = document.getElementById('loading');
        const info = document.getElementById('info');
        const fpsElement = document.getElementById('fps');
        const memoryElement = document.getElementById('memory');
        
        // 设置canvas大小
        function resizeCanvas() {
            canvas.width = window.innerWidth;
            canvas.height = window.innerHeight;
        }
        
        resizeCanvas();
        window.addEventListener('resize', resizeCanvas);
        
        // 性能监控
        let lastTime = performance.now();
        let frameCount = 0;
        let fps = 0;
        
        function updatePerformance() {
            frameCount++;
            const currentTime = performance.now();
            const delta = currentTime - lastTime;
            
            if (delta >= 1000) {
                fps = Math.round((frameCount * 1000) / delta);
                fpsElement.textContent = fps;
                frameCount = 0;
                lastTime = currentTime;
                
                // 更新内存使用（如果可用）
                if (performance.memory) {
                    const memoryMB = (performance.memory.usedJSHeapSize / 1024 / 1024).toFixed(2);
                    memoryElement.textContent = memoryMB;
                }
            }
            
            requestAnimationFrame(updatePerformance);
        }
        
        // 初始化引擎
        async function run() {
            try {
                console.log('Initializing game engine...');
                await init();
                console.log('Game engine initialized');
                
                // 隐藏加载提示
                loading.style.display = 'none';
                info.classList.add('show');
                
                // 启动性能监控
                updatePerformance();
                
                // 启动游戏
                start();
            } catch (error) {
                console.error('Failed to initialize game engine:', error);
                loading.textContent = 'Failed to load game engine. Check console for details.';
                loading.style.color = '#ff4444';
            }
        }
        
        run();
    </script>
</body>
</html>
EOF

echo -e "${GREEN}HTML file generated: $OUTPUT_DIR/index.html${NC}"

# 生成README
echo -e "${GREEN}Generating README...${NC}"
cat > "$OUTPUT_DIR/README.md" << EOF
# WebAssembly Build

This directory contains the WebAssembly build of the game engine.

## Running Locally

You can serve this directory using any HTTP server. For example:

\`\`\`bash
# Using Python
python3 -m http.server 8000

# Using Node.js (if you have http-server installed)
npx http-server -p 8000

# Using PHP
php -S localhost:8000
\`\`\`

Then open http://localhost:8000 in your browser.

## Files

- \`game_engine.js\` - Main JavaScript bindings
- \`game_engine_bg.wasm\` - WebAssembly binary
- \`index.html\` - HTML entry point
- \`*.wasm\` - Additional WebAssembly modules (if any)

## Browser Compatibility

- Chrome/Edge: Full support
- Firefox: Full support
- Safari: Full support (may require WebGL2)
- Mobile browsers: Supported with performance limitations

## Performance Tips

1. Use Chrome/Edge for best performance
2. Enable hardware acceleration in browser settings
3. Close other tabs to free up memory
4. Use release builds for production

## Building

To rebuild, run:

\`\`\`bash
./scripts/build_wasm.sh --release
\`\`\`

For development builds:

\`\`\`bash
./scripts/build_wasm.sh --dev
\`\`\`
EOF

echo -e "${GREEN}README generated: $OUTPUT_DIR/README.md${NC}"

# 显示构建信息
echo ""
echo -e "${GREEN}=== Build Complete ===${NC}"
echo "Output directory: $OUTPUT_DIR"
echo ""
echo "To run locally:"
echo "  cd $OUTPUT_DIR"
echo "  python3 -m http.server 8000"
echo "  Then open http://localhost:8000 in your browser"
echo ""

