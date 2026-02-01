import { useCallback, useRef } from 'react'
import { pluginRegistry } from '../plugins/registry'
import type { SearchPluginResult, SearchContext } from '../plugins/types'
import { useSearchCache } from './useSearchCache'

/**
 * 搜索插件管理 Hook
 */
export function useSearchPlugins(context: SearchContext) {
  const cache = useSearchCache()
  const contextRef = useRef(context)
  
  // 更新 context ref
  contextRef.current = context
  
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
      return cached
    }
    
    // 执行搜索
    const results = await pluginRegistry.search(query, contextRef.current)
    
    // 缓存结果
    cache.set(query, results)
    
    return results
  }, [cache])
  
  return {
    registry: pluginRegistry,
    search
  }
}
