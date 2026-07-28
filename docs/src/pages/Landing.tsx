import { Link } from 'react-router-dom'

const features = [
  { icon: '🧠', title: 'Intelligent History', desc: 'JSONL persistence, real-time sync across terminals, directory-aware, history pin/unpin.' },
  { icon: '💡', title: 'Fish-style Autosuggestions', desc: 'Ranked by frequency, recency, directory, and pinned status. Accept with → or End.' },
  { icon: '🔍', title: 'Fuzzy Reverse Search', desc: 'Interactive Ctrl+R with substring/fuzzy matching and arrow key navigation.' },
  { icon: '⚡', title: 'Fast Parser', desc: 'Pipes, redirects, heredocs, process substitution, arithmetic expansion, brace expansion, extglob.' },
  { icon: '🎨', title: 'Rich Prompt', desc: 'RPROMPT, transient prompt, async git branch, theme system, Nerd Fonts, OSC 7/133.' },
  { icon: '📋', title: 'Tab Completion', desc: 'TUI menu, fuzzy matching, programmable via complete -W/-F, flag descriptions.' },
  { icon: '🔧', title: '30+ Builtins', desc: 'cd, pushd/popd/dirs, declare/typeset, local, readonly, getopts, read, printf, and more.' },
  { icon: '🖥️', title: 'Terminal Protocols', desc: 'Kitty Graphics Protocol, OSC 8 hyperlinks, OSC 133 shell integration, East Asian Width.' },
  { icon: '🔀', title: 'Bash Fallback', desc: 'Scripts with nvm, rvm and other Bash-specific features are delegated transparently.' },
  { icon: '🌐', title: 'Cross-platform', desc: 'Linux, macOS, Windows (native, not just WSL).' },
]

const builtins = [
  ['cd', 'Change directory'],
  ['pushd / popd / dirs', 'Directory stack navigation'],
  ['export / unset', 'Manage environment variables'],
  ['alias / unalias', 'Manage command aliases'],
  ['source / .', 'Execute script in current context'],
  ['history', 'View and manage command history'],
  ['set / shopt', 'Shell options (-e, -u, -x, -o pipefail, glob flags)'],
  ['declare / typeset', 'Variable attributes (-i, -a, -A, -r, -x)'],
  ['local / readonly', 'Local and read-only variables'],
  ['getopts', 'Parse script options'],
  ['eval / exec / command', 'Command execution control'],
  ['test / [ / [[', 'Conditional expressions'],
  ['read / printf / echo', 'Input/output'],
  ['jobs / fg / bg / disown / kill', 'Job control'],
  ['type / which', 'Locate commands'],
  ['complete', 'Programmable completion'],
]

export function Landing() {
  return (
    <>
      <div className="crate-header">
        <div className="container">
          <h1>jesh <span className="version">2.0.1</span></h1>
          <div className="description">
            A modern, fast Unix shell written in Rust. Combines POSIX/Bash compatibility with
            intelligent features from Fish, Zsh, and Nushell.
          </div>
          <div className="badges">
            <img src="https://img.shields.io/github/stars/jefferson-it/jesh?style=social" alt="Stars" />
            <img src="https://img.shields.io/badge/Rust-1.84+-purple" alt="Rust" />
            <img src="https://img.shields.io/badge/license-MIT-blue" alt="License" />
            <img src="https://img.shields.io/badge/platform-linux%20%7C%20macOS%20%7C%20windows-lightgrey" alt="Platform" />
          </div>
          <div className="crate-tabs">
            <a href="/docs" className="active">📦 Crate</a>
            <Link to="/docs/getting-started">📚 Documentation</Link>
            <a href="https://github.com/jefferson-it/jesh">📂 Source</a>
          </div>
        </div>
      </div>

      <div className="container package-page">
        <div className="two-col">
          <aside className="sidebar">
            <div className="sidebar-section">
              <h3>Quick Links</h3>
              <nav>
                <a href="#install">Installation</a>
                <a href="#features">Features</a>
                <a href="#docs">Documentation</a>
                <a href="#builtins">Builtins</a>
              </nav>
            </div>
            <div className="sidebar-section">
              <h3>Links</h3>
              <nav>
                <a href="https://github.com/jefferson-it/jesh">⭐ GitHub</a>
                <a href="https://github.com/jefferson-it/jesh/issues">🐛 Issues</a>
              </nav>
            </div>
          </aside>

          <div className="main">
            <div className="warning-banner">
              ⚠️ jesh is still in active development. Some Bash features may not be fully
              supported yet. See <Link to="/docs/vs-bash">Differences vs Bash</Link> for details.
            </div>

            <h2 id="install">Installation</h2>
            <div className="install-block">
              <strong>Via Cargo</strong>
              <div className="cmd">
                <code>cargo install jesh</code>
                <CopyBtn text="cargo install jesh" />
              </div>
            </div>
            <div className="install-block">
              <strong>Build from source</strong>
              <div className="cmd">
                <code>git clone https://github.com/jefferson-it/jesh && cd jesh && cargo build --release</code>
                <CopyBtn text="git clone https://github.com/jefferson-it/jesh && cd jesh && cargo build --release" />
              </div>
            </div>

            <h2 id="features">Features</h2>
            <div className="feature-grid">
              {features.map((f, i) => (
                <div key={i} className="feature-item">
                  <strong>{f.icon} {f.title}</strong>
                  <span>{f.desc}</span>
                </div>
              ))}
            </div>

            <h2 id="builtins">Builtin Commands</h2>
            <table>
              <thead><tr><th>Command</th><th>Description</th></tr></thead>
              <tbody>
                {builtins.map(([cmd, desc], i) => (
                  <tr key={i}><td><code>{cmd}</code></td><td>{desc}</td></tr>
                ))}
              </tbody>
            </table>

            <h2 id="docs">Documentation</h2>
            <div className="quick-links">
              {[
                ['Getting Started', '/docs/getting-started'],
                ['Configuration', '/docs/configuration'],
                ['Builtins Reference', '/docs/builtins'],
                ['Scripting', '/docs/scripting'],
                ['Parser', '/docs/parser'],
                ['Globbing', '/docs/globbing'],
                ['Autocomplete', '/docs/autocomplete'],
                ['Prompt', '/docs/prompt'],
                ['Jobs & Processes', '/docs/jobs'],
                ['History', '/docs/history'],
                ['Vs Bash', '/docs/vs-bash'],
                ['Examples', '/docs/examples'],
              ].map(([label, to]) => (
                <Link key={to} to={to}>{label}</Link>
              ))}
            </div>

            <h2>Quick Start</h2>
            <p>Create a <code>~/.jeshrc</code> file:</p>
            <pre><code># jesh configuration
INIT_INFO=true
THEME="jesh-dracula"

alias ll="eza -la"
alias gs="git status"</code></pre>
            <p>Run <code>jesh</code> and start typing. Press <kbd>Ctrl+R</kbd> for fuzzy history search,
            <kbd>Tab</kbd> for completions, and <kbd>→</kbd> to accept autosuggestions.</p>
          </div>
        </div>
      </div>
    </>
  )
}

function CopyBtn({ text }: { text: string }) {
  const copy = () => {
    navigator.clipboard.writeText(text).then(() => {
      const btn = document.activeElement as HTMLElement
      if (btn) {
        const orig = btn.textContent
        btn.textContent = 'Copied!'
        setTimeout(() => { btn.textContent = orig }, 1500)
      }
    })
  }
  return (
    <button className="copy" onClick={copy}>Copy</button>
  )
}
