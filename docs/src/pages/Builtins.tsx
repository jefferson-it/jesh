import { DocPage } from '../components/DocPage'
import { useLanguage } from '../context/LanguageContext'

export default function Builtins() {
  const { t } = useLanguage()
  return (
    <DocPage title={t('builtins.title')}>
      <p>{t('builtins.intro')}</p>

      <h2>{t('builtins.categories.navigation')}</h2>
      <table>
        <thead><tr><th>{t('builtins.command')}</th><th>{t('builtins.description')}</th><th>{t('builtins.syntax')}</th></tr></thead>
        <tbody>
          <tr><td><code>cd</code></td><td>Change the current directory.</td><td><code>cd [-L|-P] [dir]</code></td></tr>
          <tr><td><code>pwd</code></td><td>Print the current working directory.</td><td><code>pwd [-L|-P]</code></td></tr>
          <tr><td><code>pushd</code></td><td>Push a directory onto the directory stack and cd to it.</td><td><code>pushd [+n|-n|dir]</code></td></tr>
          <tr><td><code>popd</code></td><td>Pop the directory stack and cd to the top entry.</td><td><code>popd [+n|-n]</code></td></tr>
          <tr><td><code>dirs</code></td><td>Display the directory stack.</td><td><code>dirs [-clpv] [+n|-n]</code></td></tr>
        </tbody>
      </table>

      <h2>{t('builtins.categories.environment')}</h2>
      <table>
        <thead><tr><th>{t('builtins.command')}</th><th>{t('builtins.description')}</th><th>{t('builtins.syntax')}</th></tr></thead>
        <tbody>
          <tr><td><code>export</code></td><td>Set or list exported environment variables.</td><td><code>export [-n] [name[=value]...]</code></td></tr>
          <tr><td><code>unset</code></td><td>Unset shell variables or functions.</td><td><code>unset [-fv] name...</code></td></tr>
          <tr><td><code>alias</code></td><td>Define or list aliases.</td><td><code>alias [-p] [name[=value]...]</code></td></tr>
          <tr><td><code>unalias</code></td><td>Remove aliases.</td><td><code>unalias [-a] name...</code></td></tr>
        </tbody>
      </table>

      <h2>{t('builtins.categories.process')}</h2>
      <table>
        <thead><tr><th>{t('builtins.command')}</th><th>{t('builtins.description')}</th><th>{t('builtins.syntax')}</th></tr></thead>
        <tbody>
          <tr><td><code>jobs</code></td><td>List active jobs.</td><td><code>jobs [-lnprs] [jobspec...]</code></td></tr>
          <tr><td><code>fg</code></td><td>Bring a job to the foreground.</td><td><code>fg [jobspec]</code></td></tr>
          <tr><td><code>bg</code></td><td>Resume a job in the background.</td><td><code>bg [jobspec]</code></td></tr>
          <tr><td><code>kill</code></td><td>Send a signal to a process.</td><td><code>kill [-s sigspec|-n signum|-sigspec] pid...</code></td></tr>
        </tbody>
      </table>

      <h2>{t('builtins.categories.shell')}</h2>
      <table>
        <thead><tr><th>{t('builtins.command')}</th><th>{t('builtins.description')}</th><th>{t('builtins.syntax')}</th></tr></thead>
        <tbody>
          <tr><td><code>set</code></td><td>Set or unset shell options (-e, -u, -x, -o pipefail, etc.).</td><td><code>set [-euvx] [-o option] [-- args...]</code></td></tr>
          <tr><td><code>shopt</code></td><td>Toggle optional shell behaviour (glob flags, etc.).</td><td><code>shopt [-pqsu] [-o] name...</code></td></tr>
          <tr><td><code>complete</code></td><td>Define programmable completion rules.</td><td><code>complete [-F func] [-W words] [-o opts] cmd</code></td></tr>
        </tbody>
      </table>

      <h2>{t('builtins.categories.files')}</h2>
      <table>
        <thead><tr><th>{t('builtins.command')}</th><th>{t('builtins.description')}</th><th>{t('builtins.syntax')}</th></tr></thead>
        <tbody>
          <tr><td><code>source</code> / <code>.</code></td><td>Execute commands from a file in the current shell.</td><td><code>source filename [args...]</code></td></tr>
          <tr><td><code>read</code></td><td>Read a line from stdin into variables.</td><td><code>read [-r] [-d delim] [-p prompt] [name...]</code></td></tr>
          <tr><td><code>printf</code></td><td>Format and print data, like C printf.</td><td><code>printf format [args...]</code></td></tr>
          <tr><td><code>echo</code></td><td>Write arguments to stdout.</td><td><code>echo [-nEe] [args...]</code></td></tr>
        </tbody>
      </table>
    </DocPage>
  )
}
