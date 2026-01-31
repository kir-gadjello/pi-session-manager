import type { SessionInfo } from '../types'

/**
 * 搜索插件接口
 * 所有搜索插件必须实现此接口
 */
export interface SearchPlugin {
  // ========== 元数据 ==========
  
  /** 插件唯一标识 */
  id: string
  
  /** 插件显示名称 */
  name: string
  
  /** 插件图标组件 */
  icon: React.ComponentType<{ className?: string }>
  
  /** 插件描述 */
  description: string
  
  /** 搜索关键词（用于插件匹配） */
  keywords: string[]
  
  /** 优先级（0-100，越高越优先显示） */
  priority: number
  
  // ========== 核心方法 ==========
  
  /**
   * 执行搜索
   * @param query 搜索查询
   * @param context 搜索上下文
   * @returns 搜索结果数组
   */
  search(
    query: string,
    context: SearchContext
  ): Promise<SearchPluginResult[]>
  
  /**
   * 处理结果选中
   * @param result 选中的结果
   * @param context 搜索上下文
   */
  onSelect(
    result: SearchPluginResult,
    context: SearchContext
  ): void
  
  // ========== 可选方法 ==========
  
  /**
   * 自定义结果项渲染
   * @param result 搜索结果
   * @returns 自定义 React 节点
   */
  renderItem?(result: SearchPluginResult): React.ReactNode
  
  /**
   * 插件挂载时调用
   */
  onMount?(): void
  
  /**
   * 插件卸载时调用
   */
  onUnmount?(): void
  
  /**
   * 判断插件是否可用
   * @param context 搜索上下文
   * @returns 是否可用
   */
  isEnabled?(context: SearchContext): boolean
}

/**
 * 搜索上下文
 * 提供给插件的全局状态和方法
 */
export interface SearchContext {
  // ========== 数据 ==========
  
  /** 所有会话列表 */
  sessions: SessionInfo[]
  
  /** 当前选中的项目 */
  selectedProject: string | null
  
  /** 当前选中的会话 */
  selectedSession: SessionInfo | null
  
  /** 是否只搜索当前项目 */
  searchCurrentProjectOnly: boolean
  
  // ========== 方法 ==========
  
  /** 设置选中的会话 */
  setSelectedSession: (session: SessionInfo | null) => void
  
  /** 设置选中的项目 */
  setSelectedProject: (project: string | null) => void
  
  /** 关闭命令面板 */
  closeCommandMenu: () => void
  
  // ========== 工具 ==========
  
  /** 国际化翻译函数 */
  t: (key: string, options?: any) => string
}

/**
 * 搜索结果
 */
export interface SearchPluginResult {
  /** 结果唯一标识 */
  id: string
  
  /** 所属插件 ID */
  pluginId: string
  
  /** 主标题 */
  title: string
  
  /** 副标题 */
  subtitle?: string
  
  /** 描述 */
  description?: string
  
  /** 图标 */
  icon?: React.ReactNode
  
  /** 元数据（插件自定义） */
  metadata?: Record<string, any>
  
  /** 匹配分数（0-1） */
  score: number
  
  /** 高亮范围 */
  highlights?: HighlightRange[]
}

/**
 * 高亮范围
 */
export interface HighlightRange {
  start: number
  end: number
  field: 'title' | 'subtitle' | 'description'
}
