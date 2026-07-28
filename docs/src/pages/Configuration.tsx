import { DocPage } from '../components/DocPage'

export default function Configuration() {
  return (
    <DocPage title="Configuration">
      <h2><code>~/.jeshrc</code> Variables</h2>
      <p>
        On startup, jesh reads <code>~/.jeshrc</code> as a shell script. You can set
        variables in this file to control shell behaviour. All configuration variables
        are optional.
      </p>

      <h3><code>INIT_INFO</code></h3>
      <p>
        Set to <code>true</code> to print a welcome banner each time jesh starts. The
        banner shows the jesh version, Rust version, and basic system info. Default is
        unset (no banner).
      </p>

      <h3><code>HOT_RELOAD</code></h3>
      <p>
        When set to <code>true</code>, jesh watches the <code>~/.jeshrc</code> file for
        changes and reloads it automatically. This lets you tweak aliases or variables
        in your editor and see the effect immediately without restarting the shell.
      </p>

      <h3><code>SHOW_TIMING</code></h3>
      <p>
        Set to <code>true</code> to display the execution time of each command after it
        finishes. The time is shown in milliseconds or seconds as appropriate. Useful
        for profiling slow commands.
      </p>

      <h3><code>JSH_TAB_MODE</code></h3>
      <p>
        Controls how <kbd>Tab</kbd> behaves. Possible values:
      </p>
      <ul>
        <li><code>complete</code> — Default. Pressing Tab triggers completions.</li>
        <li><code>menu-complete</code> — Cycle through completions on each Tab press.</li>
        <li><code>insert-tab</code> — Insert a literal tab character.</li>
      </ul>

      <h3><code>JSH_TRANSIENT_PROMPT</code></h3>
      <p>
        When set to <code>true</code>, the prompt after each command is replaced with a
        compact, minimal version (usually just <code>$ </code>). The full prompt
        (including RPROMPT, git info, etc.) is only shown for the current input line.
        This keeps the scrollback clean.
      </p>

      <h3><code>THEME</code></h3>
      <p>
        Selects a built-in color theme. jesh ships with several themes:
      </p>
      <ul>
        <li><code>jesh-dark</code> — Dark background, green accent.</li>
        <li><code>jesh-light</code> — Light background, blue accent.</li>
        <li><code>jesh-dracula</code> — Dracula-inspired palette.</li>
        <li><code>jesh-nord</code> — Nord polar night palette.</li>
        <li><code>jesh-solarized</code> — Solarized dark.</li>
      </ul>
      <p>
        Set the variable in your <code>~/.jeshrc</code>:
      </p>
      <pre><code>THEME="jesh-dracula"</code></pre>

      <h2><code>config.toml</code></h2>
      <p>
        Beyond <code>~/.jeshrc</code>, jesh reads a TOML configuration file at
        <code>~/.config/jesh/config.toml</code>. This file handles settings that are
        awkward to express as shell variables.
      </p>
      <p>Example configuration:</p>
      <pre><code>[history]
max_entries = 10000
sync = true
filter_duplicates = true
dir_aware = true

[completion]
fuzzy = true
case_sensitive = false
menu_lines = 10

[editor]
vi_mode = false
external_editor = "vim"</code></pre>

      <h3><code>[history]</code> Section</h3>
      <ul>
        <li><code>max_entries</code> — Maximum number of history entries to keep
        (default: 5000).</li>
        <li><code>sync</code> — When <code>true</code>, history is synchronized in
        real-time across all terminal sessions.</li>
        <li><code>filter_duplicates</code> — Consecutive duplicate commands are stored
        only once.</li>
        <li><code>dir_aware</code> — Prefer history entries from the current directory
        when ranking autosuggestions and search results.</li>
      </ul>

      <h2>Environment Variables</h2>
      <p>
        jesh respects several standard environment variables:
      </p>
      <ul>
        <li><code>$EDITOR</code> — Used by commands like <code>edit</code> and
        <code>fc</code> (if implemented).</li>
        <li><code>$PAGER</code> — Used by the built-in help system and commands that
        produce paginated output.</li>
        <li><code>$SHELL</code> — Set to the path of the jesh binary on startup.</li>
        <li><code>$JESH_VERSION</code> — Read-only variable containing the current
        jesh version string.</li>
        <li><code>$PWD</code> / <code>$OLDPWD</code> — Managed automatically by
        <code>cd</code> and <code>pushd</code>/<code>popd</code>.</li>
      </ul>
    </DocPage>
  )
}
