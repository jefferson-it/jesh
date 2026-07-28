import { DocPage } from '../components/DocPage'
import { useLanguage } from '../context/LanguageContext'

export default function Examples() {
  const { t } = useLanguage()

  return (
    <DocPage title={t('examples.title')}>
      <p>{t('examples.intro')}</p>

      <h2>{t('examples.migration')}</h2>
      <p>{t('examples.migrationDesc')}</p>
      <table>
        <thead><tr><th>Bash (<code>.bashrc</code>)</th><th>jesh (<code>.jeshrc</code>)</th></tr></thead>
        <tbody>
          <tr>
            <td><pre><code>{t('examples.migration.editor')}</code></pre></td>
            <td><pre><code>{t('examples.migration.editor')}</code></pre></td>
          </tr>
          <tr>
            <td><pre><code>{t('examples.migration.path')}</code></pre></td>
            <td><pre><code>{t('examples.migration.path')}</code></pre></td>
          </tr>
          <tr>
            <td><pre><code>{t('examples.migration.aliases')}</code></pre></td>
            <td><pre><code>{t('examples.migration.aliases')}</code></pre></td>
          </tr>
          <tr>
            <td><pre><code>{t('examples.migration.ps1')}</code></pre></td>
            <td><pre><code>{t('examples.migration.prompt')}</code></pre></td>
          </tr>
          <tr>
            <td><pre><code>{t('examples.migration.cargo')}</code></pre></td>
            <td><pre><code>{t('examples.migration.cargo')}</code></pre></td>
          </tr>
          <tr>
            <td><pre><code>{t('examples.migration.bashCompletion')}</code></pre></td>
            <td><pre><code>{t('examples.migration.jeshCompletion')}</code></pre></td>
          </tr>
        </tbody>
      </table>

      <h2>{t('examples.completions')}</h2>

      <h3>{t('examples.completions.static')}</h3>
      <p>{t('examples.completions.staticDesc')}</p>
      <pre><code>{t('examples.completions.staticExample')}</code></pre>
      <p>{t('examples.completions.staticNote')}</p>

      <h3>{t('examples.completions.dynamic')}</h3>
      <p>{t('examples.completions.dynamicDesc')}</p>
      <pre><code>{t('examples.completions.dynamicExample')}</code></pre>
      <p>{t('examples.completions.dynamicNote')}</p>

      <h2>{t('examples.integrations')}</h2>

      <h3>{t('examples.integrations.nvm')}</h3>
      <p>{t('examples.integrations.nvmDesc')}</p>
      <pre><code>{t('examples.integrations.nvmExample')}</code></pre>
      <p>{t('examples.integrations.nvmNote')}</p>

      <h3>{t('examples.integrations.rust')}</h3>
      <p>{t('examples.integrations.rustDesc')}</p>
      <pre><code>{t('examples.integrations.rustExample')}</code></pre>
      <p>{t('examples.integrations.rustNote')}</p>

      <h3>{t('examples.integrations.python')}</h3>
      <p>{t('examples.integrations.pythonDesc')}</p>
      <pre><code>{t('examples.integrations.pythonExample')}</code></pre>
      <p>{t('examples.integrations.pythonNote')}</p>

      <h3>{t('examples.integrations.deno')}</h3>
      <p>{t('examples.integrations.denoDesc')}</p>
      <pre><code>{t('examples.integrations.denoExample')}</code></pre>
      <p>{t('examples.integrations.denoNote')}</p>

      <h2>{t('examples.scripting')}</h2>
      <p>{t('examples.scriptingDesc')}</p>

      <h3>{t('examples.scripting.loops')}</h3>
      <pre><code>{t('examples.scripting.loopsExample')}</code></pre>

      <h3>{t('examples.scripting.conditionals')}</h3>
      <pre><code>{t('examples.scripting.conditionalsExample')}</code></pre>

      <h3>{t('examples.scripting.pipes')}</h3>
      <pre><code>{t('examples.scripting.pipesExample')}</code></pre>

      <h3>{t('examples.scripting.getopts')}</h3>
      <pre><code>{t('examples.scripting.getoptsExample')}</code></pre>
    </DocPage>
  )
}