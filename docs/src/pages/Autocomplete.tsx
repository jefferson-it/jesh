import { DocPage } from '../components/DocPage'

export default function Autocomplete() {
  return (
    <DocPage title="Autocomplete">
      <p>
        jesh provides a powerful tab-completion system inspired by Zsh's completion
        framework. Press <kbd>Tab</kbd> to complete commands, file paths, directories,
        environment variables, and user-defined completions. When multiple matches exist,
        jesh displays an interactive menu that you can navigate with the arrow keys.
      </p>

      <h2>Basic Completion</h2>
      <p>
        By default, <kbd>Tab</kbd> attempts to complete the current word. If there are
        multiple candidates, a menu is shown below the prompt. Use <kbd>↑</kbd> and
        <kbd>↓</kbd> to select an entry, and press <kbd>Enter</kbd> or <kbd>Tab</kbd>
        again to accept. Press <kbd>Esc</kbd> to close the menu without accepting.
      </p>
      <p>jesh completes the following categories automatically:</p>
      <ul>
        <li><strong>Commands</strong> — Executables on <code>$PATH</code>, aliases, functions, and builtins.</li>
        <li><strong>File paths</strong> — Relative and absolute paths with partial prefix matching.</li>
        <li><strong>Directories</strong> — When the word ends in <code>/</code>, only directories are shown.</li>
        <li><strong>Environment variables</strong> — Words starting with <code>$</code> complete variable names.</li>
        <li><strong>User names</strong> — Tilde (<code>~</code>) followed by a prefix completes system users.</li>
        <li><strong>Command arguments</strong> — When a <code>complete</code> rule exists for the command.</li>
      </ul>

      <h2>Fuzzy Matching</h2>
      <p>
        jesh supports fuzzy path matching that lets you type abbreviated segments and
        have them expanded automatically. For example:
      </p>
      <ul>
        <li><code>/u/l/b</code> matches <code>/usr/local/bin</code></li>
        <li><code>/v/l/syslog</code> matches <code>/var/log/syslog</code></li>
        <li><code>/e/c/ng</code> matches <code>/etc/nginx/nginx.conf</code></li>
      </ul>
      <p>
        Each slash-separated segment is matched independently. Within a segment, the
        typed characters are matched in order against the directory or file name but do
        not need to be contiguous. This makes navigating deep directory trees much
        faster than typing full paths.
      </p>
      <p>
        You can disable fuzzy matching in <code>~/.config/jesh/config.toml</code>:
      </p>
      <pre><code>{`[completion]
fuzzy = false`}</code></pre>

      <h2>Static Word Lists</h2>
      <p>
        Use the <code>complete</code> builtin with <code>-W</code> to provide a static
        list of words for a command. jesh shows these words when completing the first
        argument (or subsequent arguments, depending on context).
      </p>
      <pre><code>{`complete -W "start stop restart status reload" myservice
complete -W "up down left right" move
complete -W "production staging development" deploy`}</code></pre>
      <p>
        Word lists support per-argument completion. You can specify different word lists
        for different argument positions using the <code>-X</code> filter pattern or by
        chaining multiple <code>complete</code> calls for the same command.
      </p>

      <h2>Dynamic Completers</h2>
      <p>
        For completions that require runtime computation, use <code>complete -F</code>
        with a shell function. The function receives the command name, the current word,
        and the previous word as arguments. It writes candidates to the
        <code>COMPREPLY</code> array.
      </p>
      <pre><code>{`_myapp_complete() {
  local word="\${COMP_WORDS[COMP_CWORD]}"
  case "$word" in
    --*) COMPREPLY=($(compgen -W "--verbose --config --help --version" "$word")) ;;
    *)   COMPREPLY=($(compgen -W "$(myapp list-projects)" "$word")) ;;
  esac
}
complete -F _myapp_complete myapp`}</code></pre>
      <p>
        The dynamic completer function runs in the shell context, so it has access to
        all shell variables, functions, and external commands. jesh populates the
        standard completion variables:
      </p>
      <ul>
        <li><code>COMP_WORDS</code> — Array of words on the command line.</li>
        <li><code>COMP_CWORD</code> — Index of the current word being completed.</li>
        <li><code>COMP_LINE</code> — The entire command line as a string.</li>
        <li><code>COMP_POINT</code> — The cursor position in the command line.</li>
      </ul>

      <h2>Flag and Description Display</h2>
      <p>
        When candidates are displayed in the completion menu, jesh shows a description
        alongside each option when available. For file completions, the description
        includes the file size, type, and modification time. For command completions,
        a one-line summary is shown when the command provides a help flag or when a
        description is configured in the completion rule.
      </p>
      <p>
        You can supply descriptions with the <code>complete -D</code> option or by
        appending a tab character followed by the description text in
        <code>COMPREPLY</code> entries:
      </p>
      <pre><code>{`COMPREPLY=(
  "--verbose\tEnable verbose output"
  "--config\tPath to configuration file"
  "--help\tShow this help message"
)`}</code></pre>

      <h2>Integration with Modern CLI Tools</h2>
      <p>
        jesh's completion system works with tools that generate completions dynamically.
        Popular tools and their integration points include:
      </p>
      <ul>
        <li><strong>cargo</strong> — Completions for subcommands, flags, and crate names.</li>
        <li><strong>git</strong> — Branch names, remotes, subcommands, and file paths.</li>
        <li><strong>npm / yarn / pnpm</strong> — Script names from <code>package.json</code>.</li>
        <li><strong>docker / podman</strong> — Containers, images, volumes, and networks.</li>
        <li><strong>kubectl</strong> — Resources, namespaces, and context names.</li>
        <li><strong>rustup</strong> — Toolchains, targets, and components.</li>
        <li><strong>deno</strong> — Permissions, scripts, and runtime flags.</li>
      </ul>
      <p>
        Many of these tools provide their own completion scripts. Source them in
        <code>~/.jeshrc</code> with the <code>source</code> builtin or rely on the
        Bash fallback mechanism to delegate completion generation to Bash transparently.
      </p>

      <h2>Menu Configuration</h2>
      <p>
        The completion menu can be customised in <code>~/.config/jesh/config.toml</code>:
      </p>
      <pre><code>{`[completion]
menu_lines = 10
case_sensitive = false
fuzzy = true
auto_list = true`}</code></pre>
      <ul>
        <li><code>menu_lines</code> — Maximum number of lines in the completion menu (default: 10).</li>
        <li><code>case_sensitive</code> — When <code>false</code>, matching ignores case.</li>
        <li><code>fuzzy</code> — Enable or disable fuzzy segment matching for paths.</li>
        <li><code>auto_list</code> — When <code>true</code>, pressing <kbd>Tab</kbd> with no unique match shows the menu immediately.</li>
      </ul>
    </DocPage>
  )
}
