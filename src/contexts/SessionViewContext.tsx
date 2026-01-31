import { createContext, useContext, useState, ReactNode } from 'react'

interface SessionViewContextType {
  showThinking: boolean
  toggleThinking: () => void
  toolsExpanded: boolean
  toggleToolsExpanded: () => void
}

const SessionViewContext = createContext<SessionViewContextType | undefined>(undefined)

export function SessionViewProvider({ children }: { children: ReactNode }) {
  const [showThinking, setShowThinking] = useState(true)
  const [toolsExpanded, setToolsExpanded] = useState(false)

  const toggleThinking = () => setShowThinking(prev => !prev)
  const toggleToolsExpanded = () => setToolsExpanded(prev => !prev)

  return (
    <SessionViewContext.Provider
      value={{
        showThinking,
        toggleThinking,
        toolsExpanded,
        toggleToolsExpanded,
      }}
    >
      {children}
    </SessionViewContext.Provider>
  )
}

export function useSessionView() {
  const context = useContext(SessionViewContext)
  if (!context) {
    throw new Error('useSessionView must be used within SessionViewProvider')
  }
  return context
}