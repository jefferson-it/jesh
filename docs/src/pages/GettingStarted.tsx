import { DocPage } from '../components/DocPage'

export default function GettingStarted() {
  return (
    <DocPage title="Getting Started">
      <h2>Installation</h2>
      <p>
        The easiest way to install jesh is via <code>cargo</code>, Rust's package manager:
      </p>
      <pre><code>{`cargo install jesh`}</code></pre>
      <p>
        This will download the source, compile it, and place the <code>jesh</code> binary in
        your <code>~/.cargo/bin</code> directory. Make sure that directory is on your
        <code>$PATH</code>.
      </p>
      <p>
        To build from source, clone the repository and compile with <code>cargo build</code>:
      </p>
      <pre><code>{`git clone https://github.com/jefferson-it/jesh.git
cd jesh
cargo build --release`}</code></pre>
      <p>
        The resulting binary will be at <code>target/release/jesh</code>. You can copy it
        anywhere on your <code>$PATH</code>.
      </p>

      <h2>First Run</h2>
      <p>
        Simply type <code>jesh</code> in your terminal to launch the shell. You will be
        greeted with an interactive prompt. If you have not yet created a configuration
        file, jesh uses sensible defaults — a POSIX-compatible prompt, basic history
        tracking, and default key bindings.
      </p>

      <h2>Configuration File</h2>
      <p>
        jesh reads <code>~/.jeshrc</code> on startup. This file supports variable
        assignments, aliases, and function definitions. Here is a minimal example:
      </p>
      <pre><code>{`# ~/.jeshrc — jesh configuration
INIT_INFO=true
THEME="jesh-dracula"

alias ll="eza -la"
alias gs="git status"
alias grep="grep --color=auto"`}</code></pre>
      <p>
        Set <code>INIT_INFO=true</code> to display a welcome banner with version and
        system information. The <code>THEME</code> variable selects a built-in color
        scheme. See the Configuration page for a full list of variables.
      </p>

      <h2>Basic Usage</h2>
      <p>
        jesh behaves like a standard Unix shell. Type a command and press
        <kbd>Enter</kbd> to execute it. Pipes, redirects, and environment variables
        all work as expected:
      </p>
      <pre><code>{`~> ls -la | grep ".rs"
~> echo "Hello, $USER" > greeting.txt
~> cd projects && make`}</code></pre>

      <h3>Tab Completion</h3>
      <p>
        Press <kbd>Tab</kbd> to complete commands, file paths, and arguments. If there
        are multiple matches, a TUI menu appears. Use the arrow keys to navigate the
        menu and <kbd>Enter</kbd> to select. The completion system is programmable via
        the <code>complete</code> builtin.
      </p>

      <h3>History Navigation</h3>
      <p>
        Press <kbd>Up</kbd> and <kbd>Down</kbd> to scroll through previous commands.
        jesh's history is persisted to disk in JSONL format at
        <code>~/.local/share/jesh/history.jsonl</code> and supports directory-aware
        filtering, pinning, and real-time sync across terminal sessions.
      </p>

      <h3>Reverse Search</h3>
      <p>
        Press <kbd>Ctrl+R</kbd> to enter interactive reverse search. Start typing any
        portion of a previous command; jesh performs fuzzy/substring matching and
        displays results ranked by frequency, recency, and directory relevance. Use
        <kbd>Ctrl+R</kbd> again to cycle through older matches, or use the arrow keys
        to navigate the list. Press <kbd>Enter</kbd> to execute the selected command
        or <kbd>Tab</kbd> to place it on the input line for editing.
      </p>

      <h3>Autosuggestions</h3>
      <p>
        As you type, jesh displays a dimmed suggestion based on your history. Press
        <kbd>→</kbd> (right arrow) or <kbd>End</kbd> to accept the suggestion. This
        feature is inspired by Fish and uses the same ranking algorithm as the reverse
        search.
      </p>

      <h2>Next Steps</h2>
      <p>
        Read the Configuration page to customize your shell, explore the Builtins
        reference for the full command list, and check Scripting if you want to write
        jesh scripts.
      </p>
    </DocPage>
  )
}
