import type { Content } from '../types'
import BashExecution from './BashExecution'
import ReadExecution from './ReadExecution'
import WriteExecution from './WriteExecution'
import EditExecution from './EditExecution'
import GenericToolCall from './GenericToolCall'
import { useSessionView } from '../contexts/SessionViewContext'

interface ToolCallListProps {
  toolCalls: Content[]
  entries?: any[]
}

export default function ToolCallList({ toolCalls, entries = [] }: ToolCallListProps) {
  const { toolsExpanded } = useSessionView()
  const findToolResult = (toolCallId: string) => {
    return entries.find(
      (e: any) => e.type === 'message' &&
      e.message?.role === 'toolResult' &&
      e.message?.toolCallId === toolCallId
    )
  }

  const getToolResultContent = (toolCallId: string) => {
    const result = findToolResult(toolCallId)
    if (!result?.message?.content) return null
    // 返回 content 数组的第一个元素（输出内容）
    return result.message.content[0] || null
  }

  return (
    <div className="tool-call-list">
      {toolCalls.map((toolCall, index) => {
        const name = toolCall.name || 'unknown'
        const args = toolCall.arguments || {}
        const toolCallId = toolCall.id || ''
        const result = findToolResult(toolCallId)
        const toolResultContent = getToolResultContent(toolCallId)

        const isError = result?.message?.isError || toolResultContent?.isError || false
        // 输出内容可能在不同位置：content[0].text 或 output
        const output = toolResultContent?.text || toolResultContent?.output || result?.message?.output || ''
        // diff 可能在多个位置：content[0].details.diff, message.details.diff, 或 content[0].diff
        const diff = toolResultContent?.details?.diff || toolResultContent?.diff || result?.message?.details?.diff
        const timestamp = result?.timestamp

        // Route to specific component based on tool name
        switch (name) {
          case 'bash':
            return (
              <BashExecution
                key={index}
                command={args.command || ''}
                output={output}
                exitCode={result?.message?.exitCode}
                cancelled={result?.message?.cancelled}
                timestamp={timestamp}
                entryId={result?.id || `tool-${index}`}
                expanded={toolsExpanded}
              />
            )

          case 'read':
            return (
              <ReadExecution
                key={index}
                filePath={args.file_path || args.path || ''}
                offset={args.offset}
                limit={args.limit}
                output={output}
                images={toolResultContent?.content?.filter((c: any) => c.type === 'image') || []}
                timestamp={timestamp}
                expanded={toolsExpanded}
              />
            )

          case 'write':
            return (
              <WriteExecution
                key={index}
                filePath={args.file_path || args.path || ''}
                content={args.content || ''}
                output={output}
                timestamp={timestamp}
                expanded={toolsExpanded}
              />
            )

          case 'edit':
            // Debug: 打印 edit 工具的数据结构
            console.log('Edit tool data:', {
              toolCallId,
              hasResult: !!result,
              hasDiff: !!diff,
              diffType: typeof diff,
              diffLength: diff?.length,
              toolResultContent,
              resultMessage: result?.message,
              resultContent: result?.message?.content,
              // 检查是否有 oldText 和 newText
              hasOldText: !!toolResultContent?.oldText,
              hasNewText: !!toolResultContent?.newText,
              detailsKeys: result?.message?.details ? Object.keys(result.message.details) : []
            })
            
            // 输出完整的 diff 内容
            if (diff) {
              console.log('=== FULL DIFF CONTENT ===')
              console.log(diff)
              console.log('=== END DIFF ===')
              // 保存到全局变量以便在控制台中查看
              ;(window as any).lastEditDiff = diff
            }

            // 输出完整的 details
            if (result?.message?.details) {
              console.log('=== FULL DETAILS ===')
              console.log(result.message.details)
              console.log('=== END DETAILS ===')
              ;(window as any).lastEditDetails = result.message.details
            }
            
            return (
              <EditExecution
                key={index}
                filePath={args.file_path || args.path || ''}
                diff={diff}
                output={output}
                timestamp={timestamp}
                expanded={toolsExpanded}
              />
            )

          default:
            return (
              <GenericToolCall
                key={index}
                name={name}
                arguments={args}
                output={output}
                isError={isError}
                timestamp={timestamp}
                expanded={toolsExpanded}
              />
            )
        }
      })}
    </div>
  )
}