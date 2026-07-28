use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use crossterm::style::{Color, Stylize};
#[cfg(unix)]
use nix::unistd::{getppid, getuid, getgroups};
use unicode_width::UnicodeWidthStr;

use crate::completion::CompletionDb;
use crate::parser::{Word, WordSegment};
use crate::utils;

fn command_exists(name: &str) -> bool {
    let path_var = match env::var_os("PATH") {
        Some(p) => p,
        None => return false,
    };
    env::split_paths(&path_var).any(|dir| {
        let full = dir.join(name);
        full.is_file() && full.metadata().ok().map_or(false, |m| {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                m.permissions().mode() & 0o111 != 0
            }
            #[cfg(windows)]
            {
                let _ = m;
                true
            }
        })
    })
}

fn register_modern_cli_aliases(aliases: &mut HashMap<String, String>) {
    if command_exists("eza") {
        aliases.insert("ls".to_string(), "eza --color=auto --icons".to_string());
        aliases.insert("ll".to_string(), "eza -la --color=auto --icons".to_string());
        aliases.insert("lt".to_string(), "eza --tree --level=2 --color=auto --icons".to_string());
    } else if command_exists("exa") {
        aliases.insert("ls".to_string(), "exa --color=auto --icons".to_string());
        aliases.insert("ll".to_string(), "exa -la --color=auto --icons".to_string());
    }
    if command_exists("bat") {
        aliases.insert("cat".to_string(), "bat --paging=never".to_string());
    }
    if command_exists("rg") {
        aliases.insert("grep".to_string(), "rg --color=always".to_string());
    }
    if command_exists("fd") {
        aliases.insert("find".to_string(), "fd --type f --hidden --exclude .git".to_string());
    }
    if command_exists("htop") {
        aliases.insert("top".to_string(), "htop".to_string());
    }
    if command_exists("zoxide") {
        aliases.insert("z".to_string(), "zoxide".to_string());
    }
}

pub mod history;

#[derive(Debug, Clone, Default)]
pub struct VarAttrs {
    pub integer: bool,
    pub array: bool,
    pub assoc: bool,
    pub readonly: bool,
    pub exported: bool,
    pub local: bool,
}

pub struct ShellState {
    pub last_exit_status: i32,
    pub home_dir: PathBuf,
    pub init_info: bool,
    pub aliases: Arc<Mutex<HashMap<String, String>>>,
    pub old_pwd: Option<PathBuf>,
    /// Shell-local variables (`NAME=value`), distinct from process env vars.
    /// Looked up before falling back to `env::var`.
    pub shell_vars: Arc<Mutex<HashMap<String, String>>>,
    /// Variable attributes (integer, array, assoc, readonly, exported, local)
    pub var_attrs: Arc<Mutex<HashMap<String, VarAttrs>>>,
    /// Names of shell vars that have been `export`ed to the process env.
    pub exported: HashSet<String>,
    /// Name jesh was invoked as / script path, used for `$0`.
    pub arg0: String,
    /// When true, "command not found" errors are swallowed instead of
    /// printed. Used while loading `.jshrc`, since it may contain bash-only
    /// constructs (functions, `[ ]` tests) this shell doesn't parse — each
    /// such line fails as an unknown command, and printing all of those on
    /// every startup would be noisy for configs migrated from bash/zsh.
    pub quiet_errors: bool,
    /// Paths passed to `source`/`.` that look like real bash scripts
    /// (define functions, use `[[`, etc.) rather than simple jesh-style
    /// config. jesh can't interpret bash functions itself, so commands that
    /// turn out to be unknown are retried through `bash -ic "source <file>;
    /// <cmd> <args>"` for each of these files — this is how things like
    /// `nvm use 18` keep working after `.jshrc` sources nvm.sh.
    pub bash_sourced_files: Vec<PathBuf>,
    /// User-defined shell functions (`name() { body }`), keyed by name.
    /// The body is the raw text between `{` and `}`, run as a nested
    /// script with `$1`, `$2`, ... bound to the call's arguments.
    pub functions: Arc<Mutex<HashMap<String, String>>>,
    /// Stack of positional-parameter frames for nested function calls;
    /// the top frame is used to resolve `$1`, `$2`, `$@`, `$#` while a
    /// function body is executing.
    pub positional_stack: Vec<Vec<String>>,
    /// Stack of variable attribute frames for nested function calls (for `local`).
    /// Each frame maps variable names to their saved attributes for restoration on return.
    pub var_attrs_stack: Vec<HashMap<String, VarAttrs>>,
    /// Stack of variable value frames for nested function calls (for `local`).
    pub var_values_stack: Vec<HashMap<String, String>>,
    /// Last-seen modification time of `.jshrc`, used to detect edits for
    /// hot-reloading. `None` until the file is first loaded.
    pub jeshrc_mtime: Option<SystemTime>,
    /// Cached OS logo (emoji) for the prompt, populated on first access.
    cached_os_logo: Option<String>,
    /// Cached commands known to NOT exist in bash (neg cache for try_bash_fallback).
    /// This avoids spawning bash for every unknown command.
    bash_cmd_neg_cache: HashSet<String>,
    /// Whether the shell is currently running in an interactive session.
    pub is_interactive: bool,
    pub history_mgr: Arc<history::HistoryManager>,
    pub completions: Arc<Mutex<CompletionDb>>,
    pub dir_stack: Vec<PathBuf>,
    pub pipestatus: Vec<i32>,
    pub pipefail: bool,
    pub readonly_vars: HashSet<String>,
    pub glob_nullglob: bool,
    pub glob_failglob: bool,
    pub glob_dotglob: bool,
    pub glob_nocaseglob: bool,
    pub glob_extglob: bool,
    pub bg_jobs: Arc<Mutex<Vec<BgJob>>>,
    pub errexit: bool,
    pub nounset: bool,
    pub xtrace: bool,
    pub cached_git_branch: Arc<Mutex<Option<String>>>,
    pub cached_prompt_time: Arc<Mutex<std::time::Instant>>,
}

#[derive(Debug, Clone)]
pub struct BgJob {
    pub pid: u32,
    pub command: String,
    pub start_time: SystemTime,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Qualifier {
    Dir,
    File,
    Symlink,
    Exec,
    NotDir,
    NotFile,
    NotSymlink,
    NotExec,
}

impl ShellState {
    pub fn new() -> Self {
        let home = env::var_os("HOME")
            .or_else(|| env::var_os("USERPROFILE"))
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/"));

        // Ensure PATH is set to a reasonable default if not present
        // This handles cases where PATH is empty or missing (e.g. when invoked via `sh -c`)
        let default_path = "/usr/local/bin:/usr/bin:/bin";
        let path = env::var_os("PATH");
        if path.as_ref().map(|p| p.is_empty()).unwrap_or(true) {
            unsafe {
                env::set_var("PATH", default_path);
            }
        }

        // Override SHELL env var to point to our jesh
        if let Ok(exe) = env::current_exe() {
            unsafe {
                env::set_var("SHELL", exe);
            }
        }

        let aliases_map = Arc::new(Mutex::new(HashMap::new()));

        {
            let mut map = aliases_map.lock().unwrap();
            map.insert("ls".to_string(), "ls --color=auto".to_string());
            map.insert("grep".to_string(), "grep --color=auto".to_string());
            map.insert("ll".to_string(), "ls -la --color=auto".to_string());
            map.insert("c".to_string(), "clear".to_string());
            register_modern_cli_aliases(&mut map);
        }

        let history_mgr = Arc::new(history::HistoryManager::new());
        history_mgr.load_history();

        let jeshrc_path = home.join(".jeshrc");
        let mut bash_sourced_files: Vec<PathBuf> = Vec::new();
        let mut shell_vars: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
        let mut exported: HashSet<String> = HashSet::new();
        let arg0 = String::new();
        let quiet_errors = false;
        let mut functions: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
        let mut positional_stack: Vec<Vec<String>> = Vec::new();
        let mut old_pwd: Option<PathBuf> = None;
        let mut cached_os_logo: Option<String> = None;
        let mut bash_cmd_neg_cache: HashSet<String> = HashSet::new();
        let mut is_interactive = false;
        let mut completions = Arc::new(Mutex::new(CompletionDb::new()));
        let mut dir_stack: Vec<PathBuf> = Vec::new();
        let jeshrc_mtime: Option<SystemTime> = None;

        Self {
            last_exit_status: 0,
            home_dir: home,
            init_info: true,
            aliases: aliases_map,
            old_pwd: None,
            shell_vars: Arc::new(Mutex::new(HashMap::new())),
            exported: HashSet::new(),
            arg0: "jesh".to_string(),
            quiet_errors: false,
            bash_sourced_files: Vec::new(),
            functions: Arc::new(Mutex::new(HashMap::new())),
            positional_stack: Vec::new(),
            jeshrc_mtime: None,
            cached_os_logo: None,
            bash_cmd_neg_cache: HashSet::new(),
            is_interactive: false,
            history_mgr,
            completions,
            dir_stack: Vec::new(),
            pipestatus: Vec::new(),
            pipefail: false,
            readonly_vars: HashSet::new(),
            var_attrs: Arc::new(Mutex::new(HashMap::new())),
            var_attrs_stack: Vec::new(),
            var_values_stack: Vec::new(),
            glob_nullglob: false,
            glob_failglob: false,
            glob_dotglob: false,
            glob_nocaseglob: false,
            glob_extglob: false,
            bg_jobs: Arc::new(Mutex::new(Vec::new())),
            errexit: false,
            nounset: false,
            xtrace: false,
            cached_git_branch: Arc::new(Mutex::new(None)),
            cached_prompt_time: Arc::new(Mutex::new(std::time::Instant::now())),
        }
    }

    pub fn check_bg_jobs(&self) {
        #[cfg(unix)]
        {
            let mut jobs = self.bg_jobs.lock().unwrap();
            let mut i = 0;
            while i < jobs.len() {
                let pid = jobs[i].pid as libc::pid_t;
                let mut status: i32 = 0;
                let ret = unsafe { libc::waitpid(pid, &mut status, libc::WNOHANG) };
                if ret == pid {
                    let exited = libc::WIFEXITED(status);
                    let exit_code = if exited {
                        libc::WEXITSTATUS(status)
                    } else if libc::WIFSIGNALED(status) {
                        128 + libc::WTERMSIG(status)
                    } else {
                        1
                    };
                    let cmd = &jobs[i].command;
                    let elapsed = jobs[i].start_time.elapsed().map(|d| d.as_secs_f64()).unwrap_or(0.0);
                    if exit_code == 0 {
                        eprintln!("\r\x1B[32m[done]\x1B[0m {} ({}s)      ", cmd, elapsed);
                    } else {
                        eprintln!("\r\x1B[31m[done]\x1B[0m {} (exit {}) ({}s)      ", cmd, exit_code, elapsed);
                    }
                    jobs.remove(i);
                } else if ret == -1 {
                    jobs.remove(i);
                } else {
                    i += 1;
                }
            }
        }
    }

    pub fn load_jshrc(&mut self) {
        let jeshrc_path = self.home_dir.join(".jeshrc");
        if !jeshrc_path.exists() {
            let default_jshrc = "\
# jesh configuration file
INIT_INFO=true

# When true, editing this file re-loads it automatically before each
# prompt — no need to `source.jeshrc` or restart the shell.
HOT_RELOAD=true

# When true, shows elapsed time for commands that take >= 2s.
# Set to false to disable the \"(⏳ demorou Xs)\" notification.
SHOW_TIMING=true

# Tab completion mode: \"interactive\" (default) shows a horizontal menu
# with Tab/Shift+Tab navigation and Enter to select.
# Set to \"circular\" for the traditional inline cycle behavior.
# Set to \"hybrid\" for the pro autocomplete: combines circular cycling
# with the interactive menu — shows a counter (2/5), a live preview of
# the replacement, and circular navigation (Tab wraps, Shift+Tab wraps).
# JSH_TAB_MODE=interactive

alias c=\"clear\"
alias ls=\"ls --color=auto\"
alias grep=\"grep --color=auto\"

# Custom Exports
export EDITOR=texit
# Extend the inherited PATH instead of replacing it:
export PATH=$PATH:/usr/local/bin
";
            let _ = fs::write(&jeshrc_path, default_jshrc);
        }

        self.jeshrc_mtime = fs::metadata(&jeshrc_path)
            .and_then(|m| m.modified())
            .ok();

        if let Ok(content) = fs::read_to_string(&jeshrc_path) {
            self.quiet_errors = true;
            self.run_script_text(&content);
        self.quiet_errors = false;
        self.load_theme();
    }
    }

    /// If hot-reload is enabled (`HOT_RELOAD=true` in `.jeshrc`) and the file
    /// has been modified since it was last loaded, re-run it so edits take
    /// effect without `source.jeshrc` or restarting the shell. Called before
    /// each interactive prompt. The `HOT_RELOAD` flag is read from the
    /// *currently loaded* config, so setting it to false (or removing it)
    /// and reloading once disables further auto-reloading.
    pub fn maybe_hot_reload(&mut self) {
        if self.get_var("HOT_RELOAD") != "true" {
            return;
        }
        let jeshrc_path = self.home_dir.join(".jeshrc");
        let Some(mtime) = fs::metadata(&jeshrc_path).and_then(|m| m.modified()).ok() else {
            return;
        };
        if self.jeshrc_mtime == Some(mtime) {
            return;
        }
        self.load_jshrc();
    }

    /// Heuristic: does `content` use bash syntax jesh genuinely can't parse
    /// (`[[ ]]`, `local`, `case`, etc — simple one-line function defs are
    /// now natively supported, see `run_script_text`)? If so, `source`/`.`
    /// should remember the file so unknown commands can be retried through
    /// real bash.
    pub fn looks_like_bash(content: &str) -> bool {
        content.contains("[[")
            || content.contains("local ")
            || content.lines().any(|l| l.trim().starts_with("case "))
    }

    /// Retries `program args...` through `bash -ic`, sourcing every bash
    /// script previously loaded via `source`/`.`, so functions defined
    /// there (e.g. `nvm`) remain callable from jesh. Returns `None` if there
    /// are no bash-sourced files or bash isn't available.
    /// Uses a negative cache to avoid repeated spawns for commands known to not exist.
    pub fn try_bash_fallback(&mut self, program: &str, args: &[String]) -> Option<i32> {
        if self.bash_sourced_files.is_empty() {
            return None;
        }

        // Check negative cache first - skip bash spawn if we already know this command doesn't exist
        if self.bash_cmd_neg_cache.contains(program) {
            return None;
        }

        let mut script = String::new();
        for f in &self.bash_sourced_files {
            script.push_str("source ");
            script.push('\'');
            script.push_str(&f.to_string_lossy().replace('\'', "'\\''"));
            script.push_str("' >/dev/null 2>&1; ");
        }

        let mut check_script = script.clone();
        check_script.push_str("type -t ");
        check_script.push_str(program);
        check_script.push_str(" >/dev/null 2>&1");

        let check_status = Command::new("bash")
            .arg("-c")
            .arg(&check_script)
            .status()
            .ok()?;

        if !check_status.success() {
            // Cache failure to avoid repeated spawns for the same missing command
            self.bash_cmd_neg_cache.insert(program.to_string());
            return None;
        }

        script.push_str(program);
        for a in args {
            script.push(' ');
            script.push('\'');
            script.push_str(&a.replace('\'', "'\\''"));
            script.push('\'');
        }

        let status = Command::new("bash")
            .arg("-ic")
            .arg(&script)
            .status()
            .ok()?;
        Some(crate::utils::exit_code_from_status(status))
    }

    /// Runs a block of script text line by line through the same
    /// tokenize -> parse -> expand -> execute pipeline used interactively,
    /// without requiring a TTY. Used for `.jshrc`, `source`, and non-interactive
    /// stdin/script invocation. Also recognizes and stores simple shell
    /// function definitions (`name() { body }`, one line or multi-line).
    pub fn run_script_text(&mut self, content: &str) {
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        while i < lines.len() {
            let mut raw_line = lines[i].to_string();
            i += 1;

            while crate::utils::ends_with_line_continuation(&raw_line) && i < lines.len() {
                raw_line.pop(); // Remove the trailing '\'
                raw_line.push_str(lines[i]);
                i += 1;
            }

            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((name, rest)) = Self::function_header(line) {
                let mut body = String::new();
                let mut depth = 0i32;
                let mut collected_any_brace = false;

                for ch in rest.chars() {
                    if ch == '{' {
                        depth += 1;
                        collected_any_brace = true;
                        if depth == 1 {
                            continue;
                        }
                    } else if ch == '}' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    if collected_any_brace {
                        body.push(ch);
                    }
                }

                while depth > 0 && i < lines.len() {
                    let next_line = lines[i];
                    i += 1;
                    for ch in next_line.chars() {
                        if ch == '{' {
                            depth += 1;
                        } else if ch == '}' {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        body.push(ch);
                    }
                    body.push('\n');
                }

                self.functions.lock().unwrap().insert(name, body.trim().to_string());
                continue;
            }

            crate::run_line_with(self, line, |_| {
                if i < lines.len() {
                    let l = lines[i];
                    i += 1;
                    Some(l.to_string())
                } else {
                    None
                }
            });
        }
    }

    /// Recognizes a `name() {` (or `function name {` / `function name() {`)
    /// header, returning `(name, rest_of_line_after_open_brace_search)`.
    fn function_header(line: &str) -> Option<(String, &str)> {
        let line = line.trim();
        let (name, after) = if let Some(rest) = line.strip_prefix("function ") {
            let rest = rest.trim_start();
            let name_end = rest.find(|c: char| c.is_whitespace() || c == '(').unwrap_or(rest.len());
            let name = &rest[..name_end];
            (name, &rest[name_end..])
        } else {
            let paren = line.find("()")?;
            let name = line[..paren].trim();
            if name.is_empty()
                || !name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_')
            {
                return None;
            }
            (name, &line[paren + 2..])
        };

        if name.is_empty() {
            return None;
        }
        if !after.contains('{') {
            return None;
        }
        Some((name.to_string(), after))
    }

    pub fn set_positional_args(&mut self, args: Vec<String>) {
        self.positional_stack = vec![args];
    }

    /// Runs a user-defined function's body with `$1`, `$2`, ... bound to
    /// `args`, returning the function's final exit status.
    pub fn call_function(&mut self, name: &str, args: &[String]) -> i32 {
        let Some(body) = self.functions.lock().unwrap().get(name).cloned() else {
            return 127;
        };
        self.positional_stack.push(args.to_vec());
        self.push_local_scope();
        self.run_script_text(&body);
        self.pop_local_scope();
        self.positional_stack.pop();
        self.last_exit_status
    }

    /// Returns the current function call's positional parameters, if any
    /// function call is in progress.
    fn positional_params(&self) -> Option<&Vec<String>> {
        self.positional_stack.last()
    }

    /// Looks up a shell variable, falling back to the process environment,
    /// then resolves the handful of special variables (`?`, `$`, `0`, and
    /// the positional parameters `1`.."9", `@`, `#` inside a function body).
    pub fn get_var(&self, name: &str) -> String {
        match name {
            "?" => return self.last_exit_status.to_string(),
            "$" => return std::process::id().to_string(),
            "0" => return self.arg0.clone(),
            "PWD" => {
                if let Ok(cwd) = env::current_dir() {
                    return cwd.to_string_lossy().into_owned();
                }
            }
            "OLDPWD" => {
                if let Some(ref p) = self.old_pwd {
                    return p.to_string_lossy().into_owned();
                }
            }
            "@" | "*" => {
                if let Some(params) = self.positional_params() {
                    return params.join(" ");
                }
            }
            "#" => {
                if let Some(params) = self.positional_params() {
                    return params.len().to_string();
                }
                return "0".to_string();
            }
            "PIPESTATUS" => {
                return self.pipestatus.iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
            }
            "IFS" => {
                // Return the IFS value from shell vars or default
                if let Some(v) = self.shell_vars.lock().unwrap().get("IFS").cloned() {
                    return v;
                }
                return " \t\n".to_string();
            }
            "PPID" => {
                #[cfg(unix)]
                { return getppid().to_string(); }
                #[cfg(windows)]
                { return "0".to_string(); }
            }
            "UID" => {
                #[cfg(unix)]
                { return getuid().as_raw().to_string(); }
                #[cfg(windows)]
                { return "0".to_string(); }
            }
            "GROUPS" => {
                #[cfg(unix)]
                {
                    if let Ok(groups) = getgroups() {
                        return groups.iter().map(|g| g.as_raw().to_string()).collect::<Vec<_>>().join(" ");
                    }
                    return String::new();
                }
                #[cfg(windows)]
                { return String::new(); }
            }
            _ if name.len() <= 2 && name.chars().all(|c| c.is_ascii_digit()) && !name.is_empty() => {
                if let Ok(idx) = name.parse::<usize>() {
                    if idx >= 1 {
                        if let Some(params) = self.positional_params() {
                            return params.get(idx - 1).cloned().unwrap_or_default();
                        }
                        return String::new();
                    }
                }
            }
            _ => {}
        }
        if let Some(v) = self.shell_vars.lock().unwrap().get(name).cloned() {
            return v;
        }
        if self.nounset {
            let env_val = env::var(name);
            match env_val {
                Ok(v) => return v,
                Err(_) => {
                    eprintln!("jesh: {}: variable not set", name);
                    return String::new();
                }
            }
        }
        env::var(name).unwrap_or_default()
    }

    /// Resolves a `${NAME:+word}` / `${NAME:-word}` style parameter
    /// expansion. `op` is `+` or `-`; `word` is re-tokenized (so quotes and
    /// `$VAR` references inside it work normally) and expanded. POSIX
    /// semantics: `:-` substitutes `word` when NAME is unset/empty,
    /// otherwise keeps NAME's value; `:+` substitutes `word` when NAME is
    /// set/non-empty, otherwise expands to nothing.
    pub fn expand_param_op(&mut self, name: &str, op: char, word: &str) -> String {
        let current = self.get_var(name);
        match op {
            '-' => {
                // ${VAR:-word}: use word if VAR is unset/empty
                if current.is_empty() {
                    self.expand_param_word(word)
                } else {
                    current
                }
            }
            '+' => {
                // ${VAR:+word}: use word if VAR is set/non-empty
                if current.is_empty() {
                    String::new()
                } else {
                    self.expand_param_word(word)
                }
            }
            '=' => {
                // ${VAR:=word}: assign word to VAR if unset/empty, then return value
                if current.is_empty() {
                    let expanded = self.expand_param_word(word);
                    self.set_var(name, &expanded);
                    expanded
                } else {
                    current
                }
            }
            '?' => {
                // ${VAR:?word}: error if unset/empty, else return value
                if current.is_empty() {
                    let expanded = self.expand_param_word(word);
                    eprintln!("{}: {}", name, expanded);
                    std::process::exit(1);
                } else {
                    current
                }
            }
            '#' => {
                // ${VAR#pattern}: remove shortest prefix
                self.expand_prefix_removal(name, word, false)
            }
            'H' => {
                // ${VAR##pattern}: remove longest prefix
                self.expand_prefix_removal(name, word, true)
            }
            '%' => {
                // ${VAR%pattern}: remove shortest suffix
                self.expand_suffix_removal(name, word, false)
            }
            'P' => {
                // ${VAR%%pattern}: remove longest suffix
                self.expand_suffix_removal(name, word, true)
            }
            '/' => {
                // ${VAR/pat/repl}: substitute first match
                self.expand_subst(name, word, false)
            }
            'D' => {
                // ${VAR//pat/repl}: substitute all (global)
                self.expand_subst(name, word, true)
            }
            '!' => {
                // ${!VAR}: indirect reference
                let var_name = self.get_var(name);
                self.get_var(&var_name)
            }
            _ => current,
        }
    }

    fn expand_param_word(&mut self, word: &str) -> String {
        let tokens = crate::parser::lexer::tokenize(word);
        let parsed_word = tokens.into_iter().find_map(|t| match t {
            crate::parser::lexer::Token::Word(w) => Some(w),
            _ => None,
        });
        match parsed_word {
            Some(w) => self.expand_word_single(&w),
            None => String::new(),
        }
    }

    fn expand_prefix_removal(&self, name: &str, pattern: &str, is_longest: bool) -> String {
        let current = self.get_var(name);
        if current.is_empty() || pattern.is_empty() {
            return current;
        }
        if is_longest {
            // ${VAR##pattern}: remove longest matching prefix (glob)
            for len in (1..=current.len()).rev() {
                if Self::match_simple_pattern(&current[..len], pattern) {
                    return current[len..].to_string();
                }
            }
        } else {
            // ${VAR#pattern}: remove shortest matching prefix (glob)
            for len in 1..=current.len() {
                if Self::match_simple_pattern(&current[..len], pattern) {
                    return current[len..].to_string();
                }
            }
        }
        current
    }

    fn expand_suffix_removal(&self, name: &str, pattern: &str, is_longest: bool) -> String {
        let current = self.get_var(name);
        if current.is_empty() || pattern.is_empty() {
            return current;
        }
        if is_longest {
            // ${VAR%%pattern}: remove longest matching suffix (glob)
            for len in (1..=current.len()).rev() {
                if Self::match_simple_pattern(&current[current.len() - len..], pattern) {
                    return current[..current.len() - len].to_string();
                }
            }
        } else {
            // ${VAR%pattern}: remove shortest matching suffix (glob)
            for len in 1..=current.len() {
                if Self::match_simple_pattern(&current[current.len() - len..], pattern) {
                    return current[..current.len() - len].to_string();
                }
            }
        }
        current
    }

    fn glob_to_regex_str(pattern: &str) -> String {
        let mut regex_str = String::new();
        for c in pattern.chars() {
            match c {
                '*' => regex_str.push_str(".*"),
                '?' => regex_str.push('.'),
                '[' => regex_str.push('['),
                ']' => regex_str.push(']'),
                c if c.is_ascii_punctuation() => regex_str.push_str(&regex::escape(&c.to_string())),
                c => regex_str.push(c),
            }
        }
        regex_str
    }

    fn expand_subst(&self, name: &str, word: &str, global: bool) -> String {
        let current = self.get_var(name);
        if let Some(slash_pos) = word.find('/') {
            let pattern = &word[..slash_pos];
            let replacement = &word[slash_pos + 1..];
            let regex_str = Self::glob_to_regex_str(pattern);
            if let Ok(re) = regex::Regex::new(&regex_str) {
                if global {
                    re.replace_all(&current, replacement).to_string()
                } else {
                    re.replace(&current, replacement).to_string()
                }
            } else {
                current
            }
        } else {
            current
        }
    }

    pub fn expand_string(&mut self, word: &str) -> String {
        let tokens = crate::parser::lexer::tokenize(word);
        let parsed_word = tokens.into_iter().find_map(|t| match t {
            crate::parser::lexer::Token::Word(w) => Some(w),
            _ => None,
        });
        match parsed_word {
            Some(w) => self.expand_word_single(&w),
            None => String::new(),
        }
    }

    pub fn set_var(&mut self, name: &str, value: &str) {
        if name == "INIT_INFO" {
            self.init_info = value == "true";
        }
        self.shell_vars.lock().unwrap().insert(name.to_string(), value.to_string());
        if self.exported.contains(name) {
            unsafe {
                env::set_var(name, value);
            }
        }
    }

    pub fn export_var(&mut self, name: &str, value: Option<&str>) {
        if let Some(v) = value {
            self.shell_vars.lock().unwrap().insert(name.to_string(), v.to_string());
            unsafe {
                env::set_var(name, v);
            }
        } else if let Some(v) = self.shell_vars.lock().unwrap().get(name).cloned() {
            unsafe {
                env::set_var(name, &v);
            }
        }
        self.exported.insert(name.to_string());
    }

    pub fn unset_var(&mut self, name: &str) {
        self.shell_vars.lock().unwrap().remove(name);
        self.var_attrs.lock().unwrap().remove(name);
        self.exported.remove(name);
        self.readonly_vars.remove(name);
        unsafe {
            env::remove_var(name);
        }
    }

    /// Get variable attributes, creating default if not exist
    pub fn get_var_attrs(&self, name: &str) -> VarAttrs {
        self.var_attrs.lock().unwrap().get(name).cloned().unwrap_or_default()
    }

    /// Set variable attributes
    pub fn set_var_attrs(&mut self, name: &str, attrs: VarAttrs) {
        self.var_attrs.lock().unwrap().insert(name.to_string(), attrs);
    }

    /// Check if variable has integer attribute
    pub fn is_integer_var(&self, name: &str) -> bool {
        self.get_var_attrs(name).integer
    }

    /// Check if variable is readonly
    pub fn is_readonly_var(&self, name: &str) -> bool {
        self.readonly_vars.contains(name) || self.get_var_attrs(name).readonly
    }

    /// Check if variable is an array (indexed)
    pub fn is_array_var(&self, name: &str) -> bool {
        self.get_var_attrs(name).array
    }

    /// Check if variable is an associative array
    pub fn is_assoc_var(&self, name: &str) -> bool {
        self.get_var_attrs(name).assoc
    }

    /// Set a variable with integer attribute (evaluates arithmetic expression)
    pub fn set_integer_var(&mut self, name: &str, value: &str) -> Result<i64, String> {
        let expanded = crate::utils::expand_env_vars_with(value, |n| self.get_var(n));
        let result = crate::utils::eval_arithmetic(&expanded, |n| self.get_var(n))?;
        self.set_var(name, &result.to_string());
        let mut attrs = self.get_var_attrs(name);
        attrs.integer = true;
        self.set_var_attrs(name, attrs);
        Ok(result)
    }

    /// Push a new local variable scope frame (for function calls with `local`)
    pub fn push_local_scope(&mut self) {
        self.var_attrs_stack.push(HashMap::new());
        self.var_values_stack.push(HashMap::new());
    }

    /// Pop the local variable scope frame, restoring previous values
    pub fn pop_local_scope(&mut self) {
        if let Some(local_attrs) = self.var_attrs_stack.pop() {
            let mut attrs = self.var_attrs.lock().unwrap();
            for (name, saved_attrs) in local_attrs {
                if let Some(current) = attrs.get_mut(&name) {
                    *current = saved_attrs;
                } else {
                    attrs.remove(&name);
                }
            }
        }
        if let Some(local_values) = self.var_values_stack.pop() {
            let mut vars = self.shell_vars.lock().unwrap();
            for (name, saved_value) in local_values {
                if let Some(current) = vars.get_mut(&name) {
                    *current = saved_value;
                } else {
                    vars.remove(&name);
                }
            }
        }
    }

    /// Declare a local variable in the current function scope
    pub fn declare_local(&mut self, name: &str, value: Option<&str>, attrs: VarAttrs) {
        // Save current value and attrs if they exist (for restoration on scope exit)
        let current_attrs = self.get_var_attrs(name);
        if let Some(frame) = self.var_attrs_stack.last_mut() {
            if !frame.contains_key(name) {
                frame.insert(name.to_string(), current_attrs);
            }
        }
        if let Some(frame) = self.var_values_stack.last_mut() {
            if !frame.contains_key(name) {
                if let Some(v) = self.shell_vars.lock().unwrap().get(name).cloned() {
                    frame.insert(name.to_string(), v);
                }
            }
        }
        // Set new value and attrs
        if let Some(v) = value {
            self.set_var(name, v);
        } else {
            // For local without assignment, create empty variable
            self.shell_vars.lock().unwrap().entry(name.to_string()).or_insert_with(String::new);
        }
        let mut new_attrs = attrs;
        new_attrs.local = true;
        self.set_var_attrs(name, new_attrs);
    }

    /// Check if we're in a function scope (have local frames)
    pub fn in_function_scope(&self) -> bool {
        !self.var_attrs_stack.is_empty()
    }

    /// Make a variable readonly
    pub fn make_readonly(&mut self, name: &str) {
        self.readonly_vars.insert(name.to_string());
        let mut attrs = self.get_var_attrs(name);
        attrs.readonly = true;
        self.set_var_attrs(name, attrs);
    }

    /// Export a variable (mark as exported and set in environment)
    pub fn export_var_attrs(&mut self, name: &str, value: Option<&str>) {
        if let Some(v) = value {
            self.shell_vars.lock().unwrap().insert(name.to_string(), v.to_string());
            unsafe {
                env::set_var(name, v);
            }
        } else if let Some(v) = self.shell_vars.lock().unwrap().get(name).cloned() {
            unsafe {
                env::set_var(name, &v);
            }
        }
        self.exported.insert(name.to_string());
        let mut attrs = self.get_var_attrs(name);
        attrs.exported = true;
        self.set_var_attrs(name, attrs);
    }

    /// Detects a leading `NAME=value` assignment word (POSIX-style, no
    /// spaces around `=`). Returns `(name, value)` if `word` is a bare
    /// literal matching that shape.
    pub fn as_assignment(word: &Word) -> Option<(String, Word)> {
        if word.segments.is_empty() {
            return None;
        }
        let WordSegment::Literal(first_str) = &word.segments[0] else {
            return None;
        };
        let eq = first_str.find('=')?;
        if eq == 0 {
            return None;
        }
        let name = &first_str[..eq];
        if !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
            || name.chars().next().unwrap().is_ascii_digit()
        {
            return None;
        }

        let mut val_word = word.clone();
        if let WordSegment::Literal(s) = &mut val_word.segments[0] {
            *s = s[eq + 1..].to_string();
        }
        Some((name.to_string(), val_word))
    }

    /// Expands a single `Word` into one or more resulting strings.
    /// Quoted/single-segment words never glob or split; unquoted words with
    /// glob metacharacters expand against the filesystem.
    pub fn expand_word(&mut self, word: &Word) -> Vec<String> {
        // eprintln!("DEBUG expand_word ENTRY: word.quoted={}, segments={:?}", word.quoted, word.segments);
        let mut out = String::new();
        for seg in &word.segments {
            match seg {
                WordSegment::Literal(s) => out.push_str(s),
                WordSegment::VarExpand(name) => out.push_str(&self.get_var(name)),
                WordSegment::Tilde(s) => {
                    let home = self.home_dir.to_string_lossy();
                    if s == "~" {
                        out.push_str(&home);
                    } else if let Some(rest) = s.strip_prefix("~/") {
                        out.push_str(&home);
                        out.push('/');
                        out.push_str(rest);
                    } else {
                        out.push_str(s);
                    }
                }
                WordSegment::CommandSubst(src) => {
                    out.push_str(&self.run_command_subst(src));
                }
                WordSegment::ParamOp(name, op, w) => {
                    out.push_str(&self.expand_param_op(name, *op, w));
                }
                WordSegment::Arithmetic(expr) => {
                    let expanded_expr = crate::utils::expand_env_vars_with(expr, |name| self.get_var(name));
                    match crate::utils::eval_arithmetic(&expanded_expr, |name| self.get_var(name)) {
                        Ok(val) => out.push_str(&val.to_string()),
                        Err(e) => {
                            if !self.quiet_errors {
                                eprintln!("jesh: $(( {} )): {}", expr.trim(), e);
                            }
                        }
                    }
                }
            }
        }

        if word.quoted {
            return vec![out];
        }

        let braced = crate::utils::expand_braces(&out);
        // eprintln!("DEBUG expand_word: out='{}', braced={:?}", out, braced);
        let mut final_out = Vec::new();
        for item in braced {
            // eprintln!("DEBUG expand_word: trying glob on item='{}'", item);
            if let Some(matches) = self.try_glob(&item) {
                if !matches.is_empty() {
                    final_out.extend(matches);
                    continue;
                }
                // matches is empty - this can happen with nullglob
                // In that case, we don't add the pattern (it expands to nothing)
                continue;
            }
            // try_glob returned None - this means the pattern has glob chars but no matches
            // Check if failglob is enabled
            if self.glob_failglob && item.chars().any(|c| matches!(c, '*' | '?' | '[') || matches!(c, '@' | '+' | '?')) {
                eprintln!("jesh: no matches found: {}", item);
                return vec![];
            }
            final_out.push(item);
        }
        final_out
    }

    /// Expands a `Word` into a single joined string (used where multiple
    /// results/globbing don't make sense, e.g. redirect targets).
    pub fn expand_word_single(&mut self, word: &Word) -> String {
        self.expand_word(word).join(" ")
    }

    fn try_glob(&self, pattern: &str) -> Option<Vec<String>> {
        // Check for Zsh-style glob qualifiers *(X) or *(^X) where X is / . @ *
        let qualifier = {
            let bytes = pattern.as_bytes();
            let len = bytes.len();
            if len >= 4 && bytes[len-1] == b')' {
                if len >= 4 && &pattern[len-4..] == "*(/)" { Some((len-3, Qualifier::Dir)) }
                else if len >= 4 && &pattern[len-4..] == "*(.)" { Some((len-3, Qualifier::File)) }
                else if len >= 4 && &pattern[len-4..] == "*(@)" { Some((len-3, Qualifier::Symlink)) }
                else if len >= 4 && &pattern[len-4..] == "*(*)" { Some((len-3, Qualifier::Exec)) }
                else if len >= 5 && &pattern[len-5..] == "*(^/)" { Some((len-4, Qualifier::NotDir)) }
                else if len >= 5 && &pattern[len-5..] == "*(^.)" { Some((len-4, Qualifier::NotFile)) }
                else if len >= 5 && &pattern[len-5..] == "*(^@)" { Some((len-4, Qualifier::NotSymlink)) }
                else if len >= 5 && &pattern[len-5..] == "*(^*)" { Some((len-4, Qualifier::NotExec)) }
                else { None }
            } else {
                None
            }
        };

        if let Some((end, qual)) = qualifier {
            let inner = &pattern[..end];
            let matches = self.try_glob(inner)?;
            let filtered = self.filter_by_qualifier(matches, qual);
            if filtered.is_empty() {
                if self.glob_nullglob {
                    return Some(vec![]);
                }
                if self.glob_failglob {
                    return None;
                }
                return None;
            }
            return Some(filtered);
        }

        // Check if pattern has any glob metacharacters
        let has_basic_glob = pattern.chars().any(|c| matches!(c, '*' | '?' | '['));
        let has_extglob = self.glob_extglob && pattern.contains('(') && pattern.chars().any(|c| matches!(c, '@' | '*' | '+' | '?' | '!'));

        if !has_basic_glob && !has_extglob {
            return None;
        }

        // Use custom glob matching for extglob patterns
        if has_extglob {
            return self.try_glob_extended(pattern);
        }

        // Basic glob matching - use our custom regex-based approach to support nocaseglob and dotglob
        let (dir_part, base_pattern) = Self::split_pattern(pattern);
        
        let dir = if dir_part.is_empty() {
            std::path::PathBuf::from(".")
        } else {
            std::path::PathBuf::from(&dir_part)
        };

        if !dir.is_dir() {
            return None;
        }

        // For nocaseglob, convert pattern to lowercase
        let pattern_for_matching = if self.glob_nocaseglob {
            base_pattern.to_lowercase()
        } else {
            base_pattern.to_string()
        };

        // Convert glob pattern to regex
        let regex_pattern = Self::glob_to_regex(&pattern_for_matching);
        let regex = regex::Regex::new(&regex_pattern).ok()?;

        let mut results = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                
                // Apply dotglob
                if !self.glob_dotglob && name.starts_with('.') {
                    continue;
                }
                
                // Apply nocaseglob
                let match_name = if self.glob_nocaseglob {
                    name.to_lowercase()
                } else {
                    name.clone()
                };
                
                if regex.is_match(&match_name) {
                    let full_path = if dir_part.is_empty() {
                        name
                    } else {
                        format!("{}/{}", dir_part, name)
                    };
                    results.push(full_path);
                }
            }
        }

        if results.is_empty() {
            if self.glob_nullglob {
                return Some(vec![]);
            }
            if self.glob_failglob {
                return None;
            }
            return None;
        }

        results.sort();
        Some(results)
    }

    fn filter_by_qualifier(&self, results: Vec<String>, qual: Qualifier) -> Vec<String> {
        results.into_iter().filter(|path| {
            let metadata = match std::fs::symlink_metadata(path) {
                Ok(m) => m,
                Err(_) => return false,
            };
            match qual {
                Qualifier::Dir => metadata.is_dir(),
                Qualifier::File => metadata.is_file(),
                Qualifier::Symlink => metadata.file_type().is_symlink(),
                Qualifier::Exec => {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        metadata.permissions().mode() & 0o111 != 0
                    }
                    #[cfg(not(unix))]
                    { false }
                }
                Qualifier::NotDir => !metadata.is_dir(),
                Qualifier::NotFile => !metadata.is_file(),
                Qualifier::NotSymlink => !metadata.file_type().is_symlink(),
                Qualifier::NotExec => {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        metadata.permissions().mode() & 0o111 == 0
                    }
                    #[cfg(not(unix))]
                    { true }
                }
            }
        }).collect()
    }

/// Extended glob matching for patterns like @(a|b), *(a|b), +(a|b), ?(a|b), !(a|b)
    fn try_glob_extended(&self, pattern: &str) -> Option<Vec<String>> {
        // eprintln!("DEBUG try_glob_extended: pattern={}, glob_extglob={}", pattern, self.glob_extglob);
        // Split pattern into directory and basename parts
        let (dir_part, base_pattern) = Self::split_pattern(pattern);
        // eprintln!("DEBUG try_glob_extended: dir_part='{}', base_pattern='{}'", dir_part, base_pattern);
        
        let dir = if dir_part.is_empty() {
            std::path::PathBuf::from(".")
        } else {
            std::path::PathBuf::from(&dir_part)
};

        // eprintln!("DEBUG try_glob_extended: dir.is_dir()={}", dir.is_dir());
        
        if !dir.is_dir() {
            return None;
        }
        
        // For nocaseglob, convert pattern to lowercase
        let pattern_for_matching = if self.glob_nocaseglob {
            base_pattern.to_lowercase()
        } else {
            base_pattern.to_string()
        };
        // eprintln!("DEBUG try_glob_extended: pattern_for_matching='{}', has_bang={}, has_paren={}", pattern_for_matching, pattern_for_matching.contains('!'), pattern_for_matching.contains('('));
        
        // Check if pattern contains negative extglob (!(...))
        if pattern_for_matching.contains('!') && pattern_for_matching.contains('(') {
            return self.try_glob_extended_negative(&dir, &dir_part, &pattern_for_matching);
        }
        
        // Convert extglob pattern to regex for positive patterns (@, *, +, ?)
        let regex_pattern = Self::extglob_to_regex(&pattern_for_matching);
        // eprintln!("DEBUG try_glob_extended: regex_pattern='{}'", regex_pattern);
        let regex = regex::Regex::new(&regex_pattern).ok()?;
        
        let mut results = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                
                // Apply dotglob
                if !self.glob_dotglob && name.starts_with('.') {
                    continue;
                }
                
                // Apply nocaseglob
                let match_name = if self.glob_nocaseglob {
                    name.to_lowercase()
                } else {
                    name.clone()
                };
                // eprintln!("DEBUG try_glob_extended: checking '{}' against regex", match_name);
                
                if regex.is_match(&match_name) {
                    // eprintln!("DEBUG try_glob_extended: MATCH '{}'", name);
                    let full_path = if dir_part.is_empty() {
                        name
                    } else {
                        format!("{}/{}", dir_part, name)
                    };
                    results.push(full_path);
                }
            }
        }
        
        if results.is_empty() {
            // eprintln!("DEBUG try_glob_extended: no matches");
            if self.glob_nullglob {
                return Some(vec![]);
            }
            if self.glob_failglob {
                return None;
            }
            return None;
        }
        
        results.sort();
        Some(results)
    }
    
    /// Handle negative extglob patterns like !(a|b) by evaluating in Rust
    fn try_glob_extended_negative(&self, dir: &std::path::Path, dir_part: &str, pattern: &str) -> Option<Vec<String>> {
        let mut results = Vec::new();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in std::fs::read_dir(dir).ok()?.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                
                // Apply dotglob
                if !self.glob_dotglob && name.starts_with('.') {
                    continue;
                }
                
                // Apply nocaseglob
                let match_name = if self.glob_nocaseglob {
                    name.to_lowercase()
                } else {
                    name.clone()
                };
                
                // Check if the name matches the negative extglob pattern
                if Self::match_negative_extglob(&match_name, pattern) {
                    let full_path = if dir_part.is_empty() {
                        name
                    } else {
                        format!("{}/{}", dir_part, name)
                    };
                    results.push(full_path);
                }
            }
        }
        
        if results.is_empty() {
            if self.glob_nullglob {
                return Some(vec![]);
            }
            if self.glob_failglob {
                return None;
            }
            return None;
        }
        
        results.sort();
        Some(results)
    }
    
    /// Check if a name matches a negative extglob pattern like !(a|b).txt
    fn match_negative_extglob(name: &str, pattern: &str) -> bool {
        // Parse pattern for !(alternatives)suffix format
        // Find the first !(...)
        let mut chars = pattern.chars().peekable();
        let mut prefix = String::new();
        let mut in_negative = false;
        let mut alternatives = Vec::new();
        let mut suffix = String::new();
        let mut paren_depth = 0;
        let mut in_parens = false;
        
        while let Some(c) = chars.next() {
            if c == '!' && chars.peek() == Some(&'(') && !in_parens {
                // Found negative extglob start
                in_negative = true;
                chars.next(); // consume '('
                let mut alt = String::new();
                let mut depth = 1;
                while let Some(c) = chars.next() {
                    if c == '(' {
                        depth += 1;
                        alt.push(c);
                    } else if c == ')' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                        alt.push(c);
                    } else {
                        alt.push(c);
                    }
                }
                // Split alternatives by |
                alternatives = alt.split('|').map(|s| s.to_string()).collect();
                // Rest is suffix
                suffix = chars.collect();
                break;
            } else {
                prefix.push(c);
            }
        }
        
        if in_negative {
            if prefix.is_empty() {
                if suffix.is_empty() {
                    // Pattern is !(alternatives)
                    for alt in &alternatives {
                        if Self::match_simple_pattern(name, alt) {
                            return false;
                        }
                    }
                    return true;
                } else {
                    // Pattern is !(alternatives)suffix
                    if name.ends_with(&suffix) {
                        let stem = &name[..name.len() - suffix.len()];
                        for alt in &alternatives {
                            if Self::match_simple_pattern(stem, alt) {
                                return false;
                            }
                        }
                        return true;
                    }
                    return false;
                }
            }
        }
        
        // Fallback: use regex for complex patterns
        let regex_pattern = Self::extglob_to_regex(pattern);
        if let Ok(regex) = regex::Regex::new(&regex_pattern) {
            regex.is_match(pattern)
        } else {
            false
        }
    }
    
    /// Simple pattern matching for alternatives (supports *, ?, [...])
    fn match_simple_pattern(text: &str, pattern: &str) -> bool {
        // Convert simple glob pattern to regex
        let mut regex_str = String::new();
        regex_str.push('^');
        for c in pattern.chars() {
            match c {
                '*' => regex_str.push_str(".*"),
                '?' => regex_str.push('.'),
                '[' => regex_str.push('['),
                ']' => regex_str.push(']'),
                c if c.is_ascii_punctuation() => regex_str.push_str(&regex::escape(&c.to_string())),
                c => regex_str.push(c),
            }
        }
        regex_str.push('$');
        if let Ok(regex) = regex::Regex::new(&regex_str) {
            regex.is_match(text)
        } else {
            false
        }
    }

    /// Split a pattern into directory part and basename pattern
    fn split_pattern(pattern: &str) -> (String, String) {
        if let Some(pos) = pattern.rfind('/') {
            let (dir, base) = pattern.split_at(pos + 1);
            (dir.to_string(), base.to_string())
        } else {
            (String::new(), pattern.to_string())
        }
    }

    /// Convert extended glob pattern to regex
    fn extglob_to_regex(pattern: &str) -> String {
        let mut result = String::new();
        let mut chars = pattern.chars().peekable();
        
        while let Some(c) = chars.next() {
            match c {
                '@' | '*' | '+' | '?' | '!' if chars.peek() == Some(&'(') => {
                    chars.next(); // consume '('
                    let op = c;
                    let mut inner = String::new();
                    let mut depth = 1;
                    while let Some(ic) = chars.next() {
                        if ic == '(' {
                            depth += 1;
                            inner.push(ic);
                        } else if ic == ')' {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                            inner.push(ic);
                        } else {
                            inner.push(ic);
                        }
                    }
                    // Split by | for alternatives
                    let alternatives: Vec<&str> = inner.split('|').collect();
                    // Convert each alternative to regex (without ^$ anchors)
                    let alt_regex: Vec<String> = alternatives.iter()
                        .map(|a| Self::glob_to_regex_inner(a))
                        .collect();
                    let joined = alt_regex.join("|");
                    
                    match op {
                        '@' => result.push_str(&format!("(?:{})", joined)),       // exactly one
                        '*' => result.push_str(&format!("(?:{})*", joined)),      // zero or more
                        '+' => result.push_str(&format!("(?:{})+", joined)),      // one or more
                        '?' => result.push_str(&format!("(?:{})?", joined)),      // zero or one
                        '!' => result.push_str(&format!("(?:(?!{}).)*", joined)), // not matching (any chars not starting with pattern)
                        _ => {}
                    }
                }
                '[' => {
                    // Character class - pass through to glob_to_regex
                    let mut class = String::new();
                    class.push('[');
                    while let Some(ic) = chars.next() {
                        class.push(ic);
                        if ic == ']' {
                            break;
                        }
                    }
                    result.push_str(&Self::glob_to_regex(&class));
                }
                _ => {
                    // Escape special regex chars, then convert glob chars
                    result.push_str(&Self::glob_char_to_regex(c));
                }
            }
        }
        
        format!("^{}$", result)
    }

    /// Convert extended glob pattern to regex (inner version without ^$ anchors)
    fn extglob_to_regex_inner(pattern: &str) -> String {
        let mut result = String::new();
        let mut chars = pattern.chars().peekable();
        
        while let Some(c) = chars.next() {
            match c {
                '@' | '*' | '+' | '?' | '!' if chars.peek() == Some(&'(') => {
                    chars.next(); // consume '('
                    let op = c;
                    let mut inner = String::new();
                    let mut depth = 1;
                    while let Some(ic) = chars.next() {
                        if ic == '(' {
                            depth += 1;
                            inner.push(ic);
                        } else if ic == ')' {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                            inner.push(ic);
                        } else {
                            inner.push(ic);
                        }
                    }
                    let alternatives: Vec<&str> = inner.split('|').collect();
                    let alt_regex: Vec<String> = alternatives.iter()
                        .map(|a| Self::glob_to_regex_inner(a))
                        .collect();
                    let joined = alt_regex.join("|");
                    
                    match op {
                        '@' => result.push_str(&format!("(?:{})", joined)),
                        '*' => result.push_str(&format!("(?:{})*", joined)),
                        '+' => result.push_str(&format!("(?:{})+", joined)),
                        '?' => result.push_str(&format!("(?:{})?", joined)),
                        '!' => result.push_str(&format!("(?!{}).", joined)),
                        _ => {}
                    }
                }
                '[' => {
                    let mut class = String::new();
                    class.push('[');
                    while let Some(ic) = chars.next() {
                        class.push(ic);
                        if ic == ']' {
                            break;
                        }
                    }
                    result.push_str(&Self::glob_to_regex(&class));
                }
                _ => {
                    result.push_str(&Self::glob_char_to_regex(c));
                }
            }
        }
        
        result
    }

    /// Convert glob pattern to regex (without ^$ anchors, for use in extglob alternatives)
    fn glob_to_regex_inner(pattern: &str) -> String {
        let mut result = String::new();
        for c in pattern.chars() {
            result.push_str(&Self::glob_char_to_regex(c));
        }
        result
    }

    /// Convert a single glob character to regex
    fn glob_char_to_regex(c: char) -> String {
        match c {
            '*' => ".*".to_string(),
            '?' => ".".to_string(),
            '.' | '+' | '^' | '$' | '(' | ')' | '{' | '}' | '|' | '\\' => format!("\\{}", c),
            _ => c.to_string(),
        }
    }

    /// Convert glob pattern to regex (basic glob: *, ?, [...])
    fn glob_to_regex(pattern: &str) -> String {
        let mut result = String::new();
        for c in pattern.chars() {
            result.push_str(&Self::glob_char_to_regex(c));
        }
        format!("^{}$", result)
    }

    /// Runs `src` (raw source text captured from `$(...)`/backticks) as a
    /// nested script and returns its stdout with trailing newlines trimmed.
    fn run_command_subst(&mut self, src: &str) -> String {
        let tokens = crate::parser::lexer::tokenize(src);
        let list = crate::parser::parser::parse(tokens);
        // Reuse the current process env/shell vars by spawning through the
        // same expansion path, but capture stdout instead of inheriting it.
        let mut all_output = Vec::new();
        for (andor, _op) in &list.items {
            let expanded = self.expand_pipeline(&andor.pipeline, None);
            let output = crate::executor::pipeline::execute_capture(expanded);
            all_output.extend(output);
        }
        let mut s = String::from_utf8_lossy(&all_output).into_owned();
        while s.ends_with('\n') {
            s.pop();
        }
        s
    }

    /// Expands every word of every command in a `Pipeline` into an
    /// `ExpandedPipeline` ready for the executor. `heredoc_body` is attached
    /// to whichever command declared a heredoc redirect, if any.
    pub fn expand_pipeline(
        &mut self,
        pipeline: &crate::parser::Pipeline,
        heredoc_body: Option<&str>,
    ) -> crate::parser::ExpandedPipeline {
        use crate::parser::lexer::RedirectTarget;
        use crate::parser::ExpandedCommand;

        let mut commands = Vec::new();
        for cmd in &pipeline.commands {
            let mut words = self.expand_word(&cmd.program);
            for a in &cmd.args {
                words.extend(self.expand_word(a));
            }
            if words.is_empty() {
                continue;
            }

            // Resolve aliases on the program name only (first command word).
            let (program, mut rest) = {
                let mut w = words;
                let program = w.remove(0);
                (program, w)
            };
            let (program, mut rest) = self.resolve_alias(program, rest.drain(..).collect());

            let mut final_words = vec![program];
            final_words.append(&mut rest);

            let redirects: Vec<_> = cmd
                .redirects
                .iter()
                .map(|r| crate::parser::lexer::Redirect {
                    fd: r.fd,
                    append: r.append,
                    target: match &r.target {
                        RedirectTarget::File(p) => {
                            let expanded = self.expand_str(p);
                            // Re-check after expansion: `>$FD` where FD="&1" or FD="&-"
                            // should be treated as an fd redirect, not a file path.
                            if let Some(stripped) = expanded.strip_prefix('&') {
                                if stripped == "-" {
                                    RedirectTarget::Close(0)
                                } else if let Ok(n) = stripped.parse::<i32>() {
                                    RedirectTarget::Fd(n)
                                } else {
                                    RedirectTarget::File(expanded)
                                }
                            } else {
                                RedirectTarget::File(expanded)
                            }
                        }
                        RedirectTarget::Fd(n) => RedirectTarget::Fd(*n),
                        RedirectTarget::Heredoc(d, strip) => RedirectTarget::Heredoc(d.clone(), *strip),
                        RedirectTarget::HereString(s) => {
                            RedirectTarget::HereString(self.expand_str(s))
                        }
                        RedirectTarget::ProcessSubst(cmd, is_input) => {
                            RedirectTarget::ProcessSubst(self.expand_str(cmd), *is_input)
                        }
                        RedirectTarget::Close(_) => RedirectTarget::Close(0),
                        RedirectTarget::Dynamic(name) => RedirectTarget::Dynamic(name.clone()),
                        RedirectTarget::LazyWord(segs) => {
                            // Fully expand the word segments and re-classify
                            let word = Word { segments: segs.clone(), quoted: false };
                            let expanded = self.expand_word_single(&word);
                            if let Some(stripped) = expanded.strip_prefix('&') {
                                if stripped == "-" {
                                    RedirectTarget::Close(0)
                                } else if let Ok(n) = stripped.parse::<i32>() {
                                    RedirectTarget::Fd(n)
                                } else {
                                    RedirectTarget::File(expanded)
                                }
                            } else {
                                RedirectTarget::File(expanded)
                            }
                        }
                    },
                    dyn_var: r.dyn_var.clone(),
                })
                .collect();

            let is_heredoc = redirects
                .iter()
                .any(|r| matches!(r.target, RedirectTarget::Heredoc(..)));

            let expanded_env_vars: Vec<(String, String)> = cmd
                .env_vars
                .iter()
                .map(|(k, v)| (k.clone(), self.expand_word_single(v)))
                .collect();

            commands.push(ExpandedCommand {
                program: final_words.remove(0),
                args: final_words,
                env_vars: expanded_env_vars,
                redirects,
                heredoc: if is_heredoc {
                    heredoc_body.map(|s| self.expand_str(s))
                } else {
                    None
                },
            });
        }

        crate::parser::ExpandedPipeline { commands }
    }

    /// Expands `$VAR`/`~` occurring in a plain string (used for redirect
    /// targets, which come from the lexer as flattened literals that may
    /// still contain `$NAME` placeholders).
    fn expand_str(&self, s: &str) -> String {
        let expanded = crate::utils::expand_env_vars_with(s, |name| self.get_var(name));
        crate::utils::expand_tilde_with(&expanded, &self.home_dir.to_string_lossy())
    }

    fn resolve_alias(&self, program: String, rest: Vec<String>) -> (String, Vec<String>) {
        let map = self.aliases.lock().unwrap();
        if let Some(alias_val) = map.get(&program) {
            let alias_words: Vec<String> = alias_val.split_whitespace().map(|s| s.to_string()).collect();
            if !alias_words.is_empty() {
                let mut new_rest = alias_words[1..].to_vec();
                new_rest.extend(rest);
                return (alias_words[0].clone(), new_rest);
            }
        }
        (program, rest)
    }

    fn get_git_branch(&self) -> Option<String> {
        let output = Command::new("git")
            .args(["symbolic-ref", "--short", "HEAD"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .ok()?;
        if output.status.success() {
            let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !branch.is_empty() {
                return Some(branch);
            }
        }

        let mut dir = env::current_dir().ok()?;
        loop {
            let git_dir = dir.join(".git");
            if git_dir.is_dir() {
                let head_file = git_dir.join("HEAD");
                if head_file.is_file() {
                    if let Ok(content) = fs::read_to_string(head_file) {
                        let content = content.trim();
                        if content.starts_with("ref: refs/heads/") {
                            return Some(content.strip_prefix("ref: refs/heads/").unwrap().to_string());
                        } else if content.starts_with("ref: refs/tags/") {
                            return Some(content.strip_prefix("ref: refs/tags/").unwrap().to_string());
                        } else if !content.is_empty() {
                            return Some("HEAD".to_string());
                        }
                    }
                }
                break;
            }
            if !dir.pop() {
                break;
            }
        }
        None
    }

    fn get_current_dir_short(&self) -> String {
        let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        if current_dir == self.home_dir {
            return "~".to_string();
        }
        current_dir
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "/".to_string())
    }

    fn is_ssh(&self) -> bool {
        env::var("SSH_CLIENT").is_ok() || env::var("SSH_TTY").is_ok() || env::var("SSH_CONNECTION").is_ok()
    }

    /// Detect the current distro/OS.
    /// Returns (id, id_like, name). On Linux it reads `/etc/os-release`.
    fn detect_distro(&self) -> (String, String, String) {
        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = fs::read_to_string("/etc/os-release") {
                let mut id = String::new();
                let mut id_like = String::new();
                let mut name = String::new();
                for line in content.lines() {
                    if let Some(v) = line.strip_prefix("ID=") {
                        id = v.trim_matches('"').to_string();
                    } else if let Some(v) = line.strip_prefix("ID_LIKE=") {
                        id_like = v.trim_matches('"').to_string();
                    } else if let Some(v) = line.strip_prefix("NAME=") {
                        name = v.trim_matches('"').to_string();
                    }
                }
                if !id.is_empty() || !id_like.is_empty() {
                    return (id, id_like, name);
                }
            }
            return ("linux".to_string(), String::new(), "Linux".to_string());
        }
        #[cfg(target_os = "macos")]
        {
            return ("macos".to_string(), String::new(), "macOS".to_string());
        }
        #[cfg(target_os = "windows")]
        {
            return ("windows".to_string(), String::new(), "Windows".to_string());
        }
        #[allow(unreachable_code)]
        ("linux".to_string(), String::new(), "Linux".to_string())
    }

    /// Map a distro/OS id to an emoji logo.
    fn logo_for(candidate: &str) -> Option<&'static str> {
        Some(match candidate {
            "macos" => "🍎",
            "windows" => "🪟",
            // Everything else recognized here is a Linux distro id/id_like:
            // one shared penguin, since that's what actually identifies the
            // kernel/OS family the way 🍎/🪟 do for the others.
            "zorin" | "ubuntu" | "linuxmint" | "mint" | "elementary" | "pop" | "pop_os"
            | "arch" | "archarm" | "manjaro" | "endeavouros" | "endeavour" | "fedora"
            | "debian" | "raspbian" | "opensuse" | "opensuse-leap" | "opensuse-tumbleweed"
            | "gentoo" | "void" | "alpine" | "centos" | "rhel" | "kali" | "linux" => "🐧",
            _ => return None,
        })
    }

    /// Returns the OS logo (emoji) for the running system, or the value of
    /// `PROMPT_ICON` if the user has set that override.
    fn os_logo(&mut self) -> String {
        // Check for user override
        let override_icon = self.get_var("PROMPT_ICON");
        if !override_icon.is_empty() {
            return override_icon;
        }

        // Return cached logo if available (avoid repeated /etc/os-release reads)
        if let Some(ref logo) = self.cached_os_logo {
            return logo.clone();
        }

        // Detect and cache the logo
        let (id, id_like, _name) = self.detect_distro();
        let mut candidates: Vec<String> = vec![id];
        candidates.extend(id_like.split_whitespace().map(|s| s.to_string()));

        let logo = candidates
            .iter()
            .find_map(|c| Self::logo_for(c))
            .unwrap_or("🐧")
            .to_string();

        self.cached_os_logo = Some(logo.clone());
        logo
    }

    pub fn render_rprompt(&mut self) -> Option<(String, String)> {
        let mut parts_styled = Vec::new();
        let mut parts_plain = Vec::new();

        if self.last_exit_status != 0 {
            let colored = self.apply_theme_color(&self.last_exit_status.to_string(), "JSH_THEME_RPROMPT_ERR_COLOR");
            parts_styled.push(format!("✘ {} ", if colored != self.last_exit_status.to_string() { colored } else { self.last_exit_status.to_string().red().to_string() }));
            parts_plain.push(format!("✘ {} ", self.last_exit_status));
        }

        if self.is_ssh() {
            let user = env::var("USER").unwrap_or_else(|_| "user".to_string());
            let host = env::var("HOSTNAME").unwrap_or_else(|_| "host".to_string());
            let user_styled = self.apply_theme_color(&user, "JSH_THEME_RPROMPT_SSH_USER_COLOR");
            let host_styled = self.apply_theme_color(&host, "JSH_THEME_RPROMPT_SSH_HOST_COLOR");
            let user_final = if user_styled != user { user_styled } else { user.clone().bold().magenta().to_string() };
            let host_final = if host_styled != host { host_styled } else { host.clone().bold().cyan().to_string() };
            parts_styled.push(format!("{}@{} 🔐 ", user_final, host_final));
            parts_plain.push(format!("{}@{} 🔐 ", user, host));
        }

        if !parts_styled.is_empty() {
            Some((parts_styled.join(""), parts_plain.join("")))
        } else {
            None
        }
    }

    fn parse_theme_color(&self, name: &str) -> Option<Color> {
        let val = self.get_var(name);
        if val.is_empty() {
            return None;
        }
        match val.to_lowercase().as_str() {
            "black" => Some(Color::Black),
            "red" => Some(Color::DarkRed),
            "green" => Some(Color::DarkGreen),
            "yellow" => Some(Color::DarkYellow),
            "blue" => Some(Color::DarkBlue),
            "magenta" => Some(Color::DarkMagenta),
            "cyan" => Some(Color::DarkCyan),
            "white" => Some(Color::Grey),
            "darkred" | "dark_red" => Some(Color::DarkRed),
            "darkgreen" | "dark_green" => Some(Color::DarkGreen),
            "darkyellow" | "dark_yellow" => Some(Color::DarkYellow),
            "darkblue" | "dark_blue" => Some(Color::DarkBlue),
            "darkmagenta" | "dark_magenta" => Some(Color::DarkMagenta),
            "darkcyan" | "dark_cyan" => Some(Color::DarkCyan),
            "grey" | "gray" => Some(Color::Grey),
            _ => None,
        }
    }

    fn apply_theme_color(&self, text: &str, var_name: &str) -> String {
        if let Some(color) = self.parse_theme_color(var_name) {
            text.with(color).to_string()
        } else {
            text.to_string()
        }
    }

    pub fn load_theme(&mut self) {
        let theme = self.get_var("THEME");
        if theme.is_empty() {
            return;
        }
        let home = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let data_home = env::var("XDG_DATA_HOME").unwrap_or_else(|_| format!("{}/.local/share", home));
        let search_dirs = [
            PathBuf::from(&data_home).join("jesh/themes"),
            PathBuf::from(&home).join(".local/jesh/themes"),
        ];
        for dir in &search_dirs {
            let theme_file = dir.join(format!("{}.sh", theme));
            if theme_file.exists() {
                if let Ok(content) = fs::read_to_string(&theme_file) {
                    let script = format!("JSH_THEME_LOADED=1\n{}", content);
                    self.run_script_text(&script);
                }
                return;
            }
        }
    }


    pub fn render_prompt(&mut self) -> String {
        crate::utils::emit_osc7();

        let err_color = |s: &str| -> String {
            let c = self.apply_theme_color(s, "JSH_THEME_STATUS_ERR_COLOR");
            if c != s { c } else { s.red().to_string() }
        };

        let status_part = if self.last_exit_status == 0 {
            "".to_string()
        } else {
            format!("✘{}", err_color(&self.last_exit_status.to_string()))
        };

        let ssh_part = if self.is_ssh() {
            let user = env::var("USER").unwrap_or_else(|_| "user".to_string());
            let host = env::var("HOSTNAME").unwrap_or_else(|_| "host".to_string());
            let user_styled = self.apply_theme_color(&user, "JSH_THEME_SSH_USER_COLOR");
            let host_styled = self.apply_theme_color(&host, "JSH_THEME_SSH_HOST_COLOR");
            let u = if user_styled != user { user_styled } else { user.bold().magenta().to_string() };
            let h = if host_styled != host { host_styled } else { host.bold().cyan().to_string() };
            format!("{}@{} 🔐", u, h)
        } else {
            "".to_string()
        };

        let dir_styled = self.apply_theme_color(&self.get_current_dir_short(), "JSH_THEME_DIR_COLOR");
        let dir_final = if dir_styled != self.get_current_dir_short() { dir_styled } else { self.get_current_dir_short().bold().magenta().to_string() };

        let prompt_styled = self.apply_theme_color(">", "JSH_THEME_PROMPT_COLOR");
        let prompt_final = if prompt_styled != ">" { prompt_styled } else { ">".magenta().to_string() };

        let mut base_parts = Vec::new();
        if !status_part.is_empty() {
            base_parts.push(status_part.trim().to_string());
        }
        if !ssh_part.is_empty() {
            base_parts.push(ssh_part.trim().to_string());
        }
        let logo = self.os_logo();
        if !logo.is_empty() {
            base_parts.push(logo);
        }
        base_parts.push(dir_final);
        if let Some(branch) = self.get_git_branch() {
            let styled = self.apply_theme_color(&branch, "JSH_THEME_GIT_COLOR");
            let b = if styled != branch { styled } else { branch.green().to_string() };
            base_parts.push(b);
        }
        base_parts.push(prompt_final);

        base_parts.join(" ") + " "
    }

    pub fn render_prompt_clean(&mut self) -> String {
        let status_part = if self.last_exit_status == 0 {
            "".to_string()
        } else {
            format!("✘ {}", self.last_exit_status)
        };

        let ssh_part = if self.is_ssh() {
            let user = env::var("USER").unwrap_or_else(|_| "user".to_string());
            let host = env::var("HOSTNAME").unwrap_or_else(|_| "host".to_string());
            format!("{}@{} 🔐", user, host)
        } else {
            "".to_string()
        };

        let mut base_parts = Vec::new();
        if !status_part.is_empty() {
            base_parts.push(status_part.trim().to_string());
        }
        if !ssh_part.is_empty() {
            base_parts.push(ssh_part.trim().to_string());
        }
        let logo = self.os_logo();
        if !logo.is_empty() {
            base_parts.push(logo);
        }
        base_parts.push(self.get_current_dir_short());
        if let Some(branch) = self.get_git_branch() {
            base_parts.push(branch);
        }
        base_parts.push(">".to_string());

        base_parts.join(" ") + " "
    }

    pub fn get_git_branch_for(path: &str) -> Option<String> {
        let output = Command::new("git")
            .args(["symbolic-ref", "--short", "HEAD"])
            .current_dir(path)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .ok()?;
        if output.status.success() {
            let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !branch.is_empty() {
                return Some(branch);
            }
        }

        let mut dir = PathBuf::from(path);
        loop {
            let git_dir = dir.join(".git");
            if git_dir.is_dir() {
                let head_file = git_dir.join("HEAD");
                if head_file.is_file() {
                    if let Ok(content) = fs::read_to_string(head_file) {
                        let content = content.trim();
                        if content.starts_with("ref: refs/heads/") {
                            return Some(content.strip_prefix("ref: refs/heads/").unwrap().to_string());
                        } else if content.starts_with("ref: refs/tags/") {
                            return Some(content.strip_prefix("ref: refs/tags/").unwrap().to_string());
                        } else if !content.is_empty() {
                            return Some("HEAD".to_string());
                        }
                    }
                }
                break;
            }
            if !dir.pop() {
                break;
            }
        }
        None
    }
}
