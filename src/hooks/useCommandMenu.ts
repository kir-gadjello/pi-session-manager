import { useState, useCallback, useEffect } from 'react'
import type { SearchPluginResult } from '../plugins/types'

interface UseCommandMenuReturn {
  isOpen: boolean
  open: () => void
  close: () => void
  toggle: () => void
  query: string
  setQuery: (query: string) => void
  results: SearchPluginResult[]
  setResults: (results: SearchPluginResult[]) => void
  isSearching: boolean
  setIsSearching: (isSearching: boolean) => void
  selectedIndex: number
  setSelectedIndex: (index: number) => void
  reset: () => void
}

/**
 * 命令菜单状态管理 Hook
 */
export function useCommandMenu(): UseCommandMenuReturn {
  const [isOpen, setIsOpen] = useState(false)
  const [query, setQuery] = useState('')
  const [results, setResults] = useState<SearchPluginResult[]>([])
  const [isSearching, setIsSearching] = useState(false)
  const [selectedIndex, setSelectedIndex] = useState(0)
  
  const open = useCallback(() => {
    setIsOpen(true)
  }, [])
  
  const close = useCallback(() => {
    setIsOpen(false)
    // 延迟重置状态，等待关闭动画完成
    setTimeout(() => {
      setQuery('')
      setResults([])
      setSelectedIndex(0)
      setIsSearching(false)
    }, 200)
  }, [])
  
  const toggle = useCallback(() => {
    setIsOpen(prev => !prev)
  }, [])
  
  const reset = useCallback(() => {
    setQuery('')
    setResults([])
    setSelectedIndex(0)
    setIsSearching(false)
  }, [])
  
  // 当结果变化时，重置选中索引
  useEffect(() => {
    setSelectedIndex(0)
  }, [results])
  
  return {
    isOpen,
    open,
    close,
    toggle,
    query,
    setQuery,
    results,
    setResults,
    isSearching,
    setIsSearching,
    selectedIndex,
    setSelectedIndex,
    reset
  }
}
