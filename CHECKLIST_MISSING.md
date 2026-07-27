# Análise de Gaps: O que falta no Checklist do JSH?

Esta análise identifica recursos, comportamentos e padrões ausentes no checklist original do **JSH**, organizados por categoria. O objetivo é aproximar o `jesh` de shells maduros como Bash, Zsh e Fish, além de incorporar diferenciais modernos (estilo Nushell e integrações contemporâneas).

---

## 1. Básico (Essencial)
- [x] **Pilha de Diretórios (Directory Stack):** Comandos internos `pushd`, `popd` e `dirs` para navegação rápida entre diretórios.
- [x] **Comandos Builtin de Entrada/Saída:**
  - [x] `read` (essencial para capturar inputs de usuários em scripts).
  - [x] `printf` (alternativa muito mais robusta e padronizada ao `echo` para formatação).
- [x] **Builtins de Controle e Execução:**
  - [x] `eval` (executar strings como comandos do shell).
  - [x] `command` (chamar comandos ignorando aliases ou funções definidas).
  - [x] `exec` (substituir o processo do shell pelo comando executado, sem criar subprocesso).
  - [x] `true`, `false` e `:` (comandos nulos padrão que retornam 0 ou 1).
- [x] **Parâmetros Posicionais (Positional Parameters):** Suporte nativo a `$0`, `$1`, `$2` ... `$9`, `$#` (contagem de argumentos), `$@` e `$*` (todos os argumentos).

---

## 2. Parser
- [x] **Substituição de Processos (Process Substitution):** Sintaxe `<(comando)` e `>(comando)` para passar saídas/entradas de sub-processos como se fossem arquivos.
- [x] **Expansão Aritmética (Arithmetic Expansion):** Sintaxe `$((expressão))` para cálculos matemáticos inteiros diretamente no shell sem depender de `expr` ou `bc`.
- [x] **ANSI-C Quoting:** Sintaxe `$'string'` para interpretar sequências de escape diretamente (ex: `$'linha1\nlinha2'`).
- [x] **Continuação de Linha:** Interpretação da barra invertida (`\`) ao final de uma linha física para continuar o mesmo comando na linha seguinte.
- [x] **Expansão de Histórico (Bash-style):** Suporte nativo para `!!`, `!$`, `!*`, `!-n` e `!prefixo` no parser.

---

## 3. Pipes e Redirecionamentos
- [x] **Redirecionamento em Modo Append para stdout + stderr:** Suporte ao operador `&>>` (redirecionar e anexar ambos os fluxos).
- [x] **Status do Pipeline (PIPESTATUS / pipefail):** Um array contendo os códigos de retorno de todos os comandos do pipeline (ex: em `a | b`, capturar se `a` falhou, mesmo se `b` retornar 0).
- [x] **Fechamento e Cópia de Descritores:**
  - [x] Fechar descritores com `n>&-` ou `n<&-`.
  - [x] Redirecionar descritores dinamicamente via variáveis, ex: `exec {FD}>arquivo` ou `>$FD`.
  - [x] Swap de descritores de arquivo, ex: `3>&1 1>&2 2>&3`.

---

## 4. Operadores
- [x] **Quebras de Linha como Delimitadores:** Tratar a quebra de linha física exatamente como um separador de comandos (equivalente a `;`).
- [x] **Operadores de Teste Avançados:** Suporte integrado a condicionais estruturadas `[[ ... ]]` e ao builtin de teste `[ ... ]`.
- [x] **Operador de Negação Lógica:** Suporte a `!` antes de comandos (ex: `if ! grep -q ...`).

---

## 5. Variáveis e Expansão de Parâmetros
- [x] **Variáveis Especiais do Shell:** `$IFS` (delimitador de campos), `$PPID` (PID pai), `$UID`, `$GROUPS`, `$PWD`, `$OLDPWD`, `$PIPESTATUS`, `$@`, `$*`, `$#`, `$1`..`$9`. *(Implementado em shell/mod.rs:get_var)*
- [x] **Tipagem e Atributos de Variáveis (`declare` / `typeset` / `local` / `readonly` / `getopts`):**
  - [x] `-i` (inteiro). *Implementado em builtin/mod.rs*
  - [x] `-a` (array indexado - atributo). *Implementado*
  - [x] `-A` (array associativo - atributo). *Implementado*
  - [x] `-r` (somente leitura). *Implementado em builtin/mod.rs*
  - [x] `-x` (exportar). *Implementado em builtin/mod.rs*
  - [x] Builtin `local` para escopo de função. *Implementado em builtin/mod.rs e shell/mod.rs:push/pop_local_scope*
  - [x] Builtin `readonly` para variáveis somente leitura. *Implementado em builtin/mod.rs*
  - [x] Builtin `getopts` para processamento de opções de scripts. *Implementado em builtin/mod.rs*
- [x] **Expansão e Manipulação Avançada de Parâmetros:**
  - [x] `${VAR:-default}` — *Implementado*
  - [x] `${VAR:=default}` (valores padrão com atribuição) — *Implementado*
  - [x] `${VAR:?error}` (lançar erro se nula/não definida) — *Implementado*
  - [x] `${VAR:+alternative}` — *Implementado*
  - [x] `${VAR#pattern}`, `${VAR##pattern}`, `${VAR%pattern}`, `${VAR%%pattern}` (remover prefixos/sufixos curtos ou longos) — *Implementado*
  - [x] `${VAR/pattern/replacement}` e `${VAR//pattern/replacement}` (substituição simples e global) — *Implementado*
  - [x] `${!VAR}` (indireção de variáveis) — *Implementado*

---

## 6. Globbing (Expansão de Caminhos)
- [x] **Extended Globbing (extglob):** Suporte a padrões complexos como `@(...)`, `*(...)`, `+(...)`, `?(...)` e `!(...)`.
- [x] **Flags de Ajuste de Globbing:**
  - [x] `nullglob` (não dar erro ou manter a string caso nenhum arquivo dê match).
  - [x] `failglob` (gerar erro se nenhum match for encontrado).
  - [x] `dotglob` (fazer com que o `*` capture arquivos ocultos iniciados por ponto).
  - [x] `nocaseglob` (globbing insensível a maiúsculas e minúsculas).
- [ ] **Qualificadores de Globbing (Zsh-style):** Filtrar matches por tipo, ex: apenas diretórios (`*(/)`) ou apenas links simbólicos.

---

## 7. Histórico
- [x] **Pinagem de Comandos:** Comando `history pin` / `history unpin` para favorecer comandos úteis nas sugestões do prompt.
- [x] **Variáveis de Controle de Histórico:** Suporte a `$HISTSIZE`, `$HISTFILESIZE`, `$HISTIGNORE` e `$HISTCONTROL`.
- [x] **Histórico Ciente do Diretório (Directory-Aware History):** Busca prioritária por comandos executados no diretório de trabalho atual.
- [x] **Metadados de Sessão:** Rastrear qual terminal (TTY/Session ID) executou determinado comando para filtragens locais por aba.

---

## 8. Linha de Comando e Editor de Entrada
- [x] **Configuração de Atalhos (Custom Keybindings):** Permitir mapear teclas personalizadas no arquivo `config.toml`.
- [x] **Yank-ring (Histórico de Recortes):** Suporte a múltiplos níveis de colar no estilo Emacs (`Alt+Y` após `Ctrl+Y`).
- [x] **Redesenho de Tela em Redimensionamento:** Manipulação inteligente de `SIGWINCH` para re-renderizar a linha de comando sem quebrar a tela quando o terminal for redimensionado.
- [x] **Medição de Largura de Caracteres Unicode (East Asian Width):** Correção do cálculo do cursor para emojis e caracteres asiáticos (double-width).
- [x] **Mudança Visual do Cursor por Modo:** Cursor como bloco sólido `█` no modo comando do Vi, e como barra vertical `|` no modo inserção.

---

## 9. Autocompletar (Tab Completion)
- [x] **TUI Menu Selection:** Navegação visual pelas opções de completamento usando setas do teclado (como o `menu select` do Zsh).
- [x] **Busca Fuzzy no Completamento:** Autocompletar substrings fragmentadas (ex: `/u/l/b` expandir para `/usr/local/bin`).
- [x] **Descrições de Comandos e Flags:** Mostrar notas explicativas ao lado de cada opção de completamento sugerida.
- [x] **API de Autocompletar Programável:** Permitir que scripts e plugins definam regras customizadas de completamento (ex: `complete -F` do Bash).

---

## 10. Jobs
- [x] **Comando `disown`:** Desvincular um processo em background do shell pai para mantê-lo rodando após a saída do shell.
- [x] **Notificação Assíncrona de Mudança de Estado:** Notificar o usuário imediatamente quando um processo em background finaliza ou para, sem aguardar o próximo comando.
- [x] **Isolamento de Process Groups (PGID):** Proteção do shell contra sinais como `Ctrl+C` (SIGINT) emitidos para processos foreground.

---

## 11. Scripts e Execução
- [x] **Diretivas de Depuração e Erro:**
  - [x] `set -e` (parar script no primeiro erro).
  - [x] `set -u` (parar se houver variável não declarada).
  - [x] `set -o pipefail`.
  - [x] `set -x` (imprimir cada linha executada para depuração).
- [x] **Escopo de Função e a palavra-chave `local`:** Variáveis declaradas dentro de funções não vazam para o escopo global.
- [x] **Processamento de Opções (`getopts`):** Parser nativo e simples de argumentos de linha de comando para scripts.

---

## 12. Prompt
- [ ] **Prompt Direito (RPROMPT):** Informações adicionais exibidas na borda direita do terminal, que desaparecem se a linha de comando crescer muito.
- [ ] **Renderização Assíncrona do Prompt:** Executar tarefas pesadas (verificação de repositório Git remoto, status do Kubernetes, etc.) em threads de background para que o prompt nunca trave ao digitar Enter.
- [ ] **Prompt Transiente (Transient Prompt):** Diminuir e limpar o prompt das linhas anteriores já executadas para manter a tela limpa e focada no histórico.

---

## 13. Integrações e Recursos Modernos
- [ ] **Motor de Pipeline Semântico (Nushell-style):** Capacidade de tratar saídas de comandos como tabelas de dados estruturados e filtrá-las sem precisar de regex pesadas.
- [ ] **Smart Paste (Colar Inteligente):** Escapar caracteres perigosos (como `?` ou `&`) ao colar URLs ou textos contendo delimitadores do shell.
- [x] **Substituições Automáticas por CLI Modernas:** Configurações automáticas para usar ferramentas modernas de CLI quando disponíveis (ex: `eza`/`exa` em vez de `ls`, `bat` em vez de `cat`, `zoxide` em vez de `cd`).

---

## 14. Compatibilidade e Protocolos Modernos
- [ ] **Suporte a Windows Nativo:** Compilação e execução nativas para Windows (CMD/PowerShell API), não apenas via WSL.
- [ ] **Integração com Protocolos Avançados de Terminal:**
  - [ ] **Kitty Graphics Protocol:** Renderizar imagens diretamente no shell.
  - [ ] **Hyperlinks OSC 8:** Tornar links e caminhos clicáveis nativamente no terminal.
  - [ ] **Shell Integration OSC 133:** Enviar sequências de escape semânticas para informar ao terminal inteligente onde começam e terminam comandos e prompts.

---

# 15. Site Web + Documentação

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
