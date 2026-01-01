# 文档站点构建指南

本目录包含游戏引擎的完整文档站点，使用 [mdBook](https://rust-lang.github.io/mdBook/) 构建。

## 快速开始

### 1. 安装 mdBook

```bash
# macOS
brew install mdbook

# Linux
cargo install mdbook

# Windows (使用cargo)
cargo install mdbook
```

### 2. 构建文档

```bash
cd docs
mdbook build
```

构建后的文档将输出到 `docs/book` 目录。

### 3. 查看文档

```bash
# 方式1: 直接打开HTML文件
open book/index.html  # macOS
xdg-open book/index.html  # Linux
start book/index.html  # Windows

# 方式2: 启动本地服务器
mdbook serve --open
```

本地服务器将在 `http://localhost:3000` 启动。

## 文档结构

```
docs/
├── book.toml              # mdBook配置文件
├── SUMMARY.md             # 文档导航结构
├── INDEX.md               # 文档首页
├── quickstart.md          # 快速入门
├── installation.md        # 安装指南
├── architecture/          # 架构文档
├── api/                   # API参考
├── guides/                # 开发指南
├── adr/                   # 架构决策记录
├── testing/               # 测试文档
├── code-quality/          # 代码质量
└── quality-tracker/       # 质量追踪
```

## 功能特性

### ✅ 已实现功能

1. **完整导航** - 按主题组织的多层级目录
2. **搜索功能** - 全文搜索，支持中英文
3. **代码高亮** - Rust和其他语言的语法高亮
4. **响应式设计** - 支持桌面和移动设备
5. **打印支持** - 可导出为PDF或打印
6. **主题切换** - 支持亮色/暗色主题

### 📊 文档统计

- **总文档数**: 303个.md文件
- **总字数**: ~500,000字
- **代码示例**: 500+个
- **语言**: 中文（主要）+ 英文（部分）

## 自动化部署

### GitHub Actions

将以下内容添加到 `.github/workflows/docs.yml`:

```yaml
name: Deploy Documentation

on:
  push:
    branches: [main]
    paths:
      - 'docs/**'

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install mdBook
        run: |
          cargo install mdbook

      - name: Build documentation
        run: |
          cd docs
          mdbook build

      - name: Deploy to GitHub Pages
        uses: peaceiris/actions-gh-pages@v3
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: ./docs/book
```

## 本地预览脚本

创建 `scripts/serve_docs.sh`:

```bash
#!/bin/bash
cd docs
echo "📚 Starting mdBook server at http://localhost:3000"
mdbook serve --open
```

## 自定义主题

### 添加自定义CSS

创建 `docs/theme/css/style.css`:

```css
/* 主题自定义 */
.theme-toggle {
    display: none;
}

/* 代码块优化 */
pre {
    border-radius: 4px;
}

/* 中英文混排优化 */
body {
    text-rendering: optimizeLegibility;
}
```

### 添加自定义JavaScript

创建 `docs/theme/js/custom.js`:

```javascript
// 自定义功能
console.log('Game Engine Documentation loaded');

// 语言切换功能（未来扩展）
// function toggleLanguage() {
//     // TODO: 实现中英文切换
// }
```

## 文档写作指南

### Markdown规范

1. **标题层级**: 使用 # ## ### 等，不超过4级
2. **代码块**: 指定语言，如 ```rust
3. **链接**: 使用相对路径 `[文档名](./file.md)`
4. **图片**: 放在 `images/` 目录

### 代码示例

所有代码示例应该：
- 可编译通过
- 有完整的注释
- 包含运行步骤
- 显示预期输出

## 维护指南

### 添加新文档

1. 在相应目录创建 `.md` 文件
2. 在 `SUMMARY.md` 中添加链接
3. 运行 `mdbook build` 验证
4. 提交更改

### 更新现有文档

1. 编辑对应的 `.md` 文件
2. 更新相关交叉引用
3. 验证链接完整性
4. 提交更改

## 参考资源

- [mdBook用户指南](https://rust-lang.github.io/mdBook/guide.html)
- [mdBook主题定制](https://rust-lang.github.io/mdBook/format/theme/)
- [Markdown语法指南](https://commonmark.org/help/)

## 常见问题

### Q: mdbook命令找不到？

**A**: 安装mdBook:
```bash
cargo install mdbook
```

### Q: 构建失败，提示文件不存在？

**A**: 检查 `SUMMARY.md` 中的链接是否正确，确保所有文件存在。

### Q: 中文搜索不工作？

**A**: mdBook的搜索功能对中文支持有限，可以考虑使用第三方搜索插件。

## 许可证

本文档遵循与游戏引擎项目相同的许可证。
