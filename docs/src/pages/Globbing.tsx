import { DocPage } from '../components/DocPage'
import { useLanguage } from '../context/LanguageContext'

export default function Globbing() {
  const { t } = useLanguage()

  return (
    <DocPage title={t('globbing.title')}>
      <p>{t('globbing.intro')}</p>

      <h2>{t('globbing.basic')}</h2>
      <p>{t('globbing.basicDesc')}</p>
      <table>
        <thead><tr><th>{t('globbing.basicTable.pattern')}</th><th>{t('globbing.basicTable.meaning')}</th><th>{t('globbing.basicTable.example')}</th></tr></thead>
        <tbody>
          <tr><td><code>*</code></td><td>{t('globbing.basicTable.star')}</td><td><code>*.rs</code></td></tr>
          <tr><td><code>?</code></td><td>{t('globbing.basicTable.question')}</td><td><code>file?.txt</code></td></tr>
          <tr><td><code>[abc]</code></td><td>{t('globbing.basicTable.bracket')}</td><td><code>[abc]*.txt</code></td></tr>
          <tr><td><code>[a-z]</code></td><td>{t('globbing.basicTable.range')}</td><td><code>[a-z]*.sh</code></td></tr>
          <tr><td><code>[!abc]</code></td><td>{t('globbing.basicTable.negate')}</td><td><code>[!0-9]*.log</code></td></tr>
        </tbody>
      </table>

      <h2>{t('globbing.extended')}</h2>
      <p>{t('globbing.extendedDesc')}</p>
      <table>
        <thead><tr><th>{t('globbing.extendedTable.pattern')}</th><th>{t('globbing.extendedTable.meaning')}</th><th>{t('globbing.extendedTable.example')}</th></tr></thead>
        <tbody>
          <tr><td><code>**</code></td><td>{t('globbing.extendedTable.doubleStar')}</td><td><code>**/*.rs</code></td></tr>
          <tr><td><code>?(pattern)</code></td><td>{t('globbing.extendedTable.questionParen')}</td><td><code>?(*.md|*.txt)</code></td></tr>
          <tr><td><code>*(pattern)</code></td><td>{t('globbing.extendedTable.starParen')}</td><td><code>*(.rs|.toml)</code></td></tr>
          <tr><td><code>+(pattern)</code></td><td>{t('globbing.extendedTable.plusParen')}</td><td><code>+(.rs|.toml)</code></td></tr>
          <tr><td><code>@(pattern)</code></td><td>{t('globbing.extendedTable.atParen')}</td><td><code>@(.rs|.toml)</code></td></tr>
          <tr><td><code>!(pattern)</code></td><td>{t('globbing.extendedTable.negateParen')}</td><td><code>!(*.log|*.tmp)</code></td></tr>
        </tbody>
      </table>

      <h2>{t('globbing.globQualifiers')}</h2>
      <p>{t('globbing.globQualifiersDesc')}</p>
      <table>
        <thead><tr><th>{t('globbing.globQualifiersTable.qualifier')}</th><th>{t('globbing.globQualifiersTable.meaning')}</th></tr></thead>
        <tbody>
          <tr><td><code>(/)</code></td><td>{t('globbing.globQualifiersTable.dir')}</td></tr>
          <tr><td><code>(.)</code></td><td>{t('globbing.globQualifiersTable.file')}</td></tr>
          <tr><td><code>(@)</code></td><td>{t('globbing.globQualifiersTable.symlink')}</td></tr>
          <tr><td><code>(*)</code></td><td>{t('globbing.globQualifiersTable.exec')}</td></tr>
          <tr><td><code>(r)</code></td><td>{t('globbing.globQualifiersTable.readable')}</td></tr>
          <tr><td><code>(w)</code></td><td>{t('globbing.globQualifiersTable.writable')}</td></tr>
          <tr><td><code>(x)</code></td><td>{t('globbing.globQualifiersTable.executable')}</td></tr>
          <tr><td><code>(L+100)</code></td><td>{t('globbing.globQualifiersTable.sizeGt')}</td></tr>
          <tr><td><code>(L-10k)</code></td><td>{t('globbing.globQualifiersTable.sizeLt')}</td></tr>
          <tr><td><code>(mh+1)</code></td><td>{t('globbing.globQualifiersTable.modHoursGt')}</td></tr>
          <tr><td><code>(mh-24)</code></td><td>{t('globbing.globQualifiersTable.modHoursLt')}</td></tr>
        </tbody>
      </table>
      <p>{t('globbing.globQualifiersNote')}</p>

      <h2>{t('globbing.braceExpansion')}</h2>
      <p>{t('globbing.braceExpansionDesc')}</p>
      <pre><code>{t('globbing.braceExpansionExample')}</code></pre>

      <h2>{t('globbing.options')}</h2>
      <p>{t('globbing.optionsDesc')}</p>
      <ul>
        <li><code>shopt -s extglob</code> — {t('globbing.options.extglob')}</li>
        <li><code>shopt -s globstar</code> — {t('globbing.options.globstar')}</li>
        <li><code>shopt -s dotglob</code> — {t('globbing.options.dotglob')}</li>
        <li><code>shopt -s nullglob</code> — {t('globbing.options.nullglob')}</li>
        <li><code>shopt -s failglob</code> — {t('globbing.options.failglob')}</li>
        <li><code>shopt -s nocaseglob</code> — {t('globbing.options.nocaseglob')}</li>
      </ul>
    </DocPage>
  )
}