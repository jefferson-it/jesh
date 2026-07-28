import { Link, useLocation } from 'react-router-dom'
import { useTheme } from '../context/ThemeContext'
import { useLanguage } from '../context/LanguageContext'
import { LanguageSelector } from './LanguageSelector'

const navLinks = [
  { to: '/docs/getting-started', label: 'nav.gettingStarted' },
  { to: '/docs/configuration', label: 'nav.config' },
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

  return (
    <>
      <div className="nav-container">
        <div className="container">
          <Link to="/docs" className="nav-logo">🐚 jesh</Link>
          <div className="nav-links">
            {navLinks.map(l => (
              <Link key={l.to} to={l.to}>{t(l.label)}</Link>
            ))}
            <a href="https://github.com/jefferson-it/jesh">{t('nav.github')}</a>
            <LanguageSelector />
            <button id="theme-btn" className="theme-btn" onClick={toggle} aria-label={t('theme.toggle')}>
              {theme === 'dark' ? '☀️' : '🌙'}
            </button>
          </div>
        </div>
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
