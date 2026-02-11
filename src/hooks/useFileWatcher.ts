import { useEffect, useRef } from 'react'
import { listen, UnlistenFn } from '@tauri-apps/api/event'

interface UseFileWatcherOptions {
  enabled?: boolean
  onSessionsChanged: (useCache: boolean) => void
  debounceMs?: number // 防抖时间（毫秒）
}

/**
 * 文件监听 Hook
 * 监听后端的文件变化事件，触发会话列表刷新
 * 带防抖机制，避免频繁刷新
 */
export function useFileWatcher({
  enabled = true,
  onSessionsChanged,
  debounceMs = 5000, // 默认 5 秒防抖（减少刷新频率）
}: UseFileWatcherOptions) {
  const debounceTimerRef = useRef<NodeJS.Timeout | null>(null)
  const onSessionsChangedRef = useRef(onSessionsChanged)

  // 保持回调引用最新
  useEffect(() => {
    onSessionsChangedRef.current = onSessionsChanged
  }, [onSessionsChanged])

  useEffect(() => {
    if (!enabled) {
      return
    }

    let unlisten: UnlistenFn | null = null

    const setupListener = async () => {
      try {
        unlisten = await listen('sessions-changed', (_event) => {
          if (debounceTimerRef.current) {
            clearTimeout(debounceTimerRef.current)
          }

          debounceTimerRef.current = setTimeout(() => {
            // 使用缓存刷新，避免昂贵的全量扫描
            onSessionsChangedRef.current(false)
            debounceTimerRef.current = null
          }, debounceMs)
        })
      } catch (error) {
        console.error('[FileWatcher] Failed to setup listener:', error)
      }
    }

    setupListener()

    return () => {
      if (debounceTimerRef.current) {
        clearTimeout(debounceTimerRef.current)
        debounceTimerRef.current = null
      }

      if (unlisten) {
        unlisten()
      }
    }
  }, [enabled, debounceMs])
}
