import { DocPage } from '../components/DocPage'

export default function VsBash() {
  return (
    <DocPage title="jesh vs Bash">
      <p>
        jesh aims to be a drop-in replacement for Bash in daily interactive use while
        also providing modern features like structured history, async git prompts, fuzzy
        completions, and a cleaner scripting environment. This page compares jesh and
        Bash across language features, builtins, and compatibility.
      </p>

      <h2>Feature Comparison</h2>
      <table>
        <thead><tr><th>Feature</th><th>Bash</th><th>jesh</th></tr></thead>
        <tbody>
          <tr><td>Basic command syntax</td><td>✅</td><td>✅</td></tr>
          <tr><td>Pipes and redirections</td><td>✅</td><td>✅</td></tr>
          <tr><td>Job control (<code>&amp;</code>, <code>fg</code>, <code>bg</code>, <code>jobs</code>)</td><td>✅</td><td>✅</td></tr>
          <tr><td>Shell functions</td><td>✅</td><td>✅</td></tr>
          <tr><td>Aliases</td><td>✅</td><td>✅</td></tr>
          <tr><td>Indexed arrays (<code>arr=(a b c)</code>)</td><td>✅</td><td>✅</td></tr>
          <tr><td>Associative arrays (<code>declare -A</code>)</td><td>✅</td><td>✅</td></tr>
          <tr><td><code>[[ ]]</code> conditionals</td><td>✅</td><td>✅</td></tr>
          <tr><td><code>case</code> statements</td><td>✅</td><td>✅</td></tr>
          <tr><td><code>for</code>, <code>while</code>, <code>until</code> loops</td><td>✅</td><td>✅</td></tr>
          <tr><td><code>source</code> / <code>.</code></td><td>✅</td><td>✅</td></tr>
          <tr><td><code>getopts</code> option parsing</td><td>✅</td><td>✅</td></tr>
          <tr><td><code>set -e</code>, <code>-u</code>, <code>-x</code>, <code>-o pipefail</code></td><td>✅</td><td>✅</td></tr>
          <tr><td>History expansion (<code>!!</code>, <code>!$</code>)</td><td>✅</td><td>✅</td></tr>
          <tr><td>Process substitution (<code>&lt;(cmd)</code>)</td><td>✅</td><td>✅</td></tr>
          <tr><td>Arithmetic expansion (<code>$((expr))</code>)</td><td>✅</td><td>✅</td></tr>
          <tr><td>Brace expansion (<code>{'{a,b,c}'}</code>)</td><td>✅</td><td>✅</td></tr>
          <tr><td>Extended glob (<code>extglob</code>)</td><td>✅</td><td>✅</td></tr>
          <tr><td>Glob qualifiers</td><td>—</td><td>✅</td></tr>
          <tr><td>Automatic <code>cd</code> on bare path</td><td>—</td><td>✅</td></tr>
          <tr><td>Structured history (JSONL)</td><td>—</td><td>✅</td></tr>
          <tr><td>Async git prompt</td><td>—</td><td>✅</td></tr>
          <tr><td>Fuzzy path completion</td><td>—</td><td>✅</td></tr>
          <tr><td>Interactive completion menu</td><td>—</td><td>✅</td></tr>
          <tr><td>Transient prompt</td><td>—</td><td>✅</td></tr>
          <tr><td>RPROMPT (right-aligned prompt)</td><td>—</td><td>✅</td></tr>
          <tr><td>Hot-reload config</td><td>—</td><td>✅</td></tr>
          <tr><td>Real-time history sync</td><td>—</td><td>✅</td></tr>
          <tr><td><code>coproc</code></td><td>✅</td><td>❌</td></tr>
          <tr><td>Namerefs (<code>declare -n</code>)</td><td>✅</td><td>❌</td></tr>
          <tr><td><code>{'${!name[@]}'}</code> key expansion</td><td>✅</td><td>❌</td></tr>
          <tr><td><code>printf -v</code> (assign to variable)</td><td>✅</td><td>❌</td></tr>
          <tr><td><code>mapfile</code> / <code>readarray</code></td><td>✅</td><td>❌</td></tr>
          <tr><td><code>select</code> menu loop</td><td>✅</td><td>❌</td></tr>
          <tr><td><code>local -n</code> (dynamic scoping refs)</td><td>✅</td><td>❌</td></tr>
        </tbody>
      </table>

      <h2>Bash Fallback Mechanism</h2>
      <p>
        jesh recognises that many users have years of accumulated <code>.bashrc</code>
        files containing Bash-specific syntax. Rather than requiring a complete rewrite,
        jesh provides a transparent fallback: when it detects Bash-only syntax during
        sourcing (e.g. through <code>.bashrc</code> or <code>.bash_profile</code>), it
        can delegate execution of that file to <code>bash -ic</code>.
      </p>
      <p>
        The detection works by parsing the sourced script. Constructs that jesh cannot
        handle natively — such as <code>coproc</code>, <code>declare -n</code>, or
        <code>mapfile</code> — trigger the fallback. The script is then executed in a
        Bash subshell and its side effects (variable exports, function definitions,
        alias creation) are imported back into jesh where possible.
      </p>
      <p>
        In practice, most <code>.bashrc</code> files work with jesh without modification.
        The fallback is automatic and transparent. You can also force Bash execution
        with the <code>--bash</code> flag when running a specific script:
      </p>
      <pre><code>{`$ jesh --bash my-legacy-script.sh`}</code></pre>

      <h2>Migration Guide</h2>
      <p>
        Transitioning from Bash to jesh is straightforward for most users. Follow these
        steps:
      </p>
      <ol>
        <li><strong>Set jesh as your login shell</strong> — Run <code>chsh -s $(which jesh)</code>.</li>
        <li><strong>Create <code>~/.jeshrc</code></strong> — Move your aliases, exports, and function definitions from <code>~/.bashrc</code> to <code>~/.jeshrc</code>. Most syntax works as-is.</li>
        <li><strong>Source your <code>.bashrc</code> for compatibility</strong> — Add <code>source ~/.bashrc</code> at the end of your <code>.jeshrc</code> to pick up any remaining Bash-specific setup.</li>
        <li><strong>Add completions</strong> — Migrate your <code>complete</code> rules. jesh supports the same <code>complete -W</code> and <code>complete -F</code> syntax.</li>
        <li><strong>Customise the prompt</strong> — Replace <code>PS1</code> with jesh's prompt syntax, or keep <code>PS1</code> and it will be emulated.</li>
        <li><strong>Remove Bash-specific constructs</strong> — If you use <code>coproc</code>, <code>mapfile</code>, <code>declare -n</code>, or <code>select</code>, consider rewriting them in portable shell or keeping them in a Bash-sourced section.</li>
      </ol>
      <blockquote>
        <p>
          <strong>Tip:</strong> Start by using jesh as an interactive shell while keeping
          Bash as your login shell. This way you can test jesh without committing fully.
          Run <code>jesh</code> from Bash to try it out.
        </p>
      </blockquote>
    </DocPage>
  )
}
