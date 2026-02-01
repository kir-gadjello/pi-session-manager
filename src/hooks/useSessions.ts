import { useState, useEffect, useCallback, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useTranslation } from 'react-i18next'
import type { SessionInfo } from '../types'

export interface UseSessionsReturn {
  sessions: SessionInfo[]
  loading: boolean
  selectedSession: SessionInfo | null
  setSelectedSession: (session: SessionInfo | null) => void
  loadSessions: () => Promise<void>
  handleDeleteSession: (session: SessionInfo) => Promise<void>
  handleRenameSession: (session: SessionInfo, newName: string) => Promise<void>
}

export function useSessions(): UseSessionsReturn {
  const { t } = useTranslation()
  const [sessions, setSessions] = useState<SessionInfo[]>([])
  const [selectedSession, setSelectedSession] = useState<SessionInfo | null>(null)
  const [loading, setLoading] = useState(true)
  const selectedSessionRef = useRef<SessionInfo | null>(null)

  useEffect(() => {
    selectedSessionRef.current = selectedSession
  }, [selectedSession])

  const loadSessions = useCallback(async () => {
    console.log('[useSessions] loadSessions called')
    try {
      setLoading(true)
      const result = await invoke<SessionInfo[]>('scan_sessions')
      console.log('[useSessions] scan_sessions returned', result.length, 'sessions')
      setSessions(result)

      const currentSelection = selectedSessionRef.current
      if (currentSelection) {
        const matchedByPath = result.find(s => s.path === currentSelection.path)
        const matchedById = result.find(s => s.id === currentSelection.id)
        const matched = matchedByPath || matchedById

        if (matched) {
          const pathChanged = matched.path !== currentSelection.path
          const nameChanged = matched.name !== currentSelection.name
          const hasChanges = pathChanged || nameChanged ||
            matched.message_count !== currentSelection.message_count ||
            matched.modified !== currentSelection.modified

          if (!hasChanges) {
            console.log('[useSessions] No changes detected, keeping current selection stable:', matched.id)
          } else if (pathChanged || nameChanged) {
            console.log('[useSessions] Updating selected session (path/name changed):', matched.id)
            setSelectedSession(matched)
          } else {
            console.log('[useSessions] Session metadata changed, updating silently:', matched.id)
            setSelectedSession(prev => {
              if (!prev) return matched
              return Object.assign(prev, matched)
            })
          }
        } else {
          try {
            await invoke('read_session_file', { path: currentSelection.path })
            console.log('[useSessions] Selected session file still readable but not in scan results, keeping selection:', currentSelection.id)
          } catch (error) {
            console.warn('[useSessions] Selected session file not readable, clearing selection:', error)
            setSelectedSession(null)
          }
        }
      }
    } catch (error) {
      console.error('[useSessions] Failed to load sessions:', error)
      alert(`${t('app.errors.loadSessions')}: ${error}`)
    } finally {
      setLoading(false)
    }
  }, [t])

  const handleDeleteSession = useCallback(async (session: SessionInfo) => {
    if (!confirm(t('app.confirm.deleteSession', { name: session.name || t('common.untitled') }))) {
      return
    }

    try {
      await invoke('delete_session', { path: session.path })
      setSessions(prev => prev.filter(s => s.id !== session.id))
      if (selectedSession?.id === session.id) {
        setSelectedSession(null)
      }
    } catch (error) {
      console.error('Failed to delete session:', error)
      alert(t('app.errors.deleteSession'))
    }
  }, [selectedSession, t])

  const handleRenameSession = useCallback(async (session: SessionInfo, newName: string) => {
    try {
      await invoke('rename_session', {
        path: session.path,
        newName
      })

      setSessions(prev => prev.map(s =>
        s.id === session.id ? { ...s, name: newName } : s
      ))

      if (selectedSession?.id === session.id) {
        setSelectedSession(prev => prev ? { ...prev, name: newName } : null)
      }
    } catch (error) {
      console.error('Failed to rename session:', error)
      alert(t('app.errors.renameSession'))
    }
  }, [selectedSession, t])

  return {
    sessions,
    loading,
    selectedSession,
    setSelectedSession,
    loadSessions,
    handleDeleteSession,
    handleRenameSession,
  }
}
