use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;

const SYSTEM_DIR: &str = "/opt/jeffutils/bjesh/comp";
const USER_DIR: &str = ".local/.bjesh/completations";

pub struct CompletionDb {
    system_dir: PathBuf,
    user_dir: PathBuf,
    entries: HashMap<String, Vec<String>>,
    /// Programmable completers: maps command name to shell function name
    /// that should be called to generate completions.
    /// The function receives `$1`=current word, `$2`=previous word,
    /// `$3`=all words so far, and should output completion lines to stdout.
    pub completers: HashMap<String, String>,
    /// Custom word lists registered via `complete -W "..." cmd`
    pub word_lists: HashMap<String, Vec<String>>,
}

impl CompletionDb {
    pub fn new() -> Self {
        let home = std::env::var("HOME").map(PathBuf::from).unwrap_or_default();
        let user_dir = home.join(USER_DIR);
        let system_dir = PathBuf::from(SYSTEM_DIR);

        let mut db = CompletionDb {
            system_dir,
            user_dir: user_dir.clone(),
            entries: HashMap::new(),
            completers: HashMap::new(),
            word_lists: HashMap::new(),
        };

        if !user_dir.exists() {
            let _ = db.try_import();
        }

        db.load();
        db
    }

    pub fn load(&mut self) {
        self.entries.clear();

        let sys = self.system_dir.clone();
        if sys.exists() {
            self.load_dir(&sys, false);
        }

        let usr = self.user_dir.clone();
        if usr.exists() {
            self.load_dir(&usr, true);
        }
    }

    pub fn get(&self, cmd: &str) -> Vec<String> {
        self.entries
            .get(cmd)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    pub fn has(&self, cmd: &str) -> bool {
        self.entries.contains_key(cmd)
    }

    /// Register a shell function as completer for a command.
    /// Equivalent to bash's `complete -F _func cmd`.
    pub fn register_completer(&mut self, cmd: &str, func_name: &str) {
        self.completers.insert(cmd.to_string(), func_name.to_string());
    }

    /// Register a word list for a command.
    /// Equivalent to bash's `complete -W "word1 word2" cmd`.
    pub fn register_word_list(&mut self, cmd: &str, words: &[String]) {
        self.word_lists.insert(cmd.to_string(), words.to_vec());
    }

    /// Get the function completer for a command, if registered.
    pub fn get_completer(&self, cmd: &str) -> Option<&str> {
        self.completers.get(cmd).map(|s| s.as_str())
    }

    /// Get the word list for a command, if registered.
    pub fn get_word_list(&self, cmd: &str) -> Vec<String> {
        self.word_lists.get(cmd).cloned().unwrap_or_default()
    }

    /// Run a function completer by calling the shell function through state.
    /// Returns the list of completion words.
    pub fn run_completer(&self, cmd: &str, state: &mut crate::shell::ShellState,
                         current_word: &str, prev_word: &str, words: &str) -> Vec<String> {
        let Some(func_name) = self.get_completer(cmd) else {
            return Vec::new();
        };
        let args = vec![
            current_word.to_string(),
            prev_word.to_string(),
            words.to_string(),
        ];
        let exit = state.call_function(func_name, &args);
        if exit != 0 {
            return Vec::new();
        }
        // The function should have set a variable with completions
        let result = state.get_var("COMP_REPLY");
        if result.is_empty() {
            return Vec::new();
        }
        result.split_whitespace().map(|s| s.to_string()).collect()
    }

    fn load_dir(&mut self, dir: &PathBuf, overwrite: bool) {
        let Ok(entries) = fs::read_dir(dir) else { return };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() || path.extension().map(|e| e != "bash" && e != "zsh" && e != "bjesh").unwrap_or(false) {
                continue;
            }
            let cmd_name = path.file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.trim_start_matches('_').to_string());
            let Some(cmd_name) = cmd_name else { continue };

            if !overwrite && self.entries.contains_key(&cmd_name) {
                continue;
            }

            let Ok(file) = fs::File::open(&path) else { continue };
            let reader = BufReader::new(file);
            let mut words = Vec::new();
            for line in reader.lines().flatten() {
                let line = line.trim().to_string();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                words.push(line);
            }
            if !words.is_empty() {
                self.entries.insert(cmd_name, words);
            }
        }
    }

    pub fn try_import(&self) -> io::Result<()> {
        if self.user_dir.exists() {
            return Ok(());
        }

        fs::create_dir_all(&self.user_dir)?;

        let bash_dirs = vec![
            PathBuf::from("/usr/share/bash-completion/completions"),
            PathBuf::from("/etc/bash_completion.d"),
        ];

        for dir in &bash_dirs {
            if dir.exists() {
                self.import_bash_dir(dir)?;
            }
        }

        let zsh_base = PathBuf::from("/usr/share/zsh");
        if zsh_base.exists() {
            if let Ok(entries) = fs::read_dir(&zsh_base) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() && path.file_name().and_then(|s| s.to_str()).map_or(false, |s| s.starts_with("functions")) {
                        let comp_dir = path.join("Completion");
                        if comp_dir.exists() {
                            if let Ok(subdirs) = fs::read_dir(&comp_dir) {
                                for sd in subdirs.flatten() {
                                    let sd_path = sd.path();
                                    if sd_path.is_dir() {
                                        self.import_zsh_dir(&sd_path)?;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn import_bash_dir(&self, dir: &PathBuf) -> io::Result<()> {
        let Ok(entries) = fs::read_dir(dir) else { return Ok(()) };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                continue;
            }
            let cmd_name = path.file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.trim_start_matches('_').to_string());
            let Some(cmd_name) = cmd_name else { continue };

            let out_path = self.user_dir.join(&cmd_name);
            if out_path.exists() {
                continue;
            }

            let Ok(content) = fs::read_to_string(&path) else { continue };
            let words = extract_bash_completions(&content);
            if !words.is_empty() {
                let mut out = fs::File::create(&out_path)?;
                for w in &words {
                    writeln!(out, "{}", w)?;
                }
            }
        }
        Ok(())
    }

    fn import_zsh_dir(&self, dir: &PathBuf) -> io::Result<()> {
        let Ok(entries) = fs::read_dir(dir) else { return Ok(()) };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() || !path.file_name().and_then(|s| s.to_str()).map_or(false, |s| s.starts_with('_')) {
                continue;
            }

            let Ok(content) = fs::read_to_string(&path) else { continue };

            let cmd_name = extract_zsh_cmd_name(&content);
            let Some(cmd_name) = cmd_name else { continue };

            let out_path = self.user_dir.join(&cmd_name);
            if out_path.exists() {
                continue;
            }

            let words = extract_zsh_completions(&content);
            if !words.is_empty() {
                let mut out = fs::File::create(&out_path)?;
                for w in &words {
                    writeln!(out, "{}", w)?;
                }
            }
        }
        Ok(())
    }
}

fn extract_bash_completions(content: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut seen = HashSet::new();

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        if let Some(caps_start) = line.find("-W '") {
            let after_w = &line[caps_start + 4..];
            if let Some(end) = after_w.find('\'') {
                let list = &after_w[..end];
                for w in list.split_whitespace() {
                    let w = w.trim().to_string();
                    if !w.is_empty() && !w.starts_with('-') && seen.insert(w.clone()) {
                        words.push(w);
                    }
                }
            }
        }

        if let Some(caps_start) = line.find("-W \"") {
            let after_w = &line[caps_start + 4..];
            if let Some(end) = after_w.find('"') {
                let list = &after_w[..end];
                for w in list.split_whitespace() {
                    let w = w.trim().to_string();
                    if !w.is_empty() && !w.starts_with('-') && seen.insert(w.clone()) {
                        words.push(w);
                    }
                }
            }
        }
    }

    words
}

fn extract_zsh_cmd_name(content: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("#compdef ") {
            let rest = line.trim_start_matches("#compdef ").trim();
            let name = rest.split_whitespace().next()?;
            let name = name.trim_start_matches('_');
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }
    None
}

fn extract_zsh_completions(content: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut seen = HashSet::new();

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        if let Some(pos) = line.find("_values ") {
            let rest = &line[pos + 8..];
            let after_desc = rest.split_once('\'').map(|(_, r)| r).unwrap_or(rest);
            let after_desc = after_desc.trim_start_matches('\'');
            for part in after_desc.split_whitespace() {
                if let Some(name) = part.split('[').next() {
                    let name = name.trim().trim_matches('\'');
                    if !name.is_empty() && !name.starts_with('-') && !name.starts_with('*') && !name.starts_with(':') && seen.insert(name.to_string()) {
                        words.push(name.to_string());
                    }
                }
            }
        }

        if let Some(pos) = line.find("_describe ") {
            let rest = &line[pos + 10..];
            for part in rest.split_whitespace() {
                let part = part.trim().trim_matches('\'');
                if !part.is_empty() && !part.starts_with('-') && !part.starts_with('(') && !part.starts_with(')') && !part.contains('[') && seen.insert(part.to_string()) {
                    if !part.contains('$') && !part.contains('{') {
                        words.push(part.to_string());
                    }
                }
            }
        }
    }

    words
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bash_extract_simple() {
        let content = r#"complete -W 'start stop restart status' myapp"#;
        let words = extract_bash_completions(content);
        assert!(words.contains(&"start".to_string()));
        assert!(words.contains(&"stop".to_string()));
        assert!(words.contains(&"restart".to_string()));
        assert!(words.contains(&"status".to_string()));
    }

    #[test]
    fn zsh_extract_cmd_name() {
        let content = "#compdef myapp\n_local stuff";
        assert_eq!(extract_zsh_cmd_name(content), Some("myapp".to_string()));
    }

    #[test]
    fn zsh_extract_values() {
        let content = r#"_values 'myapp command' start[Start] stop[Stop] status[Status]"#;
        let words = extract_zsh_completions(content);
        assert!(words.contains(&"start".to_string()));
        assert!(words.contains(&"stop".to_string()));
        assert!(words.contains(&"status".to_string()));
    }

    #[test]
    fn load_and_query() {
        let db = CompletionDb::new();
        assert!(!db.has("git"));
    }
}
