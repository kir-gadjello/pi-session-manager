import { useState, useEffect, useCallback, useRef } from 'react'
import type { SessionInfo } from '../types'

interface BadgeState {
  type: 'new' | 'updated'
}

/**
 * Badge 状态管理 Hook
 * 以程序启动时间为基准，追踪启动后新增和更新的会话
 */
export function useSessionBadges(sessions: SessionInfo[]) {
  const [badgeStates, setBadgeStates] = useState<Record<string, BadgeState>>({})
  const baselineRef = useRef<Map<string, SessionInfo> | null>(null)
  const previousSessionsRef = useRef<Map<string, SessionInfo>>(new Map())

  // 检测会话变化并更新 badge 状态
  useEffect(() => {
    // 如果 sessions 为空，不做任何处理
    if (sessions.length === 0) {
      return
    }

    // 首次加载：建立基准线（启动时的会话快照）
    if (baselineRef.current === null) {
      console.log('[BadgeManager] 🎯 Establishing baseline with', sessions.length, 'sessions at startup')
      const baseline = new Map<string, SessionInfo>()
      for (const session of sessions) {
        baseline.set(session.id, session)
      }
      baselineRef.current = baseline
      previousSessionsRef.current = baseline
      console.log('[BadgeManager] ✅ Baseline established, no badges will be shown for existing sessions')
      return
    }

    console.log('[BadgeManager] 🔍 Checking for changes since startup...')
    
    const baseline = baselineRef.current
    const previousSessions = previousSessionsRef.current
    const newBadges: Record<string, BadgeState> = {}

    // 检测新增和更新的会话
    for (const session of sessions) {
      const baselineSession = baseline.get(session.id)
      const prevSession = previousSessions.get(session.id)
      
      if (!baselineSession) {
        // 启动后新增的会话 → NEW badge
        console.log('[BadgeManager] 🆕 New session (after startup):', session.id, session.name || session.first_message.substring(0, 50))
        newBadges[session.id] = { type: 'new' }
      } else if (prevSession && session.message_count > prevSession.message_count) {
        // 启动后更新的会话 → UPDATED badge（不覆盖 NEW）
        if (!badgeStates[session.id] || badgeStates[session.id].type !== 'new') {
          console.log('[BadgeManager] 🔄 Session updated (after startup):', session.id, {
            messageCount: `${prevSession.message_count} -> ${session.message_count}`,
          })
          newBadges[session.id] = { type: 'updated' }
        }
      }
    }

    // 更新 badge 状态
    if (Object.keys(newBadges).length > 0) {
      console.log('[BadgeManager] ✅ Adding', Object.keys(newBadges).length, 'new badges')
      setBadgeStates(prev => ({ ...prev, ...newBadges }))
    }

    // 更新 previousSessions
    const newPreviousSessions = new Map<string, SessionInfo>()
    for (const session of sessions) {
      newPreviousSessions.set(session.id, session)
    }
    previousSessionsRef.current = newPreviousSessions
  }, [sessions, badgeStates])

  // 清除指定会话的 badge
  const clearBadge = useCallback((sessionId: string) => {
    console.log('[BadgeManager] 🗑️ Clearing badge for session:', sessionId)
    setBadgeStates(prev => {
      const newStates = { ...prev }
      delete newStates[sessionId]
      return newStates
    })
  }, [])

  // 清除所有 badge
  const clearAllBadges = useCallback(() => {
    console.log('[BadgeManager] 🗑️ Clearing all badges')
    setBadgeStates({})
  }, [])

  // 获取指定会话的 badge 类型
  const getBadgeType = useCallback((sessionId: string): 'new' | 'updated' | null => {
    return badgeStates[sessionId]?.type || null
  }, [badgeStates])

  return {
    badgeStates,
    getBadgeType,
    clearBadge,
    clearAllBadges,
  }
}
