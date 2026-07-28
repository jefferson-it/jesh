import { DocPage } from '../components/DocPage'
import { useLanguage } from '../context/LanguageContext'

export default function Jobs() {
  const { t } = useLanguage()

  return (
    <DocPage title={t('jobs.title')}>
      <p>{t('jobs.intro')}</p>

      <h2>{t('jobs.background')}</h2>
      <p>{t('jobs.backgroundDesc')}</p>
      <pre><code>{t('jobs.backgroundExample')}</code></pre>
      <p>{t('jobs.backgroundNote')}</p>

      <h2>{t('jobs.managing')}</h2>
      <p>{t('jobs.managingDesc')}</p>

      <h3><code>jobs</code> — {t('jobs.jobs.title')}</h3>
      <p>{t('jobs.jobs.desc')}</p>
      <pre><code>{t('jobs.jobs.example')}</code></pre>
      <p>{t('jobs.jobs.indicators')}</p>
      <ul>
        <li><code>-l</code> — {t('jobs.jobs.opts.l')}</li>
        <li><code>-p</code> — {t('jobs.jobs.opts.p')}</li>
        <li><code>-r</code> — {t('jobs.jobs.opts.r')}</li>
        <li><code>-s</code> — {t('jobs.jobs.opts.s')}</li>
      </ul>

      <h3><code>fg</code> — {t('jobs.fg.title')}</h3>
      <p>{t('jobs.fg.desc')}</p>
      <pre><code>{t('jobs.fg.example')}</code></pre>
      <p>{t('jobs.fg.note')}</p>

      <h3><code>bg</code> — {t('jobs.bg.title')}</h3>
      <p>{t('jobs.bg.desc')}</p>
      <pre><code>{t('jobs.bg.example')}</code></pre>
      <p>{t('jobs.bg.note')}</p>

      <h3><code>disown</code> — {t('jobs.disown.title')}</h3>
      <p>{t('jobs.disown.desc')}</p>
      <pre><code>{t('jobs.disown.example')}</code></pre>
      <p>{t('jobs.disown.note')}</p>
      <ul>
        <li><code>-h</code> — {t('jobs.disown.opts.h')}</li>
        <li><code>-a</code> — {t('jobs.disown.opts.a')}</li>
      </ul>

      <h2>{t('jobs.shortcuts')}</h2>
      <table>
        <thead><tr><th>{t('jobs.shortcutsTable.shortcut')}</th><th>{t('jobs.shortcutsTable.action')}</th><th>{t('jobs.shortcutsTable.signal')}</th></tr></thead>
        <tbody>
          <tr><td><kbd>Ctrl+Z</kbd></td><td>{t('jobs.shortcutsTable.ctrlZ')}</td><td><code>SIGTSTP</code></td></tr>
          <tr><td><kbd>Ctrl+C</kbd></td><td>{t('jobs.shortcutsTable.ctrlC')}</td><td><code>SIGINT</code></td></tr>
          <tr><td><kbd>Ctrl+D</kbd></td><td>{t('jobs.shortcutsTable.ctrlD')}</td><td>—</td></tr>
        </tbody>
      </table>

      <h3><kbd>Ctrl+Z</kbd> — {t('jobs.ctrlZ.title')}</h3>
      <p>{t('jobs.ctrlZ.desc')}</p>

      <h3><kbd>Ctrl+C</kbd> — {t('jobs.ctrlC.title')}</h3>
      <p>{t('jobs.ctrlC.desc')}</p>

      <h3><kbd>Ctrl+D</kbd> — {t('jobs.ctrlD.title')}</h3>
      <p>{t('jobs.ctrlD.desc')}</p>

      <h2>{t('jobs.processGroup')}</h2>
      <p>{t('jobs.processGroup.desc')}</p>
      <p>{t('jobs.processGroup.note')}</p>

      <h2>{t('jobs.asyncNotify')}</h2>
      <p>{t('jobs.asyncNotify.desc')}</p>
      <pre><code>{t('jobs.asyncNotify.example')}</code></pre>
      <p>{t('jobs.asyncNotify.note')}</p>
    </DocPage>
  )
}