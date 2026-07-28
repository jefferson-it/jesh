import { Link } from 'react-router-dom'
import { useLanguage } from '../context/LanguageContext'

export function Landing() {
  const { t } = useLanguage()

  return (
    <>
      <div className="crate-header">
        <div className="container">
          <h1>{t('landing.title')} <span className="version">{t('crateHeader.version')}</span></h1>
          <div className="description">{t('landing.description')}</div>
          <div className="badges">
            <img src="https://img.shields.io/github/stars/jefferson-it/jesh?style=social" alt="Stars" />
            <img src="https://img.shields.io/badge/Rust-1.84+-purple" alt="Rust" />
            <img src="https://img.shields.io/badge/license-MIT-blue" alt="License" />
            <img src="https://img.shields.io/badge/platform-linux%20%7C%20macOS%20%7C%20windows-lightgrey" alt="Platform" />
          </div>
          <div className="crate-tabs">
            <a href="/docs" className="active">{t('crateHeader.crate')}</a>
            <Link to="/docs/getting-started">{t('crateHeader.documentation')}</Link>
            <a href="https://github.com/jefferson-it/jesh">{t('crateHeader.source')}</a>
          </div>
        </div>
      </div>

      <div className="container package-page">
        <div className="two-col">
          <aside className="sidebar">
            <div className="sidebar-section">
              <h3>{t('landing.quickLinks')}</h3>
              <nav>
                <a href="#install">{t('landing.installation.curl')}</a>
                <a href="#features">{t('landing.features.title')}</a>
                <a href="#builtins">{t('landing.builtins.title')}</a>
                <a href="#docs">{t('landing.documentation')}</a>
              </nav>
            </div>
            <div className="sidebar-section">
              <h3>Links</h3>
              <nav>
                <a href="https://github.com/jefferson-it/jesh">⭐ GitHub</a>
                <a href="https://github.com/jefferson-it/jesh/issues">🐛 Issues</a>
              </nav>
            </div>
          </aside>

          <div className="main">
            <div className="warning-banner">
              ⚠️ {t('landing.warning')} <Link to="/docs/vs-bash">{t('landing.seeVsBash')}</Link>
            </div>

            <h2 id="install">{t('landing.installation.curl')}</h2>
            <div className="install-block primary">
              <div className="cmd">
                <code>{t('landing.installation.curlCmd')}</code>
                <CopyBtn text={t('landing.installation.curlCmd')} />
              </div>
            </div>

            <h2 id="install-or-build">{t('landing.installation.orBuild')}</h2>
            <div className="install-block">
              <strong>{t('landing.installation.source')}</strong>
              <div className="cmd">
                <code>{t('landing.installation.sourceCmd')}</code>
                <CopyBtn text={t('landing.installation.sourceCmd')} />
              </div>
            </div>
            <div className="install-block">
              <strong>{t('landing.installation.cargo')}</strong>
              <div className="cmd">
                <code>{t('landing.installation.cargoCmd')}</code>
                <CopyBtn text={t('landing.installation.cargoCmd')} />
              </div>
            </div>

            <h2 id="features">{t('landing.features.title')}</h2>
            <div className="feature-grid">
              {[
                { icon: '🧠', title: 'landing.features.intelligentHistory.title', desc: 'landing.features.intelligentHistory.desc' },
                { icon: '💡', title: 'landing.features.autosuggestions.title', desc: 'landing.features.autosuggestions.desc' },
                { icon: '🔍', title: 'landing.features.fuzzySearch.title', desc: 'landing.features.fuzzySearch.desc' },
                { icon: '⚡', title: 'landing.features.fastParser.title', desc: 'landing.features.fastParser.desc' },
                { icon: '🎨', title: 'landing.features.richPrompt.title', desc: 'landing.features.richPrompt.desc' },
                { icon: '📋', title: 'landing.features.tabCompletion.title', desc: 'landing.features.tabCompletion.desc' },
                { icon: '🔧', title: 'landing.features.builtins.title', desc: 'landing.features.builtins.desc' },
                { icon: '🖥️', title: 'landing.features.terminalProtocols.title', desc: 'landing.features.terminalProtocols.desc' },
                { icon: '🔀', title: 'landing.features.bashFallback.title', desc: 'landing.features.bashFallback.desc' },
                { icon: '🌐', title: 'landing.features.crossPlatform.title', desc: 'landing.features.crossPlatform.desc' },
              ].map((f, i) => (
                <div key={i} className="feature-item">
                  <strong>{f.icon} {t(f.title)}</strong>
                  <span>{t(f.desc)}</span>
                </div>
              ))}
            </div>

            <h2 id="builtins">{t('landing.builtins.title')}</h2>
            <table>
              <thead><tr><th>{t('landing.builtins.command')}</th><th>{t('landing.builtins.description')}</th></tr></thead>
              <tbody>
                {[
                  ['cd', 'landing.builtins.cd'],
                  ['pushd / popd / dirs', 'landing.builtins.pushd'],
                  ['export / unset', 'landing.builtins.export'],
                  ['alias / unalias', 'landing.builtins.alias'],
                  ['source / .', 'landing.builtins.source'],
                  ['history', 'landing.builtins.history'],
                  ['set / shopt', 'landing.builtins.set'],
                  ['declare / typeset', 'landing.builtins.declare'],
                  ['local / readonly', 'landing.builtins.local'],
                  ['getopts', 'landing.builtins.getopts'],
                  ['eval / exec / command', 'landing.builtins.eval'],
                  ['test / [ / [[', 'landing.builtins.test'],
                  ['read / printf / echo', 'landing.builtins.read'],
                  ['jobs / fg / bg / disown / kill', 'landing.builtins.jobs'],
                  ['type / which', 'landing.builtins.type'],
                  ['complete', 'landing.builtins.complete'],
                ].map(([cmd, descKey], i) => (
                  <tr key={i}><td><code>{cmd}</code></td><td>{t(descKey)}</td></tr>
                ))}
              </tbody>
            </table>

            <h2 id="docs">{t('landing.documentation')}</h2>
            <div className="quick-links">
              {[
                ['gettingStarted.title', '/docs/getting-started'],
                ['configuration.title', '/docs/configuration'],
                ['builtins.title', '/docs/builtins'],
                ['scripting.title', '/docs/scripting'],
                ['parser.title', '/docs/parser'],
                ['globbing.title', '/docs/globbing'],
                ['autocomplete.title', '/docs/autocomplete'],
                ['prompt.title', '/docs/prompt'],
                ['jobs.title', '/docs/jobs'],
                ['history.title', '/docs/history'],
                ['vsBash.title', '/docs/vs-bash'],
                ['examples.title', '/docs/examples'],
              ].map(([labelKey, to]) => (
                <Link key={to} to={to}>{t(labelKey)}</Link>
              ))}
            </div>

            <h2>{t('landing.quickStart.title')}</h2>
            <p>{t('landing.quickStart.createConfig')}</p>
            <pre><code>{t('landing.quickStart.configExample')}</code></pre>
            <p dangerouslySetInnerHTML={{ __html: t('landing.quickStart.runJesh') }}></p>
          </div>
        </div>
      </div>
    </>
  )
}

function CopyBtn({ text }: { text: string }) {
  const copy = () => {
    navigator.clipboard.writeText(text).then(() => {
      const btn = document.activeElement as HTMLElement
      if (btn) {
        const orig = btn.textContent
        btn.textContent = 'Copied!'
        setTimeout(() => { btn.textContent = orig }, 1500)
      }
    })
  }
  return <button className="copy" onClick={copy}>Copy</button>
}