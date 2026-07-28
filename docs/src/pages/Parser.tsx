import { DocPage } from '../components/DocPage'
import { useLanguage } from '../context/LanguageContext'

export default function Parser() {
  const { t } = useLanguage()

  return (
    <DocPage title={t('parser.title')}>
      <p>{t('parser.intro')}</p>

      <h2>{t('parser.quoting.title')}</h2>

      <h3>{t('parser.quoting.single')}</h3>
      <p dangerouslySetInnerHTML={{ __html: t('parser.quoting.singleDesc') }}></p>
      <pre><code>{t('parser.quoting.singleExample')}</code></pre>
      <p>{t('parser.quoting.singleNote')}</p>

      <h3>{t('parser.quoting.double')}</h3>
      <p dangerouslySetInnerHTML={{ __html: t('parser.quoting.doubleDesc') }}></p>
      <pre><code>{t('parser.quoting.doubleExample')}</code></pre>
      <p>{t('parser.quoting.doubleNote')}</p>

      <h3>{t('parser.quoting.escape')}</h3>
      <p dangerouslySetInnerHTML={{ __html: t('parser.quoting.escapeDesc') }}></p>
      <pre><code>{t('parser.quoting.escapeExample')}</code></pre>

      <h3>{t('parser.quoting.ansi')}</h3>
      <p dangerouslySetInnerHTML={{ __html: t('parser.quoting.ansiDesc') }}></p>
      <pre><code>{t('parser.quoting.ansiExample')}</code></pre>
      <p dangerouslySetInnerHTML={{ __html: t('parser.quoting.ansiNote') }}></p>

      <h2>{t('parser.lineContinuation')}</h2>
      <p dangerouslySetInnerHTML={{ __html: t('parser.lineContinuationDesc') }}></p>
      <pre><code>{t('parser.lineContinuationExample')}</code></pre>

      <h2>{t('parser.processSubstitution')}</h2>
      <p dangerouslySetInnerHTML={{ __html: t('parser.processSubstitutionDesc') }}></p>
      <pre><code>{t('parser.processSubstitutionExample')}</code></pre>
      <p>{t('parser.processSubstitutionNote')}</p>

      <h2>{t('parser.historyExpansion')}</h2>
      <p dangerouslySetInnerHTML={{ __html: t('parser.historyExpansionDesc') }}></p>
      <table>
        <thead><tr><th>{t('parser.historyExpansion.expr')}</th><th>{t('parser.historyExpansion.desc')}</th><th>{t('parser.historyExpansion.example')}</th></tr></thead>
        <tbody>
          <tr><td><code>!!</code></td><td>{t('parser.historyExpansion.bangbang')}</td><td><code>!!</code></td></tr>
          <tr><td><code>!$</code></td><td>{t('parser.historyExpanding.bang$')}</td><td><code>mkdir dir; cd !$</code></td></tr>
          <tr><td><code>!^</code></td><td>{t('parser.historyExpansion.bang^')}</td><td><code>!^</code></td></tr>
          <tr><td><code>!n</code></td><td>{t('parser.historyExpansion.bangN')}</td><td><code>!42</code></td></tr>
          <tr><td><code>!-n</code></td><td>{t('parser.historyExpansion.bang-N')}</td><td><code>!-3</code></td></tr>
          <tr><td><code>!prefix</code></td><td>{t('parser.historyExpansion.bangPrefix')}</td><td><code>!git</code></td></tr>
          <tr><td><code>!?text</code></td><td>{t('parser.historyExpansion.bang?')}</td><td><code>!?commit</code></td></tr>
          <tr><td><code>!string:s/old/new/</code></td><td>{t('parser.historyExpansion.bangSubst')}</td><td><code>!git:s/push/pull/</code></td></tr>
        </tbody>
      </table>
      <p>{t('parser.historyExpansion.note')}</p>

      <h2>{t('parser.arithmetic')}</h2>
      <p dangerouslySetInnerHTML={{ __html: t('parser.arithmeticDesc') }}></p>
      <pre><code>{t('parser.arithmeticExample')}</code></pre>

      <h2>{t('parser.comments')}</h2>
      <p dangerouslySetInnerHTML={{ __html: t('parser.commentsDesc') }}></p>
      <pre><code>{t('parser.commentsExample')}</code></pre>
      <p>{t('parser.commentsNote')}</p>
    </DocPage>
  )
}