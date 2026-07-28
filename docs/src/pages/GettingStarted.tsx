import { DocPage } from '../components/DocPage'
import { useLanguage } from '../context/LanguageContext'

export default function GettingStarted() {
  const { t } = useLanguage()

  return (
    <DocPage title={t('gettingStarted.title')}>
      <h2>{t('gettingStarted.installation.title')}</h2>

      <h3>{t('gettingStarted.installation.curl')}</h3>
      <pre><code>{t('gettingStarted.installation.curlCmd')}</code></pre>
      <p>{t('gettingStarted.installation.curlDesc')}</p>

      <h3>{t('gettingStarted.installation.binary')}</h3>
      <p>
        <a href="https://github.com/jefferson-it/jesh/releases" target="_blank" rel="noopener noreferrer">
          {t('gettingStarted.installation.binaryDesc')}
        </a>
      </p>
      <p dangerouslySetInnerHTML={{ __html: t('gettingStarted.installation.binaryNote') }}></p>

      <h3>{t('gettingStarted.installation.cargo')}</h3>
      <pre><code>{t('gettingStarted.installation.cargoCmd')}</code></pre>
      <p dangerouslySetInnerHTML={{ __html: t('gettingStarted.installation.cargoDesc') }}></p>

      <h3>{t('gettingStarted.installation.source')}</h3>
      <pre><code>{t('gettingStarted.installation.sourceCmd')}</code></pre>

      <h3>{t('gettingStarted.installation.aur')}</h3>
      <pre><code>{t('gettingStarted.installation.aurCmd')}</code></pre>

      <h2>{t('gettingStarted.firstSteps.title')}</h2>
      <p dangerouslySetInnerHTML={{ __html: t('gettingStarted.firstSteps.step1') }}></p>
      <p dangerouslySetInnerHTML={{ __html: t('gettingStarted.firstSteps.step2') }}></p>
      <p dangerouslySetInnerHTML={{ __html: t('gettingStarted.firstSteps.step3') }}></p>

      <h2>{t('gettingStarted.configuration.title')}</h2>
      <p dangerouslySetInnerHTML={{ __html: t('gettingStarted.configuration.desc') }}></p>

      <h2>{t('gettingStarted.basicUsage.title')}</h2>
      <p dangerouslySetInnerHTML={{ __html: t('gettingStarted.basicUsage.desc') }}></p>
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