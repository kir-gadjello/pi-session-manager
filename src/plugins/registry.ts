import type { SearchPlugin, SearchContext, SearchPluginResult } from './types'

/**
 * 插件注册表
 * 管理所有搜索插件的注册、查询和执行
 */
export class PluginRegistry {
  private plugins: Map<string, SearchPlugin> = new Map()
  
  /**
   * 注册插件
   * @param plugin 搜索插件
   * @throws 如果插件 ID 已存在
   */
  register(plugin: SearchPlugin): void {
    if (this.plugins.has(plugin.id)) {
      throw new Error(`Plugin with id "${plugin.id}" already registered`)
    }
    
    this.plugins.set(plugin.id, plugin)
    plugin.onMount?.()
    
    console.log(`[PluginRegistry] Registered plugin: ${plugin.id}`)
  }
  
  /**
   * 注销插件
   * @param pluginId 插件 ID
   */
  unregister(pluginId: string): void {
    const plugin = this.plugins.get(pluginId)
    if (plugin) {
      plugin.onUnmount?.()
      this.plugins.delete(pluginId)
      console.log(`[PluginRegistry] Unregistered plugin: ${pluginId}`)
    }
  }
  
  /**
   * 获取插件
   * @param pluginId 插件 ID
   * @returns 插件实例或 undefined
   */
  get(pluginId: string): SearchPlugin | undefined {
    return this.plugins.get(pluginId)
  }
  
  /**
   * 获取所有插件
   * @returns 插件数组（按优先级排序）
   */
  getAll(): SearchPlugin[] {
    return Array.from(this.plugins.values())
      .sort((a, b) => b.priority - a.priority)
  }
  
  /**
   * 获取可用插件
   * @param context 搜索上下文
   * @returns 可用插件数组
   */
  getEnabled(context: SearchContext): SearchPlugin[] {
    return this.getAll().filter(plugin => 
      plugin.isEnabled ? plugin.isEnabled(context) : true
    )
  }
  
  /**
   * 执行搜索
   * @param query 搜索查询
   * @param context 搜索上下文
   * @returns 合并后的搜索结果
   */
  async search(
    query: string,
    context: SearchContext
  ): Promise<SearchPluginResult[]> {
    if (!query.trim()) {
      return []
    }
    
    const enabledPlugins = this.getEnabled(context)
    
    // 并行执行所有插件的搜索
    const results = await Promise.all(
      enabledPlugins.map(async plugin => {
        try {
          const pluginResults = await plugin.search(query, context)
          return pluginResults.map(result => ({
            ...result,
            pluginId: plugin.id,
            // 综合分数 = 结果分数 × 插件优先级权重
            score: result.score * (plugin.priority / 100)
          }))
        } catch (error) {
          console.error(`[PluginRegistry] Plugin ${plugin.id} search failed:`, error)
          return []
        }
      })
    )
    
    // 合并并排序结果
    return results
      .flat()
      .sort((a, b) => b.score - a.score)
  }
}

// 全局单例
export const pluginRegistry = new PluginRegistry()
