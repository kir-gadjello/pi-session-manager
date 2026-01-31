import { useEffect, useState } from 'react'

/**
 * 获取当前实际使用的主题（解析 system 主题）
 * @returns 'dark' | 'light'
 */
export function useResolvedTheme(): 'dark' | 'light' {
  const [resolvedTheme, setResolvedTheme] = useState<'dark' | 'light'>(() => {
    // 初始化：检查 DOM 类名或系统偏好
    if (document.documentElement.classList.contains('theme-dark')) {
      return 'dark'
    }
    if (document.documentElement.classList.contains('theme-light')) {
      return 'light'
    }
    // 如果没有类名，使用系统偏好
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
  })

  useEffect(() => {
    const updateTheme = () => {
      if (document.documentElement.classList.contains('theme-dark')) {
        setResolvedTheme('dark')
      } else if (document.documentElement.classList.contains('theme-light')) {
        setResolvedTheme('light')
      } else {
        // system 主题：使用系统偏好
        setResolvedTheme(
          window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
        )
      }
    }

    // 监听 DOM 类名变化
    const observer = new MutationObserver(updateTheme)
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ['class'],
    })

    // 监听系统主题变化
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
    const handleChange = () => {
      // 只有在 system 主题时才响应系统变化
      if (
        !document.documentElement.classList.contains('theme-dark') &&
        !document.documentElement.classList.contains('theme-light')
      ) {
        updateTheme()
      }
    }
    mediaQuery.addEventListener('change', handleChange)

    return () => {
      observer.disconnect()
      mediaQuery.removeEventListener('change', handleChange)
    }
  }, [])

  return resolvedTheme
}
