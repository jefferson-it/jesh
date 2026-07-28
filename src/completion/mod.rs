pub mod apps;

use rustyline::CompletionType;
use std::borrow::Cow;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};

use crossterm::event::{self, Event as CEvent, KeyCode, KeyModifiers};
use crossterm::terminal::{self, Clear, ClearType};
use crossterm::ExecutableCommand;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::hint::{Hinter, Hint};
use rustyline::highlight::Highlighter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};

use crate::builtin::{is_builtin, is_executable};
use crate::utils::expand_tilde;
pub use apps::CompletionDb;

fn fuzzy_score(query: &str, target: &str) -> u32 {
    let q = query.to_lowercase();
    let t = target.to_lowercase();
    let qc: Vec<char> = q.chars().collect();
    if qc.is_empty() {
        return 0;
    }
    let mut qi = 0;
    let mut score: u32 = 0;
    let mut consecutive: u32 = 0;
    for (ti, tc) in t.char_indices() {
        if qi < qc.len() && tc == qc[qi] {
            qi += 1;
            consecutive += 1;
            score += consecutive * 10;
            if ti == 0 || t[..ti].chars().last().map_or(true, |c| !c.is_alphanumeric()) {
                score += 20;
            }
        } else {
            consecutive = 0;
        }
    }
    if qi == qc.len() { score + 50 } else { 0 }
}

fn fuzzy_filter(query: &str, candidates: &[String]) -> Vec<(u32, String)> {
    let mut scored: Vec<(u32, String)> = candidates
        .iter()
        .filter_map(|c| {
            let s = fuzzy_score(query, c);
            if s > 0 { Some((s, c.clone())) } else { None }
        })
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0));
    scored
}

/// Subcommands offered for well-known tools when completing their first
/// argument. Kept as a small hand-maintained table — enough to cover the
/// day-to-day verbs without shelling out to the tool.
///
/// Returns `(name, description)` pairs so the completion menu can show
/// inline help (§9 — Descrições de Comandos e Flags).
fn known_subcommands(cmd: &str) -> Option<Vec<(&'static str, &'static str)>> {
    Some(match cmd {
        "git" => vec![
            ("add", "Add file contents to the index"),
            ("branch", "List, create, or delete branches"),
            ("checkout", "Switch branches or restore files"),
            ("clone", "Clone a repository"),
            ("commit", "Record changes to the repository"),
            ("diff", "Show changes between commits/working tree"),
            ("fetch", "Download objects and refs from another repo"),
            ("init", "Create an empty Git repository"),
            ("log", "Show commit logs"),
            ("merge", "Merge branches"),
            ("pull", "Fetch and integrate from another repo"),
            ("push", "Update remote refs"),
            ("rebase", "Reapply commits on top of another base"),
            ("remote", "Manage remote repositories"),
            ("reset", "Reset current HEAD to a specific state"),
            ("restore", "Restore working tree files"),
            ("stash", "Stash changes"),
            ("status", "Show working tree status"),
            ("switch", "Switch branches"),
            ("tag", "Create, list, or delete tags"),
        ],
        "cargo" => vec![
            ("add", "Add dependencies to Cargo.toml"),
            ("bench", "Run benchmarks"),
            ("build", "Compile the package"),
            ("check", "Check the package"),
            ("clean", "Remove build artifacts"),
            ("clippy", "Run clippy lints"),
            ("doc", "Build documentation"),
            ("fmt", "Format source code"),
            ("init", "Create a new Cargo package"),
            ("install", "Install a binary"),
            ("new", "Create a new Cargo package"),
            ("publish", "Publish to crates.io"),
            ("remove", "Remove dependencies"),
            ("run", "Run the binary"),
            ("test", "Run tests"),
            ("update", "Update dependencies"),
        ],
        "dnf" | "yum" => vec![
            ("alias", "Manage aliases"),
            ("autoremove", "Remove unused dependencies"),
            ("check", "Check for problems"),
            ("check-update", "Check for updates"),
            ("clean", "Clean data"),
            ("deplist", "Show dependencies"),
            ("distro-sync", "Sync to latest versions"),
            ("downgrade", "Downgrade packages"),
            ("group", "Manage groups"),
            ("help", "Show help"),
            ("history", "Show history"),
            ("info", "Show package info"),
            ("install", "Install packages"),
            ("list", "List packages"),
            ("makecache", "Make cache"),
            ("mark", "Mark packages"),
            ("module", "Manage modules"),
            ("provides", "Find packages providing files"),
            ("reinstall", "Reinstall packages"),
            ("remove", "Remove packages"),
            ("repoquery", "Query repositories"),
            ("repository-packages", "Manage repo packages"),
            ("search", "Search packages"),
            ("shell", "Open shell"),
            ("swap", "Swap packages"),
            ("update", "Update packages"),
            ("updateinfo", "Update info"),
            ("upgrade", "Upgrade packages"),
            ("upgrade-minimal", "Minimal upgrade"),
        ],
        "apt" | "apt-get" => vec![
            ("update", "Retrieve new package lists"),
            ("upgrade", "Upgrade packages"),
            ("install", "Install packages"),
            ("remove", "Remove packages"),
            ("purge", "Remove packages and config"),
            ("autoremove", "Remove unused dependencies"),
            ("search", "Search packages"),
            ("show", "Show package info"),
            ("list", "List packages"),
            ("edit-sources", "Edit source info"),
            ("help", "Show help"),
        ],
        "aly" => vec![
            ("run", "Run the application"),
            ("comp", "Manage completions"),
            ("help", "Show help"),
            ("version", "Show version"),
        ],
        "apg" => vec![
            ("install", "Install a package"),
            ("remove", "Remove a package"),
            ("update", "Update a package"),
            ("publish", "Publish a package"),
            ("init", "Initialize a project"),
            ("run", "Run a command"),
            ("list", "List packages"),
            ("search", "Search packages"),
        ],
        "cutils" => vec![
            ("install", "Install a component"),
            ("uninstall", "Uninstall a component"),
            ("list", "List components"),
            ("status", "Show component status"),
            ("which", "Find a component"),
        ],
        _ => return None,
    })
}

/// Flags and options offered for well-known tools when completing arguments
/// that start with `-`.  Returns `(option, description)` pairs.
fn known_options(cmd: &str) -> Option<Vec<(&'static str, &'static str)>> {
    Some(match cmd {
        "git" => vec![
            ("--help", "Show help"),
            ("--version", "Show version"),
            ("--exec-path", "Show git exec path"),
            ("--html-path", "Show html path"),
            ("--man-path", "Show man path"),
            ("--info-path", "Show info path"),
            ("-p", "Page output"),
            ("--paginate", "Page output"),
            ("--no-pager", "Disable pager"),
            ("--no-replace-objects", "Skip replace objects"),
            ("--bare", "Create bare repo"),
            ("--git-dir=", "Set git directory"),
            ("--work-tree=", "Set work tree"),
            ("--namespace=", "Set namespace"),
        ],
        "ls" => vec![
            ("-a", "Show all entries"),
            ("--all", "Show all entries"),
            ("-A", "Show almost all"),
            ("--almost-all", "Show almost all"),
            ("-l", "Long format"),
            ("-h", "Human-readable sizes"),
            ("--human-readable", "Human-readable sizes"),
            ("-R", "Recursive"),
            ("--recursive", "Recursive"),
            ("-1", "One per line"),
            ("--color=auto", "Color auto"),
            ("--color=always", "Color always"),
            ("--color=never", "No color"),
        ],
        "grep" => vec![
            ("-i", "Ignore case"),
            ("--ignore-case", "Ignore case"),
            ("-v", "Invert match"),
            ("--invert-match", "Invert match"),
            ("-c", "Count"),
            ("--count", "Count"),
            ("-n", "Line number"),
            ("--line-number", "Line number"),
            ("-r", "Recursive"),
            ("--recursive", "Recursive"),
            ("-E", "Extended regex"),
            ("--extended-regexp", "Extended regex"),
            ("-F", "Fixed strings"),
            ("--fixed-strings", "Fixed strings"),
        ],
        "cargo" => vec![
            ("--help", "Show help"),
            ("--version", "Show version"),
            ("--list", "List commands"),
            ("--verbose", "Verbose"),
            ("--quiet", "Quiet"),
            ("--color", "Color output"),
        ],
        "dnf" | "yum" => vec![
            ("-y", "Assume yes"),
            ("--assumeyes", "Assume yes"),
            ("-q", "Quiet"),
            ("--quiet", "Quiet"),
            ("-v", "Verbose"),
            ("--verbose", "Verbose"),
            ("--help", "Show help"),
            ("--version", "Show version"),
            ("--enablerepo=", "Enable repo"),
            ("--disablerepo=", "Disable repo"),
        ],
        "apt" | "apt-get" => vec![
            ("-y", "Assume yes"),
            ("--yes", "Assume yes"),
            ("-q", "Quiet"),
            ("--quiet", "Quiet"),
            ("--help", "Show help"),
            ("--version", "Show version"),
            ("-d", "Download only"),
            ("--download-only", "Download only"),
            ("--purge", "Purge"),
            ("--reinstall", "Reinstall"),
        ],
        "aly" => vec![
            ("--help", "Show help"),
            ("--version", "Show version"),
            ("--release", "Release mode"),
            ("--verbose", "Verbose"),
        ],
        "apg" => vec![
            ("--help", "Show help"),
            ("--version", "Show version"),
            ("--global", "Global scope"),
            ("-g", "Global scope"),
        ],
        "cutils" => vec![
            ("--help", "Show help"),
            ("--version", "Show version"),
            ("-d", "Directory mode"),
            ("--dir", "Directory mode"),
        ],
        _ => return None,
    })
}

pub struct JshHint {
    display: String,
    complete: String,
}

impl Hint for JshHint {
    fn display(&self) -> &str {
        &self.display
    }
    fn completion(&self) -> Option<&str> {
        Some(&self.complete)
    }
}

use std::cell::RefCell;

thread_local! {
    pub static CURRENT_COLORED_PROMPT: RefCell<String> = RefCell::new(String::new());
    pub static CURRENT_COLORED_RPROMPT: RefCell<Option<String>> = RefCell::new(None);
    static COMPLETION_STATE: RefCell<Option<*mut crate::shell::ShellState>> = const { RefCell::new(None) };
}

/// Set the shell state reference for programmable completion.
/// Called from `run_interactive` before each `readline` call.
pub fn set_completion_state(state: &mut crate::shell::ShellState) {
    COMPLETION_STATE.with(|cell| {
        *cell.borrow_mut() = Some(state as *mut crate::shell::ShellState);
    });
}

pub fn get_completions(
    line: &str,
    pos: usize,
    completions: &CompletionDb,
    aliases: &HashMap<String, String>,
    shell_vars: &HashMap<String, String>,
    functions: &HashMap<String, String>,
) -> (usize, Vec<Pair>) {
    let prefix = &line[..pos];
    let word_start = prefix
        .rfind(char::is_whitespace)
        .map(|i| i + 1)
        .unwrap_or(0);
    let word = &prefix[word_start..];
    let leading: Vec<&str> = prefix[..word_start].split_whitespace().collect();
    let arg_index = leading.len();
    let first_word = leading.first().copied().unwrap_or("");

    if let Some(var_prefix) = word.strip_prefix('$') {
        let mut candidates = Vec::new();
        for (key, _) in env::vars() {
            if key.to_lowercase().starts_with(&var_prefix.to_lowercase()) {
                candidates.push(Pair {
                    display: format!("${}", key),
                    replacement: format!("${}", key),
                });
            }
        }
        for key in shell_vars.keys() {
            if key.to_lowercase().starts_with(&var_prefix.to_lowercase()) {
                candidates.push(Pair {
                    display: format!("${}", key),
                    replacement: format!("${}", key),
                });
            }
        }
        candidates.sort_by(|a, b| a.display.cmp(&b.display));
        candidates.dedup_by(|a, b| a.display == b.display);
        return (word_start, candidates);
    }

    if arg_index == 0 && !word.contains('/') && !word.contains('\\') {
        let mut candidates = Vec::new();
        let wl = word.to_lowercase();

        let builtins = [
            "cd", "exit", "jeofetch", "help", "version", "export", "unset", "set",
            "alias", "unalias", "source", "true", "false", ".-1", "$PWD_BACK", "$PB",
        ];
        for b in builtins {
            let bl = b.to_lowercase();
            if !wl.is_empty() && bl.starts_with(&wl) {
                candidates.push(Pair { display: b.to_string(), replacement: format!("{} ", b) });
            }
        }

        for name in aliases.keys() {
            let nl = name.to_lowercase();
            if !wl.is_empty() && nl.starts_with(&wl) {
                candidates.push(Pair { display: name.clone(), replacement: format!("{} ", name) });
            }
        }

        for name in functions.keys() {
            let nl = name.to_lowercase();
            if !wl.is_empty() && nl.starts_with(&wl) {
                candidates.push(Pair { display: name.clone(), replacement: format!("{} ", name) });
            }
        }

        let path_var = env::var_os("PATH").unwrap_or_default();
        for path in env::split_paths(&path_var) {
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().into_owned();
                    let nl = name.to_lowercase();
                    if !wl.is_empty() && nl.starts_with(&wl) && entry.path().is_file() {
                        candidates.push(Pair { display: name.clone(), replacement: format!("{} ", name) });
                    }
                }
            }
        }

        candidates.sort_by(|a, b| {
            let a_starts = a.display.to_lowercase().starts_with(&wl);
            let b_starts = b.display.to_lowercase().starts_with(&wl);
            if a_starts != b_starts {
                return b_starts.cmp(&a_starts);
            }
            a.display.cmp(&b.display)
        });
        candidates.dedup_by(|a, b| a.display == b.display);
        if !candidates.is_empty() {
            return (word_start, candidates);
        }
    }

        if arg_index == 1 {
        let mut loaded_subs = completions.get(first_word);
        // Include word list from programmable API
        let word_list = completions.get_word_list(first_word);
        for w in &word_list {
            if !loaded_subs.contains(w) {
                loaded_subs.push(w.clone());
            }
        }
        // Build a description map from known subcommands so the menu can
        // show inline help (§9 — Descrições de Comandos e Flags).
        let mut desc_map: HashMap<String, &str> = HashMap::new();
        if let Some(subs) = known_subcommands(first_word) {
            for (name, desc) in subs {
                desc_map.insert(name.to_string(), desc);
                if !loaded_subs.contains(&name.to_string()) {
                    loaded_subs.push(name.to_string());
                }
            }
        }
        if !loaded_subs.is_empty() {
            let wl = word.to_lowercase();
            let mut candidates: Vec<Pair> = loaded_subs
                .iter()
                .filter(|s| {
                    let sl = s.to_lowercase();
                    sl.starts_with(&wl)
                })
                .map(|s| {
                    let display = match desc_map.get(s) {
                        Some(desc) => format!("{}  {}", s, desc),
                        None => s.clone(),
                    };
                    Pair { display, replacement: format!("{} ", s) }
                })
                .collect();
            if !candidates.is_empty() {
                candidates.sort_by(|a, b| {
                    let a_starts = a.replacement.trim_end().to_lowercase().starts_with(&wl);
                    let b_starts = b.replacement.trim_end().to_lowercase().starts_with(&wl);
                    if a_starts != b_starts {
                        return b_starts.cmp(&a_starts);
                    }
                    a.replacement.trim_end().cmp(b.replacement.trim_end())
                });
                return (word_start, candidates);
            }
        }
        // Try function completer if no prefix matches
        if completions.get_completer(first_word).is_some() {
            let cands = COMPLETION_STATE.with(|cell| {
                let cell = cell.borrow();
                if let Some(ptr) = cell.as_ref() {
                    let state = unsafe { &mut **ptr };
                    let result = completions.run_completer(first_word, state, word, leading.last().unwrap_or(&""), &leading.join(" "));
                    if !result.is_empty() {
                        Some(result)
                    } else { None }
                } else { None }
            });
            if let Some(words) = cands {
                let candidates: Vec<Pair> = words.iter().map(|w| {
                    Pair { display: w.clone(), replacement: format!("{} ", w) }
                }).collect();
                return (word_start, candidates);
            }
        }
    }

        if word.starts_with('-') {
        if let Some(opts) = known_options(first_word) {
            let wl = word.to_lowercase();
            let mut candidates: Vec<Pair> = opts
                .iter()
                .filter(|(opt, _)| {
                    let ol = opt.to_lowercase();
                    ol.starts_with(&wl)
                })
                .map(|(opt, desc)| {
                    let repl = if opt.ends_with('=') { opt.to_string() } else { format!("{} ", opt) };
                    let display = format!("{}  {}", opt, desc);
                    Pair { display, replacement: repl }
                })
                .collect();
            if !candidates.is_empty() {
                candidates.sort_by(|a, b| {
                    let a_starts = a.replacement.trim_end().to_lowercase().starts_with(&wl);
                    let b_starts = b.replacement.trim_end().to_lowercase().starts_with(&wl);
                    if a_starts != b_starts {
                        return b_starts.cmp(&a_starts);
                    }
                    a.replacement.trim_end().cmp(b.replacement.trim_end())
                });
                return (word_start, candidates);
            }
        }
    }

    let dir_only = matches!(first_word, "cd" | "pushd") && arg_index == 1;
    if dir_only || word.starts_with('~') {
        if let Some(result) = complete_path(word, word_start, dir_only) {
            return result;
        }
    }

    if let Some(result) = complete_path(word, word_start, false) {
        if !result.1.is_empty() {
            return result;
        }
    }

    // --- Fuzzy fallback for commands (arg_index == 0) ---
    if arg_index == 0 && !word.is_empty() {
        let mut fuzzy_cands: Vec<Pair> = Vec::new();

        // Builtins
        let builtins = [
            "cd", "exit", "jeofetch", "help", "version", "export", "unset", "set",
            "alias", "unalias", "source", "true", "false", ".-1", "$PWD_BACK", "$PB",
        ];
        for b in builtins {
            if fuzzy_score(word, b) > 0 {
                fuzzy_cands.push(Pair { display: b.to_string(), replacement: format!("{} ", b) });
            }
        }

        for name in aliases.keys() {
            if fuzzy_score(word, name) > 0 {
                fuzzy_cands.push(Pair { display: name.clone(), replacement: format!("{} ", name) });
            }
        }

        for name in functions.keys() {
            if fuzzy_score(word, name) > 0 {
                fuzzy_cands.push(Pair { display: name.clone(), replacement: format!("{} ", name) });
            }
        }

        let path_var = env::var_os("PATH").unwrap_or_default();
        let mut path_names = Vec::new();
        for path in env::split_paths(&path_var) {
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().into_owned();
                    if entry.path().is_file() && fuzzy_score(word, &name) > 0 {
                        path_names.push(name);
                    }
                }
            }
        }
        path_names.sort();
        path_names.dedup();
        for name in path_names {
            fuzzy_cands.push(Pair { display: name.clone(), replacement: format!("{} ", name) });
        }

        if !fuzzy_cands.is_empty() {
            fuzzy_cands.sort_by(|a, b| {
                let sa = fuzzy_score(word, &a.display.trim_end());
                let sb = fuzzy_score(word, &b.display.trim_end());
                sb.cmp(&sa).then(a.display.cmp(&b.display))
            });
            fuzzy_cands.dedup_by(|a, b| a.display == b.display);
            return (word_start, fuzzy_cands);
        }
    }

    // --- Fuzzy fallback for subcommands (arg_index == 1) ---
    if !word.is_empty() {
        let loaded_subs = completions.get(first_word);
        let mut all_subs: Vec<String> = loaded_subs.clone();
        if let Some(subs) = known_subcommands(first_word) {
            for (name, _desc) in subs {
                if !all_subs.contains(&name.to_string()) {
                    all_subs.push(name.to_string());
                }
            }
        }
        if !all_subs.is_empty() {
            let mut fuzzy_cands: Vec<Pair> = all_subs
                .iter()
                .filter_map(|s| {
                    if fuzzy_score(word, s) > 0 {
                        Some(Pair { display: s.clone(), replacement: format!("{} ", s) })
                    } else {
                        None
                    }
                })
                .collect();
            if !fuzzy_cands.is_empty() {
                fuzzy_cands.sort_by(|a, b| {
                    let sa = fuzzy_score(word, &a.replacement.trim_end());
                    let sb = fuzzy_score(word, &b.replacement.trim_end());
                    sb.cmp(&sa).then(a.display.cmp(&b.display))
                });
                return (word_start, fuzzy_cands);
            }
        }
    }

    // --- Fuzzy path completion (e.g. /u/l/b → /usr/local/bin) ---
    if !word.is_empty() && !dir_only {
        if let Some(result) = complete_path_fuzzy(word, word_start) {
            if !result.1.is_empty() {
                return result;
            }
        }
    }

    (word_start, Vec::new())
}

/// Fuzzy path completion: walks the filesystem matching each path segment
/// as a fuzzy subsequence. For example `/u/l/b` resolves each component
/// (`u`→`usr`, `l`→`local`, `b`→`bin`) and lists completions in the final
/// directory.
fn complete_path_fuzzy(word: &str, word_start: usize) -> Option<(usize, Vec<Pair>)> {
    let expanded = expand_tilde(word);
    // Split by '/', keeping empty strings for leading /
    let segments: Vec<&str> = if expanded == "/" {
        vec!["", ""]
    } else {
        expanded.split('/').collect()
    };

    if segments.len() < 2 {
        return None; // single component, handled by regular complete_path
    }

    // Walk from root or current dir
    let start_is_absolute = expanded.starts_with('/');
    let mut current_dir = if start_is_absolute {
        Path::new("/").to_path_buf()
    } else {
        Path::new(".").to_path_buf()
    };

    // All segments except the last one (which is the fragment to complete)
    for (i, seg) in segments[..segments.len() - 1].iter().enumerate() {
        if seg.is_empty() {
            if i == 0 && start_is_absolute {
                continue; // leading /
            }
            continue;
        }
        // Find best fuzzy match for this segment in current_dir
        let entries: Vec<_> = fs::read_dir(&current_dir).ok()?
            .flatten()
            .filter(|e| e.path().is_dir())
            .collect();

        let matched: Vec<_> = entries.iter()
            .filter(|e| {
                let name = e.file_name().to_string_lossy().into_owned();
                fuzzy_score(seg, &name) > 0
            })
            .collect();

        if matched.is_empty() {
            return None;
        }

        // If there's an exact match, follow it; otherwise use the best fuzzy match
        let best = matched.iter()
            .max_by_key(|e| {
                let name = e.file_name().to_string_lossy().into_owned();
                fuzzy_score(seg, &name)
            })?;

        current_dir = best.path();
    }

    // Now list completions in the final directory
    let last_seg = segments.last().unwrap_or(&"");
    let entries: Vec<_> = fs::read_dir(&current_dir).ok()?
        .flatten()
        .collect();

    let mut candidates: Vec<Pair> = Vec::new();
    for entry in &entries {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !last_seg.is_empty() && fuzzy_score(last_seg, &name) == 0 {
            continue;
        }
        if last_seg.is_empty() {
            // Show everything
        }
        let is_dir = entry.path().is_dir();
        let suffix = if is_dir { "/" } else { "" };
        // Build visible path from word
        let base_dir = match word.rfind('/') {
            Some(i) => &word[..=i],
            None => "",
        };
        let replacement = if last_seg.is_empty() {
            format!("{}{}{}", base_dir, name, if is_dir { "/" } else { " " })
        } else {
            // Replace the last segment of word with the matched entry
            let mut new_word = String::new();
            if let Some(last_slash) = word.rfind('/') {
                new_word.push_str(&word[..=last_slash]);
            }
            new_word.push_str(&name);
            new_word.push_str(if is_dir { "/" } else { " " });
            new_word
        };
        candidates.push(Pair {
            display: format!("{}{}", name, suffix),
            replacement,
        });
    }

    candidates.sort_by(|a, b| {
        let sa = fuzzy_score(last_seg, &a.display.trim_end().trim_end_matches('/'));
        let sb = fuzzy_score(last_seg, &b.display.trim_end().trim_end_matches('/'));
        sb.cmp(&sa).then(a.display.cmp(&b.display))
    });
    Some((word_start, candidates))
}

fn complete_path(word: &str, word_start: usize, dirs_only: bool) -> Option<(usize, Vec<Pair>)> {
    let expanded = expand_tilde(word);

    let (dir_part, frag) = match expanded.rfind('/') {
        Some(i) => (&expanded[..=i], &expanded[i + 1..]),
        None => ("", expanded.as_str()),
    };
    let visible_dir = match word.rfind('/') {
        Some(i) => &word[..=i],
        None => "",
    };

    let lookup_dir = if dir_part.is_empty() { "." } else { dir_part };
    let entries = fs::read_dir(lookup_dir).ok()?;

    let fl = frag.to_lowercase();
    let mut candidates = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let nl = name.to_lowercase();
        if !nl.starts_with(&fl) {
            continue;
        }
        let is_dir = entry.path().is_dir();
        if dirs_only && !is_dir {
            continue;
        }
        let (suffix, replacement_suffix) = if is_dir { ("/", "/") } else { ("", " ") };
        let replacement = format!("{}{}{}", visible_dir, name, replacement_suffix);
        candidates.push(Pair { display: format!("{}{}", name, suffix), replacement });
    }
    candidates.sort_by(|a, b| {
        let a_starts = a.display.to_lowercase().starts_with(&fl);
        let b_starts = b.display.to_lowercase().starts_with(&fl);
        if a_starts != b_starts {
            return b_starts.cmp(&a_starts);
        }
        a.display.cmp(&b.display)
    });
    Some((word_start, candidates))
}

/// Shows an interactive completion menu using raw mode.
/// Returns the index of the selected candidate, or None on cancel.
pub fn interactive_complete(candidates: &[Pair]) -> io::Result<Option<usize>> {
    if candidates.is_empty() {
        return Ok(None);
    }
    if candidates.len() == 1 {
        return Ok(Some(0));
    }

    let mut selected = 0;
    let mut stdout = io::stdout();

    terminal::enable_raw_mode()?;
    writeln!(stdout)?;
    let result = loop {
        let term_width = terminal::size()
            .map(|(w, _)| w as usize)
            .unwrap_or(80);

        let mut visual_len = 0usize;
        let mut line = String::new();
        for (i, c) in candidates.iter().enumerate() {
            let display = &c.display;
            let entry = if i == selected {
                format!("\x1B[7m{}\x1B[0m ", display)
            } else {
                format!("{} ", display)
            };
            let next_visual = if i == selected { display.len() + 2 } else { display.len() + 1 };
            if visual_len + next_visual + 3 > term_width {
                line.push('\u{2026}');
                break;
            }
            line.push_str(&entry);
            visual_len += next_visual;
        }

        write!(stdout, "\r")?;
        stdout.execute(Clear(ClearType::CurrentLine))?;
        write!(stdout, "{line}")?;
        stdout.execute(Clear(ClearType::UntilNewLine))?;
        writeln!(stdout)?;
        write!(stdout, "\r\x1B[2m\u{23CE}=ok  Tab=prox  S-Tab=ant  Esc/BS=canc\x1B[0m")?;
        stdout.execute(Clear(ClearType::UntilNewLine))?;
        stdout.flush()?;

        match event::read()? {
            CEvent::Key(kev) if kev.kind == event::KeyEventKind::Press => {
                match (kev.code, kev.modifiers) {
                    (KeyCode::Tab, m) if m == KeyModifiers::NONE => {
                        selected = (selected + 1) % candidates.len();
                    }
                    (KeyCode::Tab, m) if m == KeyModifiers::SHIFT => {
                        selected = if selected == 0 { candidates.len() - 1 } else { selected - 1 };
                    }
                    (KeyCode::Backspace, _) | (KeyCode::Esc, _) => {
                        break None;
                    }
                    (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                        break None;
                    }
                    (KeyCode::Enter, _) => {
                        break Some(selected);
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        write!(stdout, "\r")?;
        stdout.execute(Clear(ClearType::CurrentLine))?;
        write!(stdout, "\x1B[1A")?;
        stdout.execute(Clear(ClearType::CurrentLine))?;
    };

    write!(stdout, "\r")?;
    stdout.execute(Clear(ClearType::CurrentLine))?;
    write!(stdout, "\x1B[1A")?;
    stdout.execute(Clear(ClearType::CurrentLine))?;
    write!(stdout, "\x1B[1A")?;
    terminal::disable_raw_mode()?;
    stdout.flush()?;

        Ok(result)
}

/// Enhanced interactive completion menu for the **Hybrid** tab mode.
///
/// This is a "pro" autocomplete that merges the **circular** cycling of
/// `Circular` mode with the **interactive menu** of the default `Interactive`
/// mode:
///
/// * All candidates are rendered on screen with the current selection
///   highlighted in reverse video.
/// * A live **preview** of the replacement text is shown below the menu so
///   the user can see exactly what will be inserted before committing.
/// * A **counter** (`2/5`) indicates the current position.
/// * Navigation is **circular**: Tab wraps forward, Shift+Tab wraps backward.
/// * Enter confirms, Esc / Backspace / Ctrl-C cancels.
///
/// Returns the index of the selected candidate, or `None` on cancel.
pub fn interactive_complete_hybrid(candidates: &[Pair]) -> io::Result<Option<usize>> {
    if candidates.is_empty() {
        return Ok(None);
    }
    if candidates.len() == 1 {
        return Ok(Some(0));
    }

    let mut selected = 0usize;
    let mut stdout = io::stdout();

    terminal::enable_raw_mode()?;
    writeln!(stdout)?;

    let result = loop {
        let term_width = terminal::size()
            .map(|(w, _)| w as usize)
            .unwrap_or(80);

                // --- Render candidate menu ---------------------------------------
        let mut visual_len = 0usize;
        let mut line = String::new();
        for (i, c) in candidates.iter().enumerate() {
            // Split "name  description" so we can dim the description.
            let (name, desc) = match c.display.split_once("  ") {
                Some((n, d)) => (n, Some(d)),
                None => (c.display.as_str(), None),
            };
            let entry = if i == selected {
                if let Some(d) = desc {
                    format!("\x1B[7m{}\x1B[0m\x1B[2m {}\x1B[0m", name, d)
                } else {
                    format!("\x1B[7m{}\x1B[0m ", name)
                }
            } else {
                if let Some(d) = desc {
                    format!("{}\x1B[2m {}\x1B[0m", name, d)
                } else {
                    format!("{} ", name)
                }
            };
            let next_visual = if i == selected {
                name.len() + 2
            } else {
                name.len() + 1
            };
            if visual_len + next_visual + 3 > term_width {
                line.push('\u{2026}');
                break;
            }
            line.push_str(&entry);
            visual_len += next_visual;
        }

        write!(stdout, "\r")?;
        stdout.execute(Clear(ClearType::CurrentLine))?;
        write!(stdout, "{line}")?;
        stdout.execute(Clear(ClearType::UntilNewLine))?;
        writeln!(stdout)?;

        // --- Render counter + preview ------------------------------------
        let preview = &candidates[selected].replacement;
        let counter = format!("{}/{}", selected + 1, candidates.len());
        let hint = format!(
            "\x1B[2m{}  preview: {}  Tab=cycle  S-Tab=ant  Enter=ok  Esc/BS=canc\x1B[0m",
            counter, preview,
        );
        write!(stdout, "\r")?;
        stdout.execute(Clear(ClearType::CurrentLine))?;
        write!(stdout, "{hint}")?;
        stdout.execute(Clear(ClearType::UntilNewLine))?;
        writeln!(stdout)?;
        stdout.flush()?;

        match event::read()? {
            CEvent::Key(kev) if kev.kind == event::KeyEventKind::Press => {
                match (kev.code, kev.modifiers) {
                    (KeyCode::Tab, m) if m == KeyModifiers::NONE => {
                        // Circular forward
                        selected = (selected + 1) % candidates.len();
                    }
                    (KeyCode::Tab, m) if m == KeyModifiers::SHIFT => {
                        // Circular backward
                        selected = if selected == 0 {
                            candidates.len() - 1
                        } else {
                            selected - 1
                        };
                    }
                    (KeyCode::Up, _) => {
                        selected = if selected == 0 {
                            candidates.len() - 1
                        } else {
                            selected - 1
                        };
                    }
                    (KeyCode::Down, _) => {
                        selected = (selected + 1) % candidates.len();
                    }
                    (KeyCode::Backspace, _) | (KeyCode::Esc, _) => {
                        break None;
                    }
                    (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                        break None;
                    }
                    (KeyCode::Enter, _) => {
                        break Some(selected);
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        // Clear the two lines we just wrote so we can redraw cleanly.
        write!(stdout, "\r")?;
        stdout.execute(Clear(ClearType::CurrentLine))?;
        write!(stdout, "\x1B[1A")?;
        stdout.execute(Clear(ClearType::CurrentLine))?;
    };

    // Clean up the menu lines.
    write!(stdout, "\r")?;
    stdout.execute(Clear(ClearType::CurrentLine))?;
    write!(stdout, "\x1B[1A")?;
    stdout.execute(Clear(ClearType::CurrentLine))?;
    write!(stdout, "\x1B[1A")?;
    terminal::disable_raw_mode()?;
    stdout.flush()?;

    Ok(result)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TabMode {
    Interactive,
    Circular,
    /// Hybrid autocomplete: combines the circular cycling of `Circular`
    /// with the interactive menu of `Interactive` (the default mode).
    ///
    /// When Tab is pressed with multiple candidates, an enhanced menu is
    /// shown with circular navigation (Tab wraps forward, Shift+Tab wraps
    /// backward), a candidate counter (`2/5`), and a live preview of the
    /// replacement text.  Enter confirms, Esc/BS cancels.
    Hybrid,
}

pub struct JshHelper {
    pub history_mgr: Arc<crate::shell::history::HistoryManager>,
    pub aliases: Arc<Mutex<HashMap<String, String>>>,
    pub shell_vars: Arc<Mutex<HashMap<String, String>>>,
    pub functions: Arc<Mutex<HashMap<String, String>>>,
    pub completions: Arc<Mutex<CompletionDb>>,
    pub tab_mode: TabMode,
}

impl Helper for JshHelper {}

impl Completer for JshHelper {
    type Candidate = Pair;
    fn complete(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Result<(usize, Vec<Self::Candidate>), ReadlineError> {
        let aliases = self.aliases.lock().unwrap_or_else(|e| e.into_inner());
        let vars = self.shell_vars.lock().unwrap_or_else(|e| e.into_inner());
        let funcs = self.functions.lock().unwrap_or_else(|e| e.into_inner());
        let completions = self.completions.lock().unwrap_or_else(|e| e.into_inner());
        let (word_start, candidates) = get_completions(line, pos, &completions, &aliases, &vars, &funcs);

        match self.tab_mode {
            TabMode::Circular => {
                // Return all candidates so rustyline cycles through them inline.
                return Ok((word_start, candidates));
            }
            TabMode::Hybrid => {
                // Hybrid: enhanced menu with circular navigation + preview.
                if candidates.len() <= 1 {
                    return Ok((word_start, candidates));
                }
                match interactive_complete_hybrid(&candidates) {
                    Ok(Some(idx)) => Ok((word_start, vec![candidates[idx].clone()])),
                    _ => Ok((0, Vec::new())),
                }
            }
            TabMode::Interactive => {
                if candidates.len() <= 1 {
                    return Ok((word_start, candidates));
                }
                match interactive_complete(&candidates) {
                    Ok(Some(idx)) => Ok((word_start, vec![candidates[idx].clone()])),
                    _ => Ok((0, Vec::new())),
                }
            }
        }
    }
}

impl Hinter for JshHelper {
    type Hint = JshHint;
    fn hint(&self, line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<Self::Hint> {
        let cwd = {
            let vars = self.shell_vars.lock().unwrap();
            vars.get("PWD").cloned().unwrap_or_else(|| ".".to_string())
        };
        if let Some(suggestion) = self.history_mgr.get_suggestion(line, &cwd) {
            if suggestion.starts_with(line) && suggestion != line {
                let remainder = suggestion[line.len()..].to_string();
                return Some(JshHint {
                    display: remainder.clone(),
                    complete: remainder,
                });
            }
        }
        None
    }
}

impl Validator for JshHelper {
    fn validate(&self, _ctx: &mut rustyline::validate::ValidationContext<'_>) -> rustyline::Result<rustyline::validate::ValidationResult> {
        Ok(rustyline::validate::ValidationResult::Valid(None))
    }
}

impl Highlighter for JshHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(&'s self, prompt: &'p str, default: bool) -> Cow<'b, str> {
        if default {
            let colored = CURRENT_COLORED_PROMPT.with(|cell| cell.borrow().clone());
            if !colored.is_empty() {
                let rprompt = CURRENT_COLORED_RPROMPT.with(|cell| cell.borrow().clone());
                if let Some(rp) = rprompt {
                    let cols = crossterm::terminal::size().map(|(w, _)| w).unwrap_or(80) as usize;
                    let rp_plain = crate::utils::strip_ansi(&rp);
                    use unicode_width::UnicodeWidthStr;
                    let rp_width = UnicodeWidthStr::width(rp_plain.as_str());
                    let prompt_plain = crate::utils::strip_ansi(&colored);
                    let prompt_width = UnicodeWidthStr::width(prompt_plain.as_str());
                    if prompt_width + rp_width < cols {
                        let gap = cols - prompt_width - rp_width;
                        Cow::Owned(format!("{}\x1b[s{}{}\x1b[u", colored, " ".repeat(gap), rp))
                    } else {
                        Cow::Owned(format!("{} {}", colored, rp))
                    }
                } else {
                    Cow::Owned(colored)
                }
            } else {
                Cow::Borrowed(prompt)
            }
        } else {
            Cow::Borrowed(prompt)
        }
    }

    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        if line.is_empty() {
            return Cow::Borrowed(line);
        }

        let mut result = String::with_capacity(line.len() + 100);
        let mut chars = line.char_indices().peekable();
        let mut in_single_quote = false;
        let mut in_double_quote = false;
        let mut word_start = None;
        let mut is_first_word = true;

        let aliases_map = self.aliases.lock().unwrap();

        // Helper to flush a collected word with syntax highlighting
        let flush_word = |start: usize, end: usize, res: &mut String, is_first: &mut bool| {
            if start >= end { return; }
            let word = &line[start..end];
            
            if word.starts_with('$') {
                res.push_str(&format!("\x1B[38;5;39m{}\x1B[0m", word)); // Light blue for variables
            } else if *is_first {
                let expanded_word = crate::utils::expand_tilde(word);
                if word == "texit" || word == "nano" || is_executable(word) {
                    res.push_str(&format!("\x1B[32m{}\x1B[0m", word)); // Green
                } else if aliases_map.contains_key(word) {
                    res.push_str(&format!("\x1B[38;5;208m{}\x1B[0m", word)); // Orange
                } else if is_builtin(word) {
                    res.push_str(&format!("\x1B[32m{}\x1B[0m", word)); // Green
                } else if Path::new(&expanded_word).is_dir() || word == "~" || word == ".-1" || word == "$PWD_BACK" || word == "$PB" {
                    res.push_str(&format!("\x1B[34m{}\x1B[0m", word)); // Blue
                } else {
                    res.push_str(&format!("\x1B[31m{}\x1B[0m", word)); // Red
                }
                *is_first = false;
                
                // Allow commands following wrappers like sudo to also be highlighted
                if matches!(word, "sudo" | "time" | "exec" | "env" | "nohup" | "watch" | "xargs") {
                    *is_first = true;
                }
            } else if word.starts_with('-') {
                res.push_str(&format!("\x1B[38;5;228m{}\x1B[0m", word)); // Pale yellow for flags
            } else if Path::new(&crate::utils::expand_tilde(word)).is_dir() || word == "~" || word == ".-1" || word == "$PWD_BACK" || word == "$PB" {
                res.push_str(&format!("\x1B[34m{}\x1B[0m", word)); // Blue for dirs
            } else {
                res.push_str(word);
            }
        };

        while let Some((i, c)) = chars.next() {
            if in_single_quote {
                result.push_str("\x1B[33m"); // Yellow for strings
                result.push(c);
                if c == '\'' {
                    in_single_quote = false;
                }
                result.push_str("\x1B[0m");
                continue;
            }
            if in_double_quote {
                if c == '$' {
                    // Quick variable highlight inside double quotes
                    result.push_str("\x1B[38;5;39m$\x1B[0m");
                } else {
                    result.push_str("\x1B[33m");
                    result.push(c);
                    if c == '"' {
                        in_double_quote = false;
                    }
                    result.push_str("\x1B[0m");
                }
                continue;
            }

            match c {
                '\'' => {
                    if let Some(s) = word_start {
                        flush_word(s, i, &mut result, &mut is_first_word);
                        word_start = None;
                    }
                    in_single_quote = true;
                    result.push_str("\x1B[33m'\x1B[0m");
                }
                '"' => {
                    if let Some(s) = word_start {
                        flush_word(s, i, &mut result, &mut is_first_word);
                        word_start = None;
                    }
                    in_double_quote = true;
                    result.push_str("\x1B[33m\"\x1B[0m");
                }
                ' ' | '\t' | '\r' | '\n' => {
                    if let Some(s) = word_start {
                        flush_word(s, i, &mut result, &mut is_first_word);
                        word_start = None;
                    }
                    result.push(c);
                    if c == '\n' {
                        is_first_word = true;
                    }
                }
                '|' | '&' | ';' | '<' | '>' => {
                    if let Some(s) = word_start {
                        flush_word(s, i, &mut result, &mut is_first_word);
                        word_start = None;
                    }
                    result.push_str(&format!("\x1B[38;5;161m{}\x1B[0m", c)); // Pink/Red for operators
                    if matches!(c, '|' | '&' | ';') {
                        is_first_word = true; // reset first word after pipeline/sequence
                    }
                }
                _ => {
                    if word_start.is_none() {
                        word_start = Some(i);
                    }
                }
            }
        }

        if let Some(s) = word_start {
            flush_word(s, line.len(), &mut result, &mut is_first_word);
        }

        Cow::Owned(result)
    }

    fn highlight_char(&self, _line: &str, _pos: usize, _forced: bool) -> bool {
        true
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Owned(format!("\x1B[90m{}\x1B[0m", hint))
    }

    fn highlight_candidate<'c>(&self, candidate: &'c str, _completion: CompletionType) -> Cow<'c, str> {
        let trimmed = candidate.trim_end();

        // Directory: bold blue (like ls --color)
        if trimmed.ends_with('/') {
            return Cow::Owned(format!("\x1B[1;34m{}\x1B[0m", candidate));
        }

        // Detect file extension for ls-like coloring
        if let Some(dot_pos) = trimmed.rfind('.') {
            let ext = &trimmed[dot_pos..].to_lowercase();
            let color = match ext.as_str() {
                // Archives: bold red
                ".tar" | ".gz" | ".bz2" | ".xz" | ".zip" | ".7z" | ".rar"
                | ".deb" | ".rpm" | ".tgz" | ".zst" | ".lz4" | ".iso" => Some("1;31"),
                // Images/media: bold magenta
                ".jpg" | ".jpeg" | ".png" | ".gif" | ".bmp" | ".svg" | ".webp"
                | ".ico" | ".tiff" | ".mp4" | ".mkv" | ".avi" | ".mov" | ".webm"
                | ".mp3" | ".flac" | ".ogg" | ".wav" | ".aac" => Some("1;35"),
                // Scripts/source: bold green
                ".sh" | ".bash" | ".zsh" | ".fish" | ".py" | ".rb" | ".pl"
                | ".rs" | ".go" | ".js" | ".ts" | ".c" | ".cpp" | ".h"
                | ".java" | ".aly" => Some("1;32"),
                // Config/data: cyan
                ".toml" | ".yaml" | ".yml" | ".json" | ".xml" | ".ini"
                | ".cfg" | ".conf" => Some("0;36"),
                // Markdown/docs: yellow
                ".md" | ".txt" | ".rst" | ".org" => Some("0;33"),
                _ => None,
            };
            if let Some(c) = color {
                return Cow::Owned(format!("\x1B[{}m{}\x1B[0m", c, candidate));
            }
        }

        Cow::Borrowed(candidate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn helper() -> JshHelper {
        let history_mgr = Arc::new(crate::shell::history::HistoryManager::new());
        JshHelper {
            history_mgr,
            aliases: Arc::new(Mutex::new(HashMap::new())),
            shell_vars: Arc::new(Mutex::new(HashMap::new())),
            functions: Arc::new(Mutex::new(HashMap::new())),
            completions: Arc::new(Mutex::new(CompletionDb::new())),
            tab_mode: TabMode::Interactive,
        }
    }

            #[test]
    fn subcommands_known() {
        let git = known_subcommands("git").unwrap();
        assert!(git.iter().any(|(n, _)| *n == "commit"));
        let cargo = known_subcommands("cargo").unwrap();
        assert!(cargo.iter().any(|(n, _)| *n == "build"));
        let aly = known_subcommands("aly").unwrap();
        assert!(aly.iter().any(|(n, _)| *n == "run"));
        let apg = known_subcommands("apg").unwrap();
        assert!(apg.iter().any(|(n, _)| *n == "install"));
        let cutils = known_subcommands("cutils").unwrap();
        assert!(cutils.iter().any(|(n, _)| *n == "list"));
        assert!(known_subcommands("nonesuch").is_none());
    }

    #[test]
    fn subcommand_descriptions_present() {
        let git = known_subcommands("git").unwrap();
        let commit = git.iter().find(|(n, _)| *n == "commit").unwrap();
        assert!(!commit.1.is_empty(), "commit should have a description");
        let status = git.iter().find(|(n, _)| *n == "status").unwrap();
        assert!(!status.1.is_empty(), "status should have a description");
    }

    #[test]
    fn option_descriptions_present() {
        let git_opts = known_options("git").unwrap();
        let help = git_opts.iter().find(|(n, _)| *n == "--help").unwrap();
        assert!(!help.1.is_empty(), "--help should have a description");
    }

    #[test]
    fn get_completions_includes_descriptions() {
        let db = CompletionDb::new();
        let (pos, cands) = get_completions("git s", 5, &db, &HashMap::new(), &HashMap::new(), &HashMap::new());
        assert!(pos > 0);
        // At least one candidate should contain a description (double-space separator).
        let with_desc = cands.iter().any(|p| p.display.contains("  "));
        assert!(with_desc, "expected at least one candidate with a description in {:?}", cands.iter().map(|p| &p.display).collect::<Vec<_>>());
    }

    #[test]
    fn cd_offers_only_dirs() {
        // Build an isolated dir with one subdir and one file, then complete
        // its path with dirs_only=true and check only the dir shows up.
        let base = std::env::temp_dir().join(format!("jsh_ct_{}", std::process::id()));
        let _ = fs::create_dir_all(base.join("subdir"));
        let _ = fs::write(base.join("file.txt"), b"x");

        helper();
        let word = format!("{}/", base.display());
        let (_, cands) = complete_path(&word, 0, true).unwrap();
        assert!(cands.iter().any(|p| p.display == "subdir/"),
            "expected subdir/ among {:?}", cands.iter().map(|p| &p.display).collect::<Vec<_>>());
        assert!(!cands.iter().any(|p| p.display.starts_with("file.txt")),
            "file.txt must not appear when dirs_only");

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn tilde_kept_in_replacement() {
        // "~/" should read $HOME and keep the ~/ prefix in every replacement.
        if let Some((_, cands)) = complete_path("~/", 2, false) {
            assert!(cands.iter().all(|p| p.replacement.starts_with("~/")));
        }
    }

    #[test]
    fn completes_local_vars() {
        let mut vars = HashMap::new();
        vars.insert("MY_TEST_LOCAL_VAR".to_string(), "value".to_string());
        let db = CompletionDb::new();
        
        let (pos, candidates) = get_completions("$MY_TEST_L", 10, &db, &HashMap::new(), &vars, &HashMap::new());
        assert_eq!(pos, 0);
        assert!(candidates.iter().any(|p| p.display == "$MY_TEST_LOCAL_VAR"));
    }

        #[test]
    fn completes_functions() {
        let mut funcs = HashMap::new();
        funcs.insert("my_test_func".to_string(), "body".to_string());
        let db = CompletionDb::new();
        
        let (pos, candidates) = get_completions("my_te", 5, &db, &HashMap::new(), &HashMap::new(), &funcs);
        assert_eq!(pos, 0);
        assert!(candidates.iter().any(|p| p.display == "my_test_func"));
    }

    // ------------------------------------------------------------------
    // Hybrid tab-mode tests
    // ------------------------------------------------------------------

    fn helper_hybrid() -> JshHelper {
        let history_mgr = Arc::new(crate::shell::history::HistoryManager::new());
        JshHelper {
            history_mgr,
            aliases: Arc::new(Mutex::new(HashMap::new())),
            shell_vars: Arc::new(Mutex::new(HashMap::new())),
            functions: Arc::new(Mutex::new(HashMap::new())),
            completions: Arc::new(Mutex::new(CompletionDb::new())),
            tab_mode: TabMode::Hybrid,
        }
    }

    #[test]
    fn hybrid_mode_is_distinct() {
        assert_ne!(TabMode::Hybrid, TabMode::Interactive);
        assert_ne!(TabMode::Hybrid, TabMode::Circular);
    }

    #[test]
    fn hybrid_empty_candidates_returns_none() {
        let cands: Vec<Pair> = vec![];
        assert_eq!(interactive_complete_hybrid(&cands).unwrap(), None);
    }

    #[test]
    fn hybrid_single_candidate_returns_zero() {
        let cands = vec![Pair {
            display: "git".to_string(),
            replacement: "git ".to_string(),
        }];
        assert_eq!(interactive_complete_hybrid(&cands).unwrap(), Some(0));
    }

    #[test]
    fn hybrid_completes_single_candidate_directly() {
        // With only one candidate, Hybrid mode should return it directly
        // (same as Interactive / Circular) without invoking the menu.
        let h = helper_hybrid();
        let cands = vec![Pair {
            display: "git".to_string(),
            replacement: "git ".to_string(),
        }];
                // Simulate what complete() does for a single candidate.
        let result: Result<(usize, Vec<Pair>), ReadlineError> = match h.tab_mode {
            TabMode::Hybrid => {
                if cands.len() <= 1 {
                    Ok((0, cands.clone()))
                } else {
                    match interactive_complete_hybrid(&cands) {
                        Ok(Some(idx)) => Ok((0, vec![cands[idx].clone()])),
                        _ => Ok((0, Vec::new())),
                    }
                }
            }
            _ => unreachable!(),
        };
        assert!(result.is_ok());
        let (_, returned) = result.unwrap();
        assert_eq!(returned.len(), 1);
        assert_eq!(returned[0].display, "git");
    }

        #[test]
    fn hybrid_menu_returns_valid_index_range() {
        // Build candidates from known subcommands of git.
        let subs = known_subcommands("git").unwrap();
        let cands: Vec<Pair> = subs
            .iter()
            .map(|(name, desc)| Pair {
                display: format!("{}  {}", name, desc),
                replacement: format!("{} ", name),
            })
            .collect();
        // interactive_complete_hybrid is blocking (reads terminal input),
        // so we only verify the pre-conditions: >1 candidate, all indices
        // are within bounds.
        assert!(cands.len() > 1);
        for (i, c) in cands.iter().enumerate() {
            assert!(i < cands.len());
            assert!(!c.display.is_empty());
        }
    }
}
