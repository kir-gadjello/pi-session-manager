import { FileText } from 'lucide-react'
import { BaseSearchPlugin } from '../base/BaseSearchPlugin'
import type { SearchContext, SearchPluginResult } from '../types'
import { shortenPath } from '../../utils/format'

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
    try {
      const results: SearchPluginResult[] = []

      // 如果启用了"只搜索当前项目"，过滤会话列表
      const sessionsToSearch = context.searchCurrentProjectOnly && context.selectedProject
        ? context.sessions.filter(s => s.cwd === context.selectedProject)
        : context.sessions

      for (const session of sessionsToSearch) {
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
            // 标题显示目录名（项目名）
            title: this.getProjectName(session.cwd),
            // 副标题显示截断的文件路径
            subtitle: shortenPath(session.path, 60),
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

    // 会话名称（如果存在）
    if (session.name) {
      parts.push(session.name)
    }

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
   * 获取项目名称
   */
  private getProjectName(path: string): string {
    const parts = path.split('/')
    return parts[parts.length - 1] || path
  }
}
