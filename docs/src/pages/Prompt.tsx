import { DocPage } from '../components/DocPage'
import { useLanguage } from '../context/LanguageContext'

export default function Prompt() {
  const { t } = useLanguage()

  return (
    <DocPage title={t('prompt.title')}>
      <p>{t('prompt.intro')}</p>

      <h2>{t('prompt.variables')}</h2>
      <p>{t('prompt.variablesDesc')}</p>
      <table>
        <thead><tr><th>{t('prompt.variablesTable.variable')}</th><th>{t('prompt.variablesTable.description')}</th><th>{t('prompt.variablesTable.example')}</th></tr></thead>
        <tbody>
          <tr><td><code>PROMPT</code> / <code>PS1</code></td><td>{t('prompt.variablesTable.prompt')}</td><td><code>'%n@%m %~ %# '</code></td></tr>
          <tr><td><code>RPROMPT</code></td><td>{t('prompt.variablesTable.rprompt')}</td><td><code>{'\u0027%? %B%F{red}%*\u0027'}</code></td></tr>
          <tr><td><code>PROMPT2</code></td><td>{t('prompt.variablesTable.prompt2')}</td><td><code>{'\'> \''}</code></td></tr>
          <tr><td><code>SPROMPT</code></td><td>{t('prompt.variablesTable.sprompt')}</td><td><code>'zsh: correct %R to %r? '</code></td></tr>
        </tbody>
      </table>

      <h3>{t('prompt.escapeSequences')}</h3>
      <p>{t('prompt.escapeSequencesDesc')}</p>
      <table>
        <thead><tr><th>{t('prompt.escapeTable.sequence')}</th><th>{t('prompt.escapeTable.expandsTo')}</th></tr></thead>
        <tbody>
          <tr><td><code>%n</code></td><td>{t('prompt.escapeTable.n')}</td></tr>
          <tr><td><code>%m</code></td><td>{t('prompt.escapeTable.m')}</td></tr>
          <tr><td><code>%M</code></td><td>{t('prompt.escapeTable.M')}</td></tr>
          <tr><td><code>%~</code></td><td>{t('prompt.escapeTable.tilde')}</td></tr>
          <tr><td><code>%/</code> / <code>%d</code></td><td>{t('prompt.escapeTable.slash')}</td></tr>
          <tr><td><code>%?</code></td><td>{t('prompt.escapeTable.question')}</td></tr>
          <tr><td><code>%#</code></td><td>{t('prompt.escapeTable.hash')}</td></tr>
          <tr><td><code>%*</code></td><td>{t('prompt.escapeTable.star')}</td></tr>
          <tr><td><code>%T</code></td><td>{t('prompt.escapeTable.T')}</td></tr>
          <tr><td><code>%t</code> / <code>%@</code></td><td>{t('prompt.escapeTable.t')}</td></tr>
          <tr><td><code>%D</code></td><td>{t('prompt.escapeTable.D')}</td></tr>
          <tr><td><code>{'%F{color}'}</code></td><td>{t('prompt.escapeTable.F')}</td></tr>
          <tr><td><code>{'%K{color}'}</code></td><td>{t('prompt.escapeTable.K')}</td></tr>
          <tr><td><code>%B</code></td><td>{t('prompt.escapeTable.B')}</td></tr>
          <tr><td><code>%b</code></td><td>{t('prompt.escapeTable.b')}</td></tr>
          <tr><td><code>%E</code></td><td>{t('prompt.escapeTable.E')}</td></tr>
          <tr><td><code>%{'{'+'...%}'}</code></td><td>{t('prompt.escapeTable.literal')}</td></tr>
        </tbody>
      </table>

      <h3>{t('prompt.infoPlaceholders')}</h3>
      <p>{t('prompt.infoPlaceholdersDesc')}</p>
      <ul>
        <li><code>$JSH_GIT_BRANCH</code> — {t('prompt.infoPlaceholders.gitBranch')}</li>
        <li><code>$JSH_GIT_DIRTY</code> — {t('prompt.infoPlaceholders.gitDirty')}</li>
        <li><code>$JSH_OS_LOGO</code> — {t('prompt.infoPlaceholders.osLogo')}</li>
        <li><code>$JSH_CONTAINER</code> — {t('prompt.infoPlaceholders.container')}</li>
        <li><code>$JSH_SSH</code> — {t('prompt.infoPlaceholders.ssh')}</li>
      </ul>

      <h2>{t('prompt.rprompt')}</h2>
      <p>{t('prompt.rpromptDesc')}</p>
      <ul>
        <li>{t('prompt.rprompt.exitCode')}</li>
        <li>{t('prompt.rprompt.ssh')}</li>
        <li>{t('prompt.rprompt.git')}</li>
        <li>{t('prompt.rprompt.time')}</li>
      </ul>
      <pre><code>{t('prompt.rpromptExample')}</code></pre>
      <p>{t('prompt.rpromptConditional')}</p>

      <h2>{t('prompt.transient')}</h2>
      <p>{t('prompt.transientDesc')}</p>
      <p>{t('prompt.transientCustom')}</p>
      <pre><code>{t('prompt.transientExample')}</code></pre>
      <p>{t('prompt.transientRprompt')}</p>

      <h2>{t('prompt.asyncGit')}</h2>
      <p>{t('prompt.asyncGitDesc')}</p>
      <p>{t('prompt.asyncGitNote')}</p>

      <h2>{t('prompt.themes')}</h2>
      <p>{t('prompt.themesDesc')}</p>
      <pre><code>{t('prompt.themesExample')}</code></pre>
      <p>{t('prompt.themesList')}</p>
      <ul>
        <li><code>jesh-default</code> — {t('prompt.themes.default')}</li>
        <li><code>jesh-dark</code> — {t('prompt.themes.dark')}</li>
        <li><code>jesh-dracula</code> — {t('prompt.themes.dracula')}</li>
        <li><code>jesh-light</code> — {t('prompt.themes.light')}</li>
        <li><code>jesh-nord</code> — {t('prompt.themes.nord')}</li>
        <li><code>jesh-solarized</code> — {t('prompt.themes.solarized')}</li>
      </ul>
      <p>{t('prompt.themesVariables')}</p>
      <pre><code>{t('prompt.themesVariablesExample')}</code></pre>
      <p>{t('prompt.themesOsc')}</p>
      <pre><code>{t('prompt.themesOscExample')}</code></pre>

      <h2>{t('prompt.nerdFonts')}</h2>
      <p>{t('prompt.nerdFontsDesc')}</p>
      <pre><code>{t('prompt.nerdFontsExample')}</code></pre>
      <p>{t('prompt.nerdFontsNote')}</p>
    </DocPage>
  )
}