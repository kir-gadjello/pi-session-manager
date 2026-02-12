import { useState, useEffect, useCallback, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useTranslation } from 'react-i18next'
import type { SessionInfo } from '../types'
import { useDemoMode } from './useDemoMode'

export interface UseSessionsReturn {
  sessions: SessionInfo[]
  loading: boolean
  selectedSession: SessionInfo | null
  setSelectedSession: (session: SessionInfo | null) => void
  loadSessions: (useFullScan?: boolean) => Promise<void>
  handleDeleteSession: (session: SessionInfo) => Promise<void>
  handleRenameSession: (session: SessionInfo, newName: string) => Promise<void>
}

export function useSessions(): UseSessionsReturn {
  const { t } = useTranslation()
  const { isDemoMode, getDemoSessions } = useDemoMode()
  const [sessions, setSessions] = useState<SessionInfo[]>([])
  const [selectedSession, setSelectedSession] = useState<SessionInfo | null>(null)
  const [loading, setLoading] = useState(true)
  const selectedSessionRef = useRef<SessionInfo | null>(null)
  const sessionsRef = useRef<SessionInfo[]>([])

  // Keep refs in sync with latest state
  useEffect(() => {
    selectedSessionRef.current = selectedSession
  }, [selectedSession])

  useEffect(() => {
    sessionsRef.current = sessions
  }, [sessions])

  const loadSessions = useCallback(async (useFullScan: boolean = false) => {
    try {
      let loadedSessions: SessionInfo[] = []
      if (isDemoMode) {
        loadedSessions = getDemoSessions()
      } else {
        if (useFullScan) {
          loadedSessions = await invoke<SessionInfo[]>('scan_sessions')
        } else {
          loadedSessions = await invoke<SessionInfo[]>('get_cached_sessions')
        }
      }
      // Compare with current sessions to avoid unnecessary re-renders and scroll resets
      const currentSessions = sessionsRef.current
      const areEqual = loadedSessions.length === currentSessions.length &&
        loadedSessions.every((s, i) => {
          const old = currentSessions[i]
          return old.path === s.path &&
                 old.modified === s.modified &&
                 old.name === s.name &&
                 old.message_count === s.message_count
        })

      if (!areEqual) {
        setSessions(loadedSessions)

        // Update selected session if needed
        const currentSelection = selectedSessionRef.current
        if (currentSelection) {
          const matchedByPath = loadedSessions.find(s => s.path === currentSelection.path)
          const matchedById = loadedSessions.find(s => s.id === currentSelection.id)
          const matched = matchedByPath || matchedById

          if (matched) {
            const pathChanged = matched.path !== currentSelection.path
            const nameChanged = matched.name !== currentSelection.name
            const hasChanges = pathChanged || nameChanged ||
              matched.message_count !== currentSelection.message_count ||
              matched.modified !== currentSelection.modified

            if (pathChanged || nameChanged) {
              setSelectedSession(matched)
            } else if (hasChanges) {
              // Session metadata changed, updating silently
              setSelectedSession(prev => {
                if (!prev) return matched
                return Object.assign(prev, matched)
              })
            }
          } else {
            try {
              if (isDemoMode) {
                setSelectedSession(currentSelection)
              } else {
                await invoke('read_session_file', { path: currentSelection.path })
              }
            } catch (error) {
              console.warn('[useSessions] Selected session file not readable, clearing selection:', error)
              setSelectedSession(null)
            }
          }
        }
      }
    } catch (error) {
      console.error('[useSessions] Failed to load sessions:', error)
      alert(`${t('app.errors.loadSessions')}: ${error}`)
    } finally {
      setLoading(false)
    }
  }, [t, isDemoMode, getDemoSessions])

  const handleDeleteSession = useCallback(async (session: SessionInfo) => {
    if (!confirm(t('app.confirm.deleteSession', { name: session.name || t('common.untitled') }))) {
      return
    }

    try {
      if (isDemoMode) {
        // In demo mode, just remove from local state
        setSessions(prev => prev.filter(s => s.id !== session.id))
      } else {
        await invoke('delete_session', { path: session.path })
        setSessions(prev => prev.filter(s => s.id !== session.id))
      }
      if (selectedSession?.id === session.id) {
        setSelectedSession(null)
      }
    } catch (error) {
      console.error('Failed to delete session:', error)
      alert(t('app.errors.deleteSession'))
    }
  }, [selectedSession, t, isDemoMode])

  const handleRenameSession = useCallback(async (session: SessionInfo, newName: string) => {
    try {
      if (isDemoMode) {
        // In demo mode, just update local state
        setSessions(prev => prev.map(s =>
          s.id === session.id ? { ...s, name: newName } : s
        ))
      } else {
        await invoke('rename_session', {
          path: session.path,
          new_name: newName
        })
        setSessions(prev => prev.map(s =>
          s.id === session.id ? { ...s, name: newName } : s
        ))
      }

      if (selectedSession?.id === session.id) {
        setSelectedSession(prev => prev ? { ...prev, name: newName } : null)
      }
    } catch (error) {
      console.error('Failed to rename session:', error)
      alert(t('app.errors.renameSession'))
    }
  }, [selectedSession, t, isDemoMode])

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
