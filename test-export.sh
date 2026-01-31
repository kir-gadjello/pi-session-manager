#!/bin/bash

# 测试导出功能
SESSION_PATH="/Users/dengwenyu/.pi/agent/sessions/--private-tmp-checkpoint-v2-test--/2026-01-27T05-48-33-581Z_a4179077-31ae-4c3d-b25f-85deda4672dc.jsonl"
OUTPUT_PATH="/tmp/test-export-$(date +%s).html"

echo "Testing HTML export..."
echo "Session: $SESSION_PATH"
echo "Output: $OUTPUT_PATH"

pi --export "$SESSION_PATH" "$OUTPUT_PATH"

if [ $? -eq 0 ]; then
    echo "✅ Export successful!"
    echo "File size: $(ls -lh "$OUTPUT_PATH" | awk '{print $5}')"
    echo "Opening in browser..."
    open "$OUTPUT_PATH"
else
    echo "❌ Export failed!"
    exit 1
fi