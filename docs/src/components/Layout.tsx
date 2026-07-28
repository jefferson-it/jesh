import { Link, useLocation } from 'react-router-dom'
import { useTheme } from '../context/ThemeContext'

const navLinks = [
  { to: '/docs/getting-started', label: 'Getting Started' },
  { to: '/docs/configuration', label: 'Config' },
  { to: '/docs/builtins', label: 'Builtins' },
]

const sidebarLinks = [
  { to: '/docs/getting-started', label: 'Getting Started' },
  { to: '/docs/configuration', label: 'Configuration' },
  { to: '/docs/builtins', label: 'Builtins' },
  { to: '/docs/scripting', label: 'Scripting' },
  { to: '/docs/parser', label: 'Parser' },
  { to: '/docs/globbing', label: 'Globbing' },
  { to: '/docs/autocomplete', label: 'Autocomplete' },
  { to: '/docs/prompt', label: 'Prompt' },
  { to: '/docs/jobs', label: 'Jobs & Processes' },
  { to: '/docs/history', label: 'History' },
  { to: '/docs/vs-bash', label: 'Vs Bash' },
  { to: '/docs/examples', label: 'Examples' },
]

export function Layout({ children }: { children: React.ReactNode }) {
  const { theme, toggle } = useTheme()
  const loc = useLocation()
  const isLanding = loc.pathname === '/docs' || loc.pathname === '/docs/'

  return (
    <>
      <div className="nav-container">
        <div className="container">
          <Link to="/docs" className="nav-logo">🐚 jesh</Link>
          <div className="nav-links">
            {navLinks.map(l => (
              <Link key={l.to} to={l.to}>{l.label}</Link>
            ))}
            <a href="https://github.com/jefferson-it/jesh">GitHub</a>
            <button id="theme-btn" className="theme-btn" onClick={toggle}>
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
              <Link to="/docs">📦 Crate</Link>
              <Link to="/docs/getting-started" className="active">📚 Documentation</Link>
              <a href="https://github.com/jefferson-it/jesh">📂 Source</a>
            </div>
          </div>
        </div>
      )}

      {children}

      <div className="footer">
        <div className="container">
          jesh 2.0.1 — Built with Rust &bull;
          <a href="https://github.com/jefferson-it/jesh">GitHub</a> &bull;
          MIT License
        </div>
      </div>
    </>
  )
}

export { sidebarLinks }
