#!/usr/bin/env node

/**
 * 组件迁移辅助脚本
 *
 * 功能:
 * 1. 查找所有旧的组件导入
 * 2. 生成迁移报告
 * 3. 自动替换简单的导入路径
 * 4. 创建待办事项清单
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

// 配置
const CONFIG = {
  srcDir: './src',
  componentsDir: './src/components',
  outputFile: './COMPONENT_MIGRATION_REPORT.md',
  dryRun: false, // 设为true时不修改文件，只生成报告
};

// 路径映射表
const PATH_MAPPINGS = [
  {
    old: 'components/ui/Button',
    new: 'components/molecules/Button',
    type: 'molecule',
  },
  {
    old: 'components/ui/Spinner',
    new: 'components/atoms/Spinner',
    type: 'atom',
  },
  {
    old: 'components/ui/Skeleton',
    new: 'components/atoms/Skeleton',
    type: 'atom',
  },
  {
    old: 'components/ui/EmptyState',
    new: 'components/organisms/EmptyState',
    type: 'organism',
  },
  {
    old: 'components/Toolbar',
    new: 'components/organisms/Toolbar',
    type: 'organism',
  },
  {
    old: 'components/EntityTree',
    new: 'components/organisms/EntityTree',
    type: 'organism',
  },
  {
    old: 'components/PropertyInspector',
    new: 'components/organisms/PropertyInspector',
    type: 'organism',
  },
  {
    old: 'components/Timeline',
    new: 'components/organisms/Timeline',
    type: 'organism',
  },
  {
    old: 'components/AssetBrowser',
    new: 'components/organisms/AssetBrowser',
    type: 'organism',
  },
  {
    old: 'components/Toast',
    new: 'components/organisms/Toast',
    type: 'organism',
  },
];

/**
 * 递归查找目录下的所有文件
 */
function findFiles(dir, extension = '.tsx,.ts,.jsx,.js') {
  const extensions = extension.split(',').map(ext => ext.trim());
  const files = [];

  function traverse(currentDir) {
    const items = fs.readdirSync(currentDir);

    for (const item of items) {
      const fullPath = path.join(currentDir, item);
      const stat = fs.statSync(fullPath);

      if (stat.isDirectory()) {
        // 跳过node_modules和.git
        if (item !== 'node_modules' && item !== '.git' && item !== 'dist') {
          traverse(fullPath);
        }
      } else if (stat.isFile()) {
        const ext = path.extname(item);
        if (extensions.includes(ext)) {
          files.push(fullPath);
        }
      }
    }
  }

  traverse(dir);
  return files;
}

/**
 * 查找文件中的旧组件导入
 */
function findOldImports(filePath) {
  const content = fs.readFileSync(filePath, 'utf-8');
  const findings = [];

  for (const mapping of PATH_MAPPINGS) {
    // 匹配 import 语句
    const importRegex = new RegExp(
      `import\\s+.*?\\s+from\\s+['"](${mapping.old.replace('/', '\\/')})['"]`,
      'g'
    );

    let match;
    while ((match = importRegex.exec(content)) !== null) {
      findings.push({
        oldPath: match[1],
        newPath: mapping.new,
        type: mapping.type,
        line: content.substring(0, match.index).split('\n').length,
        fullMatch: match[0],
      });
    }
  }

  return findings;
}

/**
 * 替换文件中的导入路径
 */
function replaceImports(filePath, findings) {
  if (findings.length === 0) return 0;

  let content = fs.readFileSync(filePath, 'utf-8');
  let replaceCount = 0;

  for (const finding of findings) {
    const escapedOldPath = finding.oldPath.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    const importRegex = new RegExp(
      `(import\\s+.*?\\s+from\\s+['"])${escapedOldPath}(['"])`,
      'g'
    );

    const newContent = content.replace(importRegex, `$1${finding.newPath}$2`);

    if (newContent !== content) {
      replaceCount++;
      content = newContent;
    }
  }

  if (replaceCount > 0 && !CONFIG.dryRun) {
    fs.writeFileSync(filePath, content, 'utf-8');
    console.log(`✅ 更新文件: ${filePath}`);
  }

  return replaceCount;
}

/**
 * 生成迁移报告
 */
function generateReport(allFindings) {
  const report = [];

  report.push('# 组件迁移报告\n');
  report.push(`生成时间: ${new Date().toLocaleString()}\n`);
  report.push(`总计发现 ${allFindings.length} 处需要迁移的导入\n`);

  // 按类型统计
  const byType = {};
  for (const finding of allFindings) {
    if (!byType[finding.type]) {
      byType[finding.type] = [];
    }
    byType[finding.type].push(finding);
  }

  report.push('\n## 按组件类型统计\n');

  for (const [type, findings] of Object.entries(byType)) {
    report.push(`\n### ${type.charAt(0).toUpperCase() + type.slice(1)} (${findings.length} 处)\n`);
    report.push('| 文件路径 | 旧路径 | 新路径 | 行号 |\n');
    report.push('|---------|--------|--------|------|\n');

    for (const finding of findings) {
      const relativePath = path.relative(process.cwd(), finding.filePath);
      report.push(
        `| \`${relativePath}\` | \`${finding.oldPath}\` | \`${finding.newPath}\` | ${finding.line} |\n`
      );
    }
  }

  // 按文件统计
  const byFile = {};
  for (const finding of allFindings) {
    if (!byFile[finding.filePath]) {
      byFile[finding.filePath] = [];
    }
    byFile[finding.filePath].push(finding);
  }

  report.push('\n## 按文件统计\n');

  for (const [filePath, findings] of Object.entries(byFile)) {
    const relativePath = path.relative(process.cwd(), filePath);
    report.push(`\n### ${relativePath} (${findings.length} 处)\n`);

    for (const finding of findings) {
      report.push(`- 行 ${finding.line}: \`${finding.oldPath}\` → \`${finding.newPath}\`\n`);
    }
  }

  return report.join('');
}

/**
 * 生成待办事项清单
 */
function generateTodoList(allFindings) {
  const todo = [];

  todo.push('# 组件迁移待办事项\n');
  todo.push('\n## 优先级 P0 - 核心组件\n');

  const priorities = {
    Button: 'P0',
    Toolbar: 'P0',
    EntityTree: 'P0',
    PropertyInspector: 'P0',
    Icon: 'P0',
    Text: 'P0',
    Input: 'P0',
  };

  const grouped = {};
  for (const finding of allFindings) {
    const componentName = finding.oldPath.split('/').pop();
    const priority = priorities[componentName] || 'P1';

    if (!grouped[priority]) {
      grouped[priority] = {};
    }

    if (!grouped[priority][componentName]) {
      grouped[priority][componentName] = [];
    }

    grouped[priority][componentName].push(finding);
  }

  for (const [priority, components] of Object.entries(grouped).sort()) {
    todo.push(`\n### ${priority} 组件\n`);

    for (const [componentName, findings] of Object.entries(components)) {
      const fileCount = new Set(findings.map(f => f.filePath)).size;
      todo.push(
        `\n- [ ] **${componentName}** - ${findings.length} 个导入, ${fileCount} 个文件\n`
      );

      // 列出需要修改的文件
      const files = [...new Set(findings.map(f => path.relative(process.cwd(), f.filePath)))];
      for (const file of files) {
        todo.push(`  - [ ] \`${file}\`\n`);
      }
    }
  }

  todo.push('\n## 验证清单\n');
  todo.push('\n### 每个组件迁移后\n');
  todo.push('- [ ] 运行 TypeScript 编译检查\n');
  todo.push('- [ ] 运行单元测试\n');
  todo.push('- [ ] 手动测试相关功能\n');
  todo.push('- [ ] 检查控制台无错误\n');
  todo.push('- [ ] 更新相关文档\n');

  return todo.join('');
}

/**
 * 主函数
 */
function main() {
  console.log('🔍 开始扫描组件导入...\n');

  // 查找所有文件
  console.log('📁 扫描源文件...');
  const files = findFiles(CONFIG.srcDir);
  console.log(`找到 ${files.length} 个源文件\n`);

  // 查找旧的导入
  console.log('🔍 查找旧的组件导入...');
  const allFindings = [];

  for (const file of files) {
    const findings = findOldImports(file);
    for (const finding of findings) {
      finding.filePath = file;
      allFindings.push(finding);
    }
  }

  console.log(`找到 ${allFindings.length} 处需要迁移的导入\n`);

  if (allFindings.length === 0) {
    console.log('✅ 没有需要迁移的导入!');
    return;
  }

  // 生成报告
  console.log('📊 生成迁移报告...');
  const report = generateReport(allFindings);
  fs.writeFileSync(CONFIG.outputFile, report, 'utf-8');
  console.log(`报告已保存到: ${CONFIG.outputFile}\n`);

  // 生成待办事项
  const todoFile = './COMPONENT_MIGRATION_TODOS.md';
  console.log('📝 生成待办事项...');
  const todoList = generateTodoList(allFindings);
  fs.writeFileSync(todoFile, todoList, 'utf-8');
  console.log(`待办事项已保存到: ${todoFile}\n`);

  // 如果不是dry run,执行替换
  if (!CONFIG.dryRun) {
    console.log('🔧 开始替换导入路径...\n');

    let totalReplaced = 0;
    const processedFiles = new Set();

    for (const finding of allFindings) {
      if (!processedFiles.has(finding.filePath)) {
        const fileFindings = allFindings.filter(f => f.filePath === finding.filePath);
        const count = replaceImports(finding.filePath, fileFindings);
        totalReplaced += count;
        processedFiles.add(finding.filePath);
      }
    }

    console.log(`\n✅ 完成! 总共替换了 ${totalReplaced} 处导入\n`);
    console.log('📋 下一步:');
    console.log('  1. 检查生成的迁移报告');
    console.log('  2. 运行 npm run type-check 检查类型错误');
    console.log('  3. 运行 npm test 检查测试');
    console.log('  4. 手动测试应用功能\n');
  } else {
    console.log('🔍 Dry run模式 - 不会修改文件\n');
    console.log('📋 下一步:');
    console.log('  1. 检查生成的迁移报告');
    console.log('  2. 确认无误后，设置 dryRun: false 重新运行\n');
  }
}

// 解析命令行参数
const args = process.argv.slice(2);
if (args.includes('--dry-run') || args.includes('-d')) {
  CONFIG.dryRun = true;
  console.log('🔍 Dry run模式\n');
}

if (args.includes('--help') || args.includes('-h')) {
  console.log(`
组件迁移辅助脚本

用法:
  node scripts/migrate-components.js [选项]

选项:
  --dry-run, -d    不修改文件，只生成报告
  --help, -h       显示帮助信息

示例:
  node scripts/migrate-components.js              # 执行迁移
  node scripts/migrate-components.js --dry-run    # 生成报告但不修改文件
`);
  process.exit(0);
}

// 运行
try {
  main();
} catch (error) {
  console.error('❌ 错误:', error.message);
  process.exit(1);
}
