import { DocPage } from '../components/DocPage'

export default function History() {
  return (
    <DocPage title="History">
      <p>
        jesh stores command history in structured JSON Lines format (JSONL), with one
        JSON object per line. This format is easy to parse, extend, and sync across
        multiple terminal sessions. History is stored on disk at
        <code>~/.local/share/jesh/history/</code> and managed through builtin commands
        and keyboard shortcuts.
      </p>

      <h2>Storage Format</h2>
      <p>
        Each command is recorded as a JSON object with the following fields:
      </p>
      <table>
        <thead><tr><th>Field</th><th>Type</th><th>Description</th></tr></thead>
        <tbody>
          <tr><td><code>cmd</code></td><td>string</td><td>The command text.</td></tr>
          <tr><td><code>cwd</code></td><td>string</td><td>Working directory when the command was run.</td></tr>
          <tr><td><code>exit</code></td><td>number</td><td>Exit code of the command.</td></tr>
          <tr><td><code>ts</code></td><td>string</td><td>ISO 8601 timestamp of execution.</td></tr>
          <tr><td><code>count</code></td><td>number</td><td>How many times this command has been run.</td></tr>
          <tr><td><code>last</code></td><td>string</td><td>ISO 8601 timestamp of last execution.</td></tr>
          <tr><td><code>pinned</code></td><td>boolean</td><td>Whether the command is pinned.</td></tr>
          <tr><td><code>session</code></td><td>string</td><td>TTY session identifier.</td></tr>
        </tbody>
      </table>
      <p>Example history entry:</p>
      <pre><code>{`{"cmd":"cargo build --release","cwd":"/home/user/projects/jesh","exit":0,"ts":"2026-07-27T10:30:00Z","count":15,"last":"2026-07-27T10:30:00Z","pinned":false,"session":"pts/3"}`}</code></pre>

      <h2>Navigation</h2>
      <p>
        Use <kbd>↑</kbd> (Up Arrow) and <kbd>↓</kbd> (Down Arrow) to navigate through
        history. jesh uses a directory-aware algorithm that prefers commands run in the
        current directory, ranking them higher than commands run in other directories.
        This means <code>cargo build</code> appears first when you are in a Rust project,
        even if you have run many other commands elsewhere.
      </p>
      <p>
        The navigation is local-first: the most recent entries are loaded from a memory
        cache for instant response. Entries from the current session are weighted
        highest, followed by entries from the current directory across all sessions, and
        then global history.
      </p>

      <h2>Reverse Search (<kbd>Ctrl+R</kbd>)</h2>
      <p>
        Press <kbd>Ctrl+R</kbd> to enter interactive reverse search mode. As you type,
        jesh performs fuzzy matching against the full history database and displays up
        to 5 matching results in a menu. Use <kbd>↑</kbd> and <kbd>↓</kbd> to select
        an entry and press <kbd>Enter</kbd> to accept it. Press <kbd>Ctrl+R</kbd> again
        to cycle to older matches.
      </p>
      <p>
        The fuzzy search matches substrings and accounts for typos. For example,
        searching for <code>cago bild</code> will match <code>cargo build --release</code>.
        Pinned entries are highlighted or shown first in the results.
      </p>

      <h2>History Builtins</h2>

      <h3><code>history</code></h3>
      <p>
        Print the command history. Without arguments, it prints the recent history to
        stdout with line numbers:
      </p>
      <pre><code>{`$ history
 1  cargo build
 2  git status
 3  vim src/main.rs
 4  cargo test`}</code></pre>

      <h3><code>history pin &lt;cmd&gt;</code></h3>
      <p>
        Pin a command so it is always shown prominently in search results and
        autosuggestions. Pinned commands are never pruned when <code>$HISTFILESIZE</code>
        is exceeded:
      </p>
      <pre><code>{`$ history pin "cargo build --release"`}</code></pre>

      <h3><code>history unpin &lt;cmd&gt;</code></h3>
      <p>
        Remove the pinned status from a command. The command stays in history but is no
        longer protected from pruning:
      </p>
      <pre><code>{`$ history unpin "cargo build --release"`}</code></pre>

      <h3><code>history clear</code></h3>
      <p>
        Clear all history entries from memory and disk. This is irreversible:
      </p>
      <pre><code>{`$ history clear`}</code></pre>

      <h3><code>history tty</code></h3>
      <p>
        Show only the commands that were entered in the current terminal session.
        Useful for reviewing what you have done in this specific window:
      </p>
      <pre><code>{`$ history tty`}</code></pre>

      <h2>Control Variables</h2>
      <table>
        <thead><tr><th>Variable</th><th>Default</th><th>Description</th></tr></thead>
        <tbody>
          <tr><td><code>$HISTSIZE</code></td><td>5000</td><td>Maximum number of entries kept in memory.</td></tr>
          <tr><td><code>$HISTFILESIZE</code></td><td>10000</td><td>Maximum number of entries kept on disk.</td></tr>
          <tr><td><code>$HISTIGNORE</code></td><td>unset</td><td>Colon-separated patterns to ignore (e.g. <code>ls *:rm *:cd</code>).</td></tr>
          <tr><td><code>$HISTCONTROL</code></td><td>unset</td><td>Colon-separated control options: <code>ignoredups</code>, <code>ignorespace</code>, <code>erasedups</code>.</td></tr>
        </tbody>
      </table>
      <p>
        Set these in <code>~/.jeshrc</code>:
      </p>
      <pre><code>{`HISTSIZE=10000
HISTFILESIZE=50000
HISTIGNORE="ls *:rm *:cd *:cargo clean"
HISTCONTROL="ignoredups:ignorespace"`}</code></pre>
      <ul>
        <li><code>ignoredups</code> — Consecutive duplicate commands are stored only once.</li>
        <li><code>ignorespace</code> — Commands starting with a space are not recorded.</li>
        <li><code>erasedups</code> — Duplicate commands are removed from history entirely, preserving only the most recent occurrence.</li>
      </ul>

      <h2>Real-Time Sync</h2>
      <p>
        jesh synchronises history across all terminal sessions in real time. When a
        command finishes, it is appended to the shared JSONL file immediately. Other
        running jesh instances detect the change via incremental file seek — they read
        only the new bytes appended to the file, avoiding the need to re-read the entire
        history. This ensures that <kbd>Ctrl+R</kbd> in one terminal shows commands
        typed in another terminal almost instantly.
      </p>
      <p>
        The sync mechanism is lock-free. Each session writes independently and reads
        use <code>inotify</code> (on Linux) or <code>kqueue</code> (on macOS) to
        receive file change notifications.
      </p>

      <h2>Autosuggestions</h2>
      <p>
        As you type, jesh suggests completions from history. The suggestion appears as
        dimmed text to the right of the cursor, showing the most likely command given
        the current input and directory context. Press <kbd>→</kbd> (Right Arrow) or
        <kbd>Ctrl+F</kbd> to accept the full suggestion. Keep typing to refine the
        match.
      </p>
      <p>
        The autosuggestion algorithm considers the current directory prefix, command
        frequency, recency, and pinned status to rank candidates. Suggestions are
        computed asynchronously so that typing is never delayed.
      </p>
    </DocPage>
  )
}
