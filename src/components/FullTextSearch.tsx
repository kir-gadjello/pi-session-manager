import { useState, useEffect, useRef, useCallback } from 'react'
import { Search, X, Loader2, User, Bot, FileText, ChevronRight, Globe, Filter } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { invoke } from '@tauri-apps/api/core'
import type { FullTextSearchHit, FullTextSearchResponse, SessionInfo } from '../types'

interface FullTextSearchProps {
  isOpen: boolean
  onClose: () => void
  onSelectResult: (session: SessionInfo, entryId: string) => void
}

export default function FullTextSearch({ isOpen, onClose, onSelectResult }: FullTextSearchProps) {
  const { t } = useTranslation()
  const [query, setQuery] = useState('')
  const [roleFilter, setRoleFilter] = useState<'all' | 'user' | 'assistant'>('all')
  const [globPattern, setGlobPattern] = useState('')
  const [results, setResults] = useState<FullTextSearchHit[]>([])
  const [totalHits, setTotalHits] = useState(0)
  const [isSearching, setIsSearching] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [page, setPage] = useState(0)
  const [hasMore, setHasMore] = useState(false)
  
  const searchTimeoutRef = useRef<NodeJS.Timeout>()
  const inputRef = useRef<HTMLInputElement>(null)

  const performSearch = useCallback(async (searchQuery: string, role: string, glob: string, pageNum: number, append = false) => {
    if (!searchQuery.trim()) {
      setResults([])
      setTotalHits(0)
      setHasMore(false)
      setError(null)
      return
    }

    setIsSearching(true)
    setError(null)
    try {
      const response = await invoke<FullTextSearchResponse>('full_text_search', {
        query: searchQuery,
        roleFilter: role,
        globPattern: glob || null,
        page: pageNum,
        pageSize: 20
      })

      if (append) {
        setResults(prev => [...prev, ...response.hits])
      } else {
        setResults(response.hits)
      }
      setTotalHits(response.total_hits)
      setHasMore(response.has_more)
    } catch (err) {
      console.error('Full text search failed:', err)
      setError(typeof err === 'string' ? err : 'Search failed')
    } finally {
      setIsSearching(false)
    }
  }, [])

  useEffect(() => {
    if (searchTimeoutRef.current) {
      clearTimeout(searchTimeoutRef.current)
    }

    searchTimeoutRef.current = setTimeout(() => {
      setPage(0)
      performSearch(query, roleFilter, globPattern, 0)
    }, 300)

    return () => clearTimeout(searchTimeoutRef.current)
  }, [query, roleFilter, globPattern, performSearch])

  const handleLoadMore = () => {
    const nextPage = page + 1
    setPage(nextPage)
    performSearch(query, roleFilter, globPattern, nextPage, true)
  }

  const handleSelect = async (hit: FullTextSearchHit) => {
    try {
      // 获取完整的 SessionInfo 以便在 App 中设置
      const session = await invoke<SessionInfo>('get_session_by_path', { path: hit.session_path })
      if (session) {
        onSelectResult(session, hit.entry_id)
        onClose()
      }
    } catch (error) {
      console.error('Failed to get session for result:', error)
    }
  }

  // Handle ESC key
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && isOpen) {
        onClose()
      }
    }
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [isOpen, onClose])

  // Focus input on open
  useEffect(() => {
    if (isOpen) {
      setTimeout(() => inputRef.current?.focus(), 100)
    }
  }, [isOpen])

  const highlightText = (text: string, query: string) => {
    if (!query.trim()) return text
    const parts = query.trim().split(/\s+/)
    const pattern = new RegExp(`(${parts.join('|')})`, 'gi')
    return text.split(pattern).map((part, i) => 
      pattern.test(part) ? <mark key={i} className="bg-blue-500/30 text-blue-200 rounded-sm px-0.5 no-underline">{part}</mark> : part
    )
  }

  if (!isOpen) return null

  return (
    <div className="fixed inset-0 z-[60] flex items-start justify-center pt-[10vh] bg-black/40 backdrop-blur-sm animate-in fade-in duration-200" onClick={onClose}>
      <div 
        className="w-full max-w-3xl bg-[#1a1b26] border border-[#2a2b36] rounded-xl shadow-2xl flex flex-col max-h-[80vh] overflow-hidden animate-in zoom-in-95 duration-200"
        onClick={e => e.stopPropagation()}
      >
        {/* Search Header */}
        <div className="p-4 border-b border-[#2a2b36] bg-[#1f2029]">
          <div className="relative flex items-center gap-3 mb-4">
            <Search className="w-5 h-5 text-blue-400" />
            <input
              ref={inputRef}
              type="text"
              value={query}
              onChange={e => setQuery(e.target.value)}
              placeholder={t('search.fullText.placeholder')}
              className="flex-1 bg-transparent border-0 outline-none text-base text-foreground placeholder:text-muted-foreground font-medium"
            />
            {query && (
              <button onClick={() => setQuery('')} className="p-1 hover:bg-[#2a2b36] rounded-md transition-colors">
                <X className="w-4 h-4 text-muted-foreground" />
              </button>
            )}
          </div>

          {/* Filters Panel - Always Visible */}
          <div className="flex flex-wrap items-center gap-4">
            <div className="flex items-center gap-1 bg-[#252636] p-1 rounded-lg border border-[#2a2b36]">
                <button
                  onClick={() => setRoleFilter('all')}
                  className={`px-3 py-1.5 text-xs rounded-md transition-all flex items-center gap-1.5 ${roleFilter === 'all' ? 'bg-blue-500 text-white shadow-lg' : 'text-muted-foreground hover:text-foreground hover:bg-[#2a2b36]'}`}
                >
                  <div className="flex -space-x-1">
                    <User className="w-3 h-3" />
                    <Bot className="w-3 h-3" />
                  </div>
                  {t('search.fullText.role.all')} (用户+助手)
                </button>
              <button
                onClick={() => setRoleFilter('user')}
                className={`px-3 py-1.5 text-xs rounded-md transition-all flex items-center gap-1.5 ${roleFilter === 'user' ? 'bg-blue-500 text-white shadow-lg' : 'text-muted-foreground hover:text-foreground hover:bg-[#2a2b36]'}`}
              >
                <User className="w-3 h-3" />
                {t('search.fullText.role.user')}
              </button>
              <button
                onClick={() => setRoleFilter('assistant')}
                className={`px-3 py-1.5 text-xs rounded-md transition-all flex items-center gap-1.5 ${roleFilter === 'assistant' ? 'bg-blue-500 text-white shadow-lg' : 'text-muted-foreground hover:text-foreground hover:bg-[#2a2b36]'}`}
              >
                <Bot className="w-3 h-3" />
                {t('search.fullText.role.assistant')}
              </button>
            </div>

            <div className="flex-1 min-w-[240px] relative">
              <Globe className="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-muted-foreground" />
              <input
                type="text"
                value={globPattern}
                onChange={e => setGlobPattern(e.target.value)}
                placeholder={t('search.fullText.globPlaceholder')}
                className="w-full pl-8 pr-3 py-1.5 bg-[#252636] border border-[#2a2b36] rounded-lg text-xs outline-none focus:border-blue-500/50 transition-colors placeholder:text-muted-foreground/50 text-foreground"
              />
            </div>
          </div>
        </div>

        {/* Results Info */}
        <div className="px-4 py-2 bg-[#1a1b26] border-b border-[#2a2b36] flex items-center justify-between">
          <div className="text-[11px] text-muted-foreground">
            {isSearching ? (
              <div className="flex items-center gap-2">
                <Loader2 className="w-3 h-3 animate-spin text-blue-400" />
                <span>{t('search.searching')}</span>
              </div>
            ) : (
              <span>{t('search.fullText.resultsFound', { count: totalHits })}</span>
            )}
          </div>
          <div className="flex items-center gap-2 text-[10px] text-muted-foreground/50">
            <kbd className="px-1.5 py-0.5 bg-[#252636] border border-[#2a2b36] rounded text-[9px] font-mono uppercase tracking-wider">ESC</kbd>
            <span>{t('common.close')}</span>
          </div>
        </div>

        {/* Results List */}
        <div className="flex-1 overflow-y-auto p-2 min-h-0 custom-scrollbar bg-[#16161e]">
          {error ? (
            <div className="h-40 flex flex-col items-center justify-center text-red-400/80 bg-red-500/5 rounded-lg m-2 border border-red-500/10">
              <div className="p-3 bg-red-500/10 rounded-full mb-3">
                <X className="w-6 h-6" />
              </div>
              <p className="text-sm font-medium">{error}</p>
            </div>
          ) : results.length > 0 ? (
            <div className="space-y-1">
              {results.map((hit, idx) => (
                <button
                  key={`${hit.session_id}-${hit.entry_id}-${idx}`}
                  onClick={() => handleSelect(hit)}
                  className="w-full text-left p-3 rounded-lg hover:bg-[#252636] border border-transparent hover:border-[#3a3b46] transition-all group"
                >
                  <div className="flex items-start justify-between gap-3 mb-1.5">
                    <div className="flex items-center gap-2 min-w-0">
                      <FileText className="w-3.5 h-3.5 text-blue-400 flex-shrink-0" />
                      <span className="text-sm font-medium truncate text-foreground/90 group-hover:text-blue-400 transition-colors">
                        {hit.session_name || hit.session_path.split('/').pop()}
                      </span>
                    </div>
                    <div className="flex items-center gap-2 flex-shrink-0">
                      <span className={`px-1.5 py-0.5 rounded-[4px] text-[10px] font-bold uppercase tracking-widest border ${hit.role === 'user' ? 'bg-blue-500/10 text-blue-400 border-blue-500/20' : 'bg-purple-500/10 text-purple-400 border-purple-500/20'}`}>
                        {hit.role}
                      </span>
                      <span className="text-[10px] text-muted-foreground/60 whitespace-nowrap font-mono">
                        {new Date(hit.timestamp).toLocaleString()}
                      </span>
                    </div>
                  </div>
                  
                  <div className="pl-5.5 relative">
                    <div className="absolute left-1.5 top-1.5 bottom-1.5 w-0.5 bg-[#2a2b36] rounded-full group-hover:bg-blue-500/30 transition-colors" />
                    <p className="text-xs text-muted-foreground/80 leading-relaxed line-clamp-3 italic bg-[#1a1b26]/50 p-2 rounded border border-transparent group-hover:border-[#2a2b36] transition-all">
                      ...{highlightText(hit.snippet, query)}...
                    </p>
                  </div>

                  <div className="mt-2 pl-5.5 flex items-center gap-1.5 text-[10px] text-muted-foreground/40 overflow-hidden">
                    <span className="truncate max-w-[400px] font-mono">{hit.session_path}</span>
                    <ChevronRight className="w-2.5 h-2.5 flex-shrink-0 opacity-50" />
                    <span className="text-muted-foreground/60 font-mono">#{hit.entry_id.slice(0, 8)}</span>
                  </div>
                </button>
              ))}
              
              {hasMore && (
                <div className="p-4 flex justify-center">
                  <button
                    onClick={handleLoadMore}
                    disabled={isSearching}
                    className="px-6 py-2 bg-[#252636] hover:bg-[#2a2b36] border border-[#3a3b46] rounded-lg text-sm text-muted-foreground hover:text-foreground transition-all flex items-center gap-2 font-medium"
                  >
                    {isSearching ? <Loader2 className="w-4 h-4 animate-spin" /> : null}
                    {t('common.loadMore')}
                  </button>
                </div>
              )}
            </div>
          ) : !isSearching && query ? (
            <div className="h-40 flex flex-col items-center justify-center text-muted-foreground/60">
              <Search className="w-10 h-10 mb-3 opacity-20 text-blue-400" />
              <p className="text-sm font-medium">{t('search.noResults')}</p>
            </div>
          ) : !isSearching && (
            <div className="h-40 flex flex-col items-center justify-center text-muted-foreground/40">
              <div className="relative mb-4">
                <Search className="w-12 h-12 opacity-10" />
                <Filter className="w-5 h-5 absolute -bottom-1 -right-1 opacity-20 text-blue-400" />
              </div>
              <p className="text-sm font-medium">{t('search.fullText.startTyping')}</p>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
