import { FileText } from 'lucide-react'
import { BaseSearchPlugin } from '../base/BaseSearchPlugin'
import type { SearchContext, SearchPluginResult } from '../types'

/**
 * 会话搜索插件
 * 搜索会话名称和元数据
 */
export class SessionSearchPlugin extends BaseSearchPlugin {
  id = 'session-search'
  name = '会话搜索'
  icon = FileText
  description = '搜索会话名称和元数据'
  keywords = ['session', 'file', 'conversation', '会话', '文件', '对话']
  priority = 60
  
  async search(
    query: string,
    context: SearchContext
  ): Promise<SearchPluginResult[]> {
    console.log('[SessionSearchPlugin] Starting search for:', query)
    
    try {
      const results: SearchPluginResult[] = []
      
      for (const session of context.sessions) {
        // 搜索会话名称
        const nameScore = session.name 
          ? this.fuzzyMatch(query, session.name)
          : 0
        
        // 搜索第一条消息
        const messageScore = this.fuzzyMatch(query, session.first_message) * 0.8
        
        // 搜索路径
        const pathScore = this.fuzzyMatch(query, session.path) * 0.5
        
        // 搜索项目路径
        const cwdScore = this.fuzzyMatch(query, session.cwd) * 0.3
        
        // 综合分数
        const score = Math.max(nameScore, messageScore, pathScore, cwdScore)
        
        if (score > 0) {
          results.push({
            id: `session-${session.id}`,
            pluginId: this.id,
            title: session.name || this.truncateText(session.first_message, 60),
            subtitle: this.getProjectName(session.cwd),
            description: this.getSessionDescription(session, context),
            icon: <FileText className="w-4 h-4 text-green-400" />,
            metadata: {
              session
            },
            score,
            highlights: [
              ...this.calculateHighlights(query, session.name || '', 'title'),
              ...this.calculateHighlights(query, session.first_message, 'title')
            ]
          })
        }
      }
      
      const finalResults = results.sort((a, b) => b.score - a.score).slice(0, 20)
      console.log('[SessionSearchPlugin] Returning results:', finalResults.length)
      return finalResults
    } catch (error) {
      console.error('[SessionSearchPlugin] Search failed:', error)
      return []
    }
  }
  
  onSelect(result: SearchPluginResult, context: SearchContext): void {
    // 打开会话
    const session = result.metadata?.session
    
    if (session) {
      context.setSelectedSession(session)
      context.closeCommandMenu()
    }
  }
  
  /**
   * 获取会话描述
   */
  private getSessionDescription(session: any, context: SearchContext): string {
    const parts: string[] = []
    
    // 消息数量
    parts.push(context.t('session.messageCount', {
      count: session.message_count,
      defaultValue: `${session.message_count} 条消息`
    }))
    
    // 修改时间
    parts.push(this.formatRelativeTime(session.modified))
    
    return parts.join(' • ')
  }
  
  /**
   * 格式化相对时间
   */
  private formatRelativeTime(timestamp: string): string {
    try {
      const date = new Date(timestamp)
      const now = new Date()
      const diff = now.getTime() - date.getTime()
      
      // 小于 1 分钟
      if (diff < 60 * 1000) {
        return '刚刚'
      }
      
      // 小于 1 小时
      if (diff < 60 * 60 * 1000) {
        const minutes = Math.floor(diff / (60 * 1000))
        return `${minutes} 分钟前`
      }
      
      // 小于 24 小时
      if (diff < 24 * 60 * 60 * 1000) {
        const hours = Math.floor(diff / (60 * 60 * 1000))
        return `${hours} 小时前`
      }
      
      // 小于 7 天
      if (diff < 7 * 24 * 60 * 60 * 1000) {
        const days = Math.floor(diff / (24 * 60 * 60 * 1000))
        return `${days} 天前`
      }
      
      // 小于 30 天
      if (diff < 30 * 24 * 60 * 60 * 1000) {
        const weeks = Math.floor(diff / (7 * 24 * 60 * 60 * 1000))
        return `${weeks} 周前`
      }
      
      // 显示日期
      return date.toLocaleDateString('zh-CN', {
        year: 'numeric',
        month: '2-digit',
        day: '2-digit'
      })
    } catch {
      return timestamp
    }
  }
  
  /**
   * 截断文本
   */
  private truncateText(text: string, maxLength: number): string {
    if (text.length <= maxLength) return text
    return text.slice(0, maxLength) + '...'
  }
  
  /**
   * 获取项目名称
   */
  private getProjectName(path: string): string {
    const parts = path.split('/')
    return parts[parts.length - 1] || path
  }
}
