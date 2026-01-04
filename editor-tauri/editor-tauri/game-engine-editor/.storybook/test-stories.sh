#!/bin/bash

# Storybook Validation Script
# Tests Storybook configuration and stories

set -e

echo "🔍 Validating Storybook Configuration..."
echo ""

# Check if Storybook is installed
if [ ! -d "node_modules/@storybook" ]; then
  echo "❌ Storybook is not installed"
  echo "Run: npm install"
  exit 1
fi

echo "✅ Storybook dependencies installed"

# Check configuration files
echo ""
echo "📁 Checking Storybook configuration files..."

config_files=(
  ".storybook/main.ts"
  ".storybook/preview.ts"
  ".storybook/theme.ts"
  ".storybook/manager.ts"
)

for file in "${config_files[@]}"; do
  if [ -f "$file" ]; then
    echo "✅ $file"
  else
    echo "❌ $file not found"
    exit 1
  fi
done

# Check for story files
echo ""
echo "📖 Checking for story files..."

story_count=$(find src -name "*.stories.tsx" -o -name "*.stories.ts" | wc -l)

if [ $story_count -eq 0 ]; then
  echo "⚠️  No story files found"
else
  echo "✅ Found $story_count story file(s)"

  # List all story files
  find src -name "*.stories.tsx" -o -name "*.stories.ts" | while read -r file; do
    echo "   - $file"
  done
fi

# Validate TypeScript
echo ""
echo "🔷 Validating TypeScript..."

if npx tsc --noEmit --project tsconfig.json 2>&1 | grep -q "error"; then
  echo "❌ TypeScript errors found"
  npx tsc --noEmit --project tsconfig.json
  exit 1
else
  echo "✅ No TypeScript errors"
fi

# Build Storybook (dry run)
echo ""
echo "🏗️  Testing Storybook build..."

if npx storybook build --dry-run 2>&1 | grep -q "error"; then
  echo "❌ Storybook build errors found"
  npx storybook build --dry-run
  exit 1
else
  echo "✅ Storybook builds successfully"
fi

echo ""
echo "✨ All checks passed!"
echo ""
echo "📚 Next steps:"
echo "   Run Storybook:     npm run storybook"
echo "   Build Storybook:   npm run build-storybook"
echo ""
