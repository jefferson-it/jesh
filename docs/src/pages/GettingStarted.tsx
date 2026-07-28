import { DocPage } from '../components/DocPage'
import { useLanguage } from '../context/LanguageContext'

export default function GettingStarted() {
  const { t } = useLanguage()

  return (
    <DocPage title={t('gettingStarted.title')}>
      <h2>{t('gettingStarted.installation.title')}</h2>
      <p>
        {t('gettingStarted.installation.cargoDesc')}
      </p>
      <pre><code>{t('gettingStarted.installation.cargoCmd')}</code></pre>
      <p>
        {t('gettingStarted.installation.cargoNote')}
      </p>
      <p>
        {t('gettingStarted.installation.sourceDesc')}
      </p>
      <pre><code>{t('gettingStarted.installation.sourceCmd')}</code></pre>
      <p>
        {t('gettingStarted.installation.binaryNote')}
      </p>

      <h2>{t('gettingStarted.firstSteps.title')}</h2>
      <p dangerouslySetInnerHTML={{ __html: t('gettingStarted.firstSteps.step1') }}></p>
      <p dangerouslySetInnerHTML={{ __html: t('gettingStarted.firstSteps.step2') }}></p>
      <p dangerouslySetInnerHTML={{ __html: t('gettingStarted.firstSteps.step3') }}></p>

      <h2>{t('gettingStarted.configuration.title')}</h2>
      <p dangerouslySetInnerHTML={{ __html: t('gettingStarted.configuration.desc') }}></p>

      <h2>{t('gettingStarted.basicUsage.title')}</h2>
      <p>{t('gettingStarted.basicUsage.desc')}</p>
      <pre><code>{t('gettingStarted.basicUsage.examples')}</code></pre>

      <h3>{t('gettingStarted.completion.title')}</h3>
      <p dangerouslySetInnerHTML={{ __html: t('gettingStarted.completion.desc') }}></p>

      <h3>{t('gettingStarted.history.title')}</h3>
      <p dangerouslySetInnerHTML={{ __html: t('gettingStarted.history.desc') }}></p>

      <h3>{t('gettingStarted.reverseSearch.title')}</h3>
      <p dangerouslySetInnerHTML={{ __html: t('gettingStarted.reverseSearch.desc') }}></p>

      <h3>{t('gettingStarted.autosuggestions.title')}</h3>
      <p dangerouslySetInnerHTML={{ __html: t('gettingStarted.autosuggestions.desc') }}></p>

      <h2>{t('gettingStarted.nextSteps')}</h2>
    </DocPage>
  )
}