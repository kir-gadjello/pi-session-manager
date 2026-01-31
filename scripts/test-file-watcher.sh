#!/bin/bash

# 测试文件监听自动刷新功能

echo "=== 文件监听自动刷新测试 ==="
echo ""

SESSIONS_DIR="$HOME/.pi/agent/sessions"
TEST_DIR="$SESSIONS_DIR/--test-auto-refresh--"
TEST_FILE="$TEST_DIR/$(date +%Y-%m-%dT%H-%M-%S)-000Z_test-$(uuidgen).jsonl"

echo "1. 创建测试目录..."
mkdir -p "$TEST_DIR"

echo "2. 创建测试会话文件..."
cat > "$TEST_FILE" << 'EOF'
{"type":"session","id":"test-auto-refresh","timestamp":"2026-01-31T15:30:00Z","cwd":"/tmp/test"}
{"type":"message","id":"msg1","parentId":null,"timestamp":"2026-01-31T15:30:01Z","message":{"role":"user","content":[{"type":"text","text":"测试自动刷新功能"}]}}
{"type":"message","id":"msg2","parentId":"msg1","timestamp":"2026-01-31T15:30:02Z","message":{"role":"assistant","content":[{"type":"text","text":"文件监听正常工作！"}]}}
{"type":"session_info","id":"info1","parentId":null,"timestamp":"2026-01-31T15:30:03Z","name":"测试自动刷新"}
EOF

echo "✅ 测试文件已创建: $TEST_FILE"
echo ""
echo "📋 请检查应用界面："
echo "   1. 会话列表应该自动刷新"
echo "   2. 应该看到新会话 '测试自动刷新'"
echo "   3. 无需手动按 Cmd+R"
echo ""
echo "⏳ 等待 5 秒后自动清理..."
sleep 5

echo ""
echo "3. 清理测试文件..."
rm -f "$TEST_FILE"
rmdir "$TEST_DIR" 2>/dev/null

echo "✅ 测试完成！"
echo ""
echo "💡 提示："
echo "   - 如果看到新会话出现，说明文件监听工作正常"
echo "   - 如果没有自动刷新，请检查控制台日志"
echo "   - 查看后端日志: 搜索 '[FileWatcher]' 或 'File watcher'"
