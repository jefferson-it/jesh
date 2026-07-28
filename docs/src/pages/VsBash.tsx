import { DocPage } from '../components/DocPage'
import { useLanguage } from '../context/LanguageContext'

export default function VsBash() {
  const { t } = useLanguage()

  return (
    <DocPage title={t('vsBash.title')}>
      <p>{t('vsBash.intro')}</p>

      <h2>{t('vsBash.comparison')}</h2>
      <table>
        <thead><tr><th>{t('vsBash.comparisonTable.feature')}</th><th>{t('vsBash.comparisonTable.bash')}</th><th>{t('vsBash.comparisonTable.jesh')}</th></tr></thead>
        <tbody>
          {[
            ['vsBash.comparisonTable.basicSyntax', '✅', '✅'],
            ['vsBash.comparisonTable.pipesRedirects', '✅', '✅'],
            ['vsBash.comparisonTable.jobControl', '✅', '✅'],
            ['vsBash.comparisonTable.functions', '✅', '✅'],
            ['vsBash.comparisonTable.aliases', '✅', '✅'],
            ['vsBash.comparisonTable.indexedArrays', '✅', '✅'],
            ['vsBash.comparisonTable.assocArrays', '✅', '✅'],
            ['vsBash.comparisonTable.conditionals', '✅', '✅'],
            ['vsBash.comparisonTable.case', '✅', '✅'],
            ['vsBash.comparisonTable.loops', '✅', '✅'],
            ['vsBash.comparisonTable.source', '✅', '✅'],
            ['vsBash.comparisonTable.getopts', '✅', '✅'],
            ['vsBash.comparisonTable.setOptions', '✅', '✅'],
            ['vsBash.comparisonTable.historyExpansion', '✅', '✅'],
            ['vsBash.comparisonTable.processSub', '✅', '✅'],
            ['vsBash.comparisonTable.arithmetic', '✅', '✅'],
            ['vsBash.comparisonTable.braceExpansion', '✅', '✅'],
            ['vsBash.comparisonTable.extglob', '✅', '✅'],
            ['vsBash.comparisonTable.globQualifiers', '—', '✅'],
            ['vsBash.comparisonTable.autoCd', '—', '✅'],
            ['vsBash.comparisonTable.structuredHistory', '—', '✅'],
            ['vsBash.comparisonTable.asyncGitPrompt', '—', '✅'],
            ['vsBash.comparisonTable.fuzzyCompletion', '—', '✅'],
            ['vsBash.comparisonTable.interactiveMenu', '—', '✅'],
            ['vsBash.comparisonTable.transientPrompt', '—', '✅'],
            ['vsBash.comparisonTable.rprompt', '—', '✅'],
            ['vsBash.comparisonTable.hotReload', '—', '✅'],
            ['vsBash.comparisonTable.realTimeSync', '—', '✅'],
            ['vsBash.comparisonTable.coproc', '✅', '❌'],
            ['vsBash.comparisonTable.namerefs', '✅', '❌'],
            ['vsBash.comparisonTable.keyExpansion', '✅', '❌'],
            ['vsBash.comparisonTable.printfV', '✅', '❌'],
            ['vsBash.comparisonTable.mapfile', '✅', '❌'],
            ['vsBash.comparisonTable.select', '✅', '❌'],
            ['vsBash.comparisonTable.localN', '✅', '❌'],
          ].map(([f, b, j], i) => (
            <tr key={i}>
              <td>{t(f)}</td>
              <td>{b}</td>
              <td>{j}</td>
            </tr>
          ))}
        </tbody>
      </table>

      <h2>{t('vsBash.fallback')}</h2>
      <p>{t('vsBash.fallbackDesc')}</p>
      <p>{t('vsBash.fallbackDetection')}</p>
      <p>{t('vsBash.fallbackWorks')}</p>
      <p>{t('vsBash.fallbackForce')}</p>
      <pre><code>{t('vsBash.fallbackExample')}</code></pre>

      <h2>{t('vsBash.migration')}</h2>
      <p>{t('vsBash.migrationDesc')}</p>
      <ol>
        <li><strong>{t('vsBash.migration.step1')}</strong> — {t('vsBash.migration.step1Desc')}</li>
        <li><strong>{t('vsBash.migration.step2')}</strong> — {t('vsBash.migration.step2Desc')}</li>
        <li><strong>{t('vsBash.migration.step3')}</strong> — {t('vsBash.migration.step3Desc')}</li>
        <li><strong>{t('vsBash.migration.step4')}</strong> — {t('vsBash.migration.step4Desc')}</li>
        <li><strong>{t('vsBash.migration.step5')}</strong> — {t('vsBash.migration.step5Desc')}</li>
        <li><strong>{t('vsBash.migration.step6')}</strong> — {t('vsBash.migration.step6Desc')}</li>
      </ol>
      <blockquote>
        <p><strong>{t('vsBash.migration.tip')}</strong> {t('vsBash.migration.tipDesc')}</p>
      </blockquote>
    </DocPage>
  )
}