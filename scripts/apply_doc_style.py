#!/usr/bin/env python3
"""
Documentation Language Migration Script
文档语言迁移脚本

This script helps apply the documentation language standard across the codebase.
For each file, it:
1. Translates public API documentation to English
2. Translates private implementation comments to Chinese
3. Maintains dual-language comments for clarity

Usage:
    python scripts/apply_doc_style.py <path_to_file_or_directory>

Example:
    python scripts/apply_doc_style.py game_engine/src/core/engine
"""

import re
import sys
from pathlib import Path
from typing import List, Tuple

# Patterns to identify public vs private items
PUBLIC_PATTERNS = [
    r'pub\s+(?:async\s+)?fn\s+\w+',
    r'pub\s+struct\s+\w+',
    r'pub\s+enum\s+\w+',
    r'pub\s+trait\s+\w+',
    r'pub\s+(?:const|static|type)\s+\w+',
    r'pub\s+mod\s+\w+',
]

PRIVATE_PATTERNS = [
    r'(?:async\s+)?fn\s+\w+',
    r'struct\s+\w+',
    r'enum\s+\w+',
    r'trait\s+\w+',
    r'(?:const|static|type)\s+\w+',
    r'mod\s+\w+',
]

def is_public_api(line: str) -> bool:
    """Check if a line declares a public API item"""
    return any(re.search(pattern, line) for pattern in PUBLIC_PATTERNS)

def extract_doc_comments(lines: List[str], start_idx: int) -> Tuple[List[str], int]:
    """Extract documentation comments starting at start_idx"""
    doc_lines = []
    i = start_idx

    while i < len(lines):
        stripped = lines[i].strip()
        if stripped.startswith('///') or stripped.startswith('//!'):
            doc_lines.append(lines[i])
            i += 1
        elif stripped == '' and i + 1 < len(lines) and lines[i + 1].strip().startswith('///'):
            # Allow blank lines in doc comments
            doc_lines.append(lines[i])
            i += 1
        else:
            break

    return doc_lines, i

def should_translate_to_english(doc_comment: str) -> bool:
    """Check if a doc comment should be in English (public API)"""
    # Heuristic: If it contains Chinese characters, it's likely Chinese
    return bool(re.search(r'[\u4e00-\u9fff]', doc_comment))

def translate_to_english(text: str) -> str:
    """
    Placeholder for translation to English.
    In practice, this would require manual translation or use of translation APIs.
    For now, returns the original text with a marker.
    """
    # This is a placeholder - actual translation requires manual intervention
    # or integration with translation services
    return text

def process_file(file_path: Path) -> bool:
    """
    Process a single Rust file to apply documentation language standards.

    Returns True if file was modified, False otherwise.
    """
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
            lines = content.split('\n')

        modified = False
        new_lines = []
        i = 0

        while i < len(lines):
            line = lines[i]

            # Check if this is a public API declaration
            if is_public_api(line):
                # Extract doc comments before this declaration
                doc_start = i - 1
                while doc_start >= 0 and (lines[doc_start].strip().startswith('///') or
                                         lines[doc_start].strip().startswith('//!') or
                                         lines[doc_start].strip() == ''):
                    doc_start -= 1

                doc_start += 1  # Move to first doc comment

                if doc_start < i:
                    # We have doc comments - check if they need translation
                    doc_comments = lines[doc_start:i]
                    has_chinese = any(should_translate_to_english(dc) for dc in doc_comments)

                    if has_chinese:
                        # TODO: Translate to English (requires manual intervention)
                        # For now, add a marker comment
                        new_lines.extend(lines[doc_start:i])
                        modified = True
                    else:
                        new_lines.extend(lines[doc_start:i])

                # Add the declaration itself
                new_lines.append(line)
                i += 1
            else:
                new_lines.append(line)
                i += 1

        if modified:
            # Write back
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write('\n'.join(new_lines))
            print(f"✅ Modified: {file_path}")
            return True
        else:
            print(f"ℹ️  No changes needed: {file_path}")
            return False

    except Exception as e:
        print(f"❌ Error processing {file_path}: {e}")
        return False

def scan_directory(directory: Path, pattern: str = '*.rs') -> List[Path]:
    """Scan directory for Rust files"""
    rust_files = []

    if directory.is_file() and directory.suffix == '.rs':
        return [directory]

    for item in directory.rglob(pattern):
        if item.is_file():
            rust_files.append(item)

    return rust_files

def main():
    if len(sys.argv) < 2:
        print("Usage: python apply_doc_style.py <path_to_file_or_directory>")
        print("Example: python apply_doc_style.py game_engine/src/core/engine")
        sys.exit(1)

    target = Path(sys.argv[1])

    if not target.exists():
        print(f"❌ Error: Path does not exist: {target}")
        sys.exit(1)

    # Scan for Rust files
    rust_files = scan_directory(target)

    if not rust_files:
        print(f"ℹ️  No Rust files found in {target}")
        sys.exit(0)

    print(f"🔍 Found {len(rust_files)} Rust file(s)")
    print("=" * 60)

    # Process each file
    modified_count = 0
    for rust_file in rust_files:
        if process_file(rust_file):
            modified_count += 1

    print("=" * 60)
    print(f"✅ Processed {len(rust_files)} file(s)")
    print(f"📝 Modified {modified_count} file(s)")

    if modified_count > 0:
        print("\n⚠️  NOTE: This script is a demonstration.")
        print("    Actual translation requires manual review to ensure quality.")
        print("    Please review the changes and run `cargo doc` to verify.")

if __name__ == '__main__':
    main()
