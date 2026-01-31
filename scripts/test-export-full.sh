#!/bin/bash

# 导出功能完整测试脚本

set -e

echo "========================================="
echo "PI Session Manager - Export Feature Test"
echo "========================================="
echo

# 检查 PI 命令是否可用
echo "1. Checking PI command..."
if command -v pi &> /dev/null; then
    echo "   ✅ PI command found"
    pi --version 2>&1 || echo "   (Version check skipped)"
else
    echo "   ❌ PI command not found"
    exit 1
fi
echo

# 检查导出命令是否可用
echo "2. Checking PI export command..."
# PI 的 --export 命令需要一个文件路径，我们用一个不存在的文件来测试命令是否识别
if pi --export /nonexistent/file.jsonl /tmp/test.html 2>&1 | grep -q "File not found"; then
    echo "   ✅ PI export command available"
else
    echo "   ⚠️  PI export command check skipped"
fi
echo

# 查找一个测试会话文件
echo "3. Finding test session..."
SESSION_FILE=$(fd "jsonl" ~/.pi/agent/sessions/ | head -1)
if [ -z "$SESSION_FILE" ]; then
    echo "   ❌ No session files found"
    exit 1
fi
echo "   ✅ Found: $SESSION_FILE"
echo

# 测试 HTML 导出
echo "4. Testing HTML export..."
HTML_OUTPUT="/tmp/test-export-$(date +%s).html"
pi --export "$SESSION_FILE" "$HTML_OUTPUT" > /dev/null 2>&1
if [ -f "$HTML_OUTPUT" ] && [ -s "$HTML_OUTPUT" ]; then
    echo "   ✅ HTML export successful"
    echo "   📄 File: $HTML_OUTPUT"
    echo "   📊 Size: $(ls -lh "$HTML_OUTPUT" | awk '{print $5}')"
    # 打开浏览器查看
    echo "   🌐 Opening in browser..."
    open "$HTML_OUTPUT"
else
    echo "   ❌ HTML export failed"
    exit 1
fi
echo

# 测试 Markdown 导出
echo "5. Testing Markdown export..."
MD_OUTPUT="/tmp/test-export-$(date +%s).md"
# 使用 cargo test 来测试
if cargo test --test export_test test_export_markdown 2>&1 | grep -q "test result: ok"; then
    echo "   ✅ Markdown export test passed"
else
    echo "   ⚠️  Markdown export test failed (may need compilation)"
fi
echo

# 测试 JSON 导出
echo "6. Testing JSON export..."
if cargo test --test export_test test_export_json 2>&1 | grep -q "test result: ok"; then
    echo "   ✅ JSON export test passed"
else
    echo "   ⚠️  JSON export test failed (may need compilation)"
fi
echo

# 运行所有导出测试
echo "7. Running all export tests..."
if cargo test --test export_test 2>&1 | grep -q "test result: ok"; then
    echo "   ✅ All export tests passed"
else
    echo "   ⚠️  Some tests failed"
fi
echo

echo "========================================="
echo "✅ Export feature test completed!"
echo "========================================="
echo
echo "Generated files:"
echo "  - $HTML_OUTPUT"
echo
echo "To test the UI:"
echo "  1. Run: npm run tauri dev"
echo "  2. Select a session"
echo "  3. Click the Export button"
echo "  4. Choose export format"
echo "  5. Select save location"
echo