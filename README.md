# jesh — Modern Unix Shell in Rust

**jesh** is an interactive shell and scripting engine written in Rust, blending POSIX/Bash compatibility with smart features from Fish, Zsh, and Nushell.

## Quick Start

```bash
# Cargo
cargo install jesh

# Build from source
git clone https://github.com/jefferson-it/jesh
cd jesh
cargo build --release
./target/release/jesh
```

## Features

### Shell & Scripting
- POSIX-compatible parser: pipes, redirects, heredocs, process substitution `<(cmd)`
- Arithmetic expansion `$((expr))`, brace expansion `{1..10}`, ANSI-C quoting `$'...'`
- Extended globbing (extglob) + Zsh-style qualifiers
- Bash fallback — delegates `.bashrc` scripts to bash
- Flow control: `if`/`else`/`case`/`while`/`until`/`for`
- `declare`/`typeset` with `-i`/`-a`/`-A`/`-r`/`-x`, `local`, `readonly`, `getopts`
- `set -e`/`-u`/`-x`/`-o pipefail`

### 40+ Builtins
`cd`, `pwd`, `exit`, `echo`, `export`, `unset`, `alias`, `unalias`, `source`, `.`, `history`, `type`, `which`, `pushd`, `popd`, `dirs`, `read`, `printf`, `eval`, `exec`, `command`, `true`, `false`, `:`, `test`, `[`, `[[`, `declare`, `typeset`, `local`, `readonly`, `getopts`, `disown`, `set`, `shopt`, `complete`, `jobs`, `fg`, `bg`, `kill`, `jeofetch`

### Smart History
- JSONL persistence with metadata (timestamp, directory, exit code, frequency)
- Cross-session sync via incremental seek
- `history pin`/`unpin` for favorite commands
- Directory-aware ranking
- `$HISTSIZE`/`$HISTFILESIZE`/`$HISTIGNORE`/`$HISTCONTROL`

### Autosuggestions & Completion
- Fish-style suggestions ranked by frequency + recency + directory (<5ms)
- TUI menu selection with fuzzy search (`/u/l/b` → `/usr/local/bin`)
- Programmable via `complete -W`/`-F`
- Command and flag descriptions

### Prompt

- RPROMPT (right prompt) with exit status, git branch, SSH
- Transient prompt — simplifies previous lines after execution
- Async rendering — git fetch in background
- Theme system via `$THEME` (Dracula, Dark, custom)
- Nerd Fonts, OSC 7, OSC 133 support

### Line Editing
- Emacs & Vi mode with block/bar cursor
- Multi-line editing, syntax highlighting, bracket matching
- Smart paste (escapes meta-characters)
- Yank ring (Alt+Y after Ctrl+Y)
- Configurable keybindings

### Job Control
- Background `&`, `fg`, `bg`, `jobs`, `disown`
- Ctrl+Z/Ctrl+C/Ctrl+D
- Process group isolation (PGID)
- Async job termination notifications

### Terminal Protocols
- Kitty Graphics Protocol — inline images
- OSC 8 hyperlinks — clickable links
- OSC 7 directory notifications
- East Asian Width — correct cursor for CJK/emoji

### Integrations
- `zoxide` — smart `z` navigation
- `eza`/`exa` replaces `ls`, `bat` replaces `cat`
- Semantic pipeline engine (Nushell-style structured data)

## Configuration

Create `~/.jeshrc`:

```bash
INIT_INFO=true
THEME="jesh-dracula"
alias ll="eza -la"
alias gs="git status"
```

## Compatibility

| System  | Status              |
|---------|---------------------|
| Linux   | Native              |
| macOS   | Native              |
| Windows | Native (not just WSL) |
| FreeBSD | Compilable          |

## Performance

- Startup < 30ms
- History suggestions < 5ms
- Lazy loading, PATH/autocomplete/git caching

## License

MIT

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for build, test, and style guidelines.
