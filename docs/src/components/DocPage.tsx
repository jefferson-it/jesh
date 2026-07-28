import { Link } from 'react-router-dom'
import { sidebarLinks } from '../components/Layout'
import { useLanguage } from '../context/LanguageContext'

export function DocPage({ title, children }: { title: string; children: React.ReactNode }) {
  const { t } = useLanguage()
  return (
    <div className="container">
      <div className="two-col">
        <aside className="sidebar">
          <div className="sidebar-section">
            <h3>{t('landing.documentation')}</h3>
            <nav>
              {sidebarLinks.map(l => (
                <Link key={l.to} to={l.to}>{t(l.label)}</Link>
              ))}
            </nav>
          </div>
        </aside>
        <div className="main">
          <div className="content">
            <h1>{title}</h1>
            {children}
          </div>
        </div>
      </div>
    </div>
  )
}
