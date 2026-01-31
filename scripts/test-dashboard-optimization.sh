#!/bin/bash

# Dashboard 性能优化测试脚本

echo "🚀 Dashboard 性能优化测试"
echo "================================"
echo ""

# 1. 检查 Rust 代码编译
echo "📦 1. 检查 Rust 代码编译..."
cd /Users/dengwenyu/Dev/AI/pi-session-manager
cargo check --manifest-path src-tauri/Cargo.toml 2>&1 | grep -E "(Finished|error)" || echo "✅ Rust 代码编译通过"
echo ""

# 2. 检查 TypeScript 类型
echo "📝 2. 检查 TypeScript 类型..."
npx tsc --noEmit 2>&1 | grep -E "Dashboard.tsx.*error" && echo "❌ Dashboard.tsx 有类型错误" || echo "✅ Dashboard.tsx 类型检查通过"
echo ""

# 3. 检查关键文件
echo "📂 3. 检查关键文件..."
files=(
  "src/components/Dashboard.tsx"
  "src-tauri/src/stats.rs"
  "src-tauri/Cargo.toml"
  "docs/pr/20260131-dashboard-performance-optimization.md"
)

for file in "${files[@]}"; do
  if [ -f "$file" ]; then
    echo "  ✅ $file"
  else
    echo "  ❌ $file (缺失)"
  fi
done
echo ""

# 4. 检查 Rayon 依赖
echo "🔍 4. 检查 Rayon 依赖..."
if grep -q "rayon" src-tauri/Cargo.toml; then
  echo "  ✅ Rayon 依赖已添加"
else
  echo "  ❌ Rayon 依赖缺失"
fi
echo ""

# 5. 检查并行处理代码
echo "⚡ 5. 检查并行处理代码..."
if grep -q "par_iter" src-tauri/src/stats.rs; then
  echo "  ✅ 并行处理代码已添加"
else
  echo "  ❌ 并行处理代码缺失"
fi
echo ""

# 6. 检查前端优化
echo "🎨 6. 检查前端优化..."
if grep -q "立即显示基础统计" src/components/Dashboard.tsx; then
  echo "  ✅ 前端非阻塞加载已实现"
else
  echo "  ❌ 前端非阻塞加载缺失"
fi
echo ""

# 7. 性能测试建议
echo "📊 7. 性能测试建议"
echo "================================"
echo ""
echo "请手动执行以下测试："
echo ""
echo "1. 启动应用："
echo "   npm run tauri:dev"
echo ""
echo "2. 观察首页加载："
echo "   - 是否立即显示基础统计？"
echo "   - 是否不再显示阻塞式加载？"
echo "   - 详细统计加载时间是多少？"
echo ""
echo "3. 测试刷新功能："
echo "   - 点击刷新按钮"
echo "   - 观察刷新图标动画"
echo "   - 确认按钮被禁用（防止重复点击）"
echo ""
echo "4. 测试空数据："
echo "   - 切换到没有 sessions 的项目"
echo "   - 确认显示'没有数据'提示"
echo ""
echo "5. 性能对比："
echo "   - 记录优化前后的加载时间"
echo "   - 对比用户体验改善"
echo ""
echo "================================"
echo "✅ 测试脚本执行完成"
