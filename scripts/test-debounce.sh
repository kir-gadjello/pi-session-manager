#!/bin/bash

echo "=== 测试防抖机制 ==="
echo ""
echo "📋 测试场景："
echo "  1. 快速创建 3 个文件（间隔 0.5 秒）"
echo "  2. 后端会触发 3 次事件"
echo "  3. 前端应该只刷新 1 次（2 秒防抖）"
echo ""

SESSIONS_DIR="$HOME/.pi/agent/sessions"
TEST_DIR="$SESSIONS_DIR/--test-debounce--"
mkdir -p "$TEST_DIR"

echo "⏳ 等待 3 秒让应用启动..."
sleep 3

echo ""
echo "🚀 开始测试..."
echo ""

# 快速创建 3 个文件
for i in 1 2 3; do
  TEST_FILE="$TEST_DIR/$(date +%Y-%m-%dT%H-%M-%S)-000Z_test-$i-$(uuidgen).jsonl"
  cat > "$TEST_FILE" << EOF
{"type":"session","id":"test-debounce-$i","timestamp":"2026-01-31T18:00:00Z","cwd":"/tmp/test"}
{"type":"message","id":"msg1","parentId":null,"timestamp":"2026-01-31T18:00:01Z","message":{"role":"user","content":[{"type":"text","text":"测试防抖 $i"}]}}
{"type":"session_info","id":"info1","parentId":null,"timestamp":"2026-01-31T18:00:03Z","name":"测试防抖 $i"}
EOF
  echo "✅ 创建文件 $i: $TEST_FILE"
  sleep 0.5
done

echo ""
echo "📊 预期结果："
echo "  - 后端日志：应该看到 3 次 'Detected .jsonl file changes'"
echo "  - 前端日志：应该看到 3 次 '[FileWatcher] 🔔 Event received'"
echo "  - 前端日志：应该看到 2 次 '[FileWatcher] ⏱️ Clearing previous debounce timer'"
echo "  - 前端日志：应该只看到 1 次 '[FileWatcher] ✅ Debounce timer fired'"
echo "  - 前端日志：应该只看到 1 次 '[App] 📡 File watcher triggered'"
echo "  - 会话列表：应该只刷新 1 次"
echo ""

echo "⏳ 等待 5 秒观察结果..."
sleep 5

echo ""
echo "🧹 清理测试文件..."
rm -rf "$TEST_DIR"
echo "✅ 测试完成！"
