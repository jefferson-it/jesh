import { DocPage } from '../components/DocPage'
import { useLanguage } from '../context/LanguageContext'

export default function Configuration() {
  const { t } = useLanguage()

  return (
    <DocPage title={t('configuration.title')}>
      <h2><code>~/.jeshrc</code> {t('configuration.variables')}</h2>
      <p dangerouslySetInnerHTML={{ __html: t('configuration.variablesDesc') }}></p>

      <h3><code>INIT_INFO</code></h3>
      <p dangerouslySetInnerHTML={{ __html: t('configuration.initInfo') }}></p>

      <h3><code>HOT_RELOAD</code></h3>
      <p dangerouslySetInnerHTML={{ __html: t('configuration.hotReload') }}></p>

      <h3><code>SHOW_TIMING</code></h3>
      <p dangerouslySetInnerHTML={{ __html: t('configuration.showTiming') }}></p>

      <h3><code>JSH_TAB_MODE</code></h3>
      <p dangerouslySetInnerHTML={{ __html: t('configuration.tabMode') }}></p>
      <ul>
        <li><code>complete</code> — {t('configuration.tabMode.complete')}</li>
        <li><code>menu-complete</code> — {t('configuration.tabMode.menuComplete')}</li>
        <li><code>insert-tab</code> — {t('configuration.tabMode.insertTab')}</li>
      </ul>

      <h3><code>JSH_TRANSIENT_PROMPT</code></h3>
      <p dangerouslySetInnerHTML={{ __html: t('configuration.transientPrompt') }}></p>

      <h3><code>THEME</code></h3>
      <p dangerouslySetInnerHTML={{ __html: t('configuration.theme') }}></p>
      <ul>
        <li><code>jesh-dark</code> — {t('configuration.themes.dark')}</li>
        <li><code>jesh-light</code> — {t('configuration.themes.light')}</li>
        <li><code>jesh-dracula</code> — {t('configuration.themes.dracula')}</li>
        <li><code>jesh-nord</code> — {t('configuration.themes.nord')}</li>
        <li><code>jesh-solarized</code> — {t('configuration.themes.solarized')}</li>
      </ul>
      <p>{t('configuration.setTheme')}</p>
      <pre><code>THEME="jesh-dracula"</code></pre>

      <h2><code>config.toml</code></h2>
      <p dangerouslySetInnerHTML={{ __html: t('configuration.tomlDesc') }}></p>
      <p>{t('configuration.example')}</p>
      <pre><code>[history]
max_entries = 10000
sync = true
filter_duplicates = true
dir_aware = true

[completion]
fuzzy = true
case_sensitive = false
menu_lines = 10

[editor]
vi_mode = false
external_editor = "vim"</code></pre>

      <h3><code>[history]</code> {t('configuration.sections.history')}</h3>
      <ul>
        <li><code>max_entries</code> — {t('configuration.history.maxEntries')}</li>
        <li><code>sync</code> — {t('configuration.history.sync')}</li>
        <li><code>filter_duplicates</code> — {t('configuration.history.filterDuplicates')}</li>
        <li><code>dir_aware</code> — {t('configuration.history.dirAware')}</li>
      </ul>

      <h2>{t('configuration.envVars')}</h2>
      <p>{t('configuration.envVarsDesc')}</p>
      <ul>
        <li><code>$EDITOR</code> — {t('configuration.envVars.editor')}</li>
        <li><code>$PAGER</code> — {t('configuration.envVars.pager')}</li>
        <li><code>$SHELL</code> — {t('configuration.envVars.shell')}</li>
        <li><code>$JESH_VERSION</code> — {t('configuration.envVars.version')}</li>
        <li><code>$PWD</code> / <code>$OLDPWD</code> — {t('configuration.envVars.pwd')}</li>
      </ul>
    </DocPage>
  )
}