import { DocPage } from '../components/DocPage'

export default function Builtins() {
  return (
    <DocPage title="Builtins">
      <p>
        jesh provides a comprehensive set of builtin commands that run directly in the
        shell process without spawning a subprocess. Builtins are always available,
        even when <code>$PATH</code> is empty or broken.
      </p>

      <h2>Directory Navigation</h2>
      <table>
        <thead><tr><th>Command</th><th>Description</th><th>Syntax</th></tr></thead>
        <tbody>
          <tr>
            <td><code>cd</code></td>
            <td>Change the current directory.</td>
            <td><code>cd [-L|-P] [dir]</code></td>
          </tr>
          <tr>
            <td><code>pwd</code></td>
            <td>Print the current working directory.</td>
            <td><code>pwd [-L|-P]</code></td>
          </tr>
          <tr>
            <td><code>pushd</code></td>
            <td>Push a directory onto the directory stack and cd to it.</td>
            <td><code>pushd [+n|-n|dir]</code></td>
          </tr>
          <tr>
            <td><code>popd</code></td>
            <td>Pop the directory stack and cd to the top entry.</td>
            <td><code>popd [+n|-n]</code></td>
          </tr>
          <tr>
            <td><code>dirs</code></td>
            <td>Display the directory stack.</td>
            <td><code>dirs [-clpv] [+n|-n]</code></td>
          </tr>
        </tbody>
      </table>

      <h2>Shell State</h2>
      <table>
        <thead><tr><th>Command</th><th>Description</th><th>Syntax</th></tr></thead>
        <tbody>
          <tr>
            <td><code>exit</code></td>
            <td>Exit the shell with an optional status.</td>
            <td><code>exit [n]</code></td>
          </tr>
          <tr>
            <td><code>echo</code></td>
            <td>Write arguments to stdout.</td>
            <td><code>echo [-nEe] [args...]</code></td>
          </tr>
          <tr>
            <td><code>export</code></td>
            <td>Set or list exported environment variables.</td>
            <td><code>export [-n] [name[=value]...]</code></td>
          </tr>
          <tr>
            <td><code>unset</code></td>
            <td>Unset shell variables or functions.</td>
            <td><code>unset [-fv] name...</code></td>
          </tr>
          <tr>
            <td><code>alias</code></td>
            <td>Define or list aliases.</td>
            <td><code>alias [-p] [name[=value]...]</code></td>
          </tr>
          <tr>
            <td><code>unalias</code></td>
            <td>Remove aliases.</td>
            <td><code>unalias [-a] name...</code></td>
          </tr>
          <tr>
            <td><code>history</code></td>
            <td>View, search, or manage command history.</td>
            <td><code>history [-c|-d n|pin|unpin n]</code></td>
          </tr>
          <tr>
            <td><code>type</code></td>
            <td>Display information about a command name.</td>
            <td><code>type [-afptP] name...</code></td>
          </tr>
          <tr>
            <td><code>which</code></td>
            <td>Locate a command and display its path.</td>
            <td><code>which [-a] name...</code></td>
          </tr>
          <tr>
            <td><code>source</code> / <code>.</code></td>
            <td>Execute commands from a file in the current shell.</td>
            <td><code>source filename [args...]</code></td>
          </tr>
        </tbody>
      </table>

      <h2>Input / Output</h2>
      <table>
        <thead><tr><th>Command</th><th>Description</th><th>Syntax</th></tr></thead>
        <tbody>
          <tr>
            <td><code>read</code></td>
            <td>Read a line from stdin into variables.</td>
            <td><code>read [-r] [-d delim] [-p prompt] [name...]</code></td>
          </tr>
          <tr>
            <td><code>printf</code></td>
            <td>Format and print data, like C printf.</td>
            <td><code>printf format [args...]</code></td>
          </tr>
        </tbody>
      </table>

      <h2>Execution Control</h2>
      <table>
        <thead><tr><th>Command</th><th>Description</th><th>Syntax</th></tr></thead>
        <tbody>
          <tr>
            <td><code>eval</code></td>
            <td>Concatenate and execute arguments as a shell command.</td>
            <td><code>eval [args...]</code></td>
          </tr>
          <tr>
            <td><code>exec</code></td>
            <td>Replace the shell with a given command.</td>
            <td><code>exec [-cl] command [args...]</code></td>
          </tr>
          <tr>
            <td><code>command</code></td>
            <td>Run a command bypassing aliases and functions.</td>
            <td><code>command [-pvV] cmd [args...]</code></td>
          </tr>
          <tr>
            <td><code>true</code></td>
            <td>Return exit status 0.</td>
            <td><code>true</code></td>
          </tr>
          <tr>
            <td><code>false</code></td>
            <td>Return exit status 1.</td>
            <td><code>false</code></td>
          </tr>
          <tr>
            <td><code>:</code></td>
            <td>No-op; expands arguments but does nothing.</td>
            <td><code>: [args...]</code></td>
          </tr>
        </tbody>
      </table>

      <h2>Conditionals</h2>
      <table>
        <thead><tr><th>Command</th><th>Description</th><th>Syntax</th></tr></thead>
        <tbody>
          <tr>
            <td><code>test</code> / <code>[</code></td>
            <td>Evaluate a conditional expression (POSIX).</td>
            <td><code>test expr</code> or <code>[ expr ]</code></td>
          </tr>
          <tr>
            <td><code>[[</code></td>
            <td>Extended conditional expression with regex matching.</td>
            <td><code>[[ expr ]]</code></td>
          </tr>
        </tbody>
      </table>

      <h2>Variable Attributes</h2>
      <table>
        <thead><tr><th>Command</th><th>Description</th><th>Syntax</th></tr></thead>
        <tbody>
          <tr>
            <td><code>declare</code> / <code>typeset</code></td>
            <td>Declare variables with attributes (-i, -a, -A, -r, -x).</td>
            <td><code>declare [-afFgGIilnrtux] [name[=value]...]</code></td>
          </tr>
          <tr>
            <td><code>local</code></td>
            <td>Declare a locally-scoped variable inside a function.</td>
            <td><code>local [name[=value]...]</code></td>
          </tr>
          <tr>
            <td><code>readonly</code></td>
            <td>Mark variables as read-only.</td>
            <td><code>readonly [-af] name...</code></td>
          </tr>
          <tr>
            <td><code>getopts</code></td>
            <td>Parse positional parameters for option flags.</td>
            <td><code>getopts optstring name [args...]</code></td>
          </tr>
        </tbody>
      </table>

      <h2>Job Control</h2>
      <table>
        <thead><tr><th>Command</th><th>Description</th><th>Syntax</th></tr></thead>
        <tbody>
          <tr>
            <td><code>jobs</code></td>
            <td>List active jobs.</td>
            <td><code>jobs [-lnprs] [jobspec...]</code></td>
          </tr>
          <tr>
            <td><code>fg</code></td>
            <td>Bring a job to the foreground.</td>
            <td><code>fg [jobspec]</code></td>
          </tr>
          <tr>
            <td><code>bg</code></td>
            <td>Resume a job in the background.</td>
            <td><code>bg [jobspec]</code></td>
          </tr>
          <tr>
            <td><code>disown</code></td>
            <td>Remove a job from the shell's job table.</td>
            <td><code>disown [-ahr] [jobspec...]</code></td>
          </tr>
          <tr>
            <td><code>kill</code></td>
            <td>Send a signal to a process.</td>
            <td><code>kill [-s sigspec|-n signum|-sigspec] pid...</code></td>
          </tr>
        </tbody>
      </table>

      <h2>Shell Options</h2>
      <table>
        <thead><tr><th>Command</th><th>Description</th><th>Syntax</th></tr></thead>
        <tbody>
          <tr>
            <td><code>set</code></td>
            <td>Set or unset shell options (-e, -u, -x, -o pipefail, etc.).</td>
            <td><code>set [-euvx] [-o option] [-- args...]</code></td>
          </tr>
          <tr>
            <td><code>shopt</code></td>
            <td>Toggle optional shell behaviour (glob flags, etc.).</td>
            <td><code>shopt [-pqsu] [-o] name...</code></td>
          </tr>
          <tr>
            <td><code>complete</code></td>
            <td>Define programmable completion rules.</td>
            <td><code>complete [-F func] [-W words] [-o opts] cmd</code></td>
          </tr>
        </tbody>
      </table>
    </DocPage>
  )
}
