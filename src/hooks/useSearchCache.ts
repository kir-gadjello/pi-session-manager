import { useRef } from 'react'
import type { SearchPluginResult } from '../plugins/types'

interface CacheEntry {
  results: SearchPluginResult[]
  timestamp: number
}

const CACHE_SIZE = 100
const CACHE_TTL = 5 * 60 * 1000 // 5 分钟

/**
 * 搜索缓存 Hook
 * 使用 LRU 策略缓存搜索结果
 */
export function useSearchCache() {
  const cacheRef = useRef<Map<string, CacheEntry>>(new Map())
  
  /**
   * 获取缓存结果
   * @param query 查询字符串
   * @returns 缓存的结果或 null
   */
  const get = (query: string): SearchPluginResult[] | null => {
    const entry = cacheRef.current.get(query)
    
    if (!entry) return null
    
    // 检查是否过期
    if (Date.now() - entry.timestamp > CACHE_TTL) {
      cacheRef.current.delete(query)
      return null
    }
    
    return entry.results
  }
  
  /**
   * 设置缓存结果
   * @param query 查询字符串
   * @param results 搜索结果
   */
  const set = (query: string, results: SearchPluginResult[]): void => {
    // LRU: 如果缓存满了，删除最旧的
    if (cacheRef.current.size >= CACHE_SIZE) {
      const firstKey = cacheRef.current.keys().next().value
      if (firstKey !== undefined) {
        cacheRef.current.delete(firstKey)
      }
    }
    
    cacheRef.current.set(query, {
      results,
      timestamp: Date.now()
    })
  }
  
  /**
   * 清空缓存
   */
  const clear = (): void => {
    cacheRef.current.clear()
  }
  
  return { get, set, clear }
}
