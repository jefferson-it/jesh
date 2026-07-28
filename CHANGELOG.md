# Changelog

## [2.0.1] — 2026-07-27

### Fixed
- RPROMPT rendering in `render_rprompt()` — now properly included in prompt output
- Smart paste — `smart_paste_escape()` wraps with single quotes and escapes internal `'`
- Removed dead code: unreachable `hyperlink`, `kitty`, `table` builtin handlers
- OSC 133 sequences (`\e]133;C` and `\e]133;D;{code}`) now properly emitted

### Added
- `set -e` (errexit) — abort script on first error
- `set -u` (nounset) — warn on undefined variable usage
- `set -x` (xtrace) — print each command before execution
- `zoxide` integration — alias `z` and `zoxide add` hook after `cd`
- Configurable keybindings via `[keybindings]` in `config.toml`
- Transient prompt via `JSH_TRANSIENT_PROMPT=true`
- Theme system with `$THEME`, loading from `~/.local/jesh/themes/<nome>.sh`
- Three example themes: `jesh-default`, `jesh-dark`, `jesh-dracula`

## [2.0.0] — 2026-06-15

### Added
- Initial public release
- Interactive shell with readline support
- History system with JSONL persistence
- Fish-style autosuggestions
- Fuzzy reverse search (Ctrl+R)
- Parser with pipes, redirects, heredocs, process substitution
- Arithmetic expansion `$((expr))`
- ANSI-C quoting `$'...'`
- Brace expansion `{1..10}`, `{a,b,c}`
- Extended globbing (extglob) with qualifiers
- Complete builtin set: `cd`, `export`, `unset`, `alias`, `unalias`, `source`, `.`, `history`, `pushd`, `popd`, `dirs`, `set`, `shopt`, `complete`, `eval`, `exec`, `command`, `read`, `printf`, `true`, `false`, `:`, `test`, `[`, `[[`, `declare`, `typeset`, `local`, `readonly`, `getopts`, `disown`
- Job control: `jobs`, `fg`, `bg`, `disown`, Ctrl+Z, Ctrl+C
- Tab completion with TUI menu selection and fuzzy search
- Programmable completion via `complete -W`/`-F`
- Bash fallback for `.bashrc` scripts
- Semantic pipeline engine (Nushell-style)
- Kitty Graphics Protocol support
- OSC 8 hyperlinks and OSC 7 directory notifications
- Cross-platform: Linux, macOS, Windows
