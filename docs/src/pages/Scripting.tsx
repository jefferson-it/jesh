import { DocPage } from '../components/DocPage'
import { useLanguage } from '../context/LanguageContext'

export default function Scripting() {
  const { t } = useLanguage()

  return (
    <DocPage title={t('scripting.title')}>
      <p dangerouslySetInnerHTML={{ __html: t('scripting.intro') }}></p>

      <h2>{t('scripting.sections.variables')}</h2>
      <p dangerouslySetInnerHTML={{ __html: t('scripting.variables.desc') }}></p>
      <pre><code>{t('scripting.variables.example')}</code></pre>

      <h3>{t('scripting.variables.local')}</h3>
      <p dangerouslySetInnerHTML={{ __html: t('scripting.variables.localDesc') }}></p>
      <pre><code>{t('scripting.variables.localExample')}</code></pre>

      <h3>{t('scripting.variables.env')}</h3>
      <p dangerouslySetInnerHTML={{ __html: t('scripting.variables.envDesc') }}></p>
      <pre><code>{t('scripting.variables.envExample')}</code></pre>

      <h3>{t('scripting.variables.special')}</h3>
      <table>
        <thead><tr><th>{t('scripting.variables.specialTable.var')}</th><th>{t('scripting.variables.specialTable.desc')}</th></tr></thead>
        <tbody>
          {[
            ['$?', 'scripting.variables.special.exitStatus'],
            ['$$', 'scripting.variables.special.pid'],
            ['$!', 'scripting.variables.special.lastBgPid'],
            ['$0–$9', 'scripting.variables.special.positional'],
            ['$@', 'scripting.variables.special.allParamsQuoted'],
            ['$#', 'scripting.variables.special.paramCount'],
            ['$*', 'scripting.variables.special.allParamsSingle'],
            ['$PWD', 'scripting.variables.special.pwd'],
            ['$OLDPWD', 'scripting.variables.special.oldpwd'],
            ['$PIPESTATUS', 'scripting.variables.special.pipestatus'],
            ['$IFS', 'scripting.variables.special.ifs'],
            ['$LINENO', 'scripting.variables.special.lineno'],
            ['$BASH_SOURCE', 'scripting.variables.special.bashSource'],
            ['$FUNCNAME', 'scripting.variables.special.funcName'],
          ].map(([v, d], i) => (
            <tr key={i}><td><code>{v}</code></td><td>{t(d)}</td></tr>
          ))}
        </tbody>
      </table>

      <h2>{t('scripting.sections.expansions')}</h2>

      <h3>{t('scripting.expansions.cmdSub')}</h3>
      <p dangerouslySetInnerHTML={{ __html: t('scripting.expansions.cmdSubDesc') }}></p>
      <pre><code>{t('scripting.expansions.cmdSubExample')}</code></pre>

      <h3>{t('scripting.expansions.paramExp')}</h3>
      <p dangerouslySetInnerHTML={{ __html: t('scripting.expansions.paramExpDesc') }}></p>
      <pre><code>{t('scripting.expansions.paramExpExample')}</code></pre>

      <h3>{t('scripting.expansions.arith')}</h3>
      <p dangerouslySetInnerHTML={{ __html: t('scripting.expansions.arithDesc') }}></p>
      <pre><code>{t('scripting.expansions.arithExample')}</code></pre>

      <h3>{t('scripting.expansions.brace')}</h3>
      <p dangerouslySetInnerHTML={{ __html: t('scripting.expansions.braceDesc') }}></p>
      <pre><code>{t('scripting.expansions.braceExample')}</code></pre>

      <h2>{t('scripting.sections.functions')}</h2>
      <p dangerouslySetInnerHTML={{ __html: t('scripting.functions.desc') }}></p>
      <pre><code>{t('scripting.functions.example')}</code></pre>
      <p dangerouslySetInnerHTML={{ __html: t('scripting.functions.notes') }}></p>

      <h2>{t('scripting.sections.declare')}</h2>
      <p dangerouslySetInnerHTML={{ __html: t('scripting.declare.desc') }}></p>
      <ul>
        <li><code>-i</code> — {t('scripting.declare.attrs.int')}</li>
        <li><code>-a</code> — {t('scripting.declare.attrs.array')}</li>
        <li><code>-A</code> — {t('scripting.declare.attrs.assoc')}</li>
        <li><code>-r</code> — {t('scripting.declare.attrs.readonly')}</li>
        <li><code>-x</code> — {t('scripting.declare.attrs.export')}</li>
        <li><code>-l</code> — {t('scripting.declare.attrs.lower')}</li>
        <li><code>-u</code> — {t('scripting.declare.attrs.upper')}</li>
      </ul>
      <pre><code>{t('scripting.declare.example')}</code></pre>

      <h2>{t('scripting.sections.setOptions')}</h2>
      <p>{t('scripting.setOptions.desc')}</p>
      <ul>
        <li><code>set -e</code> — {t('scripting.setOptions.e')}</li>
        <li><code>set -u</code> — {t('scripting.setOptions.u')}</li>
        <li><code>set -x</code> — {t('scripting.setOptions.x')}</li>
        <li><code>set -o pipefail</code> — {t('scripting.setOptions.pipefail')}</li>
        <li><code>set -o noglob</code> — {t('scripting.setOptions.noglob')}</li>
        <li><code>set -o allexport</code> — {t('scripting.setOptions.allexport')}</li>
        <li><code>set -o notify</code> — {t('scripting.setOptions.notify')}</li>
      </ul>

      <h2>{t('scripting.sections.controlFlow')}</h2>

      <h3>{t('scripting.controlFlow.conditionals')}</h3>
      <pre><code>{t('scripting.controlFlow.ifExample')}</code></pre>

      <h3>{t('scripting.controlFlow.case')}</h3>
      <pre><code>{t('scripting.controlFlow.caseExample')}</code></pre>

      <h3>{t('scripting.controlFlow.loops')}</h3>
      <pre><code>{t('scripting.controlFlow.loopsExample')}</code></pre>

      <h2>{t('scripting.sections.getopts')}</h2>
      <p dangerouslySetInnerHTML={{ __html: t('scripting.getopts.desc') }}></p>
      <pre><code>{t('scripting.getopts.example')}</code></pre>
      <p dangerouslySetInnerHTML={{ __html: t('scripting.getopts.notes') }}></p>
    </DocPage>
  )
}