import { useCallback } from 'react'
import { pluginRegistry } from '../plugins/registry'
import type { SearchPluginResult, SearchContext } from '../plugins/types'
import { useSearchCache } from './useSearchCache'

/**
 * 搜索插件管理 Hook
 */
export function useSearchPlugins(context: SearchContext) {
  const cache = useSearchCache()
  
  /**
   * 执行搜索
   * @param query 查询字符串
   * @returns 搜索结果数组
   */
  const search = useCallback(async (query: string): Promise<SearchPluginResult[]> => {
    if (!query.trim()) {
      return []
    }
    
    // 检查缓存
    const cached = cache.get(query)
    if (cached) {
      console.log('[useSearchPlugins] Cache hit:', query)
      return cached
    }
    
    // 执行搜索
    console.log('[useSearchPlugins] Searching:', query)
    const results = await pluginRegistry.search(query, context)
    
    // 缓存结果
    cache.set(query, results)
    
    return results
  }, [context, cache])
  
  return {
    registry: pluginRegistry,
    search
  }
}
