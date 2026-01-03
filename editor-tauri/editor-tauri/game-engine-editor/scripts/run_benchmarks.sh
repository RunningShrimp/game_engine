#!/bin/bash
# Benchmark Execution Script
#
# Runs the complete benchmark suite with various options

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default options
BENCHMARK_DIR="benches"
OUTPUT_DIR="target/criterion"
SAVE_BASELINE=false
COMPARE_BASELINE=false
BASELINE_NAME="main"
VERBOSE=false
GENERATE_FLAMEGRAPHS=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --save-baseline)
            SAVE_BASELINE=true
            BASELINE_NAME="$2"
            shift 2
            ;;
        --compare)
            COMPARE_BASELINE=true
            BASELINE_NAME="$2"
            shift 2
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --flamegraphs)
            GENERATE_FLAMEGRAPHS=true
            shift
            ;;
        --gpu-only)
            GPU_ONLY=true
            shift
            ;;
        --editor-only)
            EDITOR_ONLY=true
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --save-baseline NAME    Save results as baseline"
            echo "  --compare NAME          Compare against baseline"
            echo "  --verbose               Verbose output"
            echo "  --flamegraphs           Generate flamegraphs"
            echo "  --gpu-only              Run only GPU benchmarks"
            echo "  --editor-only           Run only editor benchmarks"
            echo "  --help                  Show this help message"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Game Engine Editor Benchmark Suite${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""

# Check if cargo-criterion is installed
if ! command -v cargo-criterion &> /dev/null; then
    echo -e "${YELLOW}Installing cargo-criterion...${NC}"
    cargo install cargo-criterion
fi

# Change to benchmark directory
cd "$BENCHMARK_DIR"

# Build benchmarks first
echo -e "${YELLOW}Building benchmarks...${NC}"
cargo build --benches
echo -e "${GREEN}Build complete!${NC}"
echo ""

# Prepare cargo bench command
BENCH_CMD="cargo bench"

if [ "$VERBOSE" = true ]; then
    BENCH_CMD="$BENCH_CMD -- --verbose"
fi

if [ "$SAVE_BASELINE" = true ]; then
    BENCH_CMD="$BENCH_CMD -- --save-baseline $BASELINE_NAME"
    echo -e "${YELLOW}Saving baseline: $BASELINE_NAME${NC}"
fi

if [ "$COMPARE_BASELINE" = true ]; then
    BENCH_CMD="$BENCH_CMD -- --baseline $BASELINE_NAME"
    echo -e "${YELLOW}Comparing against baseline: $BASELINE_NAME${NC}"
fi

# Run benchmarks
if [ "$GPU_ONLY" = true ]; then
    echo -e "${GREEN}Running GPU benchmarks...${NC}"
    eval "$BENCH_CMD --bench gpu_benchmark"
elif [ "$EDITOR_ONLY" = true ]; then
    echo -e "${GREEN}Running Editor benchmarks...${NC}"
    eval "$BENCH_CMD --bench editor_benchmark"
else
    echo -e "${GREEN}Running all benchmarks...${NC}"
    echo "This may take 15-30 minutes..."
    echo ""

    # Run individual benchmark suites
    echo -e "${YELLOW}[1/5] GPU Benchmarks...${NC}"
    eval "$BENCH_CMD --bench gpu_benchmark"

    echo -e "${YELLOW}[2/5] Editor Benchmarks...${NC}"
    eval "$BENCH_CMD --bench editor_benchmark"

    echo -e "${YELLOW}[3/5] Performance Benchmarks...${NC}"
    eval "$BENCH_CMD --bench performance_benchmark"

    echo -e "${YELLOW}[4/5] Memory Benchmarks...${NC}"
    eval "$BENCH_CMD --bench memory_benchmark"

    echo -e "${YELLOW}[5/5] Comprehensive Benchmarks...${NC}"
    eval "$BENCH_CMD --bench comprehensive_benchmark"
fi

echo ""
echo -e "${GREEN}Benchmarks complete!${NC}"
echo ""

# Generate flamegraphs if requested
if [ "$GENERATE_FLAMEGRAPHS" = true ]; then
    echo -e "${YELLOW}Generating flamegraphs...${NC}"

    if ! command -v cargo-flamegraph &> /dev/null; then
        echo -e "${YELLOW}Installing cargo-flamegraph...${NC}"
        cargo install cargo-flamegraph
    fi

    for bench in gpu editor performance memory comprehensive; do
        echo -e "${YELLOW}Generating flamegraph for ${bench}...${NC}"
        cargo flamegraph --bench ${bench}_benchmark -- --profile-time 30
    done

    echo -e "${GREEN}Flamegraphs generated in target/flamegraph/${NC}"
fi

# Generate summary
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Summary${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "Results location: $OUTPUT_DIR"
echo ""

if [ -d "$OUTPUT_DIR" ]; then
    # Count benchmarks
    BENCHMARK_COUNT=$(find "$OUTPUT_DIR" -mindepth 1 -maxdepth 1 -type d | wc -l)
    echo "Total benchmark groups: $BENCHMARK_COUNT"
    echo ""

    # Show HTML report location
    HTML_REPORT="$OUTPUT_DIR/report/index.html"
    if [ -f "$HTML_REPORT" ]; then
        echo "HTML report: $HTML_REPORT"
        echo ""
        echo "To view the report:"
        echo "  open $HTML_REPORT  # macOS"
        echo "  xdg-open $HTML_REPORT  # Linux"
        echo "  start $HTML_REPORT  # Windows"
    fi
fi

echo ""
echo -e "${GREEN}Done!${NC}"
