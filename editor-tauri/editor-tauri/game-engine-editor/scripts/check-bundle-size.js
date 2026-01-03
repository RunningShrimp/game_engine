#!/usr/bin/env node

/**
 * Bundle 大小检查脚本
 * 分析构建产物并生成大小报告
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const distDir = path.resolve(__dirname, '../dist');

// 文件大小格式化
function formatBytes(bytes) {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

// 估算 gzip 大小
function estimateGzipSize(bytes) {
  // 简单估算：gzip 压缩率约为 60-70%
  return Math.floor(bytes * 0.65);
}

// 递归读取目录
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

// 分析 JS bundle
function analyzeJSBundles(files) {
  const jsFiles = files.filter(
    (f) =>
      f.name.endsWith('.js') &&
      !f.name.includes('manifest') &&
      !f.relativePath.includes('assets/fonts')
  );

  console.log('\n📦 JavaScript Bundles:');
  console.log('─'.repeat(80));

  jsFiles
    .sort((a, b) => b.size - a.size)
    .forEach((file) => {
      const gzipSize = estimateGzipSize(file.size);
      const percentage = ((gzipSize / (500 * 1024)) * 100).toFixed(1);
      const status = gzipSize > 500 * 1024 ? '❌' : gzipSize > 300 * 1024 ? '⚠️' : '✅';

      console.log(
        `${status} ${file.name.padEnd(40)} ` +
          `${formatBytes(file.size).padStart(10)} ` +
          `(gzip: ${formatBytes(gzipSize).padStart(10)}) ` +
          `${percentage}%`
      );
    });

  const totalSize = jsFiles.reduce((sum, f) => sum + f.size, 0);
  const totalGzip = estimateGzipSize(totalSize);

  console.log('─'.repeat(80));
  console.log(
    `总计: ${formatBytes(totalSize)} (gzip: ${formatBytes(totalGzip)})`
  );

  return { totalSize, totalGzip, count: jsFiles.length };
}

// 分析 CSS 文件
function analyzeCSSFiles(files) {
  const cssFiles = files.filter((f) => f.name.endsWith('.css'));

  if (cssFiles.length === 0) {
    console.log('\n🎨 CSS Files: None');
    return { totalSize: 0, totalGzip: 0, count: 0 };
  }

  console.log('\n🎨 CSS Files:');
  console.log('─'.repeat(80));

  cssFiles
    .sort((a, b) => b.size - a.size)
    .forEach((file) => {
      const gzipSize = estimateGzipSize(file.size);
      console.log(
        `   ${file.name.padEnd(40)} ` +
          `${formatBytes(file.size).padStart(10)} ` +
          `(gzip: ${formatBytes(gzipSize).padStart(10)})`
      );
    });

  const totalSize = cssFiles.reduce((sum, f) => sum + f.size, 0);
  const totalGzip = estimateGzipSize(totalSize);

  console.log('─'.repeat(80));
  console.log(
    `总计: ${formatBytes(totalSize)} (gzip: ${formatBytes(totalGzip)})`
  );

  return { totalSize, totalGzip, count: cssFiles.length };
}

// 分析资源文件
function analyzeAssets(files) {
  const assetExtensions = ['.png', '.jpg', '.jpeg', '.svg', '.woff2', '.woff'];
  const assetFiles = files.filter((f) =>
    assetExtensions.some((ext) => f.name.endsWith(ext))
  );

  if (assetFiles.length === 0) {
    console.log('\n🖼️  Assets: None');
    return { totalSize: 0, count: 0 };
  }

  console.log('\n🖼️  Assets:');
  console.log('─'.repeat(80));

  const byType = {};
  assetFiles.forEach((file) => {
    const ext = path.extname(file.name);
    if (!byType[ext]) byType[ext] = [];
    byType[ext].push(file);
  });

  Object.entries(byType).forEach(([ext, files]) => {
    const typeSize = files.reduce((sum, f) => sum + f.size, 0);
    console.log(
      `   ${ext.padEnd(8)} ${files.length.toString().padStart(4)} files ` +
        `${formatBytes(typeSize).padStart(10)}`
    );
  });

  const totalSize = assetFiles.reduce((sum, f) => sum + f.size, 0);

  console.log('─'.repeat(80));
  console.log(`总计: ${formatBytes(totalSize)} (${assetFiles.length} files)`);

  return { totalSize, count: assetFiles.length };
}

// 性能评分
function calculateScore(js, css) {
  let score = 100;
  const warnings = [];
  const errors = [];

  // 检查 JS bundle 大小
  if (js.totalGzip > 500 * 1024) {
    score -= 30;
    errors.push(`JS bundle 总大小超过 500KB (gzip): ${formatBytes(js.totalGzip)}`);
  } else if (js.totalGzip > 300 * 1024) {
    score -= 15;
    warnings.push(`JS bundle 总大小较大 (gzip): ${formatBytes(js.totalGzip)}`);
  }

  // 检查单个 chunk 大小
  // ... (这里可以添加更详细的检查)

  return { score, warnings, errors };
}

// 主函数
function main() {
  console.log('\n📊 Bundle 分析报告');
  console.log('='.repeat(80));

  if (!fs.existsSync(distDir)) {
    console.error('\n❌ 错误: dist 目录不存在，请先运行 build 命令');
    console.log('   运行: npm run build');
    process.exit(1);
  }

  const files = getFiles(distDir);

  if (files.length === 0) {
    console.log('\n⚠️  警告: dist 目录为空');
    process.exit(0);
  }

  // 分析各类文件
  const jsStats = analyzeJSBundles(files);
  const cssStats = analyzeCSSFiles(files);
  const assetStats = analyzeAssets(files);

  // 性能评分
  const { score, warnings, errors } = calculateScore(jsStats, cssStats);

  // 显示评分
  console.log('\n⭐ 性能评分:');
  console.log('─'.repeat(80));
  const scoreEmoji = score >= 90 ? '🏆' : score >= 70 ? '👍' : score >= 50 ? '⚠️' : '❌';
  console.log(`${scoreEmoji} 得分: ${score}/100`);

  // 显示警告和错误
  if (errors.length > 0) {
    console.log('\n❌ 错误:');
    errors.forEach((err) => console.log(`   - ${err}`));
  }

  if (warnings.length > 0) {
    console.log('\n⚠️  警告:');
    warnings.forEach((warn) => console.log(`   - ${warn}`));
  }

  // 优化建议
  console.log('\n💡 优化建议:');
  if (jsStats.totalGzip > 500 * 1024) {
    console.log('   - 考虑进一步拆分大型组件');
    console.log('   - 检查是否有重复的依赖包');
    console.log('   - 使用动态导入减少初始加载');
  }
  if (jsStats.count > 10) {
    console.log(`   - 当前有 ${jsStats.count} 个 JS chunk，考虑合并较小的 chunk`);
  }
  if (assetStats.totalSize > 1024 * 1024) {
    console.log('   - 资源文件较大，考虑使用图片压缩工具');
    console.log('   - 使用 WebP 格式替代 PNG/JPEG');
  }

  console.log('\n' + '='.repeat(80));

  // 返回退出码
  if (errors.length > 0) {
    process.exit(1);
  } else if (warnings.length > 0) {
    process.exit(2);
  } else {
    process.exit(0);
  }
}

main();
