#!/bin/bash

# 清理 Badge 数据库

echo "=== 清理 Badge 数据库 ==="
echo ""

echo "📋 说明："
echo "  - 清理 localStorage 中的 badge 状态"
echo "  - 所有 NEW/UPDATED badge 将消失"
echo "  - 重启应用后，只有新增/修改的会话才会显示 badge"
echo ""

echo "🧹 清理方法："
echo ""
echo "方法 1: 在浏览器控制台执行（推荐）"
echo "----------------------------------------"
echo "localStorage.removeItem('pi-session-manager-badge-states')"
echo "location.reload()"
echo ""

echo "方法 2: 手动清理"
echo "----------------------------------------"
echo "1. 打开应用"
echo "2. 按 F12 打开开发者工具"
echo "3. 切换到 Console 标签"
echo "4. 粘贴以下代码并回车："
echo ""
echo "   localStorage.removeItem('pi-session-manager-badge-states')"
echo "   console.log('✅ Badge 数据已清理')"
echo "   location.reload()"
echo ""

echo "方法 3: 清理所有 localStorage（慎用）"
echo "----------------------------------------"
echo "localStorage.clear()"
echo "location.reload()"
echo ""

echo "✅ 清理后的行为："
echo "  - 所有现有会话：无 badge"
echo "  - 新创建的会话：绿色 NEW badge"
echo "  - 更新的会话：蓝色 UPDATED badge"
echo ""

echo "💡 提示："
echo "  - Badge 数据存储在浏览器 localStorage 中"
echo "  - 清理后不影响会话数据本身"
echo "  - 重启应用会自动建立新的基线"
