# Análise de Gaps: O que falta no Checklist do JSH?

Esta análise identifica recursos, comportamentos e padrões ausentes no checklist original do **JSH**, organizados por categoria. O objetivo é aproximar o `jesh` de shells maduros como Bash, Zsh e Fish, além de incorporar diferenciais modernos (estilo Nushell e integrações contemporâneas).

---

## 1. Básico (Essencial)
- [x] **Pilha de Diretórios (Directory Stack):** Comandos internos `pushd`, `popd` e `dirs` para navegação rápida entre diretórios.
- [ ] **Comandos Builtin de Entrada/Saída:**
  - [x] `read` (essencial para capturar inputs de usuários em scripts).
  - [ ] `printf` (alternativa muito mais robusta e padronizada ao `echo` para formatação).
- [ ] **Builtins de Controle e Execução:**
  - [ ] `eval` (executar strings como comandos do shell).
  - [ ] `command` (chamar comandos ignorando aliases ou funções definidas).
  - [ ] `exec` (substituir o processo do shell pelo comando executado, sem criar subprocesso).
  - [ ] `true`, `false` e `:` (comandos nulos padrão que retornam 0 ou 1).
- [ ] **Parâmetros Posicionais (Positional Parameters):** Suporte nativo a `$0`, `$1`, `$2` ... `$9`, `$#` (contagem de argumentos), `$@` e `$*` (todos os argumentos).

---

## 2. Parser
- [ ] **Substituição de Processos (Process Substitution):** Sintaxe `<(comando)` e `>(comando)` para passar saídas/entradas de sub-processos como se fossem arquivos.
- [x] **Expansão Aritmética (Arithmetic Expansion):** Sintaxe `$((expressão))` para cálculos matemáticos inteiros diretamente no shell sem depender de `expr` ou `bc`.
- [x] **ANSI-C Quoting:** Sintaxe `$'string'` para interpretar sequências de escape diretamente (ex: `$'linha1\nlinha2'`).
- [ ] **Continuação de Linha:** Interpretação da barra invertida (`\`) ao final de uma linha física para continuar o mesmo comando na linha seguinte.
- [ ] **Expansão de Histórico (Bash-style):** Suporte nativo para `!!`, `!$`, `!*`, `!-n` e `!prefixo` no parser.

---

## 3. Pipes e Redirecionamentos
- [ ] **Redirecionamento em Modo Append para stdout + stderr:** Suporte ao operador `&>>` (redirecionar e anexar ambos os fluxos).
- [ ] **Status do Pipeline (PIPESTATUS / pipefail):** Um array contendo os códigos de retorno de todos os comandos do pipeline (ex: em `a | b`, capturar se `a` falhou, mesmo se `b` retornar 0).
- [ ] **Fechamento e Cópia de Descritores:**
  - [ ] Fechar descritores com `n>&-` ou `n<&-`.
  - [ ] Redirecionar descritores dinamicamente via variáveis, ex: `exec {FD}>arquivo` ou `>$FD`.
  - [ ] Swap de descritores de arquivo, ex: `3>&1 1>&2 2>&3`.

---

## 4. Operadores
- [ ] **Quebras de Linha como Delimitadores:** Tratar a quebra de linha física exatamente como um separador de comandos (equivalente a `;`).
- [ ] **Operadores de Teste Avançados:** Suporte integrado a condicionais estruturadas `[[ ... ]]` e ao builtin de teste `[ ... ]`.
- [ ] **Operador de Negação Lógica:** Suporte a `!` antes de comandos (ex: `if ! grep -q ...`).

---

## 5. Variáveis e Expansão de Parâmetros
- [ ] **Variáveis Especiais do Shell:** `$IFS` (delimitador de campos), `$PPID` (PID pai), `$UID`, `$GROUPS`, `$PWD`, `$OLDPWD`.
- [ ] **Tipagem e Atributos de Variáveis (`declare` / `typeset`):**
  - [ ] `-i` (inteiro).
  - [ ] `-a` (array indexado).
  - [ ] `-A` (array associativo).
  - [ ] `-r` (somente leitura).
  - [ ] `-x` (exportar).
- [ ] **Expansão e Manipulação Avançada de Parâmetros:**
  - [ ] `${VAR:-default}` e `${VAR:=default}` (valores padrão).
  - [ ] `${VAR:?error}` (lançar erro se nula/não definida).
  - [ ] `${VAR:+alternative}` (usar alternativa se definida).
  - [ ] `${VAR#pattern}`, `${VAR##pattern}`, `${VAR%pattern}`, `${VAR%%pattern}` (remover prefixos/sufixos curtos ou longos).
  - [ ] `${VAR/pattern/replacement}` e `${VAR//pattern/replacement}` (substituição simples e global).
  - [ ] `${!VAR}` (indireção de variáveis, ex: ler variável cujo nome está guardado em outra).

---

## 6. Globbing (Expansão de Caminhos)
- [ ] **Extended Globbing (extglob):** Suporte a padrões complexos como `@(...)`, `*(...)`, `+(...)`, `?(...)` e `!(...)`.
- [ ] **Flags de Ajuste de Globbing:**
  - [ ] `nullglob` (não dar erro ou manter a string caso nenhum arquivo dê match).
  - [ ] `failglob` (gerar erro se nenhum match for encontrado).
  - [ ] `dotglob` (fazer com que o `*` capture arquivos ocultos iniciados por ponto).
  - [ ] `nocaseglob` (globbing insensível a maiúsculas e minúsculas).
- [ ] **Qualificadores de Globbing (Zsh-style):** Filtrar matches por tipo, ex: apenas diretórios (`*(/)`) ou apenas links simbólicos.

---

## 7. Histórico
- [ ] **Pinagem de Comandos:** Comando `history pin` / `history unpin` para favorecer comandos úteis nas sugestões do prompt.
- [ ] **Variáveis de Controle de Histórico:** Suporte a `$HISTSIZE`, `$HISTFILESIZE`, `$HISTIGNORE` e `$HISTCONTROL`.
- [ ] **Histórico Ciente do Diretório (Directory-Aware History):** Busca prioritária por comandos executados no diretório de trabalho atual.
- [ ] **Metadados de Sessão:** Rastrear qual terminal (TTY/Session ID) executou determinado comando para filtragens locais por aba.

---

## 8. Linha de Comando e Editor de Entrada
- [ ] **Configuração de Atalhos (Custom Keybindings):** Permitir mapear teclas personalizadas no arquivo `config.toml`.
- [ ] **Yank-ring (Histórico de Recortes):** Suporte a múltiplos níveis de colar no estilo Emacs (`Alt+Y` após `Ctrl+Y`).
- [ ] **Redesenho de Tela em Redimensionamento:** Manipulação inteligente de `SIGWINCH` para re-renderizar a linha de comando sem quebrar a tela quando o terminal for redimensionado.
- [ ] **Medição de Largura de Caracteres Unicode (East Asian Width):** Correção do cálculo do cursor para emojis e caracteres asiáticos (double-width).
- [ ] **Mudança Visual do Cursor por Modo:** Cursor como bloco sólido `█` no modo comando do Vi, e como barra vertical `|` no modo inserção.

---

## 9. Autocompletar (Tab Completion)
- [ ] **TUI Menu Selection:** Navegação visual pelas opções de completamento usando setas do teclado (como o `menu select` do Zsh).
- [ ] **Busca Fuzzy no Completamento:** Autocompletar substrings fragmentadas (ex: `/u/l/b` expandir para `/usr/local/bin`).
- [ ] **Descrições de Comandos e Flags:** Mostrar notas explicativas ao lado de cada opção de completamento sugerida.
- [ ] **API de Autocompletar Programável:** Permitir que scripts e plugins definam regras customizadas de completamento (ex: `complete -F` do Bash).

---

## 10. Jobs
- [ ] **Comando `disown`:** Desvincular um processo em background do shell pai para mantê-lo rodando após a saída do shell.
- [ ] **Notificação Assíncrona de Mudança de Estado:** Notificar o usuário imediatamente quando um processo em background finaliza ou para, sem aguardar o próximo comando.
- [ ] **Isolamento de Process Groups (PGID):** Proteção do shell contra sinais como `Ctrl+C` (SIGINT) emitidos para processos foreground.

---

## 11. Scripts e Execução
- [ ] **Diretivas de Depuração e Erro:**
  - [ ] `set -e` (parar script no primeiro erro).
  - [ ] `set -u` (parar se houver variável não declarada).
  - [ ] `set -o pipefail`.
  - [ ] `set -x` (imprimir cada linha executada para depuração).
- [ ] **Escopo de Função e a palavra-chave `local`:** Variáveis declaradas dentro de funções não vazam para o escopo global.
- [ ] **Processamento de Opções (`getopts`):** Parser nativo e simples de argumentos de linha de comando para scripts.

---

## 12. Prompt
- [ ] **Prompt Direito (RPROMPT):** Informações adicionais exibidas na borda direita do terminal, que desaparecem se a linha de comando crescer muito.
- [ ] **Renderização Assíncrona do Prompt:** Executar tarefas pesadas (verificação de repositório Git remoto, status do Kubernetes, etc.) em threads de background para que o prompt nunca trave ao digitar Enter.
- [ ] **Prompt Transiente (Transient Prompt):** Diminuir e limpar o prompt das linhas anteriores já executadas para manter a tela limpa e focada no histórico.

---

## 13. Integrações e Recursos Modernos
- [ ] **Motor de Pipeline Semântico (Nushell-style):** Capacidade de tratar saídas de comandos como tabelas de dados estruturados e filtrá-las sem precisar de regex pesadas.
- [ ] **Smart Paste (Colar Inteligente):** Escapar caracteres perigosos (como `?` ou `&`) ao colar URLs ou textos contendo delimitadores do shell.
- [ ] **Substituições Automáticas por CLI Modernas:** Configurações automáticas para usar ferramentas modernas de CLI quando disponíveis (ex: `eza`/`exa` em vez de `ls`, `bat` em vez de `cat`, `zoxide` em vez de `cd`).

---

## 14. Compatibilidade e Protocolos Modernos
- [ ] **Suporte a Windows Nativo:** Compilação e execução nativas para Windows (CMD/PowerShell API), não apenas via WSL.
- [ ] **Integração com Protocolos Avançados de Terminal:**
  - [ ] **Kitty Graphics Protocol:** Renderizar imagens diretamente no shell.
  - [ ] **Hyperlinks OSC 8:** Tornar links e caminhos clicáveis nativamente no terminal.
  - [ ] **Shell Integration OSC 133:** Enviar sequências de escape semânticas para informar ao terminal inteligente onde começam e terminam comandos e prompts.
