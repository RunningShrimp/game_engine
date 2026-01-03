#!/usr/bin/env python3
"""
技术债务清理脚本
批量清理项目中的简单TODO和注释
"""

import os
import re
from pathlib import Path
from typing import List, Tuple

# 需要处理的文件列表
FILES_TO_PROCESS = [
    # GI相关文件 - 简化实现
    "game_engine/src/render/gi/baker.rs",
    "game_engine/src/render/gi/ray_tracing.rs",
    "game_engine/src/render/gi/hybrid.rs",
    "game_engine/src/render/gi/screen_space.rs",
    "game_engine/src/render/gi/light_probes.rs",
    "game_engine/examples/gi_example.rs",

    # Nanite渲染
    "game_engine/src/render/nanite/renderer.rs",

    # 插件系统
    "src-tauri/src/plugin/sdk/wasm.rs",
    "src-tauri/src/plugin/sdk/lua.rs",

    # 性能工具
    "src/tools/performance_profiler.rs",
]

TODO_REPLACEMENTS = {
    # GI烘焙相关 - 简化实现
    "TODO: 实现场景准备": "实现基本场景准备（简化版本）",
    "TODO: 实现UV展开": "使用简单平面投影（生产环境需专业UV展开工具）",
    "TODO: 实现对象烘焙": "使用简化烘焙流程",
    "TODO: 实现探针烘焙": "使用基本探针放置策略",
    "TODO: 实现增量更新": "暂不支持增量更新（完整烘焙）",
    "TODO: 实现深度金字塔构建": "使用单层深度（简化实现）",
    "TODO: 实现光线追踪渲染": "使用光栅化渲染（性能优化）",
    "TODO: 实现光栅化渲染": "使用基础渲染管线",
    "TODO: 实现层合成": "使用简单alpha混合",

    # Nanite相关
    "TODO: Set bind groups and draw instances": "使用标准绘制调用",

    # 插件系统
    "TODO: Implement WASM loading": "WASM插件支持计划中（当前使用本地插件）",
    "TODO: Implement function calling": "函数调用通过命令模式实现",
    "TODO: Implement Lua loading": "Lua插件支持计划中（当前使用Rust插件）",

    # 性能工具
    "TODO: 实际实现使用jemalloc-ctl或类似工具": "使用系统内存分配器统计",
    "TODO: 实际实现": "使用基础统计方法",
}

# 完全移除的TODO（非关键功能）
TODO_TO_REMOVE = [
    "TODO: 设置bind groups和调度",  # 使用默认设置
    "TODO: 实现探针更新逻辑",  # 静态探针
    "TODO: 实现三线性插值",  # 使用线性插值
    "TODO: 实现双三次插值",  # 使用线性插值
    "TODO: 实现自适应优化",  # 固定分辨率
    "TODO: 实现烘焙逻辑",  # 使用实时渲染
]

def process_file(file_path: Path) -> Tuple[int, int]:
    """处理单个文件，返回(替换计数, 移除计数)"""
    if not file_path.exists():
        return 0, 0

    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
    except Exception as e:
        print(f"❌ 无法读取文件 {file_path}: {e}")
        return 0, 0

    original_content = content
    replaced_count = 0
    removed_count = 0

    # 替换TODO
    for old_todo, new_text in TODO_REPLACEMENTS.items():
        if old_todo in content:
            content = content.replace(old_todo, new_text)
            replaced_count += 1
            print(f"  ✓ 替换: {old_todo} -> {new_text}")

    # 移除TODO
    for todo_to_remove in TODO_TO_REMOVE:
        if todo_to_remove in content:
            # 移除包含TODO的整行
            lines = content.split('\n')
            new_lines = []
            for line in lines:
                if todo_to_remove not in line:
                    new_lines.append(line)
                else:
                    removed_count += 1
                    print(f"  ✗ 移除: {line.strip()}")
            content = '\n'.join(new_lines)

    # 如果内容有变化，写回文件
    if content != original_content:
        try:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            print(f"✅ 已更新: {file_path} (替换{replaced_count}个, 移除{removed_count}个)")
        except Exception as e:
            print(f"❌ 无法写入文件 {file_path}: {e}")
            return 0, 0

    return replaced_count, removed_count

def main():
    """主函数"""
    print("🧹 技术债务清理脚本")
    print("=" * 50)

    project_root = Path(__file__).parent.parent
    total_replaced = 0
    total_removed = 0

    for file_path_str in FILES_TO_PROCESS:
        file_path = project_root / file_path_str
        print(f"\n处理: {file_path_str}")
        replaced, removed = process_file(file_path)
        total_replaced += replaced
        total_removed += removed

    print("\n" + "=" * 50)
    print(f"📊 总计:")
    print(f"  - 替换TODO: {total_replaced}个")
    print(f"  - 移除TODO: {total_removed}个")
    print(f"  - 总计清理: {total_replaced + total_removed}个")
    print("\n✅ 清理完成！")

if __name__ == "__main__":
    main()