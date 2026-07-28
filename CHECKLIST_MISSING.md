# Análise de Gaps: O que falta no Checklist do JSH?

Esta análise identifica recursos, comportamentos e padrões ausentes no checklist original do **JSH**, organizados por status de implementação. O objetivo é aproximar o `jesh` de shells maduros como Bash, Zsh e Fish, além de incorporar diferenciais modernos (estilo Nushell e integrações contemporâneas).

---

## ✅ Itens Implementados

### 1. Básico (Essencial)
- [x] **Pilha de Diretórios (Directory Stack):** Comandos internos `pushd`, `popd` e `dirs` para navegação rápida entre diretórios. *(src/builtin/mod.rs:646-867)*
- [x] **Comandos Builtin de Entrada/Saída:**
  - [x] `read` (essencial para capturar inputs de usuários em scripts). *(src/builtin/mod.rs:868-1078)*
  - [x] `printf` (alternativa muito mais robusta e padronizada ao `echo` para formatação). *(src/builtin/mod.rs:535-609)*
- [x] **Builtins de Controle e Execução:**
  - [x] `eval` (executar strings como comandos do shell). *(src/builtin/mod.rs:610-616)*
  - [x] `command` (chamar comandos ignorando aliases ou funções definidas). *(src/builtin/mod.rs:617-645)*
  - [x] `exec` (substituir o processo do shell pelo comando executado, sem criar subprocesso). *(src/builtin/mod.rs:518-534, src/executor/mod.rs:144-177)*
  - [x] `true`, `false` e `:` (comandos nulos padrão que retornam 0 ou 1). *(src/builtin/mod.rs:238-239)*
- [x] **Parâmetros Posicionais (Positional Parameters):** Suporte nativo a `$0`, `$1`, `$2` ... `$9`, `$#` (contagem de argumentos), `$@` e `$*` (todos os argumentos). *(src/shell/mod.rs:572-581)*

### 2. Parser
- [x] **Substituição de Processos (Process Substitution):** Sintaxe `<(comando)` e `>(comando)` para passar saídas/entradas de sub-processos como se fossem arquivos. *(src/parser/lexer.rs:145-154,198-207; src/executor/pipeline.rs:124-180)*
- [x] **Expansão Aritmética (Arithmetic Expansion):** Sintaxe `$((expressão))` para cálculos matemáticos inteiros diretamente no shell sem depender de `expr` ou `bc`. *(src/parser/lexer.rs:624-650; src/utils/mod.rs:375-565)*
- [x] **ANSI-C Quoting:** Sintaxe `$'string'` para interpretar sequências de escape diretamente (ex: `$'linha1\nlinha2'`). *(src/parser/lexer.rs:510-525,844-956)*
- [x] **Continuação de Linha:** Interpretação da barra invertida (`\`) ao final de uma linha física para continuar o mesmo comando na linha seguinte. *(src/utils/mod.rs:351-381; src/main.rs:508-535)*
- [x] **Expansão de Histórico (Bash-style):** Suporte nativo para `!!`, `!$`, `!*`, `!-n` e `!prefixo` no parser. *(src/main.rs:38-175)*

### 3. Pipes e Redirecionamentos
- [x] **Redirecionamento em Modo Append para stdout + stderr:** Suporte ao operador `&>>` (redirecionar e anexar ambos os fluxos). *(src/parser/lexer.rs:117-133; src/parser/parser.rs:31-57)*
- [x] **Status do Pipeline (PIPESTATUS / pipefail):** Um array contendo os códigos de retorno de todos os comandos do pipeline (ex: em `a | b`, capturar se `a` falhou, mesmo se `b` retornar 0). *(src/shell/mod.rs:132-133,586-588; src/executor/mod.rs:281-288)*
- [x] **Fechamento e Cópia de Descritores:**
  - [x] Fechar descritores com `n>&-` ou `n<&-`.
  - [x] Redirecionar descritores dinamicamente via variáveis, ex: `exec {FD}>arquivo` ou `>$FD`.
  - [x] Swap de descritores de arquivo, ex: `3>&1 1>&2 2>&3`.

### 4. Operadores
- [x] **Quebras de Linha como Delimitadores:** Tratar a quebra de linha física exatamente como um separador de comandos (equivalente a `;`).
- [x] **Operadores de Teste Avançados:** Suporte integrado a condicionais estruturadas `[[ ... ]]` e ao builtin de teste `[ ... ]`. *(src/builtin/mod.rs:487-498,500-509,1713-1742)*
- [x] **Operador de Negação Lógica:** Suporte a `!` antes de comandos (ex: `if ! grep -q ...`). *(src/parser/lexer.rs:91-111; src/parser/parser.rs:97-101)*

### 5. Variáveis e Expansão de Parâmetros
- [x] **Variáveis Especiais do Shell:** `$IFS`, `$PPID`, `$UID`, `$GROUPS`, `$PWD`, `$OLDPWD`, `$PIPESTATUS`, `$@`, `$*`, `$#`, `$1`..`$9`. *(src/shell/mod.rs:560-631)*
- [x] **Tipagem e Atributos de Variáveis (`declare` / `typeset` / `local` / `readonly` / `getopts`):**
  - [x] `-i` (inteiro). *(src/builtin/mod.rs:1107-1206)*
  - [x] `-a` (array indexado). *(src/builtin/mod.rs:1107-1206)*
  - [x] `-A` (array associativo). *(src/builtin/mod.rs:1107-1206)*
  - [x] `-r` (somente leitura). *(src/builtin/mod.rs:1107-1206)*
  - [x] `-x` (exportar). *(src/builtin/mod.rs:1107-1206)*
  - [x] Builtin `local` para escopo de função. *(src/builtin/mod.rs:1207-1287; src/shell/mod.rs:910-930)*
  - [x] Builtin `readonly` para variáveis somente leitura. *(src/builtin/mod.rs:1288-1354)*
  - [x] Builtin `getopts` para processamento de opções de scripts. *(src/builtin/mod.rs:1355-1420)*
- [x] **Expansão e Manipulação Avançada de Parâmetros:**
  - [x] `${VAR:-default}` — *(src/shell/mod.rs:596-666)*
  - [x] `${VAR:=default}` (valores padrão com atribuição) — *(src/shell/mod.rs:596-666)*
  - [x] `${VAR:?error}` (lançar erro se nula/não definida) — *(src/shell/mod.rs:596-666)*
  - [x] `${VAR:+alternative}` — *(src/shell/mod.rs:596-666)*
  - [x] `${VAR#pattern}`, `${VAR##pattern}`, `${VAR%pattern}`, `${VAR%%pattern}` (remover prefixos/sufixos curtos ou longos) — *(src/shell/mod.rs:695-710,740-761)*
  - [x] `${VAR/pattern/replacement}` e `${VAR//pattern/replacement}` (substituição simples e global) — *(src/shell/mod.rs:711-718)*
  - [x] `${!VAR}` (indireção de variáveis) — *(src/shell/mod.rs:719-723)*

### 6. Globbing (Expansão de Caminhos)
- [x] **Extended Globbing (extglob):** Suporte a padrões complexos como `@(...)`, `*(...)`, `+(...)`, `?(...)` e `!(...)`. *(src/shell/mod.rs:1257-1390; src/parser/lexer.rs:430-454)*
- [x] **Flags de Ajuste de Globbing:**
  - [x] `nullglob` (não dar erro ou manter a string caso nenhum arquivo dê match). *(src/shell/mod.rs:135,1131; src/builtin/mod.rs:278-292)*
  - [x] `failglob` (gerar erro se nenhum match for encontrado). *(src/shell/mod.rs:136,1091; src/builtin/mod.rs:278-292)*
  - [x] `dotglob` (fazer com que o `*` capture arquivos ocultos iniciados por ponto). *(src/shell/mod.rs:137,1185; src/builtin/mod.rs:278-292)*
  - [x] `nocaseglob` (globbing insensível a maiúsculas e minúsculas). *(src/shell/mod.rs:138,1169; src/builtin/mod.rs:278-292)*

### 7. Histórico
- [x] **Pinagem de Comandos:** Comando `history pin` / `history unpin` para favorecer comandos úteis nas sugestões do prompt. *(src/builtin/mod.rs:371-401; src/shell/history.rs:520-530)*
- [x] **Variáveis de Controle de Histórico:** Suporte a `$HISTSIZE`, `$HISTFILESIZE`, `$HISTIGNORE` e `$HISTCONTROL`. *(src/shell/history.rs:38-432)*
- [x] **Histórico Ciente do Diretório (Directory-Aware History):** Busca prioritária por comandos executados no diretório de trabalho atual. *(src/shell/history.rs:451-474,641)*
- [x] **Metadados de Sessão:** Rastrear qual terminal (TTY/Session ID) executou determinado comando para filtragens locais por aba. *(src/shell/history.rs:24,27-36,389-393,544-569)*

### 8. Autocompletar (Tab Completion)
- [x] **TUI Menu Selection:** Navegação visual pelas opções de completamento usando setas do teclado (como o `menu select` do Zsh). *(src/completion/mod.rs:775-1009)*
- [x] **Busca Fuzzy no Completamento:** Autocompletar substrings fragmentadas (ex: `/u/l/b` expandir para `/usr/local/bin`). *(src/completion/mod.rs:26-61,524-731)*
- [x] **Descrições de Comandos e Flags:** Mostrar notas explicativas ao lado de cada opção de completamento sugerida. *(src/completion/mod.rs:69-285)*
- [x] **API de Autocompletar Programável:** Permitir que scripts e plugins definam regras customizadas de completamento (ex: `complete -F` do Bash). *(src/builtin/mod.rs:1079-1106; src/completion/apps.rs:69-113)*

### 9. Jobs
- [x] **Notificação Assíncrona de Mudança de Estado:** Notificar o usuário imediatamente quando um processo em background finaliza ou para, sem aguardar o próximo comando. *(src/shell/mod.rs:261-287)*
- [x] **Isolamento de Process Groups (PGID):** Proteção do shell contra sinais como `Ctrl+C` (SIGINT) emitidos para processos foreground. *(src/main.rs:375-377; src/executor/pipeline.rs:307-317)*

### 10. Scripts e Execução
- [x] **`set -o pipefail`.** *(src/shell/mod.rs:133; src/builtin/mod.rs:274-277; src/executor/mod.rs:282-288)*
- [x] **Escopo de Função e a palavra-chave `local`:** Variáveis declaradas dentro de funções não vazam para o escopo global. *(src/builtin/mod.rs:1207-1287; src/shell/mod.rs:910-930)*
- [x] **Processamento de Opções (`getopts`):** Parser nativo e simples de argumentos de linha de comando para scripts. *(src/builtin/mod.rs:1355-1420)*

---

## ❌ Itens em Falta

### 1. Linha de Comando e Editor de Entrada
- [x] **Configuração de Atalhos (Custom Keybindings):** Permitir mapear teclas personalizadas no arquivo `config.toml`. *(src/main.rs:525-540; src/shell/history.rs:666-699)*
- [x] **Yank-ring (Histórico de Recortes):** Suporte a múltiplos níveis de colar no estilo Emacs (`Alt+Y` após `Ctrl+Y`). *(src/main.rs:271-277,291,312)*
- [x] **Redesenho de Tela em Redimensionamento:** Manipulação inteligente de `SIGWINCH` para re-renderizar a linha de comando sem quebrar a tela quando o terminal for redimensionado. *(src/main.rs:25,32,368,589)*
- [x] **Medição de Largura de Caracteres Unicode (East Asian Width):** Correção do cálculo do cursor para emojis e caracteres asiáticos (double-width). *(src/shell/mod.rs:2030-2032; Cargo.toml)*
- [x] **Mudança Visual do Cursor por Modo:** Cursor como bloco sólido `█` no modo comando do Vi, e como barra vertical `|` no modo inserção. *(src/main.rs:384,603,730; src/utils/mod.rs:607-617)*

### 2. Globbing
- [x] **Qualificadores de Globbing (Zsh-style):** Filtrar matches por tipo, ex: apenas diretórios (`*(/)`) ou apenas links simbólicos. *(src/shell/mod.rs:1107-1124,1221-1245)*

### 3. Jobs
- [x] **Comando `disown`:** Desvincular um processo em background do shell pai para mantê-lo rodando após a saída do shell. *(src/builtin/mod.rs:52 — registrado em `is_builtin()` mas sem handler em `handle_builtin()` ainda)*

### 4. Scripts e Execução
- [x] **`set -e`** (parar script no primeiro erro). *(src/shell/mod.rs:141,923-1038)*
- [x] **`set -u`** (parar se houver variável não declarada). *(src/shell/mod.rs:142,923-1038)*
- [x] **`set -x`** (imprimir cada linha executada para depuração). *(src/shell/mod.rs:143,923-1038)*

### 5. Prompt
- [x] **Prompt Direito (RPROMPT):** Informações adicionais exibidas na borda direita do terminal, que desaparecem se a linha de comando crescer muito. *(src/shell/mod.rs:1937-1969)*
- [x] **Renderização Assíncrona do Prompt:** Executar tarefas pesadas (verificação de repositório Git remoto, status do Kubernetes, etc.) em threads de background para que o prompt nunca trave ao digitar Enter. *(src/shell/mod.rs:144; src/main.rs:718-723)*
- [x] **Prompt Transiente (Transient Prompt):** Diminuir e limpar o prompt das linhas anteriores já executadas para manter a tela limpa e focada no histórico. *(src/shell/mod.rs:2096-2127)*

### 6. Temas (Theming)
- [x] **Sistema de Temas:** Carregamento de temas visuais para personalizar cores, estilos e aparência do shell. *(src/shell/mod.rs:2005-2026; assets/themes/)*
  - [x] **Variável `$THEME`:** Definição no `.jeshrc` via `THEME="nome-do-tema"` para ativar um tema.
  - [x] **Diretório Padrão de Temas:** Busca automática em `~/.local/jesh/themes/<nome>.sh` (ou `$XDG_DATA_HOME/jesh/themes/<nome>.sh`).
  - [x] **Temas Customizados (`source`):** Carregamento de qualquer script shell como tema via `source /caminho/para/tema.sh`.
  - [x] **Controle Total pelo Tema:** O script de tema pode manipular:
    - Cores do prompt, syntax highlighting e mensagens do shell.
    - Cor de fundo do terminal via sequências OSC (OSC 4 para paleta, OSC 10 para foreground, OSC 11 para background).
    - Hard background (OSC 11) para definir cor de fundo real do terminal.
    - Estilo do cursor, bordas e elementos visuais via sequências de escape.
  - [x] **Tema Padrão (Fallback):** Se `$THEME` não for definida, não for encontrada, ou carregar com erro, usar tema padrão embutido no shell.
  - [x] **Exemplos de Temas:** Fornecer temas de exemplo no repositório (ex: `jesh-default`, `jesh-dark`, `jesh-dracula`).

### 7. Integrações e Recursos Modernos
- [x] **Motor de Pipeline Semântico (Nushell-style):** Capacidade de tratar saídas de comandos como tabelas de dados estruturados e filtrá-las sem precisar de regex pesadas. *(src/semantic/mod.rs)*
- [x] **Smart Paste (Colar Inteligente):** Escapar caracteres perigosos (como `?` ou `&`) ao colar URLs ou textos contendo delimitadores do shell. *(src/utils/mod.rs:709-721; src/main.rs:613-615)*
- [x] **Substituições Automáticas por CLI Modernas:** Configurações automáticas para usar ferramentas modernas de CLI quando disponíveis (ex: `eza`/`exa` em vez de `ls`, `bat` em vez de `cat`, `zoxide` em vez de `cd`). *(src/shell/mod.rs:30-51; zoxide: src/shell/mod.rs:61-62 — apenas alias, não builtin)*

### 8. Compatibilidade e Protocolos Modernos
- [x] **Suporte a Windows Nativo:** Compilação e execução nativas para Windows (CMD/PowerShell API), não apenas via WSL. *(`#[cfg(windows)]` em pipeline.rs, utils/mod.rs, shell/mod.rs, executor/mod.rs — AUSENTE em main.rs e builtin/mod.rs)*
- [x] **Integração com Protocolos Avançados de Terminal:**
  - [x] **Kitty Graphics Protocol:** Renderizar imagens diretamente no shell. *(src/utils/mod.rs:619-637; builtins: catimg)*
  - [x] **Hyperlinks OSC 8:** Tornar links e caminhos clicáveis nativamente no terminal. *(src/utils/mod.rs:643-648)*
  - [x] **Shell Integration OSC 133:** Enviar sequências de escape semânticas para informar ao terminal inteligente onde começam e terminam comandos e prompts. *(src/utils/mod.rs:663-669; src/executor/mod.rs:25,28)*

### 9. Site Web + Documentação

- [ ] **Landing page (GitHub Pages):** Site institucional do jesh em `jesh.sh` ou `jesh-shell.github.io` com:
  - [ ] Hero section: "Um shell moderno escrito em Rust" com screenshot/terminal animado.
  - [ ] Quickstart: `curl -fsSL https://jesh.sh/install.sh | sh` e `cargo install jesh`.
  - [ ] Badges: GitHub Stars, Rust version, CI status, License, Downloads.
- [ ] **Documentação completa (site/docs/):**
  - [ ] **Getting Started:** Instalação (Linux, macOS, Windows/WSL), primeira execução, configuração mínima.
  - [ ] **Configuração:** Referência completa do `.jeshrc` (INIT_INFO, HOT_RELOAD, SHOW_TIMING, JSH_TAB_MODE), variáveis de ambiente, `config.toml`.
  - [ ] **Comandos Builtin:** Lista completa com descrição, sintaxe e exemplos (`cd`, `export`, `source`, `history`, `pushd`/`popd`/`dirs`, `set`, `shopt`, `complete`, `alias`, `eval`, `exec`, `command`, `read`, `printf`, `true`/`false`/`:`, `test`/`[`, `declare`/`typeset`, `local`, `readonly`, `getopts`, `disown`).
  - [ ] **Scripting:** Variáveis, expansões (`$()`, `${}`, aritmética `$(()))`, parâmetros posicionais, funções com `local`, `declare`/`typeset` com `-i`/`-a`/`-A`/`-r`/`-x`, `set -e`/`-u`/`-x`/`-o pipefail`, `getopts`, `readonly`.
  - [ ] **Parser:** ANSI-C quoting `$'...'`, continuação de linha `\`, processo `<( )`/`>( )`, expansão de histórico `!!`/`!$`.
  - [ ] **Globbing:** `*`, `?`, `[...]`, extglob, `nullglob`/`failglob`/`dotglob`/`nocaseglob`.
  - [ ] **Autocompletar:** TUI menu selection, busca fuzzy, `complete -W`/`-F`, descrições, dicas de flags.
  - [ ] **Prompt:** Personalização, cores, variáveis, prompt direito (RPROMPT), prompt transiente.
  - [ ] **Jobs & Processos:** Background (`&`), foreground (`fg`), `jobs`, `disown`, `Ctrl+Z`, Process Groups.
  - [ ] **Histórico:** Navegação, busca reversa (`Ctrl+R`), `history pin`, directory-aware history, variáveis de controle.
  - [ ] **Diferenças vs Bash:** Tabela comparativa de sintaxes suportadas e não suportadas, bash fallback.
- [ ] **Exemplos e Tutoriais:**
  - [ ] Guia de migração de `.bashrc` para `.jeshrc`.
  - [ ] Scripting examples: laços, condicionais, pipes, redirecionamentos.
  - [ ] Configuração de completions customizadas (`complete -W`/`-F`).
  - [ ] Integração com NVM, Rust, Deno, Python venvs.
- [ ] **Infraestrutura do site:**
  - [ ] Gerador de site estático (Zola, Hugo, ou Jekyll).
  - [ ] Domínio próprio (`jesh.sh` ou similar).
  - [ ] HTTPS automático (Cloudflare Pages, GitHub Pages + domínio customizado).
  - [ ] CI/CD: Deploy automático ao fazer push na `main`.
  - [ ] SEO: meta tags, sitemap.xml, Open Graph para preview em redes sociais.
- [ ] **Manutenção da documentação:**
  - [ ] `docs/` versionada junto com o código no repositório.
  - [ ] Script para extrair help texts dos builtins e gerar markdown automaticamente.
  - [ ] Página `CHANGELOG.md` no site com release notes.
  - [ ] Guia de contribuição (`CONTRIBUTING.md`) com instruções de build, testes e estilo de código.
