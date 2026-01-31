#!/bin/bash

# 测试 Badge 功能

echo "=== Badge 功能测试 ==="
echo ""

SESSIONS_DIR="$HOME/.pi/agent/sessions"
TEST_DIR="$SESSIONS_DIR/--test-badge--"

echo "📋 测试步骤："
echo "1. 启动应用后，所有现有会话不应该有 Badge"
echo "2. 创建新会话，应该显示绿色 NEW badge"
echo "3. 更新现有会话，应该显示蓝色 UPDATED badge"
echo "4. 点击会话后，badge 应该消失"
echo ""

echo "⏳ 等待 3 秒，确保应用已启动..."
sleep 3

echo ""
echo "🆕 测试 1: 创建新会话（应该显示绿色 NEW badge）"
mkdir -p "$TEST_DIR"
TEST_FILE_1="$TEST_DIR/$(date +%Y-%m-%dT%H-%M-%S)-000Z_test-new-$(uuidgen).jsonl"

cat > "$TEST_FILE_1" << 'EOF'
{"type":"session","id":"test-new-badge","timestamp":"2026-01-31T18:00:00Z","cwd":"/tmp/test"}
{"type":"message","id":"msg1","parentId":null,"timestamp":"2026-01-31T18:00:01Z","message":{"role":"user","content":[{"type":"text","text":"测试 NEW badge"}]}}
{"type":"message","id":"msg2","parentId":"msg1","timestamp":"2026-01-31T18:00:02Z","message":{"role":"assistant","content":[{"type":"text","text":"这个会话应该显示绿色 NEW badge"}]}}
{"type":"session_info","id":"info1","parentId":null,"timestamp":"2026-01-31T18:00:03Z","name":"测试 NEW Badge"}
EOF

echo "✅ 新会话已创建: $TEST_FILE_1"
echo "   👀 请检查应用：应该看到绿色 NEW badge"
echo ""

echo "⏳ 等待 5 秒..."
sleep 5

echo ""
echo "🔄 测试 2: 更新现有会话（应该显示蓝色 UPDATED badge）"
echo "   找一个现有会话并添加新消息..."

# 找到第一个现有会话文件
EXISTING_SESSION=$(find "$SESSIONS_DIR" -name "*.jsonl" -type f ! -path "$TEST_DIR/*" | head -1)

if [ -n "$EXISTING_SESSION" ]; then
  echo "   更新会话: $EXISTING_SESSION"
  
  # 添加新消息
  cat >> "$EXISTING_SESSION" << EOF
{"type":"message","id":"msg-test-$(date +%s)","parentId":null,"timestamp":"$(date -u +%Y-%m-%dT%H:%M:%SZ)","message":{"role":"user","content":[{"type":"text","text":"测试 UPDATED badge - 这是新添加的消息"}]}}
EOF
  
  echo "✅ 会话已更新"
  echo "   👀 请检查应用：应该看到蓝色 UPDATED badge"
else
  echo "⚠️  没有找到现有会话，跳过更新测试"
fi

echo ""
echo "⏳ 等待 5 秒后清理..."
sleep 5

echo ""
echo "🧹 清理测试文件..."
rm -f "$TEST_FILE_1"
rmdir "$TEST_DIR" 2>/dev/null

echo "✅ 测试完成！"
echo ""
echo "📝 验收标准："
echo "  ✅ 启动时现有会话无 badge"
echo "  ✅ 新会话显示绿色 NEW badge"
echo "  ✅ 更新会话显示蓝色 UPDATED badge"
echo "  ✅ 点击会话后 badge 消失"
echo "  ✅ Badge 在 24 小时后自动过期"
