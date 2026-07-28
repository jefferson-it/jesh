# jesh — Shell Unix-like Moderno em Rust

**jesh** é um shell interativo e motor de scripting escrito em Rust, combinando a compatibilidade POSIX/Bash com recursos inteligentes de shells modernos como Fish, Zsh e Nushell.

---

## Recursos

### Histórico Inteligente
- **JSONL persistente** — histórico salvo em `~/.local/share/jesh/history/` com metadados (timestamp ISO 8601, diretório, exit code, frequência)
- **Sincronização entre sessões** — múltiplos terminais compartilham histórico em tempo real via seek incremental
- **`history pin`/`unpin`** — fixe comandos favoritos para destaque nas sugestões
- **`$HISTSIZE`/`$HISTFILESIZE`/`$HISTIGNORE`/`$HISTCONTROL`**
- **Directory-aware** — prioriza comandos do diretório atual na navegação e sugestões

### Sugestões Automáticas (Fish-style)
- Ranking por frequência + recência + diretório + pinned em <5ms
- Aceite com → ou End

### Busca Reversa Fuzzy (Ctrl+R)
- Busca por substring ou padrão fuzzy
- Menu interativo com 5 resultados e navegação por setas

### Prompt Poderoso
- **RPROMPT** (prompt direito) com status de saída, SSH, git branch
- **Prompt transiente** — linhas anteriores são simplificadas após execução
- **Renderização assíncrona** — git fetch em background, sem travar
- **Temas** — `$THEME` carrega scripts de `~/.local/jesh/themes/` (ex: `jesh-dark`, `jesh-dracula`)
- Suporte a Nerd Fonts, OSC 7, OSC 133

### Parser Shell Robusto
- Pipes `|`, stderr pipe `|&`
- Redirecionamentos: `>`, `>>`, `<`, `2>`, `2>>`, `&>`, `&>>`
- Heredoc `<<` e Here String `<<<`
- Process substitution `<(comando)` e `>(comando)`
- Expansão aritmética `$((expr))`
- ANSI-C quoting `$'...'`
- Brace expansion `{1..10}`, `{a,b,c}`
- Extglob: `@(...)`, `*(...)`, `+(...)`, `?(...)`, `!(...)`
- Glob qualifiers Zsh-style: `*(/)` (dirs), `*(.)` (files), `*(@)` (symlinks)
- `nullglob`, `failglob`, `dotglob`, `nocaseglob`
- Expansão de histórico: `!!`, `!$`, `!n`, `!prefixo`, `!?texto`

### Builtins Completos
`cd`, `pwd`, `exit`, `echo`, `export`, `unset`, `alias`, `unalias`, `history`, `type`, `which`, `source`, `.`, `pushd`, `popd`, `dirs`, `read`, `printf`, `eval`, `exec`, `command`, `true`, `false`, `:`, `test`, `[`, `[[`, `declare`/`typeset`, `local`, `readonly`, `getopts`, `disown`, `set`, `shopt`, `complete`, `jobs`, `fg`, `bg`, `kill`, `jeofetch`

### Autocompletar
- TUI menu selection com setas (Zsh-style)
- Busca fuzzy (`/u/l/b` → `/usr/local/bin`)
- Programável via `complete -W`/`-F`
- Descrições de comandos e flags

### Scripting
- `if`/`else`/`elif`/`case`/`while`/`until`/`for`
- Funções com `local`
- `declare -i`/`-a`/`-A`/`-r`/`-x`
- `set -e`/`-u`/`-x`/`-o pipefail`
- `getopts` para parsing de opções
- Bash fallback — scripts `.bashrc` que usam `nvm`, `rvm` etc. são delegados ao bash

### Jobs & Sinais
- Background `&`, foreground `fg`, `bg`, `jobs`, `disown`
- Ctrl+Z, Ctrl+C, Ctrl+D
- Isolamento de Process Groups (PGID)
- Notificação assíncrona de término de jobs

### Linha de Comando
- Navegação: setas, Home/End, Ctrl+A/E, Ctrl+K/U/W, Ctrl+L
- Ctrl+←/→, Alt+B/F
- Yank-ring (Alt+Y após Ctrl+Y)
- Multi-line editing
- Vi mode com cursor bloco/barra
- Syntax highlighting
- Bracket matching
- Smart paste (escapa meta-characters ao colar)

### Protocolos de Terminal
- **Kitty Graphics Protocol** — renderize imagens com `kitty image`
- **OSC 8 Hyperlinks** — links clicáveis
- **OSC 133** — shell integration para terminais modernos
- **OSC 7** — notificação de diretório
- East Asian Width — suporte a caracteres double-width (emoji, CJK)

### Integrações
- **`zoxide`** — alias `z` para navegação inteligente
- **`eza`/`exa`** — substitui `ls` automaticamente
- **`bat`** — substitui `cat` automaticamente
- **Motor semântico** — saída de comandos tratada como tabelas (Nushell-style)

---

## Instalação

### Via Cargo (Rust)
```bash
cargo install jesh
```

### Via Curl
```bash
curl -fsSL https://jesh.sh/install.sh | sh
```

### Build Manual
```bash
git clone https://github.com/anomalyco/jesh
cd jesh
cargo build --release
./target/release/jesh
```

---

## Começando

Crie seu arquivo de configuração `~/.jeshrc`:

```bash
# jesh configuration
INIT_INFO=true
HOT_RELOAD=true
SHOW_TIMING=true
THEME="jesh-dracula"

# Aliases
alias ll="eza -la"
alias gs="git status"
alias z="zoxide"

# Prompt customization
export JSH_THEME_DIR_COLOR="cyan"
export JSH_THEME_GIT_COLOR="green"
```

---

## Tema de Cores

O jesh inclui temas em `assets/themes/`:

- `jesh-default.sh` — tema padrão
- `jesh-dark.sh` — tema escuro
- `jesh-dracula.sh` — tema Dracula

Ative com `THEME="jesh-dracula"` no `.jeshrc`. Temas customizados em `~/.local/jesh/themes/<nome>.sh` também são suportados.

---

## Documentação

A documentação completa está em `/docs/`:

- [Getting Started](/docs/getting-started/)
- [Configuration](/docs/configuration/)
- [Builtins](/docs/builtins/)
- [Scripting](/docs/scripting/)
- [Parser](/docs/parser/)
- [Globbing](/docs/globbing/)
- [Autocomplete](/docs/autocomplete/)
- [Prompt](/docs/prompt/)
- [Jobs & Processes](/docs/jobs/)
- [History](/docs/history/)
- [Differences vs Bash](/docs/vs-bash/)
- [Examples](/docs/examples/)

---

## Compatibilidade

| Sistema | Status |
|---------|--------|
| Linux | ✅ Nativo |
| macOS | ✅ Nativo |
| Windows | ✅ Nativo (não apenas WSL) |
| FreeBSD | ✅ Compilável |

---

## Performance

- Inicialização < 30ms
- Sugestões de histórico em < 5ms
- Lazy loading de funcionalidades
- Cache de PATH, autocomplete e git

---

## Licença

MIT

---

## Contribuindo

Contribuições são bem-vindas! Consulte [CONTRIBUTING.md](CONTRIBUTING.md) para guia de build, testes e estilo de código.
