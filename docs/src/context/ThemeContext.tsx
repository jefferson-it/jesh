import { createContext, useContext, useEffect, useState, type ReactNode } from 'react'

type Theme = 'light' | 'dark'

interface ThemeContextType {
  theme: Theme
  toggle: () => void
}

const ThemeContext = createContext<ThemeContextType>({ theme: 'light', toggle: () => {} })

function getInitialTheme(): Theme {
  // Se o usuário já escolheu manualmente, respeitar a preferência salva
  const stored = localStorage.getItem('jesh-theme') as Theme | null
  if (stored === 'light' || stored === 'dark') return stored

  // Caso contrário, detectar o tema do sistema
  if (window.matchMedia('(prefers-color-scheme: dark)').matches) return 'dark'
  return 'light'
}

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [theme, setTheme] = useState<Theme>(getInitialTheme)
  const [userOverride, setUserOverride] = useState<boolean>(
    () => localStorage.getItem('jesh-theme') !== null
  )

  // Listener para mudança do tema do sistema (apenas quando não há override manual)
  useEffect(() => {
    if (userOverride) return

    const mq = window.matchMedia('(prefers-color-scheme: dark)')
    const handler = (e: MediaQueryListEvent) => {
      setTheme(e.matches ? 'dark' : 'light')
    }

    mq.addEventListener('change', handler)
    return () => mq.removeEventListener('change', handler)
  }, [userOverride])

  useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme)
    if (userOverride) {
      localStorage.setItem('jesh-theme', theme)
    }
  }, [theme, userOverride])

  const toggle = () => {
    setUserOverride(true)
    setTheme(t => (t === 'dark' ? 'light' : 'dark'))
  }

  return <ThemeContext.Provider value={{ theme, toggle }}>{children}</ThemeContext.Provider>
}

export const useTheme = () => useContext(ThemeContext)
