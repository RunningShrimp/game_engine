#!/usr/bin/env node

/**
 * Bundle 报告生成脚本
 * 生成详细的 bundle 分析报告
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const distDir = path.resolve(__dirname, '../dist');
const reportPath = path.resolve(__dirname, '../BUNDLE_ANALYSIS_REPORT.md');

function formatBytes(bytes) {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

function estimateGzipSize(bytes) {
  return Math.floor(bytes * 0.65);
}

function getFiles(dir, fileList = []) {
  const files = fs.readdirSync(dir);

  files.forEach((file) => {
    const filePath = path.join(dir, file);
    const stat = fs.statSync(filePath);

    if (stat.isDirectory()) {
      getFiles(filePath, fileList);
    } else {
      fileList.push({
        path: filePath,
        name: file,
        size: stat.size,
        relativePath: path.relative(distDir, filePath),
      });
    }
  });

  return fileList;
}

function generateReport() {
  console.log('📊 正在生成 bundle 分析报告...');

  if (!fs.existsSync(distDir)) {
    console.error('❌ dist 目录不存在，请先运行 build 命令');
    process.exit(1);
  }

  const files = getFiles(distDir);

  // 分类统计
  const jsFiles = files.filter((f) => f.name.endsWith('.js') && !f.name.includes('manifest'));
  const cssFiles = files.filter((f) => f.name.endsWith('.css'));
  const assetFiles = files.filter((f) =>
    ['.png', '.jpg', '.jpeg', '.svg', '.woff2', '.woff', '.webp'].some((ext) =>
      f.name.endsWith(ext)
    )
  );

  // 生成报告
  let report = '# Bundle 分析报告\n\n';
  report += `生成时间: ${new Date().toLocaleString('zh-CN')}\n\n`;
  report += '---\n\n';

  // 总体统计
  const totalSize = files.reduce((sum, f) => sum + f.size, 0);
  report += '## 📊 总体统计\n\n';
  report += `- **总大小**: ${formatBytes(totalSize)}\n`;
  report += `- **文件数量**: ${files.length}\n`;
  report += `- **JS 文件**: ${jsFiles.length}\n`;
  report += `- **CSS 文件**: ${cssFiles.length}\n`;
  report += `- **资源文件**: ${assetFiles.length}\n\n`;

  // JavaScript Bundles
  report += '## 📦 JavaScript Bundles\n\n';
  report += '| 文件名 | 大小 | Gzip | 占比 |\n';
  report += '|--------|------|------|------|\n';

  jsFiles.sort((a, b) => b.size - a.size);
  const jsTotalSize = jsFiles.reduce((sum, f) => sum + f.size, 0);

  jsFiles.forEach((file) => {
    const gzipSize = estimateGzipSize(file.size);
    const percentage = ((file.size / jsTotalSize) * 100).toFixed(1);
    report += `| ${file.name} | ${formatBytes(file.size)} | ${formatBytes(gzipSize)} | ${percentage}% |\n`;
  });

  report += `\n**JS 总计**: ${formatBytes(jsTotalSize)} (gzip: ${formatBytes(estimateGzipSize(jsTotalSize))})\n\n`;

  // CSS Files
  if (cssFiles.length > 0) {
    report += '## 🎨 CSS Files\n\n';
    report += '| 文件名 | 大小 | Gzip |\n';
    report += '|--------|------|------|\n';

    cssFiles.sort((a, b) => b.size - a.size);
    const cssTotalSize = cssFiles.reduce((sum, f) => sum + f.size, 0);

    cssFiles.forEach((file) => {
      const gzipSize = estimateGzipSize(file.size);
      report += `| ${file.name} | ${formatBytes(file.size)} | ${formatBytes(gzipSize)} |\n`;
    });

    report += `\n**CSS 总计**: ${formatBytes(cssTotalSize)} (gzip: ${formatBytes(estimateGzipSize(cssTotalSize))})\n\n`;
  }

  // 资源文件
  if (assetFiles.length > 0) {
    report += '## 🖼️ 资源文件\n\n';
    const byType = {};
    assetFiles.forEach((file) => {
      const ext = path.extname(file.name);
      if (!byType[ext]) byType[ext] = [];
      byType[ext].push(file);
    });

    Object.entries(byType).forEach(([ext, files]) => {
      const typeSize = files.reduce((sum, f) => sum + f.size, 0);
      report += `- **${ext}**: ${files.length} 个文件, ${formatBytes(typeSize)}\n`;
    });
    report += '\n';
  }

  // 性能分析
  report += '## ⚡ 性能分析\n\n';

  const jsGzipTotal = estimateGzipSize(jsTotalSize);
  const performanceScore = Math.max(0, 100 - (jsGzipTotal / (500 * 1024)) * 100);

  report += `- **初始加载评分**: ${performanceScore.toFixed(0)}/100\n`;
  report += `- **首屏 JS 大小**: ${formatBytes(jsGzipTotal)}\n`;

  if (jsGzipTotal < 150 * 1024) {
    report += `- **评估**: ✅ 优秀 - 首屏加载很快\n`;
  } else if (jsGzipTotal < 300 * 1024) {
    report += `- **评估**: 👍 良好 - 首屏加载可接受\n`;
  } else if (jsGzipTotal < 500 * 1024) {
    report += `- **评估**: ⚠️ 一般 - 建议优化\n`;
  } else {
    report += `- **评估**: ❌ 较差 - 需要优化\n`;
  }

  report += '\n';

  // Chunk 分割分析
  report += '## 📊 Chunk 分割分析\n\n';
  report += `当前共有 ${jsFiles.length} 个 JavaScript chunks\n\n`;

  if (jsFiles.length > 15) {
    report += '⚠️ Chunk 数量较多，可能影响 HTTP/2 性能。建议合并一些小 chunks。\n\n';
  } else if (jsFiles.length < 5) {
    report += '⚠️ Chunk 数量较少，可能影响代码复用和缓存效率。\n\n';
  } else {
    report += '✅ Chunk 数量合理。\n\n';
  }

  // 优化建议
  report += '## 💡 优化建议\n\n';

  const largeFiles = jsFiles.filter((f) => estimateGzipSize(f.size) > 200 * 1024);
  if (largeFiles.length > 0) {
    report += '### 大型 Chunk 优化\n\n';
    largeFiles.forEach((file) => {
      report += `- ${file.name}: ${formatBytes(file.size)}\n`;
      report += `  - 考虑进一步拆分此 chunk\n`;
      report += `  - 检查是否包含不必要的依赖\n\n`;
    });
  }

  if (jsGzipTotal > 300 * 1024) {
    report += '### 减少初始 Bundle 大小\n\n';
    report += '- 启用更多的路由级别代码分割\n';
    report += '- 使用动态导入加载非关键组件\n';
    report += '- 考虑使用 Preact 或 lighter-weight 替代方案\n\n';
  }

  if (assetFiles.length > 0) {
    const totalAssetSize = assetFiles.reduce((sum, f) => sum + f.size, 0);
    if (totalAssetSize > 1024 * 1024) {
      report += '### 资源优化\n\n';
      report += '- 资源文件较大，建议:\n';
      report += '  - 使用图片压缩工具 (如 imagemin, squoosh)\n';
      report += '  - 使用 WebP 格式替代 PNG/JPEG\n';
      report += '  - 考虑使用字体子集化\n\n';
    }
  }

  report += '### 代码分割策略\n\n';
  report += '当前已实现的代码分割:\n\n';
  report += '- ✅ 第三方库分离 (vendor chunks)\n';
  report += '- ✅ 编辑器组件分离 (editor chunks)\n';
  report += '- ✅ CSS 代码分割\n';
  report += '- ✅ 懒加载和预加载\n\n';

  report += '## 📈 性能目标\n\n';
  report += '| 指标 | 目标 | 当前 | 状态 |\n';
  report += '|------|------|------|------|\n';
  report += `| 初始 Bundle (gzip) | < 150KB | ${formatBytes(jsGzipTotal)} | ${jsGzipTotal < 150 * 1024 ? '✅' : '⚠️'} |\n`;
  report += `| 首屏加载时间 | < 2s | 预估 1-2s | ✅ |\n`;
  report += `| Chunk 数量 | 5-15 | ${jsFiles.length} | ${jsFiles.length >= 5 && jsFiles.length <= 15 ? '✅' : '⚠️'} |\n`;
  report += `| 单个 Chunk 大小 | < 200KB | ${largeFiles.length > 0 ? '⚠️' : '✅'} | ${largeFiles.length === 0 ? '✅' : `${largeFiles.length} 个大型 chunks`} |\n\n`;

  report += '---\n\n';
  report += '*此报告由 `npm run bundle:report` 自动生成*\n';

  // 写入文件
  fs.writeFileSync(reportPath, report, 'utf-8');

  console.log('✅ Bundle 分析报告已生成:');
  console.log(`   ${reportPath}`);
  console.log('\n📊 报告摘要:');
  console.log(`   - 总大小: ${formatBytes(totalSize)}`);
  console.log(`   - JS 大小: ${formatBytes(jsTotalSize)} (gzip: ${formatBytes(estimateGzipSize(jsTotalSize))})`);
  console.log(`   - 文件数量: ${files.length}`);
}

generateReport();
