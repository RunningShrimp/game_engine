# Makefile for game_engine project

.PHONY: help test test-all coverage coverage-html coverage-clean coverage-baseline benchmark lint format check

help: ## 显示帮助信息
	@echo "游戏引擎 Makefile 命令:"
	@echo ""
	@echo "测试相关:"
	@echo "  make test              - 运行单元测试"
	@echo "  make test-all          - 运行所有测试(包括集成测试)"
	@echo ""
	@echo "覆盖率相关:"
	@echo "  make coverage           - 生成代码覆盖率报告(Lcov格式)"
	@echo "  make coverage-html       - 生成HTML格式的代码覆盖率报告"
	@echo "  make coverage-baseline   - 建立代码覆盖率基线"
	@echo "  make coverage-clean      - 清理覆盖率数据"
	@echo ""
	@echo "基准测试:"
	@echo "  make benchmark          - 运行性能基准测试"
	@echo ""
	@echo "代码质量:"
	@echo "  make lint               - 运行 linter 检查"
	@echo "  make format             - 格式化代码"
	@echo "  make check              - 运行 cargo check"
	@echo ""
	@echo "构建:"
	@echo "  make build              - 构建项目"
	@echo "  make build-release       - 构建Release版本"
	@echo ""
	@echo "清理:"
	@echo "  make clean              - 清理构建产物"

test: ## 运行单元测试
	cargo test --lib

test-all: ## 运行所有测试
	cargo test --workspace

coverage: ## 生成代码覆盖率报告
	@./scripts/coverage.sh

coverage-html: ## 生成HTML格式的代码覆盖率报告
	@./scripts/coverage.sh --html

coverage-baseline: ## 建立代码覆盖率基线
	@./scripts/establish_coverage_baseline.sh

coverage-clean: ## 清理覆盖率数据
	@./scripts/coverage.sh --clean

benchmark: ## 运行性能基准测试
	cargo test --workspace --release -- --bench --test-threads=1

lint: ## 运行 linter 检查
	cargo clippy --workspace -- -D warnings

format: ## 格式化代码
	cargo fmt --all

check: ## 运行 cargo check
	cargo check --workspace

build: ## 构建项目
	cargo build --workspace

build-release: ## 构建Release版本
	cargo build --workspace --release

clean: ## 清理构建产物
	cargo clean --workspace
