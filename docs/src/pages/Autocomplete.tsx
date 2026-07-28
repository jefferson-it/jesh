import { DocPage } from '../components/DocPage'
import { useLanguage } from '../context/LanguageContext'

export default function Autocomplete() {
  const { t } = useLanguage()

  return (
    <DocPage title={t('autocomplete.title')}>
      <p>{t('autocomplete.intro')}</p>

      <h2>{t('autocomplete.basic')}</h2>
      <p>{t('autocomplete.basicDesc')}</p>
      <p>{t('autocomplete.categories')}</p>
      <ul>
        <li><strong>{t('autocomplete.categories.commands')}</strong> — {t('autocomplete.categories.commandsDesc')}</li>
        <li><strong>{t('autocomplete.categories.files')}</strong> — {t('autocomplete.categories.filesDesc')}</li>
        <li><strong>{t('autocomplete.categories.dirs')}</strong> — {t('autocomplete.categories.dirsDesc')}</li>
        <li><strong>{t('autocomplete.categories.vars')}</strong> — {t('autocomplete.categories.varsDesc')}</li>
        <li><strong>{t('autocomplete.categories.users')}</strong> — {t('autocomplete.categories.usersDesc')}</li>
        <li><strong>{t('autocomplete.categories.args')}</strong> — {t('autocomplete.categories.argsDesc')}</li>
      </ul>

      <h2>{t('autocomplete.fuzzy')}</h2>
      <p>{t('autocomplete.fuzzyDesc')}</p>
      <ul>
        <li><code>/u/l/b</code> — {t('autocomplete.fuzzy.ex1')}</li>
        <li><code>/v/l/syslog</code> — {t('autocomplete.fuzzy.ex2')}</li>
        <li><code>/e/c/ng</code> — {t('autocomplete.fuzzy.ex3')}</li>
      </ul>
      <p>{t('autocomplete.fuzzyNote')}</p>
      <pre><code>{t('autocomplete.fuzzyConfig')}</code></pre>

      <h2>{t('autocomplete.static')}</h2>
      <p>{t('autocomplete.staticDesc')}</p>
      <pre><code>{t('autocomplete.staticExample')}</code></pre>
      <p>{t('autocomplete.staticNote')}</p>

      <h2>{t('autocomplete.dynamic')}</h2>
      <p>{t('autocomplete.dynamicDesc')}</p>
      <pre><code>{t('autocomplete.dynamicExample')}</code></pre>
      <p>{t('autocomplete.dynamicVars')}</p>
      <ul>
        <li><code>COMP_WORDS</code> — {t('autocomplete.dynamic.compWords')}</li>
        <li><code>COMP_CWORD</code> — {t('autocomplete.dynamic.compCword')}</li>
        <li><code>COMP_LINE</code> — {t('autocomplete.dynamic.compLine')}</li>
        <li><code>COMP_POINT</code> — {t('autocomplete.dynamic.compPoint')}</li>
      </ul>

      <h2>{t('autocomplete.descriptions')}</h2>
      <p>{t('autocomplete.descriptionsDesc')}</p>
      <pre><code>{t('autocomplete.descriptionsExample')}</code></pre>

      <h2>{t('autocomplete.integration')}</h2>
      <p>{t('autocomplete.integrationDesc')}</p>
      <ul>
        <li><strong>cargo</strong> — {t('autocomplete.integration.cargo')}</li>
        <li><strong>git</strong> — {t('autocomplete.integration.git')}</li>
        <li><strong>npm / yarn / pnpm</strong> — {t('autocomplete.integration.npm')}</li>
        <li><strong>docker / podman</strong> — {t('autocomplete.integration.docker')}</li>
        <li><strong>kubectl</strong> — {t('autocomplete.integration.kubectl')}</li>
        <li><strong>rustup</strong> — {t('autocomplete.integration.rustup')}</li>
        <li><strong>deno</strong> — {t('autocomplete.integration.deno')}</li>
      </ul>
      <p>{t('autocomplete.integrationNote')}</p>

      <h2>{t('autocomplete.menuConfig')}</h2>
      <p>{t('autocomplete.menuConfigDesc')}</p>
      <pre><code>{t('autocomplete.menuConfigExample')}</code></pre>
      <ul>
        <li><code>menu_lines</code> — {t('autocomplete.menuConfig.menuLines')}</li>
        <li><code>case_sensitive</code> — {t('autocomplete.menuConfig.caseSensitive')}</li>
        <li><code>fuzzy</code> — {t('autocomplete.menuConfig.fuzzy')}</li>
        <li><code>auto_list</code> — {t('autocomplete.menuConfig.autoList')}</li>
      </ul>
    </DocPage>
  )
}