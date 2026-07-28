mod builtin;
mod completion;
mod executor;
mod parser;
mod semantic;
mod shell;
mod utils;

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::io::{BufRead, IsTerminal, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
use rustyline::{Cmd, ConditionalEventHandler, Config, EditMode, Editor, Event, EventContext, EventHandler, KeyCode, KeyEvent, Modifiers, RepeatCount, Movement};

use crate::builtin::run_jeofetch;
use crate::completion::JshHelper;
use crate::parser::lexer::RedirectTarget;
use crate::shell::ShellState;

static SIGINT_FLAG: AtomicBool = AtomicBool::new(false);
static SIGWINCH_FLAG: AtomicBool = AtomicBool::new(false);

extern "C" fn sigint_handler(_sig: i32) {
    SIGINT_FLAG.store(true, Ordering::SeqCst);
}

extern "C" fn sigwinch_handler(_sig: i32) {
    SIGWINCH_FLAG.store(true, Ordering::SeqCst);
}

/// Expands `!!`, `!n`, and `!prefix` history references in a raw input
/// line, using the history manager as the source of past commands.
/// Runs before tokenizing, exactly like bash's history expansion.
fn expand_history_refs(line: &str, history_mgr: &crate::shell::history::HistoryManager) -> String {
    if !line.contains('!') {
        return line.to_string();
    }

    let state = history_mgr.state.lock().unwrap();
    let entries = &state.entries;
    if entries.is_empty() {
        return line.to_string();
    }

    let mut out = String::new();
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '!' {
            out.push(c);
            continue;
        }
        match chars.peek() {
            Some('!') => {
                chars.next();
                if let Some(entry) = entries.last() {
                    out.push_str(&entry.command);
                } else {
                    out.push_str("!!");
                }
            }
            Some('$') => {
                chars.next();
                if let Some(entry) = entries.last() {
                    if let Some(last_arg) = entry.command.split_whitespace().last() {
                        out.push_str(last_arg);
                    } else {
                        out.push_str("!$");
                    }
                } else {
                    out.push_str("!$");
                }
            }
            Some('*') => {
                chars.next();
                if let Some(entry) = entries.last() {
                    let args: Vec<&str> = entry.command.split_whitespace().collect();
                    if args.len() > 1 {
                        out.push_str(&args[1..].join(" "));
                    }
                }
            }
            Some('-') => {
                chars.next();
                let mut num = String::new();
                while let Some(&d) = chars.peek() {
                    if d.is_ascii_digit() {
                        num.push(d);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if num.is_empty() {
                    out.push_str("!-");
                } else if let Ok(n) = num.parse::<usize>() {
                    if n == 0 || n > entries.len() {
                        out.push('!');
                        out.push('-');
                        out.push_str(&num);
                    } else {
                        let idx = entries.len() - n;
                        out.push_str(&entries[idx].command);
                    }
                } else {
                    out.push('!');
                    out.push('-');
                    out.push_str(&num);
                }
            }
            Some('?') => {
                chars.next();
                let mut pattern = String::new();
                while let Some(&pc) = chars.peek() {
                    if pc.is_alphanumeric() || pc == '_' || pc == '-' {
                        pattern.push(pc);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let found = entries.iter().rev().find(|e| e.command.contains(&pattern));
                match found {
                    Some(entry) => out.push_str(&entry.command),
                    None => {
                        out.push_str("!?");
                        out.push_str(&pattern);
                    }
                }
            }
            Some(d) if d.is_ascii_digit() => {
                let mut num = String::new();
                while let Some(&d) = chars.peek() {
                    if d.is_ascii_digit() {
                        num.push(d);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let idx = num.parse::<usize>().unwrap_or(0);
                if idx >= 1 && idx <= entries.len() {
                    out.push_str(&entries[idx - 1].command);
                } else {
                    out.push('!');
                    out.push_str(&num);
                }
            }
            Some(c) if c.is_alphabetic() => {
                let mut prefix = String::new();
                while let Some(&pc) = chars.peek() {
                    if pc.is_alphanumeric() || pc == '_' || pc == '-' {
                        prefix.push(pc);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let found = entries.iter().rev().find(|e| e.command.starts_with(&prefix));
                match found {
                    Some(entry) => out.push_str(&entry.command),
                    None => {
                        out.push('!');
                        out.push_str(&prefix);
                    }
                }
            }
            _ => out.push('!'),
        }
    }
    out
}

/// Reads heredoc bodies for every heredoc redirect found in `list`, prompting
/// interactively via `read_more` (used for the REPL) or consuming lines from
/// `lines` (used for non-interactive script execution). Returns one body per
/// `AndOrList` item (parallel to `list.items`), `None` where there's no heredoc.
fn collect_heredocs(
    list: &parser::CommandList,
    mut read_more: impl FnMut(&str) -> Option<String>,
) -> Vec<Option<String>> {
    let mut bodies = Vec::with_capacity(list.items.len());
    for (andor, _op) in &list.items {
        let mut heredoc_info: Option<(String, bool)> = None;
        for cmd in &andor.pipeline.commands {
            for r in &cmd.redirects {
                if let RedirectTarget::Heredoc(d, strip) = &r.target {
                    heredoc_info = Some((d.clone(), *strip));
                }
            }
        }
        if let Some((delim, strip_tabs)) = heredoc_info {
            let mut body = String::new();
            loop {
                match read_more("> ") {
                    Some(l) if l.trim() == delim => break,
                    Some(l) => {
                        let line = if strip_tabs {
                            l.trim_start_matches('\t')
                        } else {
                            &l
                        };
                        body.push_str(line);
                        body.push('\n');
                    }
                    None => break,
                }
            }
            bodies.push(Some(body));
        } else {
            bodies.push(None);
        }
    }
    bodies
}

/// Parses and executes one raw input line against `state`, using
/// `read_more` to pull additional lines for heredoc bodies when needed.
pub fn run_line_with(state: &mut ShellState, line: &str, mut read_more: impl FnMut(&str) -> Option<String>) {
    let line = line.trim();
    if line.is_empty() {
        return;
    }

    let tokens = crate::parser::lexer::tokenize(line);
    let list = crate::parser::parser::parse(tokens);
    if list.items.is_empty() {
        return;
    }

    let heredoc_bodies = collect_heredocs(&list, &mut read_more);

    crate::executor::run_command_list(state, &list, &heredoc_bodies);
}

/// Ensures `$PWD` in the environment matches the actual working directory at startup.
/// Process working directory (`current_dir()`) is authoritative because terminal emulators
/// and file managers perform `chdir()` before spawning the shell process.
/// If `$PWD` in the inherited environment is valid and canonicalizes to the same directory
/// as `current_dir()`, `$PWD` is left unchanged. Otherwise, `$PWD` is updated to match CWD.
fn sync_pwd() {
    let cwd = match std::env::current_dir() {
        Ok(c) => c,
        Err(_) => return,
    };

    if let Ok(pwd) = std::env::var("PWD") {
        let pwd_path = std::path::Path::new(&pwd);
        if pwd_path.is_dir() {
            if let (Ok(pwd_canon), Ok(cwd_canon)) = (pwd_path.canonicalize(), cwd.canonicalize()) {
                if pwd_canon == cwd_canon {
                    return;
                }
            }
        }
    }

    unsafe {
        std::env::set_var("PWD", &cwd);
    }
}

thread_local! {
    static KILL_RING: RefCell<VecDeque<String>> = RefCell::new(VecDeque::new());
    static KILL_RING_POS: RefCell<usize> = RefCell::new(0);
}

fn kill_ring_push(text: &str) {
    KILL_RING.with(|ring| {
        let mut ring = ring.borrow_mut();
        if ring.len() >= 32 {
            ring.pop_back();
        }
        ring.push_front(text.to_string());
    });
    KILL_RING_POS.with(|pos| {
        *pos.borrow_mut() = 0;
    });
}

struct KillLineHandler;
impl ConditionalEventHandler for KillLineHandler {
    fn handle(&self, _evt: &Event, _n: RepeatCount, _positive: bool, ctx: &EventContext) -> Option<Cmd> {
        let line = ctx.line();
        let pos = ctx.pos();
        if pos < line.len() {
            let killed = &line[pos..];
            kill_ring_push(killed);
            Some(Cmd::Replace(Movement::WholeBuffer, Some(line[..pos].to_string())))
        } else {
            None
        }
    }
}

struct KillWordBackHandler;
impl ConditionalEventHandler for KillWordBackHandler {
    fn handle(&self, _evt: &Event, _n: RepeatCount, _positive: bool, ctx: &EventContext) -> Option<Cmd> {
        let line = ctx.line();
        let pos = ctx.pos();
        if pos > 0 {
            let before = &line[..pos];
            let trimmed = before.trim_end_matches(|c: char| !c.is_alphanumeric() && c != '_');
            let word_start = trimmed.rfind(|c: char| !c.is_alphanumeric() && c != '_')
                .map(|i| i + 1)
                .unwrap_or(0);
            if word_start < pos {
                let killed = &line[word_start..pos];
                kill_ring_push(killed);
                let new_line = format!("{}{}", &line[..word_start], &line[pos..]);
                Some(Cmd::Replace(Movement::WholeBuffer, Some(new_line)))
            } else {
                None
            }
        } else {
            None
        }
    }
}

struct YankHandler;
impl ConditionalEventHandler for YankHandler {
    fn handle(&self, _evt: &Event, _n: RepeatCount, _positive: bool, ctx: &EventContext) -> Option<Cmd> {
        KILL_RING.with(|ring| {
            let ring = ring.borrow();
            if let Some(entry) = ring.front() {
                let line = ctx.line();
                let pos = ctx.pos();
                let new_line = format!("{}{}{}", &line[..pos], entry, &line[pos..]);
                Some(Cmd::Replace(Movement::WholeBuffer, Some(new_line)))
            } else {
                None
            }
        })
    }
}

struct YankRingHandler;
impl ConditionalEventHandler for YankRingHandler {
    fn handle(&self, _evt: &Event, _n: RepeatCount, _positive: bool, ctx: &EventContext) -> Option<Cmd> {
        KILL_RING.with(|ring| {
            let ring = ring.borrow();
            if ring.is_empty() {
                return None;
            }
            KILL_RING_POS.with(|pos_cell| {
                let mut pos = pos_cell.borrow_mut();
                *pos = (*pos + 1) % ring.len();
                if let Some(entry) = ring.get(*pos) {
                    Some(Cmd::Replace(Movement::WholeBuffer, Some(entry.clone())))
                } else {
                    None
                }
            })
        })
    }
}

fn run_interactive(mut state: ShellState) {
    state.is_interactive = true;

    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGINT, sigint_handler as *const () as usize);
        libc::signal(libc::SIGWINCH, sigwinch_handler as *const () as usize);

        libc::signal(libc::SIGTTOU, libc::SIG_IGN);
        libc::signal(libc::SIGTTIN, libc::SIG_IGN);
        libc::signal(libc::SIGTSTP, libc::SIG_IGN);
        libc::signal(libc::SIGQUIT, libc::SIG_IGN);

        let pid = libc::getpid();
        let _ = libc::setpgid(pid, pid);
        let _ = libc::tcsetpgrp(libc::STDIN_FILENO, pid);
    }

    #[cfg(unix)]
    crate::utils::save_shell_termios();

    if std::io::stdout().is_terminal() {
        crate::utils::set_cursor_block();
    }

        let tab_mode = {
        let vars = state.shell_vars.lock().unwrap();
        match vars.get("JSH_TAB_MODE").map(|s| s.as_str()) {
            Some("circular") | Some("Circular") | Some("CIRCULAR") => crate::completion::TabMode::Circular,
            Some("hybrid") | Some("Hybrid") | Some("HYBRID") => crate::completion::TabMode::Hybrid,
            _ => crate::completion::TabMode::Interactive,
        }
    };

    // Hybrid uses CompletionType::List (the default) so rustyline shows its
    // own list alongside our enhanced interactive menu.  Only Circular mode
    // switches to CompletionType::Circular for inline cycling.
    let completion_type = if tab_mode == crate::completion::TabMode::Circular {
        rustyline::CompletionType::Circular
    } else {
        rustyline::CompletionType::List
    };

    let config = Config::builder()
        .bracketed_paste(true)
        .completion_type(completion_type)
        .edit_mode(EditMode::Vi)
        .build();

    // Enable shell integration for modern terminals
    crate::utils::emit_osc7();

    let mut rl = Editor::<JshHelper, DefaultHistory>::with_config(config)
        .expect("Erro ao inicializar editor de linha");

    struct CompleteHintHandler;
    impl ConditionalEventHandler for CompleteHintHandler {
        fn handle(&self, _evt: &Event, _n: RepeatCount, _positive: bool, ctx: &EventContext) -> Option<Cmd> {
            if ctx.pos() == ctx.line().len() {
                Some(Cmd::CompleteHint)
            } else {
                None
            }
        }
    }

    struct NavigationState {
        original_input: String,
        entries: Vec<String>,
        current_index: usize,
    }

    thread_local! {
        static NAVIGATION: std::cell::RefCell<Option<NavigationState>> = std::cell::RefCell::new(None);
    }

    struct UpArrowHandler {
        history_mgr: Arc<crate::shell::history::HistoryManager>,
        shell_vars: Arc<Mutex<HashMap<String, String>>>,
    }
    impl ConditionalEventHandler for UpArrowHandler {
        fn handle(&self, _evt: &Event, _n: RepeatCount, _positive: bool, ctx: &EventContext) -> Option<Cmd> {
            let line = ctx.line();
            let cwd = {
                let vars = self.shell_vars.lock().unwrap();
                vars.get("PWD").cloned().unwrap_or_else(|| ".".to_string())
            };
            NAVIGATION.with(|cell| {
                let mut state_opt = cell.borrow_mut();
                let is_continuing = state_opt.as_ref()
                    .is_some_and(|s| s.current_index < s.entries.len() && s.entries[s.current_index] == line);

                if is_continuing {
                    let state = state_opt.as_mut().unwrap();
                    if state.current_index + 1 < state.entries.len() {
                        state.current_index += 1;
                        let next_cmd = state.entries[state.current_index].clone();
                        Some(Cmd::Replace(Movement::WholeBuffer, Some(next_cmd)))
                    } else {
                        None
                    }
                } else {
                    let entries = self.history_mgr.get_navigation_entries(line, &cwd);
                    if entries.is_empty() {
                        return None;
                    }
                    *state_opt = Some(NavigationState {
                        original_input: line.to_string(),
                        entries: entries.clone(),
                        current_index: 0,
                    });
                    Some(Cmd::Replace(Movement::WholeBuffer, Some(entries[0].clone())))
                }
            })
        }
    }

    struct DownArrowHandler;
    impl ConditionalEventHandler for DownArrowHandler {
        fn handle(&self, _evt: &Event, _n: RepeatCount, _positive: bool, ctx: &EventContext) -> Option<Cmd> {
            let line = ctx.line();
            NAVIGATION.with(|cell| {
                let mut state_opt = cell.borrow_mut();
                let is_continuing = state_opt.as_ref()
                    .is_some_and(|s| s.current_index < s.entries.len() && s.entries[s.current_index] == line);

                if is_continuing {
                    let state = state_opt.as_mut().unwrap();
                    if state.current_index > 0 {
                        state.current_index -= 1;
                        let next_cmd = state.entries[state.current_index].clone();
                        Some(Cmd::Replace(Movement::WholeBuffer, Some(next_cmd)))
                    } else {
                        let original = state.original_input.clone();
                        *state_opt = None;
                        Some(Cmd::Replace(Movement::WholeBuffer, Some(original)))
                    }
                } else {
                    *state_opt = None;
                    None
                }
            })
        }
    }

    struct CtrlRHandler {
        history_mgr: Arc<crate::shell::history::HistoryManager>,
        shell_vars: Arc<Mutex<HashMap<String, String>>>,
    }
    impl ConditionalEventHandler for CtrlRHandler {
        fn handle(&self, _evt: &Event, _n: RepeatCount, _positive: bool, ctx: &EventContext) -> Option<Cmd> {
            let cwd = {
                let vars = self.shell_vars.lock().unwrap();
                vars.get("PWD").cloned().unwrap_or_else(|| ".".to_string())
            };
            if let Ok(Some(selected)) = crate::shell::history::interactive_reverse_search(&self.history_mgr, &cwd) {
                Some(Cmd::Replace(Movement::WholeBuffer, Some(selected)))
            } else {
                Some(Cmd::Replace(Movement::WholeBuffer, Some(ctx.line().to_string())))
            }
        }
    }

    let kb = crate::shell::history::load_keybindings();

    macro_rules! bind_key {
        ($key:expr, $mod:expr, $handler:expr) => {
            rl.bind_sequence(KeyEvent($key, $mod), $handler);
        };
    }

    let up_handler: Box<dyn ConditionalEventHandler> = match kb.up.as_deref() {
        Some("history-prev") | None => Box::new(UpArrowHandler {
            history_mgr: state.history_mgr.clone(),
            shell_vars: state.shell_vars.clone(),
        }),
        Some("reverse-search") => Box::new(CtrlRHandler {
            history_mgr: state.history_mgr.clone(),
            shell_vars: state.shell_vars.clone(),
        }),
        _ => Box::new(UpArrowHandler {
            history_mgr: state.history_mgr.clone(),
            shell_vars: state.shell_vars.clone(),
        }),
    };
    bind_key!(KeyCode::Up, Modifiers::empty(), EventHandler::Conditional(up_handler));

    let down_handler: Box<dyn ConditionalEventHandler> = match kb.down.as_deref() {
        Some("history-next") | None => Box::new(DownArrowHandler),
        _ => Box::new(DownArrowHandler),
    };
    bind_key!(KeyCode::Down, Modifiers::empty(), EventHandler::Conditional(down_handler));

    let ctrl_r_handler: Box<dyn ConditionalEventHandler> = match kb.ctrl_r.as_deref() {
        Some("reverse-search") | None => Box::new(CtrlRHandler {
            history_mgr: state.history_mgr.clone(),
            shell_vars: state.shell_vars.clone(),
        }),
        _ => Box::new(CtrlRHandler {
            history_mgr: state.history_mgr.clone(),
            shell_vars: state.shell_vars.clone(),
        }),
    };
    bind_key!(KeyCode::Char('r'), Modifiers::CTRL, EventHandler::Conditional(ctrl_r_handler));

    bind_key!(KeyCode::Right, Modifiers::empty(), EventHandler::Conditional(Box::new(CompleteHintHandler)));
    bind_key!(KeyCode::End, Modifiers::empty(), EventHandler::Conditional(Box::new(CompleteHintHandler)));
    bind_key!(KeyCode::Char('e'), Modifiers::CTRL, EventHandler::Conditional(Box::new(CompleteHintHandler)));
    bind_key!(KeyCode::Char('f'), Modifiers::CTRL, Cmd::CompleteHint);
    bind_key!(KeyCode::Char('k'), Modifiers::CTRL, EventHandler::Conditional(Box::new(KillLineHandler)));
    bind_key!(KeyCode::Char('w'), Modifiers::CTRL, EventHandler::Conditional(Box::new(KillWordBackHandler)));
    bind_key!(KeyCode::Char('y'), Modifiers::CTRL, EventHandler::Conditional(Box::new(YankHandler)));
    bind_key!(KeyCode::Char('y'), Modifiers::ALT, EventHandler::Conditional(Box::new(YankRingHandler)));

    let helper = JshHelper {
        history_mgr: state.history_mgr.clone(),
        aliases: state.aliases.clone(),
        shell_vars: state.shell_vars.clone(),
        functions: state.functions.clone(),
        completions: state.completions.clone(),
        tab_mode,
    };
    rl.set_helper(Some(helper));

    loop {
        state.check_bg_jobs();

        if SIGWINCH_FLAG.swap(false, Ordering::SeqCst) {
            let _ = crossterm::terminal::size();
        }

        let prompt_clean = state.render_prompt_clean();
        let prompt_colored = state.render_prompt();
        crate::completion::CURRENT_COLORED_PROMPT.with(|cell| {
            *cell.borrow_mut() = prompt_colored;
        });

        NAVIGATION.with(|cell| {
            *cell.borrow_mut() = None;
        });

        crate::utils::set_cursor_bar();
        let readline = rl.readline(&prompt_clean);
        match readline {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                let line = if crate::utils::pasted_text_contains_metacharacters(line) {
                    let escaped = crate::utils::smart_paste_escape(line);
                    if escaped != line {
                        eprintln!("\x1b[33mjesh: smart paste: wrapped in quotes\x1b[0m");
                        escaped
                    } else {
                        line.to_string()
                    }
                } else {
                    line.to_string()
                };

                state.maybe_hot_reload();

                let expanded_line = expand_history_refs(&line, &state.history_mgr);

                let show_timing = state.get_var("SHOW_TIMING") != "false";
                let start_time = std::time::Instant::now();

                let cwd = {
                    let vars = state.shell_vars.lock().unwrap();
                    vars.get("PWD").cloned().unwrap_or_else(|| ".".to_string())
                };

                let mut lines: Vec<String> = expanded_line.lines().map(|s| s.to_string()).collect();
                let mut i = 0;
                while i < lines.len() {
                    let mut current_line = lines[i].clone();
                    i += 1;

                    while crate::utils::ends_with_line_continuation(&current_line) {
                        current_line.pop(); // Remove the trailing '\'
                        if i < lines.len() {
                            current_line.push_str(&lines[i]);
                            i += 1;
                        } else {
                            match rl.readline("> ") {
                                Ok(next_l) => {
                                    lines.push(next_l.clone());
                                    current_line.push_str(&next_l);
                                    i += 1;
                                }
                                Err(ReadlineError::Interrupted) => {
                                    SIGINT_FLAG.store(true, Ordering::SeqCst);
                                    i = lines.len();
                                    break;
                                }
                                Err(_) => {
                                    i = lines.len();
                                    break;
                                }
                            }
                        }
                    }

                    let trimmed = current_line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }

                    run_line_with(&mut state, trimmed, |prompt| {
                        if i < lines.len() {
                            let next_l = lines[i].clone();
                            i += 1;
                            Some(next_l)
                        } else {
                            match rl.readline(prompt) {
                                Ok(l) => Some(l),
                                Err(ReadlineError::Interrupted) => {
                                    SIGINT_FLAG.store(true, Ordering::SeqCst);
                                    None
                                }
                                Err(_) => None,
                            }
                        }
                    });

                    if state.get_var("JSH_TRANSIENT_PROMPT") == "true" {
                        let cols = crossterm::terminal::size().map(|(w, _)| w).unwrap_or(80) as usize;
                        let trans_line = format!("> {} ", trimmed);
                        let truncated: String = if trans_line.chars().count() > cols - 1 {
                            trans_line.chars().take(cols - 4).chain("...".chars()).collect()
                        } else {
                            trans_line
                        };
                        eprint!("\r\x1b[K{}", truncated);
                        let _ = std::io::stderr().flush();
                    }

                    if SIGINT_FLAG.load(Ordering::SeqCst) {
                        break;
                    }
                }

                let reconstructed_input = lines.join("\n");
                state.history_mgr.add_entry(&reconstructed_input, state.last_exit_status, &cwd);
                if SIGINT_FLAG.swap(false, Ordering::SeqCst) {
                    println!("^C");
                }
                if show_timing {
                    let elapsed = start_time.elapsed();
                    if elapsed.as_secs_f64() >= 2.0 {
                        eprintln!("\u{1b}[38;5;240m(\u{23f3} demorou {:.1}s)\u{1b}[0m", elapsed.as_secs_f64());
                    }
                }

                let git_branch_cache = state.cached_git_branch.clone();
                let cwd_for_branch = cwd.clone();
                std::thread::spawn(move || {
                    let branch = crate::shell::ShellState::get_git_branch_for(&cwd_for_branch);
                    *git_branch_cache.lock().unwrap() = branch;
                });
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
            }
            Err(ReadlineError::Eof) => {
                println!("Saindo do jesh...");
                crate::utils::set_cursor_block();
                break;
            }
            Err(err) => {
                println!("Erro: {:?}", err);
                break;
            }
        }
    }
}

/// Non-interactive mode: reads a full script (from a file argument or piped
/// stdin) and runs it through `ShellState::run_script_text`, supporting
/// `;`/`&&`/`||`/pipes/heredocs/function definitions without requiring a TTY.
fn run_script<R: BufRead>(mut state: ShellState, mut reader: R) {
    let mut content = String::new();
    if let Err(e) = reader.read_to_string(&mut content) {
        eprintln!("jesh: erro ao ler script: {}", e);
        std::process::exit(1);
    }
    state.run_script_text(&content);
    std::process::exit(state.last_exit_status);
}

fn print_version() {
    println!("jesh (jeffutils) {}", env!("CARGO_PKG_VERSION"));
    println!("Copyright (C) 2026 Jefferson Silva de Souza Rios.");
    println!("Contato: jeff.silvadsouza@gmail.com");
    println!("Licenca GPLv3+: GNU GPL versao 3 ou posterior <https://gnu.org/licenses/gpl.html>");
    println!("Este e um software livre: voce e livre para altera-lo e redistribui-lo.");
    println!("NAO HA QUALQUER GARANTIA, na maxima extensao permitida em lei.");
}

fn main() {
    if std::env::args().skip(1).any(|a| a == "--version" || a == "-v") {
        print_version();
        std::process::exit(0);
    }
    let mut state = ShellState::new();

    // Sync $PWD with actual CWD before loading .jshrc or running any commands.
    sync_pwd();

    let args: Vec<String> = std::env::args().collect();

    let mut cmd_string: Option<String> = None;
    let mut script_path: Option<String> = None;
    let mut script_args: Vec<String> = Vec::new();

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        if arg == "-c" {
            if i + 1 < args.len() {
                cmd_string = Some(args[i + 1].clone());
                if i + 2 < args.len() {
                    state.arg0 = args[i + 2].clone();
                    script_args = args[i + 3..].to_vec();
                }
                break;
            } else {
                eprintln!("jesh: -c: option requires an argument");
                std::process::exit(2);
            }
        } else if arg == "-l" || arg == "--login" {
            // Ignore/skip login shell flags but don't treat them as script files
            i += 1;
        } else if arg.starts_with('-') {
            // Skip other options to avoid failing
            i += 1;
        } else {
            // First non-option argument is the script file path
            script_path = Some(arg.clone());
            script_args = args[i + 1..].to_vec();
            break;
        }
    }

    state.set_positional_args(script_args);

    if let Some(command_string) = cmd_string {
        state.run_script_text(&command_string);
        std::process::exit(state.last_exit_status);
    }

    if let Some(path) = script_path {
        state.arg0 = path.clone();
        state.load_jshrc();
        match std::fs::File::open(&path) {
            Ok(f) => run_script(state, std::io::BufReader::new(f)),
            Err(e) => {
                eprintln!("jesh: {}: {}", path, e);
                std::process::exit(1);
            }
        }
        return;
    }

    // Load config from .jshrc
    state.load_jshrc();

    if !std::io::stdin().is_terminal() {
        run_script(state, std::io::stdin().lock());
        return;
    }

    // Run jeofetch on init, but only in an interactive terminal session.
    // Skip it for non-tty invocations like `jesh -c "..."`, piped stdin
    // (e.g. `!pwd` inside Claude), or when stdout is redirected — there
    // jeofetch would just be noise in captured output.
    if state.init_info && std::io::stdin().is_terminal() && std::io::stdout().is_terminal() {
        run_jeofetch();
    }

    run_interactive(state);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_var_execution() {
        let mut state = ShellState::new();
        run_line_with(&mut state, "TEST_VAR=hello", |_| None);
        assert_eq!(state.get_var("TEST_VAR"), "hello");
    }

    #[test]
    fn test_sync_pwd_updates_pwd_env_not_cwd() {
        let original_cwd = std::env::current_dir().unwrap();
        let original_pwd = std::env::var("PWD").ok();
        let fake_pwd = if original_cwd != std::path::Path::new("/tmp") {
            "/tmp"
        } else {
            "/"
        };
        unsafe {
            std::env::set_var("PWD", fake_pwd);
        }
        sync_pwd();
        assert_eq!(std::env::current_dir().unwrap(), original_cwd);
        assert_eq!(
            std::env::var("PWD").unwrap(),
            original_cwd.to_string_lossy().as_ref()
        );
        if let Some(pwd) = original_pwd {
            unsafe { std::env::set_var("PWD", &pwd); }
        }
    }

    #[test]
    fn test_cmd_option_execution() {
        let mut state = ShellState::new();
        assert!(!state.is_interactive);
        run_line_with(&mut state, "true", |_| None);
        assert_eq!(state.last_exit_status, 0);
    }

    #[test]
    fn test_script_line_continuation() {
        let mut state = ShellState::new();
        state.run_script_text("TEST_VAR=foo\\\nbar");
        assert_eq!(state.get_var("TEST_VAR"), "foobar");
    }

    #[test]
    fn test_script_heredoc() {
        let mut state = ShellState::new();
        let temp_file = "/tmp/jsh_test_heredoc.txt";
        let _ = std::fs::remove_file(temp_file);

        state.run_script_text(&format!(
            "cat << 'EOF' > {}\nhello\nworld\nEOF",
            temp_file
        ));

        let content = std::fs::read_to_string(temp_file).unwrap();
        assert_eq!(content, "hello\nworld\n");
        let _ = std::fs::remove_file(temp_file);
    }

    #[test]
    fn test_ansi_c_quoting() {
        let mut state = ShellState::new();
        state.run_script_text("VAL=$'hello\\nworld'");
        assert_eq!(state.get_var("VAL"), "hello\nworld");

        state.run_script_text("VAL=$'a\\tb'");
        assert_eq!(state.get_var("VAL"), "a\tb");

        state.run_script_text("VAL=$'foo \\x41 bar'");
        assert_eq!(state.get_var("VAL"), "foo A bar");
    }

    #[test]
    fn test_arithmetic_expansion() {
        let mut state = ShellState::new();
        state.run_script_text("VAL=$(( 1 + 2 * 3 ))");
        assert_eq!(state.get_var("VAL"), "7");

        state.run_script_text("X=10");
        state.run_script_text("VAL=$(( X + 5 ))");
        assert_eq!(state.get_var("VAL"), "15");

        state.run_script_text("VAL=$(( $X * 2 ))");
        assert_eq!(state.get_var("VAL"), "20");

        state.run_script_text("VAL=$(( (X + 2) % 3 ))");
        assert_eq!(state.get_var("VAL"), "0");
    }
}