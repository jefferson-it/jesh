import { useState, useEffect } from 'react'
import { Link, useLocation } from 'react-router-dom'
import { useTheme } from '../context/ThemeContext'
import { useLanguage } from '../context/LanguageContext'
import { LanguageSelector } from './LanguageSelector'

const navLinks = [
  { to: '/docs/getting-started', label: 'nav.gettingStarted' },
  { to: '/docs/configuration', label: 'nav.configuration' },
  { to: '/docs/builtins', label: 'nav.builtins' },
]

const sidebarLinks = [
  { to: '/docs/getting-started', label: 'gettingStarted.title' },
  { to: '/docs/configuration', label: 'configuration.title' },
  { to: '/docs/builtins', label: 'builtins.title' },
  { to: '/docs/scripting', label: 'scripting.title' },
  { to: '/docs/parser', label: 'parser.title' },
  { to: '/docs/globbing', label: 'globbing.title' },
  { to: '/docs/autocomplete', label: 'autocomplete.title' },
  { to: '/docs/prompt', label: 'prompt.title' },
  { to: '/docs/jobs', label: 'jobs.title' },
  { to: '/docs/history', label: 'history.title' },
  { to: '/docs/vs-bash', label: 'vsBash.title' },
  { to: '/docs/examples', label: 'examples.title' },
]

export function Layout({ children }: { children: React.ReactNode }) {
  const { theme, toggle } = useTheme()
  const { t } = useLanguage()
  const loc = useLocation()
  const isLanding = loc.pathname === '/docs' || loc.pathname === '/docs/'
  const [menuOpen, setMenuOpen] = useState(false)

  // Fechar menu ao navegar
  useEffect(() => {
    setMenuOpen(false)
  }, [loc.pathname])

  // Bloquear scroll do body quando menu aberto
  useEffect(() => {
    document.body.style.overflow = menuOpen ? 'hidden' : ''
    return () => { document.body.style.overflow = '' }
  }, [menuOpen])

  return (
    <>
      <div className="nav-container">
        <div className="container">
          <Link to="/docs" className="nav-logo">🐚 jesh</Link>
          <div className="nav-links">
            {navLinks.map(l => (
              <Link key={l.to} to={l.to} className="nav-link-item">
                <span className="nav-link-label">{t(l.label)}</span>
              </Link>
            ))}
            <a href="https://github.com/jefferson-it/jesh" className="nav-link-item">
              <span className="nav-link-label">{t('nav.github')}</span>
            </a>
            <LanguageSelector />
            <button id="theme-btn" className="theme-btn" onClick={toggle} aria-label={t('theme.toggle')}>
              {theme === 'dark' ? '☀️' : '🌙'}
            </button>
            <button
              className={`hamburger${menuOpen ? ' is-open' : ''}`}
              onClick={() => setMenuOpen(o => !o)}
              aria-label="Toggle menu"
              aria-expanded={menuOpen}
            >
              <span />
              <span />
              <span />
            </button>
          </div>
        </div>
      </div>

      {/* Menu mobile */}
      {menuOpen && <div className="mobile-overlay" onClick={() => setMenuOpen(false)} />}
      <div className={`mobile-menu${menuOpen ? ' is-open' : ''}`}>
        <nav>
          {navLinks.map(l => (
            <Link key={l.to} to={l.to} onClick={() => setMenuOpen(false)}>{t(l.label)}</Link>
          ))}
          <a href="https://github.com/jefferson-it/jesh" onClick={() => setMenuOpen(false)}>
            {t('nav.github')}
          </a>
        </nav>
        <hr />
        <nav>
          {sidebarLinks.map(l => (
            <Link key={l.to} to={l.to} onClick={() => setMenuOpen(false)}>{t(l.label)}</Link>
          ))}
        </nav>
      </div>

      {!isLanding && (
        <div className="crate-header">
          <div className="container">
            <h1>jesh <span className="version">2.0.1</span></h1>
            <div className="crate-tabs">
              <Link to="/docs">{t('crateHeader.crate')}</Link>
              <Link to="/docs/getting-started" className="active">{t('crateHeader.documentation')}</Link>
              <a href="https://github.com/jefferson-it/jesh">{t('crateHeader.source')}</a>
            </div>
          </div>
        </div>
      )}

      {children}

      <div className="footer">
        <div className="container">
          {t('footer.version')} — {t('footer.builtWith')} &bull;
          <a href="https://github.com/jefferson-it/jesh">{t('footer.github')}</a> &bull;
          {t('footer.license')}
        </div>
      </div>
    </>
  )
}

export { sidebarLinks }
