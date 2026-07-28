import { DocPage } from '../components/DocPage'
import { useLanguage } from '../context/LanguageContext'

export default function History() {
  const { t } = useLanguage()

  return (
    <DocPage title={t('history.title')}>
      <p>{t('history.intro')}</p>

      <h2>{t('history.storage')}</h2>
      <p>{t('history.storageDesc')}</p>
      <table>
        <thead><tr><th>{t('history.storageTable.field')}</th><th>{t('history.storageTable.type')}</th><th>{t('history.storageTable.description')}</th></tr></thead>
        <tbody>
          <tr><td><code>cmd</code></td><td>string</td><td>{t('history.storageTable.cmd')}</td></tr>
          <tr><td><code>cwd</code></td><td>string</td><td>{t('history.storageTable.cwd')}</td></tr>
          <tr><td><code>exit</code></td><td>number</td><td>{t('history.storageTable.exit')}</td></tr>
          <tr><td><code>ts</code></td><td>string</td><td>{t('history.storageTable.ts')}</td></tr>
          <tr><td><code>count</code></td><td>number</td><td>{t('history.storageTable.count')}</td></tr>
          <tr><td><code>last</code></td><td>string</td><td>{t('history.storageTable.last')}</td></tr>
          <tr><td><code>pinned</code></td><td>boolean</td><td>{t('history.storageTable.pinned')}</td></tr>
          <tr><td><code>session</code></td><td>string</td><td>{t('history.storageTable.session')}</td></tr>
        </tbody>
      </table>
      <p>{t('history.storageExample')}</p>
      <pre><code>{t('history.storageExampleJson')}</code></pre>

      <h2>{t('history.navigation')}</h2>
      <p>{t('history.navigationDesc')}</p>
      <p>{t('history.navigationLocal')}</p>

      <h2>{t('history.reverseSearch')}</h2>
      <p>{t('history.reverseSearchDesc')}</p>
      <p>{t('history.reverseSearchFuzzy')}</p>
      <p>{t('history.reverseSearchPinned')}</p>

      <h2>{t('history.builtins')}</h2>

      <h3><code>history</code></h3>
      <p>{t('history.builtin.desc')}</p>
      <pre><code>{t('history.builtin.example')}</code></pre>

      <h3><code>history pin</code></h3>
      <p>{t('history.builtin.pinDesc')}</p>
      <pre><code>{t('history.builtin.pinExample')}</code></pre>

      <h3><code>history unpin</code></h3>
      <p>{t('history.builtin.unpinDesc')}</p>
      <pre><code>{t('history.builtin.unpinExample')}</code></pre>

      <h3><code>history clear</code></h3>
      <p>{t('history.builtin.clearDesc')}</p>
      <pre><code>{t('history.builtin.clearExample')}</code></pre>

      <h3><code>history tty</code></h3>
      <p>{t('history.builtin.ttyDesc')}</p>
      <pre><code>{t('history.builtin.ttyExample')}</code></pre>

      <h2>{t('history.variables')}</h2>
      <table>
        <thead><tr><th>{t('history.variablesTable.variable')}</th><th>{t('history.variablesTable.default')}</th><th>{t('history.variablesTable.description')}</th></tr></thead>
        <tbody>
          <tr><td><code>$HISTSIZE</code></td><td>5000</td><td>{t('history.variablesTable.histsize')}</td></tr>
          <tr><td><code>$HISTFILESIZE</code></td><td>10000</td><td>{t('history.variablesTable.histfilesize')}</td></tr>
          <tr><td><code>$HISTIGNORE</code></td><td>unset</td><td>{t('history.variablesTable.histignore')}</td></tr>
          <tr><td><code>$HISTCONTROL</code></td><td>unset</td><td>{t('history.variablesTable.histcontrol')}</td></tr>
        </tbody>
      </table>
      <p>{t('history.variablesExample')}</p>
      <pre><code>{t('history.variablesExampleCode')}</code></pre>
      <ul>
        <li><code>ignoredups</code> — {t('history.variables.ignoredups')}</li>
        <li><code>ignorespace</code> — {t('history.variables.ignorespace')}</li>
        <li><code>erasedups</code> — {t('history.variables.erasedups')}</li>
      </ul>

      <h2>{t('history.sync')}</h2>
      <p>{t('history.syncDesc')}</p>
      <p>{t('history.syncMechanism')}</p>

      <h2>{t('history.autosuggestions')}</h2>
      <p>{t('history.autosuggestionsDesc')}</p>
      <p>{t('history.autosuggestionsKey')}</p>
      <p>{t('history.autosuggestionsAlgorithm')}</p>
    </DocPage>
  )
}