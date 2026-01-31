import { useEffect, useRef } from 'react'
import { listen, UnlistenFn } from '@tauri-apps/api/event'

interface UseFileWatcherOptions {
  enabled?: boolean
  onSessionsChanged: () => void
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
  debounceMs = 2000, // 默认 2 秒防抖
}: UseFileWatcherOptions) {
  const debounceTimerRef = useRef<NodeJS.Timeout | null>(null)
  const onSessionsChangedRef = useRef(onSessionsChanged)

  // 保持回调引用最新
  useEffect(() => {
    onSessionsChangedRef.current = onSessionsChanged
  }, [onSessionsChanged])

  useEffect(() => {
    console.log('[FileWatcher] Hook mounted, enabled:', enabled, 'debounce:', debounceMs, 'ms')
    
    if (!enabled) {
      console.log('[FileWatcher] Disabled, skipping setup')
      return
    }

    let unlisten: UnlistenFn | null = null

    // 监听后端的 sessions-changed 事件
    const setupListener = async () => {
      try {
        console.log('[FileWatcher] Setting up listener for "sessions-changed" event...')
        unlisten = await listen('sessions-changed', (event) => {
          console.log('[FileWatcher] 🔔 Event received:', event)
          
          // 清除之前的定时器
          if (debounceTimerRef.current) {
            console.log('[FileWatcher] ⏱️ Clearing previous debounce timer')
            clearTimeout(debounceTimerRef.current)
          }
          
          // 设置新的防抖定时器
          console.log('[FileWatcher] ⏱️ Setting debounce timer:', debounceMs, 'ms')
          debounceTimerRef.current = setTimeout(() => {
            console.log('[FileWatcher] ✅ Debounce timer fired, triggering refresh...')
            onSessionsChangedRef.current()
            debounceTimerRef.current = null
          }, debounceMs)
        })
        console.log('[FileWatcher] ✅ Listener setup complete')
      } catch (error) {
        console.error('[FileWatcher] ❌ Failed to setup listener:', error)
      }
    }

    setupListener()

    // 清理监听器和定时器
    return () => {
      if (debounceTimerRef.current) {
        console.log('[FileWatcher] Clearing debounce timer on cleanup')
        clearTimeout(debounceTimerRef.current)
        debounceTimerRef.current = null
      }
      
      if (unlisten) {
        console.log('[FileWatcher] Cleaning up listener...')
        unlisten()
        console.log('[FileWatcher] ✅ Listener cleaned up')
      }
    }
  }, [enabled, debounceMs])
}
