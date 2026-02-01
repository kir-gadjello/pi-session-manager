import { useState, useEffect, useMemo, useRef, useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { BarChart3, FolderOpen, Settings, ArrowLeft } from 'lucide-react'
import SessionList from './components/SessionList'
import SessionListByDirectory from './components/SessionListByDirectory'
import ProjectList from './components/ProjectList'
import SessionViewer from './components/SessionViewer'
import SearchPanel from './components/SearchPanel'
import ExportDialog from './components/ExportDialog'
import RenameDialog from './components/RenameDialog'
import Dashboard from './components/Dashboard'
import SettingsPanel from './components/settings/SettingsPanel'
import { CommandPalette } from './components/command'
import { useKeyboardShortcuts } from './hooks/useKeyboardShortcuts'
import { useFileWatcher } from './hooks/useFileWatcher'
import { useSessionBadges } from './hooks/useSessionBadges'
import { useSessions } from './hooks/useSessions'
import { useSearch } from './hooks/useSearch'
import { useAppSettings } from './hooks/useAppSettings'
import { useSessionActions } from './hooks/useSessionActions'
import { registerBuiltinPlugins } from './plugins'
import type { SessionInfo, SearchResult } from './types'
import type { SearchContext } from './plugins/types'

function App() {
  const { t } = useTranslation()
  const listScrollRef = useRef<HTMLDivElement>(null)

  const {
    sessions,
    loading,
    selectedSession,
    setSelectedSession,
    loadSessions,
    handleDeleteSession,
    handleRenameSession,
  } = useSessions()

  const { terminal, piPath, customCommand, loadSettings } = useAppSettings()
  const { handleExportSession } = useSessionActions()
  const { getBadgeType, clearBadge } = useSessionBadges(sessions)

  const [selectedProject, setSelectedProject] = useState<string | null>(null)
  const [viewMode, setViewMode] = useState<'list' | 'directory' | 'project'>('project')
  const [showExportDialog, setShowExportDialog] = useState(false)
  const [showRenameDialog, setShowRenameDialog] = useState(false)
  const [showSettings, setShowSettings] = useState(false)

  const { searchResults, isSearching, handleSearch, clearSearch } = useSearch(setSelectedSession)

  const handleSelectSession = useCallback((session: SessionInfo) => {
    setSelectedSession(session)
    clearSearch()
    clearBadge(session.id)
  }, [setSelectedSession, clearSearch, clearBadge])

  useEffect(() => {
    registerBuiltinPlugins()
  }, [])

  useEffect(() => {
    loadSessions()
    loadSettings()
  }, [loadSessions, loadSettings])

  useFileWatcher({
    enabled: true,
    debounceMs: 2000,
    onSessionsChanged: () => {
      loadSessions()
    },
  })

  const shortcuts = useMemo(() => ({
    'cmd+r': () => loadSessions(),
    'cmd+f': () => document.querySelector<HTMLInputElement>('input[type="text"]')?.focus(),
    'cmd+,': () => setShowSettings(true),
    'escape': () => {
      if (showSettings) {
        setShowSettings(false)
      } else if (showExportDialog) {
        setShowExportDialog(false)
      } else if (showRenameDialog) {
        setShowRenameDialog(false)
      } else if (selectedProject) {
        setSelectedProject(null)
      } else {
        setSelectedSession(null)
        clearSearch()
      }
    },
  }), [showSettings, showExportDialog, showRenameDialog, selectedProject, loadSessions, setSelectedSession, clearSearch])

  useKeyboardShortcuts(shortcuts)

  const commandContext = useMemo<SearchContext>(() => ({
    sessions,
    selectedProject,
    selectedSession,
    setSelectedSession,
    setSelectedProject,
    closeCommandMenu: () => {},
    searchCurrentProjectOnly: false,
    t
  }), [sessions, selectedProject, selectedSession, t, setSelectedSession])

  const onRenameSession = async (newName: string) => {
    if (!selectedSession) return
    await handleRenameSession(selectedSession, newName)
    setShowRenameDialog(false)
  }

  const onExportSession = async (format: 'html' | 'md' | 'json') => {
    if (!selectedSession) return
    await handleExportSession(selectedSession, format)
    setShowExportDialog(false)
  }

  const displayedSessions = isSearching
    ? mapSearchResults(searchResults, sessions)
    : sessions

  return (
    <div className="flex h-screen bg-background text-foreground">
      <div className="w-80 border-r border-[#2c2d3b] flex flex-col">
        <div className="flex items-center justify-end px-3 py-2.5 border-b border-[#2c2d3b]">
          <div className="flex items-center gap-0.5">
            <div className="flex items-center bg-[#252636] rounded-lg p-0.5 mr-1">
              <button
                onClick={() => { setViewMode('list'); setSelectedProject(null) }}
                className={`p-1 rounded transition-colors ${viewMode === 'list' ? 'text-blue-400 bg-[#2c2d3b]' : 'text-[#6a6f85] hover:text-white'}`}
                title={t('app.viewMode.list')}
              >
                <svg className="h-3.5 w-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 10h16M4 14h16M4 18h16" />
                </svg>
              </button>
              <button
                onClick={() => { setViewMode('directory'); setSelectedProject(null) }}
                className={`p-1 rounded transition-colors ${viewMode === 'directory' ? 'text-blue-400 bg-[#2c2d3b]' : 'text-[#6a6f85] hover:text-white'}`}
                title={t('app.viewMode.directory')}
              >
                <FolderOpen className="h-3.5 w-3.5" />
              </button>
              <button
                onClick={() => { setViewMode('project'); setSelectedProject(null) }}
                className={`p-1 rounded transition-colors ${viewMode === 'project' ? 'text-blue-400 bg-[#2c2d3b]' : 'text-[#6a6f85] hover:text-white'}`}
                title={t('app.viewMode.project')}
              >
                <svg className="h-3.5 w-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                </svg>
              </button>
            </div>
            <button
              onClick={() => setSelectedSession(null)}
              className={`p-1 rounded transition-colors ml-0.5 ${!selectedSession ? 'text-[#569cd6] bg-[#569cd6]/10' : 'text-[#6a6f85] hover:text-white hover:bg-[#2c2d3b]'}`}
              title={t('dashboard.title')}
            >
              <BarChart3 className="h-3.5 w-3.5" />
            </button>
            <button
              onClick={() => setShowSettings(true)}
              className="p-1 rounded transition-colors ml-0.5 text-[#6a6f85] hover:text-white hover:bg-[#2c2d3b]"
              title={t('settings.title')}
            >
              <Settings className="h-3.5 w-3.5" />
            </button>
          </div>
        </div>

        <SearchPanel
          onSearch={(query) => handleSearch(query, sessions)}
          resultCount={searchResults.length}
          isSearching={isSearching}
        />
        <div className="flex-1 overflow-y-auto" ref={listScrollRef}>
          {viewMode === 'project' && selectedProject ? (
            <div className="flex flex-col">
              <div className="flex items-center gap-2 px-3 py-2 border-b border-border/50 bg-background/30 flex-shrink-0 sticky top-0 z-10">
                <button
                  onClick={() => setSelectedProject(null)}
                  className="p-1 hover:bg-accent rounded transition-colors flex-shrink-0"
                  title={t('project.list.back')}
                >
                  <ArrowLeft className="h-4 w-4 text-muted-foreground" />
                </button>
                <div className="flex items-center gap-1.5 min-w-0 flex-1">
                  <FolderOpen className="h-3.5 w-3.5 text-blue-400 flex-shrink-0" />
                  <span className="text-sm font-medium truncate">
                    {displayedSessions.find(s => s.cwd === selectedProject)?.cwd.split('/').pop() || selectedProject.split('/').pop()}
                  </span>
                  <span className="text-[11px] text-muted-foreground flex-shrink-0">
                    ({displayedSessions.filter(s => s.cwd === selectedProject).length})
                  </span>
                </div>
              </div>
              <div>
                <ProjectList
                  sessions={displayedSessions}
                  selectedSession={selectedSession}
                  selectedProject={selectedProject}
                  onSelectSession={handleSelectSession}
                  onSelectProject={setSelectedProject}
                  loading={loading}
                  terminal={terminal}
                  piPath={piPath}
                  customCommand={customCommand}
                  getBadgeType={getBadgeType}
                  scrollParentRef={listScrollRef}
                  showHeader={false}
                />
              </div>
            </div>
          ) : viewMode === 'project' ? (
            <ProjectList
              sessions={displayedSessions}
              selectedSession={selectedSession}
              selectedProject={selectedProject}
              onSelectSession={handleSelectSession}
              onSelectProject={setSelectedProject}
              loading={loading}
              terminal={terminal}
              piPath={piPath}
              customCommand={customCommand}
              getBadgeType={getBadgeType}
              scrollParentRef={listScrollRef}
            />
          ) : viewMode === 'directory' ? (
            <SessionListByDirectory
              sessions={displayedSessions}
              selectedSession={selectedSession}
              onSelectSession={handleSelectSession}
              onDeleteSession={handleDeleteSession}
              loading={loading}
              terminal={terminal}
              piPath={piPath}
              customCommand={customCommand}
              getBadgeType={getBadgeType}
              scrollParentRef={listScrollRef}
            />
          ) : (
            <SessionList
              sessions={displayedSessions}
              selectedSession={selectedSession}
              onSelectSession={handleSelectSession}
              onDeleteSession={handleDeleteSession}
              loading={loading}
              getBadgeType={getBadgeType}
              terminal={terminal}
              piPath={piPath}
              customCommand={customCommand}
              scrollParentRef={listScrollRef}
            />
          )}
        </div>
      </div>

      <div className="flex-1 overflow-hidden flex flex-col">
        <div className="flex-1 overflow-hidden">
          {selectedSession ? (
            <SessionViewer
              session={selectedSession}
              onExport={() => setShowExportDialog(true)}
              onRename={() => setShowRenameDialog(true)}
              onBack={() => setSelectedSession(null)}
              terminal={terminal}
              piPath={piPath}
              customCommand={customCommand}
            />
          ) : (
            <Dashboard
              sessions={selectedProject
                ? sessions.filter(s => s.cwd === selectedProject)
                : sessions
              }
              onSessionSelect={setSelectedSession}
              projectName={selectedProject || undefined}
            />
          )}
        </div>
      </div>

      {showExportDialog && selectedSession && (
        <ExportDialog
          session={selectedSession}
          onExport={onExportSession}
          onClose={() => setShowExportDialog(false)}
        />
      )}

      {showRenameDialog && selectedSession && (
        <RenameDialog
          session={selectedSession}
          onRename={onRenameSession}
          onClose={() => setShowRenameDialog(false)}
        />
      )}

      <SettingsPanel
        isOpen={showSettings}
        onClose={() => setShowSettings(false)}
      />

      <CommandPalette context={commandContext} />
    </div>
  )
}

function mapSearchResults(results: SearchResult[], allSessions: SessionInfo[]): SessionInfo[] {
  return results.map((r) => {
    const originalSession = allSessions.find(s => s.id === r.session_id)

    return {
      path: r.session_path,
      id: r.session_id,
      cwd: originalSession?.cwd || '',
      name: r.session_name || originalSession?.name,
      created: originalSession?.created || new Date().toISOString(),
      modified: originalSession?.modified || new Date().toISOString(),
      message_count: r.matches.length,
      first_message: r.first_message,
      all_messages_text: '',
      last_message: originalSession?.last_message || '',
      last_message_role: originalSession?.last_message_role || 'user',
    }
  })
}

export default App
