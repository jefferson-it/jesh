use crate::shell::ShellState;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::os::unix::fs::MetadataExt;

pub fn run_jeofetch() {
    let status = Command::new("jeofetch")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();
    if let Err(e) = status {
        eprintln!("jesh: jeofetch: {}", e);
    }
}

pub fn is_builtin(cmd: &str) -> bool {
    matches!(
        cmd,
        "cd" | "exit"
            | "exec"
            | "printf"
            | "eval"
            | "command"
            | "time"
            | "help"
            | "version"
            | "jesh-version"
            | "jesh-path"
            | "jesh-which"
            | "jesh-which"
            | "export"
            | "unset"
            | "set"
            | "alias"
            | "unalias"
            | "source"
            | "."
            | "true"
            | "false"
            | ":"
            | "history"
            | "pushd"
            | "popd"
            | "dirs"
            | "read"
            | "declare"
            | "typeset"
            | "disown"
            | "local"
            | "readonly"
            | "getopts"
            | "test"
            | "["
            | "[["
            | "shopt"
            | "complete"
    )
}

pub fn is_executable(cmd: &str) -> bool {
    use std::os::unix::fs::PermissionsExt;
    let is_exec = |p: &std::path::Path| -> bool {
        p.is_file() && p.metadata().map(|m| m.permissions().mode() & 0o111 != 0).unwrap_or(false)
    };
    if is_exec(&std::path::Path::new(cmd)) {
        return true;
    }
    if cmd.contains('/') || cmd.contains(std::path::MAIN_SEPARATOR) {
        return false;
    }
    let path_var = match env::var_os("PATH") {
        Some(p) => p,
        None => return false,
    };
    env::split_paths(&path_var).any(|dir| {
        let full = dir.join(cmd);
        is_exec(&full)
    })
}

fn print_help() {
    println!(
        "\
jesh — shell interativo

Builtins:
  cd [dir]           Muda de diretório (sem args: vai para $HOME)
  export NAME=valor   Define e exporta uma variável de ambiente
  export NAME         Exporta uma variável de shell já existente
  unset NAME          Remove uma variável de shell/ambiente
  set                 Lista variáveis de shell e de ambiente
  alias nome=valor    Define um alias
  unalias nome        Remove um alias
  source arquivo | .  Executa um script no shell atual
  true / false / :    Comandos no-op de status 0/1
  exec [cmd] [args]   Substitui o processo do shell pelo comando especificado
  printf fmt ...      Imprime string formatada (suporta %s, %d, %u, %x, %f, %c)
  eval string         Avalia argumentos como um comando do shell
  command cmd [args]  Executa comando ignorando aliases e funções
  jesh-path / jesh-info Mostra o caminho exato do binário jesh em execução
  exit                Sai do jesh

Sintaxe suportada: pipes (|), redirecionamentos (>, >>, <, <<, <<<, 2>, &>),
listas de comandos (;, &&, ||), aspas simples/duplas, escapes (\\),
substituição de comando $(...) / `...`, variáveis de shell e $?, $$, $0,
globbing (*, ?, [...]), histórico !! / !n / !prefixo."
    );
}

pub fn handle_builtin(args: &[String], state: &mut ShellState) -> Option<i32> {
    if args.is_empty() {
        return Some(0);
    }
    let cmd = &args[0];

    // Check if the command is a shortcut to go back: ".-1", "$PWD_BACK", "$PB", "-"
    let is_back_cmd = cmd == ".-1" || cmd == "$PWD_BACK" || cmd == "$PB" || cmd == "-";
    let is_cd_back_cmd = cmd == "cd" && args.len() > 1 && (args[1] == ".-1" || args[1] == "$PWD_BACK" || args[1] == "$PB" || args[1] == "-");

    if is_back_cmd || is_cd_back_cmd {
        if let Some(ref prev) = state.old_pwd {
            let prev_clone = prev.clone();
            let current = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let old_pwd = env::var("PWD").unwrap_or_else(|_| current.to_string_lossy().into_owned());
            if let Err(e) = env::set_current_dir(&prev_clone) {
                eprintln!("cd: {}", e);
                return Some(1);
            }
            println!("{}", prev_clone.display());
            state.old_pwd = Some(current);
            unsafe {
                env::set_var("OLDPWD", &old_pwd);
                if let Ok(new_pwd) = env::current_dir() {
                    env::set_var("PWD", &new_pwd);
                }
            }
            crate::utils::emit_osc7();
            return Some(0);
        } else {
            eprintln!("cd: nenhuma pasta anterior gravada.");
            return Some(1);
        }
    }

    // Auto-cd behavior: typing a directory directly moves into it. Only
    // kicks in when `cmd` isn't otherwise runnable (a real PATH executable
    // or a user function), so a local dir that happens to share a name
    // with a real command (e.g. a `./pwd/` subfolder) doesn't shadow it.
    if args.len() == 1
        && Path::new(cmd).is_dir()
        && !is_executable(cmd)
        && !state.functions.lock().unwrap().contains_key(cmd)
    {
        let current = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let old_pwd = env::var("PWD").unwrap_or_else(|_| current.to_string_lossy().into_owned());
        if let Err(e) = env::set_current_dir(cmd) {
            eprintln!("cd: {}", e);
            return Some(1);
        }
        state.old_pwd = Some(current);
        unsafe {
            env::set_var("OLDPWD", &old_pwd);
            if let Ok(new_pwd) = env::current_dir() {
                env::set_var("PWD", &new_pwd);
            }
        }
        crate::utils::emit_osc7();
        return Some(0);
    }

    match cmd.as_str() {
        "cd" => {
            let target = if args.len() > 1 {
                Path::new(&args[1]).to_path_buf()
            } else {
                state.home_dir.clone()
            };
            let current = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let old_pwd = env::var("PWD").unwrap_or_else(|_| current.to_string_lossy().into_owned());
            if let Err(e) = env::set_current_dir(&target) {
                if !state.quiet_errors {
                    eprintln!("cd: {}", e);
                }
                Some(1)
            } else {
                state.old_pwd = Some(current);
                unsafe {
                    env::set_var("OLDPWD", &old_pwd);
                    if let Ok(new_pwd) = env::current_dir() {
                        env::set_var("PWD", &new_pwd);
                    }
                }
                crate::utils::emit_osc7();
                Some(0)
            }
        }
        "jeofetch" => {
            run_jeofetch();
            Some(0)
        }
        "help" => {
            print_help();
            Some(0)
        }
        "jesh-path" | "jesh-which" | "jesh-which" => {
            if let Ok(exe) = env::current_exe() {
                println!("{}", exe.display());
            } else {
                println!("jesh");
            }
            Some(0)
        }
        "jesh-info" => {
            let exe_str = env::current_exe()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| "jesh".to_string());
            println!("jesh v{} ({})", env!("CARGO_PKG_VERSION"), exe_str);
            Some(0)
        }
        "jesh-version" => {
            let exe_str = env::current_exe()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| "jesh".to_string());
            println!("jesh v{} ({})", env!("CARGO_PKG_VERSION"), exe_str);
            Some(0)
        }
        "version" => {
            let exe_str = env::current_exe()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| "jesh".to_string());
            println!("jesh {} ({})", env!("CARGO_PKG_VERSION"), exe_str);
            Some(0)
        }
        "true" | ":" => Some(0),
        "false" => Some(1),
        "export" => {
            for arg in &args[1..] {
                if let Some(eq) = arg.find('=') {
                    let (name, value) = (&arg[..eq], &arg[eq + 1..]);
                    state.export_var(name, Some(value));
                } else {
                    state.export_var(arg, None);
                }
            }
            Some(0)
        }
        "unset" => {
            for arg in &args[1..] {
                state.unset_var(arg);
            }
            Some(0)
        }
        "set" => {
            let mutargs = &args[1..];
            // `set -o` alone: list all options and their state.
            if mutargs.len() == 1 && mutargs[0] == "-o" {
                println!("pipefail    \t{}", if state.pipefail { "on" } else { "off" });
                println!("nullglob    \t{}", if state.glob_nullglob { "on" } else { "off" });
                println!("failglob    \t{}", if state.glob_failglob { "on" } else { "off" });
                println!("dotglob     \t{}", if state.glob_dotglob { "on" } else { "off" });
                println!("nocaseglob  \t{}", if state.glob_nocaseglob { "on" } else { "off" });
                println!("extglob     \t{}", if state.glob_extglob { "on" } else { "off" });
                return Some(0);
            }
            // `set -o option` or `set +o option`
            if mutargs.len() >= 2 && (mutargs[0] == "-o" || mutargs[0] == "+o") {
                let opt_name = &mutargs[1];
                let enable = mutargs[0] == "-o";
                match opt_name.as_str() {
                    "pipefail" => {
                        state.pipefail = enable;
                        return Some(0);
                    }
                    "nullglob" => {
                        state.glob_nullglob = enable;
                        return Some(0);
                    }
                    "failglob" => {
                        state.glob_failglob = enable;
                        return Some(0);
                    }
                    "dotglob" => {
                        state.glob_dotglob = enable;
                        return Some(0);
                    }
                    "nocaseglob" => {
                        state.glob_nocaseglob = enable;
                        return Some(0);
                    }
                    "extglob" => {
                        state.glob_extglob = enable;
                        return Some(0);
                    }
                    _ => {
                        eprintln!("set: {}: opção desconhecida", opt_name);
                        return Some(1);
                    }
                }
            }
            let map = state.shell_vars.lock().unwrap();
            let mut names: Vec<String> = map.keys().cloned().collect();
            names.sort();
            for name in names {
                if let Some(val) = map.get(&name) {
                    println!("{}={}", name, val);
                }
            }
            Some(0)
        }
        "alias" => {
            if args.len() == 1 {
                let map = state.aliases.lock().unwrap();
                let mut names: Vec<&String> = map.keys().collect();
                names.sort();
                for name in names {
                    println!("alias {}='{}'", name, map[name]);
                }
            } else {
                let mut map = state.aliases.lock().unwrap();
                for arg in &args[1..] {
                    if let Some(eq) = arg.find('=') {
                        let name = &arg[..eq];
                        let value = arg[eq + 1..].trim_matches('"').trim_matches('\'');
                        map.insert(name.to_string(), value.to_string());
                    } else if let Some(v) = map.get(arg) {
                        println!("alias {}='{}'", arg, v);
                    }
                }
            }
            Some(0)
        }
        "unalias" => {
            let mut map = state.aliases.lock().unwrap();
            for arg in &args[1..] {
                map.remove(arg);
            }
            Some(0)
        }
        "source" | "." => {
            if args.len() < 2 {
                eprintln!("{}: nome de arquivo esperado", cmd);
                return Some(1);
            }
            match fs::read_to_string(&args[1]) {
                Ok(content) => {
                    if ShellState::looks_like_bash(&content) {
                        state.bash_sourced_files.push(PathBuf::from(&args[1]));
                        let prev = state.quiet_errors;
                        state.quiet_errors = true;
                        state.run_script_text(&content);
                        state.quiet_errors = prev;
                    } else {
                        state.run_script_text(&content);
                    }
                    Some(state.last_exit_status)
                }
                Err(e) => {
                    if !state.quiet_errors {
                        eprintln!("{}: {}: {}", cmd, args[1], e);
                    }
                    Some(1)
                }
            }
        }
        "history" => {
            if args.len() > 1 {
                match args[1].as_str() {
                    "pin" => {
                        if args.len() > 2 {
                            let cmd_to_pin = args[2..].join(" ");
                            if let Err(e) = state.history_mgr.pin_command(&cmd_to_pin) {
                                eprintln!("history pin: erro ao fixar comando: {}", e);
                                Some(1)
                            } else {
                                println!("Comando fixado com sucesso: {}", cmd_to_pin);
                                Some(0)
                            }
                        } else {
                            eprintln!("Uso: history pin <comando>");
                            Some(1)
                        }
                    }
                    "unpin" => {
                        if args.len() > 2 {
                            let cmd_to_unpin = args[2..].join(" ");
                            if let Err(e) = state.history_mgr.unpin_command(&cmd_to_unpin) {
                                eprintln!("history unpin: erro ao desafixar comando: {}", e);
                                Some(1)
                            } else {
                                println!("Comando desafixado com sucesso: {}", cmd_to_unpin);
                                Some(0)
                            }
                        } else {
                            eprintln!("Uso: history unpin <comando>");
                            Some(1)
                        }
                    }
                    "clear" => {
                        if let Err(e) = state.history_mgr.clear_history() {
                            eprintln!("history clear: erro ao limpar histórico: {}", e);
                            Some(1)
                        } else {
                            println!("Histórico limpo com sucesso.");
                            Some(0)
                        }
                    }
                    "tty" => {
                        state.history_mgr.print_history(Some(&crate::shell::history::current_tty()));
                        Some(0)
                    }
                    _ => {
                        eprintln!("Subcomando desconhecido. Subcomandos válidos: pin, unpin, clear, tty");
                        Some(1)
                    }
                }
            } else {
                state.history_mgr.print_history(None);
                Some(0)
            }
        }
        "shopt" => {
            // shopt [-pqsu] [-o] [optname ...]
            // For simplicity, we support: shopt -s option, shopt -u option, shopt option
            let mut i = 1;
            let mut set_val = None::<bool>; // None = query, Some(true) = -s, Some(false) = -u
            let mut optname = None::<String>;

            while i < args.len() {
                match args[i].as_str() {
                    "-s" => { set_val = Some(true); i += 1; }
                    "-u" => { set_val = Some(false); i += 1; }
                    "-p" | "-q" => { i += 1; } // ignore for now
                    "-o" => {
                        // Treat -o as synonym for -s (set option)
                        if i + 1 < args.len() {
                            optname = Some(args[i + 1].clone());
                            set_val = Some(true);
                            i += 2;
                        } else {
                            eprintln!("shopt: -o: opção requer um argumento");
                            return Some(1);
                        }
                    }
                    arg if !arg.starts_with('-') => {
                        if optname.is_none() {
                            optname = Some(arg.to_string());
                        }
                        i += 1;
                    }
                    _ => {
                        eprintln!("shopt: {}: opção inválida", args[i]);
                        return Some(1);
                    }
                }
            }

            if let Some(name) = optname {
                let enable = set_val.unwrap_or(true);
                match name.as_str() {
                    "nullglob" => { state.glob_nullglob = enable; }
                    "failglob" => { state.glob_failglob = enable; }
                    "dotglob" => { state.glob_dotglob = enable; }
                    "nocaseglob" => { state.glob_nocaseglob = enable; }
                    "extglob" => { state.glob_extglob = enable; }
                    _ => {
                        eprintln!("shopt: {}: opção inválida", name);
                        return Some(1);
                    }
                }
            } else if set_val.is_some() {
                eprintln!("shopt: opção requerida");
                return Some(1);
            } else {
                // List all options
                println!("nullglob    \t{}", if state.glob_nullglob { "on" } else { "off" });
                println!("failglob    \t{}", if state.glob_failglob { "on" } else { "off" });
                println!("dotglob     \t{}", if state.glob_dotglob { "on" } else { "off" });
                println!("nocaseglob  \t{}", if state.glob_nocaseglob { "on" } else { "off" });
                println!("extglob     \t{}", if state.glob_extglob { "on" } else { "off" });
            }
            Some(0)
        }
        "test" | "[" => {
            let test_args = &args[1..];
            if cmd == "[" {
                if test_args.last().map(|s| s.as_str()) != Some("]") {
                    eprintln!("[: esperado `]'");
                    return Some(2);
                }
                let test_args = &test_args[..test_args.len() - 1];
                Some(test_eval(test_args))
            } else {
                Some(test_eval(test_args))
            }
        }
        "[[" => {
            // [[ is a keyword with special parsing, but here we handle it as a builtin
            // with all remaining args until ]]
            let test_args = &args[1..];
            if test_args.last().map(|s| s.as_str()) != Some("]]") {
                eprintln!("[[:: esperado `]]'");
                return Some(2);
            }
            let test_args = &test_args[..test_args.len() - 1];
            Some(test_eval_bracket(test_args))
        }
        "exit" => {
            let code = args
                .get(1)
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(0);
            std::process::exit(code);
        }
        "exec" => {
            if args.len() < 2 {
                Some(0)
            } else {
                use std::os::unix::process::CommandExt;
                let mut cmd = std::process::Command::new(&args[1]);
                cmd.args(&args[2..]);
                let err = cmd.exec();
                eprintln!("jesh: exec: {}: {}", args[1], err);
                let exit_code = match err.kind() {
                    std::io::ErrorKind::NotFound => 127,
                    std::io::ErrorKind::PermissionDenied => 126,
                    _ => 1,
                };
                std::process::exit(exit_code);
            }
        }
        "printf" => {
            if args.len() < 2 {
                eprintln!("printf: formato esperado");
                return Some(1);
            }
            let fmt = &args[1];
            let values = &args[2..];
            let mut idx = 0usize;
            let mut out = String::new();
            let mut chars = fmt.chars().peekable();
            while let Some(c) = chars.next() {
                if c == '%' {
                    if let Some(next) = chars.next() {
                        match next {
                            '%' => out.push('%'),
                            's' => {
                                out.push_str(values.get(idx).map(|s| s.as_str()).unwrap_or(""));
                                idx += 1;
                            }
                            'd' | 'i' => {
                                let v = values.get(idx).and_then(|s| s.parse::<i64>().ok()).unwrap_or(0);
                                out.push_str(&v.to_string());
                                idx += 1;
                            }
                            'u' => {
                                let v = values.get(idx).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
                                out.push_str(&v.to_string());
                                idx += 1;
                            }
                            'x' => {
                                let v = values.get(idx).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
                                out.push_str(&format!("{:x}", v));
                                idx += 1;
                            }
                            'X' => {
                                let v = values.get(idx).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
                                out.push_str(&format!("{:X}", v));
                                idx += 1;
                            }
                            'f' => {
                                let v = values.get(idx).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                                out.push_str(&v.to_string());
                                idx += 1;
                            }
                            'c' => {
                                let v = values.get(idx).and_then(|s| s.chars().next()).unwrap_or('\0');
                                out.push(v);
                                idx += 1;
                            }
                            'b' => {
                                let v = values.get(idx).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
                                out.push_str(&format!("{:b}", v));
                                idx += 1;
                            }
                            'o' => {
                                let v = values.get(idx).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
                                out.push_str(&format!("{:o}", v));
                                idx += 1;
                            }
                            _ => {
                                out.push('%');
                                out.push(next);
                            }
                        }
                    } else {
                        out.push('%');
                    }
                } else {
                    out.push(c);
                }
            }
            print!("{}", out);
            let _ = Write::flush(&mut std::io::stdout());
            Some(0)
        }
        "eval" => {
            if args.len() > 1 {
                let script = args[1..].join(" ");
                state.run_script_text(&script);
            }
            Some(state.last_exit_status)
        }
        "command" => {
            if args.len() < 2 {
                return Some(0);
            }
            let argv = &args[1..];
            if let Some(status) = crate::builtin::handle_builtin(argv, state) {
                return Some(status);
            }
            if crate::builtin::is_executable(argv[0].as_str()) {
                if let Ok(mut cmd) = std::process::Command::new(argv[0].as_str())
                    .args(&argv[1..])
                    .status()
                {
                    state.last_exit_status = crate::utils::exit_code_from_status(cmd);
                } else {
                    state.last_exit_status = 1;
                }
                return Some(state.last_exit_status);
            }
            if let Some(status) = state.try_bash_fallback(argv[0].as_str(), &argv[1..]) {
                state.last_exit_status = status;
                return Some(status);
            }
            state.last_exit_status = 127;
            if !state.quiet_errors {
                eprintln!("jesh: command: {}: comando não encontrado", argv[0]);
            }
            Some(127)
        }
        "dirs" => {
            let mut clear = false;
            let mut verbose = false;
            for arg in &args[1..] {
                match arg.as_str() {
                    "-c" => clear = true,
                    "-v" => verbose = true,
                    _ => {
                        eprintln!("dirs: opção inválida: {}", arg);
                        return Some(1);
                    }
                }
            }
            if clear {
                state.dir_stack.clear();
                return Some(0);
            }
            let current = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let mut full_stack = vec![current];
            full_stack.extend(state.dir_stack.clone());

            let home = &state.home_dir;
            if verbose {
                for (i, path) in full_stack.iter().enumerate() {
                    println!("{:>2}  {}", i, format_path(path, home));
                }
            } else {
                let formatted: Vec<String> = full_stack.iter().map(|p| format_path(p, home)).collect();
                println!("{}", formatted.join(" "));
            }
            Some(0)
        }
        "pushd" => {
            if args.len() == 1 {
                if state.dir_stack.is_empty() {
                    eprintln!("pushd: a pilha de diretórios está vazia.");
                    return Some(1);
                }
                let target = state.dir_stack.remove(0);
                let current = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                let old_pwd = env::var("PWD").unwrap_or_else(|_| current.to_string_lossy().into_owned());
                if let Err(e) = env::set_current_dir(&target) {
                    eprintln!("pushd: {}: {}", target.display(), e);
                    state.dir_stack.insert(0, target);
                    return Some(1);
                }
                state.dir_stack.insert(0, current);
                state.old_pwd = Some(state.dir_stack[0].clone());
                unsafe {
                    env::set_var("OLDPWD", &old_pwd);
                    if let Ok(new_pwd) = env::current_dir() {
                        env::set_var("PWD", &new_pwd);
                    }
                }
                crate::utils::emit_osc7();

                let home = &state.home_dir;
                let current = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                let mut full_stack = vec![current];
                full_stack.extend(state.dir_stack.clone());
                let formatted: Vec<String> = full_stack.iter().map(|p| format_path(p, home)).collect();
                println!("{}", formatted.join(" "));
                Some(0)
            } else {
                let target_str = &args[1];
                let target_path = Path::new(target_str).to_path_buf();
                if target_path.is_dir() {
                    let current = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                    let old_pwd = env::var("PWD").unwrap_or_else(|_| current.to_string_lossy().into_owned());
                    if let Err(e) = env::set_current_dir(&target_path) {
                        eprintln!("pushd: {}: {}", target_path.display(), e);
                        return Some(1);
                    }
                    state.dir_stack.insert(0, current);
                    state.old_pwd = Some(state.dir_stack[0].clone());
                    unsafe {
                        env::set_var("OLDPWD", &old_pwd);
                        if let Ok(new_pwd) = env::current_dir() {
                            env::set_var("PWD", &new_pwd);
                        }
                    }
                    crate::utils::emit_osc7();

                    let home = &state.home_dir;
                    let current = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                    let mut full_stack = vec![current];
                    full_stack.extend(state.dir_stack.clone());
                    let formatted: Vec<String> = full_stack.iter().map(|p| format_path(p, home)).collect();
                    println!("{}", formatted.join(" "));
                    Some(0)
                } else if (target_str.starts_with('+') || target_str.starts_with('-')) && target_str.len() > 1 && target_str[1..].chars().all(|c| c.is_ascii_digit()) {
                    let num: usize = target_str[1..].parse().unwrap();
                    let current = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                    let mut full_stack = vec![current];
                    full_stack.extend(state.dir_stack.clone());

                    if num >= full_stack.len() {
                        eprintln!("pushd: {}: índice da pilha de diretórios fora de alcance", target_str);
                        return Some(1);
                    }

                    let rot_idx = if target_str.starts_with('+') {
                        num
                    } else {
                        full_stack.len() - num
                    };

                    if rot_idx == 0 {
                        return Some(0);
                    }

                    full_stack.rotate_left(rot_idx);
                    let target = full_stack[0].clone();
                    let old_pwd = env::var("PWD").unwrap_or_else(|_| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).to_string_lossy().into_owned());
                    if let Err(e) = env::set_current_dir(&target) {
                        eprintln!("pushd: {}: {}", target.display(), e);
                        return Some(1);
                    }
                    state.dir_stack = full_stack[1..].to_vec();
                    state.old_pwd = Some(PathBuf::from(old_pwd.clone()));
                    unsafe {
                        env::set_var("OLDPWD", &old_pwd);
                        if let Ok(new_pwd) = env::current_dir() {
                            env::set_var("PWD", &new_pwd);
                        }
                    }
                    crate::utils::emit_osc7();

                    let home = &state.home_dir;
                    let current = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                    let mut full_stack = vec![current];
                    full_stack.extend(state.dir_stack.clone());
                    let formatted: Vec<String> = full_stack.iter().map(|p| format_path(p, home)).collect();
                    println!("{}", formatted.join(" "));
                    Some(0)
                } else {
                    eprintln!("pushd: {}: diretório não encontrado", target_str);
                    Some(1)
                }
            }
        }
        "popd" => {
            if state.dir_stack.is_empty() {
                eprintln!("popd: a pilha de diretórios está vazia.");
                return Some(1);
            }
            if args.len() == 1 {
                let target = state.dir_stack.remove(0);
                let old_pwd = env::var("PWD").unwrap_or_else(|_| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).to_string_lossy().into_owned());
                if let Err(e) = env::set_current_dir(&target) {
                    eprintln!("popd: {}: {}", target.display(), e);
                    state.dir_stack.insert(0, target);
                    return Some(1);
                }
                state.old_pwd = Some(PathBuf::from(old_pwd.clone()));
                unsafe {
                    env::set_var("OLDPWD", &old_pwd);
                    if let Ok(new_pwd) = env::current_dir() {
                        env::set_var("PWD", &new_pwd);
                    }
                }
                crate::utils::emit_osc7();

                let home = &state.home_dir;
                let current = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                let mut full_stack = vec![current];
                full_stack.extend(state.dir_stack.clone());
                let formatted: Vec<String> = full_stack.iter().map(|p| format_path(p, home)).collect();
                println!("{}", formatted.join(" "));
                Some(0)
            } else {
                let target_str = &args[1];
                if (target_str.starts_with('+') || target_str.starts_with('-')) && target_str.len() > 1 && target_str[1..].chars().all(|c| c.is_ascii_digit()) {
                    let num: usize = target_str[1..].parse().unwrap();
                    let current = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                    let mut full_stack = vec![current];
                    full_stack.extend(state.dir_stack.clone());

                    if num >= full_stack.len() {
                        eprintln!("popd: {}: índice da pilha de diretórios fora de alcance", target_str);
                        return Some(1);
                    }

                    let rot_idx = if target_str.starts_with('+') {
                        num
                    } else {
                        full_stack.len() - num
                    };

                    if rot_idx == 0 {
                        let target = state.dir_stack.remove(0);
                        let old_pwd = env::var("PWD").unwrap_or_else(|_| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).to_string_lossy().into_owned());
                        if let Err(e) = env::set_current_dir(&target) {
                            eprintln!("popd: {}: {}", target.display(), e);
                            state.dir_stack.insert(0, target);
                            return Some(1);
                        }
                        state.old_pwd = Some(PathBuf::from(old_pwd.clone()));
                        unsafe {
                            env::set_var("OLDPWD", &old_pwd);
                            if let Ok(new_pwd) = env::current_dir() {
                                env::set_var("PWD", &new_pwd);
                            }
                        }
                        crate::utils::emit_osc7();
                    } else {
                        state.dir_stack.remove(rot_idx - 1);
                    }

                    let home = &state.home_dir;
                    let current = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                    let mut full_stack = vec![current];
                    full_stack.extend(state.dir_stack.clone());
                    let formatted: Vec<String> = full_stack.iter().map(|p| format_path(p, home)).collect();
                    println!("{}", formatted.join(" "));
                    Some(0)
                } else {
                    eprintln!("popd: {}: opção inválida", target_str);
                    Some(1)
                }
            }
        }
        "read" => {
            use std::io::{self, Write, IsTerminal};

            let mut prompt: Option<String> = None;
            let mut silent = false;
            let mut vars = Vec::new();

            let mut i = 1;
            while i < args.len() {
                let arg = &args[i];
                if arg == "-p" {
                    if i + 1 < args.len() {
                        prompt = Some(args[i + 1].clone());
                        i += 2;
                    } else {
                        eprintln!("read: -p: requer um argumento");
                        return Some(1);
                    }
                } else if arg.starts_with("-p") {
                    prompt = Some(arg[2..].to_string());
                    i += 1;
                } else if arg == "-s" {
                    silent = true;
                    i += 1;
                } else if arg.starts_with('-') && arg.len() > 1 {
                    let mut chars = arg.chars().skip(1).peekable();
                    while let Some(c) = chars.next() {
                        match c {
                            's' => silent = true,
                            'p' => {
                                let remaining: String = chars.collect();
                                if !remaining.is_empty() {
                                    prompt = Some(remaining);
                                } else if i + 1 < args.len() {
                                    prompt = Some(args[i + 1].clone());
                                    i += 1;
                                } else {
                                    eprintln!("read: -p: requer um argumento");
                                    return Some(1);
                                }
                                break;
                            }
                            _ => {
                                eprintln!("read: opção inválida: -{}", c);
                                return Some(1);
                            }
                        }
                    }
                    i += 1;
                } else {
                    vars.push(arg.clone());
                    i += 1;
                }
            }

            if let Some(ref p) = prompt {
                let mut stderr = io::stderr();
                let _ = stderr.write_all(p.as_bytes());
                let _ = stderr.flush();
            }

            let mut line = String::new();
            let is_term = io::stdin().is_terminal();

            let bytes_read = if silent && is_term {
                #[cfg(unix)]
                {
                    use libc::{tcgetattr, tcsetattr, ECHO, TCSANOW, termios};
                    use std::os::unix::io::AsRawFd;
                    let fd = io::stdin().as_raw_fd();
                    let mut term: termios = unsafe { std::mem::zeroed() };
                    let has_term = unsafe { tcgetattr(fd, &mut term) == 0 };
                    if has_term {
                        let mut term_silent = term;
                        term_silent.c_lflag &= !ECHO;
                        let _ = unsafe { tcsetattr(fd, TCSANOW, &term_silent) };
                        let res = io::stdin().read_line(&mut line);
                        let _ = unsafe { tcsetattr(fd, TCSANOW, &term) };
                        eprintln!(); // Print a newline since user pressed Enter but echo was off
                        match res {
                            Ok(n) => n,
                            Err(_) => 0,
                        }
                    } else {
                        match io::stdin().read_line(&mut line) {
                            Ok(n) => n,
                            Err(_) => 0,
                        }
                    }
                }
                #[cfg(not(unix))]
                {
                    match io::stdin().read_line(&mut line) {
                        Ok(n) => n,
                        Err(_) => 0,
                    }
                }
            } else {
                match io::stdin().read_line(&mut line) {
                    Ok(n) => n,
                    Err(_) => 0,
                }
            };

            if bytes_read == 0 {
                // EOF reached
                if vars.is_empty() {
                    state.set_var("REPLY", "");
                } else {
                    for var_name in &vars {
                        state.set_var(var_name, "");
                    }
                }
                return Some(1);
            }

            if line.ends_with('\n') {
                line.pop();
            }
            if line.ends_with('\r') {
                line.pop();
            }

            let ifs = state.get_var("IFS");
            let ifs_chars: Vec<char> = if ifs.is_empty() {
                vec![' ', '\t', '\n']
            } else {
                ifs.chars().collect()
            };

            let is_space = |c: char| c == ' ' || c == '\t' || c == '\n';
            let mut words = Vec::new();
            let mut current_word = String::new();

            // Trim leading IFS whitespace
            let mut input_str = line.as_str();
            while let Some(c) = input_str.chars().next() {
                if ifs_chars.contains(&c) && is_space(c) {
                    input_str = &input_str[c.len_utf8()..];
                } else {
                    break;
                }
            }

            // Trim trailing IFS whitespace
            while let Some(c) = input_str.chars().last() {
                if ifs_chars.contains(&c) && is_space(c) {
                    input_str = &input_str[..input_str.len() - c.len_utf8()];
                } else {
                    break;
                }
            }

            let mut chars = input_str.chars().peekable();
            while let Some(c) = chars.next() {
                if ifs_chars.contains(&c) {
                    let c_is_whitespace = is_space(c);
                    if c_is_whitespace {
                        if !current_word.is_empty() {
                            words.push(current_word.clone());
                            current_word.clear();
                        }
                    } else {
                        words.push(current_word.clone());
                        current_word.clear();
                        if let Some(&nc) = chars.peek() {
                            if ifs_chars.contains(&nc) && is_space(nc) {
                                chars.next();
                            }
                        }
                    }
                } else {
                    current_word.push(c);
                }
            }
            words.push(current_word);

            if vars.is_empty() {
                state.set_var("REPLY", input_str);
            } else {
                let num_vars = vars.len();
                for (idx, var_name) in vars.iter().enumerate() {
                    if idx == num_vars - 1 {
                        if idx < words.len() {
                            let mut temp_str = input_str;
                            for w_idx in 0..idx {
                                if let Some(pos) = temp_str.find(&words[w_idx]) {
                                    temp_str = &temp_str[pos + words[w_idx].len()..];
                                }
                            }
                            if let Some(pos) = temp_str.find(&words[idx]) {
                                let remainder = &temp_str[pos..];
                                state.set_var(var_name, remainder);
                            } else {
                                state.set_var(var_name, &words[idx..].join(" "));
                            }
                        } else {
                            state.set_var(var_name, "");
                        }
                    } else {
                        if idx < words.len() {
                            state.set_var(var_name, &words[idx]);
                        } else {
                            state.set_var(var_name, "");
                        }
                    }
                }
            }

            Some(0)
        }
        "complete" => {
            // complete -W "word1 word2" cmdname
            // complete -F _function cmdname
            if args.len() < 4 {
                eprintln!("complete: uso: complete -W \"palavras\" comando ou complete -F função comando");
                return Some(2);
            }
            let flag = &args[1];
            let list_or_func = &args[2];
            let cmd = &args[3];
            match flag.as_str() {
                "-W" => {
                    let words: Vec<String> = list_or_func.split_whitespace().map(|s| s.to_string()).collect();
                    let mut completions = state.completions.lock().unwrap();
                    completions.register_word_list(cmd, &words);
                    Some(0)
                }
                "-F" => {
                    let mut completions = state.completions.lock().unwrap();
                    completions.register_completer(cmd, list_or_func);
                    Some(0)
                }
                _ => {
                    eprintln!("complete: opção inválida: {}", flag);
                    Some(2)
                }
            }
        }
        "declare" | "typeset" => {
            // declare [-airx] [name[=value] ...]
            // typeset is synonym for declare
            let mut i = 1;
            let mut attrs = crate::shell::VarAttrs::default();
            let mut names = Vec::new();
            
            while i < args.len() {
                let arg = &args[i];
                if arg.starts_with('-') && arg.len() > 1 && !arg.contains('=') {
                    // Parse flags
                    for ch in arg.chars().skip(1) {
                        match ch {
                            'i' => attrs.integer = true,
                            'a' => attrs.array = true,
                            'A' => attrs.assoc = true,
                            'r' => attrs.readonly = true,
                            'x' => attrs.exported = true,
                            _ => {
                                eprintln!("{}: -{}: opção inválida", cmd, ch);
                                return Some(1);
                            }
                        }
                    }
                    i += 1;
                } else {
                    // Name or name=value
                    names.push(arg.clone());
                    i += 1;
                }
            }
            
            let mut attrs_map = state.var_attrs.lock().unwrap();
            let mut vars_map = state.shell_vars.lock().unwrap();
            
            for name_val in names {
                if let Some(eq) = name_val.find('=') {
                    let name = &name_val[..eq];
                    let value = &name_val[eq+1..];
                    // Validate name
                    if name.is_empty() || name.chars().next().unwrap().is_ascii_digit() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                        eprintln!("{}: '{}': não é um identificador válido", cmd, name);
                        return Some(1);
                    }
                    // Apply integer attribute if set
                    let final_value = if attrs.integer {
                        // Evaluate as arithmetic
                        value.to_string() // Simplified - would need arithmetic eval
                    } else {
                        value.to_string()
                    };
                    vars_map.insert(name.to_string(), final_value.clone());
                    attrs_map.insert(name.to_string(), attrs.clone());
                    if attrs.exported {
                        state.exported.insert(name.to_string());
                        unsafe { env::set_var(name, &final_value); }
                    }
                    if attrs.readonly {
                        state.readonly_vars.insert(name.to_string());
                    }
                } else {
                    let name = &name_val;
                    // Validate name
                    if name.is_empty() || name.chars().next().unwrap().is_ascii_digit() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                        eprintln!("{}: '{}': não é um identificador válido", cmd, name);
                        return Some(1);
                    }
                    // Create variable with attributes if it doesn't exist
                    if !vars_map.contains_key(name) {
                        vars_map.insert(name.to_string(), String::new());
                        attrs_map.insert(name.to_string(), attrs.clone());
                        if attrs.exported {
                            state.exported.insert(name.to_string());
                            unsafe { env::set_var(name, ""); }
                        }
                        if attrs.readonly {
                            state.readonly_vars.insert(name.to_string());
                        }
                    } else {
                        // Update attributes on existing variable
                        let mut existing_attrs = attrs_map.entry(name.to_string()).or_default();
                        if attrs.integer { existing_attrs.integer = true; }
                        if attrs.array { existing_attrs.array = true; }
                        if attrs.assoc { existing_attrs.assoc = true; }
                        if attrs.readonly { 
                            existing_attrs.readonly = true; 
                            state.readonly_vars.insert(name.to_string());
                        }
                        if attrs.exported { 
                            existing_attrs.exported = true; 
                            state.exported.insert(name.to_string());
                            if let Some(v) = vars_map.get(name) {
                                unsafe { env::set_var(name, v); }
                            }
                        }
                    }
                }
            }
            Some(0)
        }
        "local" => {
            // local [option] name[=value] ...
            // Creates local variables in function scope
            if state.positional_stack.is_empty() {
                eprintln!("local: só pode ser usado em funções");
                return Some(1);
            }
            let mut i = 1;
            let mut attrs = crate::shell::VarAttrs::default();
            attrs.local = true;
            let mut names = Vec::new();
            
            while i < args.len() {
                let arg = &args[i];
                if arg.starts_with('-') && arg.len() > 1 && !arg.contains('=') {
                    // Parse flags (same as declare)
                    for ch in arg.chars().skip(1) {
                        match ch {
                            'i' => attrs.integer = true,
                            'a' => attrs.array = true,
                            'A' => attrs.assoc = true,
                            'r' => attrs.readonly = true,
                            'x' => attrs.exported = true,
                            _ => {
                                eprintln!("local: -{}: opção inválida", ch);
                                return Some(1);
                            }
                        }
                    }
                    i += 1;
                } else {
                    names.push(arg.clone());
                    i += 1;
                }
            }
            
            // Push new frames for local variables
            state.var_attrs_stack.push(HashMap::new());
            state.var_values_stack.push(HashMap::new());
            
            let mut attrs_map = state.var_attrs.lock().unwrap();
            let mut vars_map = state.shell_vars.lock().unwrap();
            
            for name_val in names {
                if let Some(eq) = name_val.find('=') {
                    let name = &name_val[..eq];
                    let value = &name_val[eq+1..];
                    if name.is_empty() || name.chars().next().unwrap().is_ascii_digit() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                        eprintln!("local: '{}': não é um identificador válido", name);
                        return Some(1);
                    }
                    // Save old value/attrs if they exist
                    if let Some(old_val) = vars_map.get(name).cloned() {
                        state.var_values_stack.last_mut().unwrap().insert(name.to_string(), old_val);
                    }
                    if let Some(old_attrs) = attrs_map.get(name).cloned() {
                        state.var_attrs_stack.last_mut().unwrap().insert(name.to_string(), old_attrs);
                    }
                    // Set new local value
                    vars_map.insert(name.to_string(), value.to_string());
                    attrs_map.insert(name.to_string(), attrs.clone());
                } else {
                    let name = &name_val;
                    if name.is_empty() || name.chars().next().unwrap().is_ascii_digit() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                        eprintln!("local: '{}': não é um identificador válido", name);
                        return Some(1);
                    }
                    // Save old value/attrs if they exist
                    if let Some(old_val) = vars_map.get(name).cloned() {
                        state.var_values_stack.last_mut().unwrap().insert(name.to_string(), old_val);
                    }
                    if let Some(old_attrs) = attrs_map.get(name).cloned() {
                        state.var_attrs_stack.last_mut().unwrap().insert(name.to_string(), old_attrs);
                    }
                    // Create new local variable (empty)
                    vars_map.insert(name.to_string(), String::new());
                    attrs_map.insert(name.to_string(), attrs.clone());
                }
            }
            Some(0)
        }
        "readonly" => {
            // readonly [name[=value] ...] or readonly -p
            let mut i = 1;
            let mut print_mode = false;
            let mut names = Vec::new();
            
            while i < args.len() {
                let arg = &args[i];
                if arg == "-p" {
                    print_mode = true;
                    i += 1;
                } else if arg.starts_with('-') && arg.len() > 1 {
                    eprintln!("readonly: {}: opção inválida", arg);
                    return Some(1);
                } else {
                    names.push(arg.clone());
                    i += 1;
                }
            }
            
            if print_mode || names.is_empty() {
                // Print all readonly variables
                let attrs_map = state.var_attrs.lock().unwrap();
                let vars_map = state.shell_vars.lock().unwrap();
                let mut names: Vec<String> = attrs_map.keys()
                    .filter(|k| attrs_map.get(*k).map(|a| a.readonly).unwrap_or(false))
                    .cloned()
                    .collect();
                names.sort();
                for name in names {
                    if let Some(val) = vars_map.get(&name) {
                        println!("readonly {}=\"{}\"", name, val);
                    } else {
                        println!("readonly {}", name);
                    }
                }
                return Some(0);
            }
            
            let mut attrs_map = state.var_attrs.lock().unwrap();
            let mut vars_map = state.shell_vars.lock().unwrap();
            
            for name_val in names {
                if let Some(eq) = name_val.find('=') {
                    let name = &name_val[..eq];
                    let value = &name_val[eq+1..];
                    if name.is_empty() || name.chars().next().unwrap().is_ascii_digit() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                        eprintln!("readonly: '{}': não é um identificador válido", name);
                        return Some(1);
                    }
                    vars_map.insert(name.to_string(), value.to_string());
                    let mut attrs = attrs_map.entry(name.to_string()).or_default();
                    attrs.readonly = true;
                    state.readonly_vars.insert(name.to_string());
                } else {
                    let name = &name_val;
                    if name.is_empty() || name.chars().next().unwrap().is_ascii_digit() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                        eprintln!("readonly: '{}': não é um identificador válido", name);
                        return Some(1);
                    }
                    let mut attrs = attrs_map.entry(name.to_string()).or_default();
                    attrs.readonly = true;
                    state.readonly_vars.insert(name.to_string());
                }
            }
            Some(0)
        }
        "getopts" => {
            // getopts optstring var [args...]
            // Simplified implementation - just parse single-letter options
            if args.len() < 3 {
                eprintln!("getopts: uso: getopts optstring var [args...]");
                return Some(1);
            }
            let optstring = &args[1];
            let var_name = &args[2];
            let args_to_parse = &args[3..];
            
            // Get OPTIND from shell vars (default 1)
            let optind_str = state.get_var("OPTIND");
            let mut optind = optind_str.parse::<usize>().unwrap_or(1);
            if optind < 1 { optind = 1; }
            
            // Get OPTARG if set
            let _optarg = state.get_var("OPTARG");
            
            // For simplicity, we'll just parse the first argument that starts with -
            // A full implementation would track state across calls
            let mut found = false;
            let mut opt_char = '?';
            let mut opt_arg = String::new();
            
            if optind < args_to_parse.len() {
                let arg = &args_to_parse[optind];
                if arg.starts_with('-') && arg.len() >= 2 {
                    let opt = arg.chars().nth(1).unwrap_or('?');
                    if optstring.contains(opt) {
                        found = true;
                        opt_char = opt;
                        // Check if this option takes an argument
                        let opt_pos = optstring.find(opt).unwrap();
                        if opt_pos + 1 < optstring.len() && optstring.chars().nth(opt_pos + 1) == Some(':') {
                            // Option takes argument
                            if arg.len() > 2 {
                                opt_arg = arg[2..].to_string();
                            } else if optind + 1 < args_to_parse.len() {
                                optind += 1;
                                opt_arg = args_to_parse[optind].clone();
                            } else {
                                opt_char = ':';
                                opt_arg = opt.to_string();
                            }
                        }
                    }
                }
            }
            
            if found {
                state.set_var(var_name, &opt_char.to_string());
                if !opt_arg.is_empty() {
                    state.set_var("OPTARG", &opt_arg);
                } else {
                    state.set_var("OPTARG", "");
                }
                optind += 1;
                state.set_var("OPTIND", &optind.to_string());
                Some(0)
            } else {
                state.set_var(var_name, "?");
                state.set_var("OPTARG", "");
                Some(1)
            }
        }
        _ => None,
        "hyperlink" => {
            let mut text = String::new();
            let mut url = String::new();
            let mut args_iter = args.iter().skip(1);
            while let Some(arg) = args_iter.next() {
                if arg == "--text" || arg == "-t" {
                    if let Some(val) = args_iter.next() {
                        text = val.clone();
                    }
                } else if arg == "--url" || arg == "-u" {
                    if let Some(val) = args_iter.next() {
                        url = val.clone();
                    }
                } else if text.is_empty() {
                    text = arg.clone();
                } else if url.is_empty() {
                    url = arg.clone();
                }
            }
            if url.is_empty() {
                eprintln!("hyperlink: URL is required");
                return Some(1);
            }
            if text.is_empty() {
                text = url.clone();
            }
            crate::utils::osc8_hyperlink(&text, &url);
            let _ = std::io::stdout().write_all(text.as_bytes());
            Some(0)
        }
        "kitty" => {
            let sub: String = args.get(1).cloned().unwrap_or_default();
            match sub.as_str() {
                "image" | "img" => {
                    let path = args.get(2);
                    if let Some(p) = path {
                        match std::fs::read(p) {
                            Ok(data) => {
                                let fmt = if p.ends_with(".png") { "png" }
                                    else if p.ends_with(".jpg") || p.ends_with(".jpeg") { "jpeg" }
                                    else if p.ends_with(".gif") { "gif" }
                                    else { "png" };
                                crate::utils::kitty_send_image(&data, fmt, 1, 0, 0);
                            }
                            Err(e) => {
                                eprintln!("kitty: image: {}: {}", p, e);
                                return Some(1);
                            }
                        }
                        Some(0)
                    } else {
                        eprintln!("kitty image: path required");
                        Some(1)
                    }
                }
                "clear" | "rm" => {
                    let image_ids: Vec<String> = args.iter().skip(2).cloned().collect();
                    let cmd: String = if image_ids.is_empty() {
                        "\x1b_Ga=d;\x1b\\".to_string()
                    } else {
                        let ids = image_ids.join(",");
                        format!("\x1b_Ga=d;a={};\x1b\\", ids)
                    };
                    let _ = std::io::stdout().write_all(cmd.as_bytes());
                    let _ = std::io::stdout().flush();
                    Some(0)
                }
                _ => {
                    eprintln!("kitty: subcommand required (image, clear)");
                    Some(1)
                }
            }
        }
        "table" => {
            if args.len() < 2 {
                eprintln!("table: delimiter and column names required");
                eprintln!("Usage: table | col1,col2,... | [header=auto|none]");
                return Some(1);
            }
            let delim = &args[1];
            let mut headers = Vec::new();
            let mut auto_header = false;
            let mut rows: Vec<Vec<String>> = Vec::new();
            let mut col_widths: Vec<usize> = Vec::new();
            for (i, arg) in args.iter().enumerate().skip(2) {
                if arg.starts_with("header=") {
                    let val = &arg["header=".len()..];
                    if val == "auto" {
                        auto_header = true;
                    }
                } else if i == 2 {
                    headers = arg.split(delim).map(|s| s.to_string()).collect();
                    col_widths = headers.iter().map(|h| h.len()).collect();
                } else {
                    let cells: Vec<String> = arg.split(delim).map(|s| s.to_string()).collect();
                    for (j, cell) in cells.iter().enumerate() {
                        if j >= col_widths.len() {
                            col_widths.push(cell.len());
                        } else if cell.len() > col_widths[j] {
                            col_widths[j] = cell.len();
                        }
                    }
                    rows.push(cells);
                }
            }
            if auto_header && rows.len() > 1 {
                for j in 0..rows[0].len() {
                    let mut max = 3usize;
                    for row in &rows {
                        if j < row.len() && row[j].len() > max {
                            max = row[j].len();
                        }
                    }
                    if j < headers.len() && headers[j].len() < max {
                        headers[j] = format!("{:>width$}", headers[j], width = max);
                    }
                }
            }
            let header_line = headers.iter().enumerate().map(|(i, h)| {
                format!("{:<width$}", h, width = col_widths.get(i).copied().unwrap_or(h.len()))
            }).collect::<Vec<_>>().join("  ");
            let _ = std::io::stdout().write_all(format!("{}\n", header_line).as_bytes());
            let sep: String = col_widths.iter().enumerate().map(|(i, w)| {
                "-".repeat(*w + if i == 0 { 0 } else { 2 })
            }).collect::<Vec<_>>().join("  ");
            let _ = std::io::stdout().write_all(format!("{}\n", sep).as_bytes());
            for row in &rows {
                let line = row.iter().enumerate().map(|(i, cell)| {
                    format!("{:<width$}", cell, width = col_widths.get(i).copied().unwrap_or(cell.len()))
                }).collect::<Vec<_>>().join("  ");
                let _ = std::io::stdout().write_all(format!("{}\n", line).as_bytes());
            }
            Some(0)
        }
    }
}

// Test evaluation functions for test/[/[[

fn test_eval(args: &[String]) -> i32 {
    let mut args = args.iter().map(|s| s.as_str()).collect::<Vec<_>>();
    match args.len() {
        0 => 1, // empty test is false
        1 => unary_test(&args[0]),
        2 => {
            if args[0] == "!" {
                // ! <expr>
                if unary_test(&args[1]) == 0 { 1 } else { 0 }
            } else {
                // Unary operator with argument: -n, -z, -f, -d, -e, -s, etc.
                // Also handles plain string check when first arg is not an operator
                unary_op_test(args[0], args[1])
            }
        }
        3 => ternary_test(&args[0], &args[1], &args[2]),
        _ => {
            eprintln!("test: too many arguments");
            2
        }
    }
}

fn unary_test(arg: &str) -> i32 {
    // ! expr
    if arg == "!" {
        return 1;
    }
    // -n string: true if length > 0
    if arg.starts_with("-n") {
        return if arg.len() > 2 { 0 } else { 1 };
    }
    // -z string: true if length == 0
    if arg.starts_with("-z") {
        return if arg.len() > 2 { 1 } else { 0 };
    }
    // string: true if non-empty
    if !arg.is_empty() { 0 } else { 1 }
}

fn unary_op_test(op: &str, arg: &str) -> i32 {
    match op {
        "-n" => if !arg.is_empty() { 0 } else { 1 },
        "-z" => if arg.is_empty() { 0 } else { 1 },
        "-f" => if std::path::Path::new(arg).is_file() { 0 } else { 1 },
        "-d" => if std::path::Path::new(arg).is_dir() { 0 } else { 1 },
        "-e" => if std::path::Path::new(arg).exists() { 0 } else { 1 },
        "-s" => std::fs::metadata(arg).map(|m| if m.len() > 0 { 0 } else { 1 }).unwrap_or(1),
        "-L" => std::fs::read_link(arg).map(|_| 0).unwrap_or(1),
        _ => {
            // Not a recognized unary operator: treat as plain string test
            if !op.is_empty() { 0 } else { 1 }
        }
    }
}

fn binary_test(op: &str, arg1: &str, arg2: &str) -> i32 {
    // Binary operators: -nt, -ot, -ef, =, !=, -eq, -ne, -lt, -le, -gt, -ge
    match op {
        "-nt" => {
            eprintln!("test: {}: binary operator not implemented", op);
            2
        }
        "-ot" => {
            eprintln!("test: {}: binary operator not implemented", op);
            2
        }
        "-ef" => {
            eprintln!("test: {}: binary operator not implemented", op);
            2
        }
        "=" | "==" => if arg1 == arg2 { 0 } else { 1 },
        "!=" => if arg1 != arg2 { 0 } else { 1 },
        "-eq" => arg1.parse::<i64>().and_then(|n| arg2.parse::<i64>().map(|m| if n == m { 0 } else { 1 })).unwrap_or(2),
        "-ne" => arg1.parse::<i64>().and_then(|n| arg2.parse::<i64>().map(|m| if n != m { 0 } else { 1 })).unwrap_or(2),
        "-lt" => arg1.parse::<i64>().and_then(|n| arg2.parse::<i64>().map(|m| if n < m { 0 } else { 1 })).unwrap_or(2),
        "-le" => arg1.parse::<i64>().and_then(|n| arg2.parse::<i64>().map(|m| if n <= m { 0 } else { 1 })).unwrap_or(2),
        "-gt" => arg1.parse::<i64>().and_then(|n| arg2.parse::<i64>().map(|m| if n > m { 0 } else { 1 })).unwrap_or(2),
        "-ge" => arg1.parse::<i64>().and_then(|n| arg2.parse::<i64>().map(|m| if n >= m { 0 } else { 1 })).unwrap_or(2),
        _ => {
            eprintln!("test: {}: binary operator not recognized", op);
            2
        }
    }
}

fn ternary_test(arg1: &str, op: &str, arg2: &str) -> i32 {
    match op {
        "-nt" | "-ot" | "-ef" => {
            // File comparison operators
            let p1 = std::path::Path::new(arg1);
            let p2 = std::path::Path::new(arg2);
            match op {
                "-nt" => {
                    let m1 = p1.metadata().and_then(|m| m.modified()).ok();
                    let m2 = p2.metadata().and_then(|m| m.modified()).ok();
                    if let (Some(m1), Some(m2)) = (m1, m2) {
                        if m1 > m2 { 0 } else { 1 }
                    } else { 2 }
                }
                "-ot" => {
                    let m1 = p1.metadata().and_then(|m| m.modified()).ok();
                    let m2 = p2.metadata().and_then(|m| m.modified()).ok();
                    if let (Some(m1), Some(m2)) = (m1, m2) {
                        if m1 < m2 { 0 } else { 1 }
                    } else { 2 }
                }
                "-ef" => {
                    let m1 = p1.metadata().ok();
                    let m2 = p2.metadata().ok();
                    if let (Some(m1), Some(m2)) = (m1, m2) {
                        if m1.dev() == m2.dev() && m1.ino() == m2.ino() { 0 } else { 1 }
                    } else { 2 }
                }
                _ => 2,
            }
        }
        "=" | "==" => if arg1 == arg2 { 0 } else { 1 },
        "!=" => if arg1 != arg2 { 0 } else { 1 },
        "-eq" => arg1.parse::<i64>().and_then(|a1| arg2.parse::<i64>().map(|a2| if a1 == a2 { 0 } else { 1 })).unwrap_or(2),
        "-ne" => arg1.parse::<i64>().and_then(|a1| arg2.parse::<i64>().map(|a2| if a1 != a2 { 0 } else { 1 })).unwrap_or(2),
        "-lt" => arg1.parse::<i64>().and_then(|a1| arg2.parse::<i64>().map(|a2| if a1 < a2 { 0 } else { 1 })).unwrap_or(2),
        "-le" => arg1.parse::<i64>().and_then(|a1| arg2.parse::<i64>().map(|a2| if a1 <= a2 { 0 } else { 1 })).unwrap_or(2),
        "-gt" => arg1.parse::<i64>().and_then(|a1| arg2.parse::<i64>().map(|a2| if a1 > a2 { 0 } else { 1 })).unwrap_or(2),
        "-ge" => arg1.parse::<i64>().and_then(|a1| arg2.parse::<i64>().map(|a2| if a1 >= a2 { 0 } else { 1 })).unwrap_or(2),
        _ => {
            // Unary operators with argument: -n, -z, -f, -d, -e, -r, -w, -x, -s, -L, -p, -S, -b, -c, -t
            match op {
                "-n" => if !arg2.is_empty() { 0 } else { 1 },
                "-z" => if arg2.is_empty() { 0 } else { 1 },
                "-f" => if std::path::Path::new(arg2).is_file() { 0 } else { 1 },
                "-d" => if std::path::Path::new(arg2).is_dir() { 0 } else { 1 },
                "-e" => if std::path::Path::new(arg2).exists() { 0 } else { 1 },
                "-r" => std::fs::metadata(arg2).map(|m| if m.permissions().readonly() == false { 0 } else { 1 }).unwrap_or(1),
                "-w" => std::fs::metadata(arg2).map(|m| if m.permissions().readonly() == false { 0 } else { 1 }).unwrap_or(1),
                "-x" => {
                    use std::os::unix::fs::PermissionsExt;
                    std::fs::metadata(arg2).map(|m| if m.permissions().mode() & 0o111 != 0 { 0 } else { 1 }).unwrap_or(1)
                }
                "-s" => std::fs::metadata(arg2).map(|m| if m.len() > 0 { 0 } else { 1 }).unwrap_or(1),
                "-L" => std::fs::read_link(arg2).map(|_| 0).unwrap_or(1),
                _ => {
                    eprintln!("test: {}: unary operator not recognized", op);
                    2
                }
            }
        }
    }
}

// [[ extended test with regex matching
fn test_eval_bracket(args: &[String]) -> i32 {
    // For now, delegate to test_eval but with regex support
    // This is a simplified implementation
    let mut args = args.iter().map(|s| s.as_str()).collect::<Vec<_>>();
    match args.len() {
        0 => 1,
        1 => unary_test(&args[0]),
        2 => binary_test(&args[0], &args[1], &args[2]),
        3 => {
            // Check for =~, !~, ==, != with pattern matching
            if args[1] == "=~" || args[1] == "!~" {
                // Regex matching (simplified - just string contains for now)
                let pattern = args[2];
                let string = args[0];
                let matched = string.contains(pattern);
                if args[1] == "=~" { if matched { 0 } else { 1 } } else { if matched { 1 } else { 0 } }
            } else if args[1] == "==" || args[1] == "=" {
                if args[0] == args[2] { 0 } else { 1 }
            } else if args[1] == "!=" {
                if args[0] != args[2] { 0 } else { 1 }
            } else {
                ternary_test(&args[0], &args[1], &args[2])
            }
        }
        _ => {
            eprintln!("[[:: too many arguments");
            2
        }
    }
}

fn format_path(path: &Path, home: &Path) -> String {
    if let Ok(striped) = path.strip_prefix(home) {
        let mut s = String::from("~");
        if striped.as_os_str().is_empty() {
            s
        } else {
            s.push('/');
            s.push_str(&striped.to_string_lossy());
            s
        }
    } else {
        path.to_string_lossy().into_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::ShellState;

    #[test]
    fn test_pushd_popd_dirs() {
        let mut state = ShellState::new();
        let current = env::current_dir().unwrap();

        // 1. Initial stack should be empty
        assert!(state.dir_stack.is_empty());

        // 2. pushd to a directory (e.g. root "/")
        let res = handle_builtin(&["pushd".to_string(), "/".to_string()], &mut state);
        assert_eq!(res, Some(0));
        assert_eq!(env::current_dir().unwrap(), Path::new("/"));
        assert_eq!(state.dir_stack, vec![current.clone()]);

        // 3. pushd swap (no args)
        let res = handle_builtin(&["pushd".to_string()], &mut state);
        assert_eq!(res, Some(0));
        assert_eq!(env::current_dir().unwrap(), current);
        assert_eq!(state.dir_stack, vec![PathBuf::from("/")]);

        // 4. dirs output format verification or just clearing
        let res = handle_builtin(&["dirs".to_string(), "-c".to_string()], &mut state);
        assert_eq!(res, Some(0));
        assert!(state.dir_stack.is_empty());

        // 5. popd on empty stack should fail
        let res = handle_builtin(&["popd".to_string()], &mut state);
        assert_eq!(res, Some(1));

        // 6. pushd and popd
        let _ = handle_builtin(&["pushd".to_string(), "/".to_string()], &mut state);
        let res = handle_builtin(&["popd".to_string()], &mut state);
        assert_eq!(res, Some(0));
        assert_eq!(env::current_dir().unwrap(), current);
        assert!(state.dir_stack.is_empty());
    }

    #[test]
    #[cfg(unix)]
    fn test_read_builtin() {
        use std::os::unix::io::FromRawFd;
        let mut state = ShellState::new();

        // Save original stdin
        let original_stdin = unsafe { libc::dup(0) };

        // Create pipe
        let mut pipe_fds = [0; 2];
        unsafe {
            libc::pipe(pipe_fds.as_mut_ptr());
            // Redirect stdin (0) to the read end of the pipe
            libc::dup2(pipe_fds[0], 0);
            libc::close(pipe_fds[0]);
        }

        // Write some input to the pipe write end
        {
            use std::io::Write;
            let mut file = unsafe { std::fs::File::from_raw_fd(pipe_fds[1]) };
            writeln!(file, "hello   world   foo   bar").unwrap();
        }

        // Run read builtin
        let res = handle_builtin(&["read".to_string(), "var1".to_string(), "var2".to_string()], &mut state);
        assert_eq!(res, Some(0));

        // Verify variables
        assert_eq!(state.get_var("var1"), "hello");
        assert_eq!(state.get_var("var2"), "world   foo   bar");

        // Restore original stdin
        unsafe {
            libc::dup2(original_stdin, 0);
            libc::close(original_stdin);
        }
    }

    #[test]
    fn test_printf_builtin() {
        let state = &mut ShellState::new();
        assert_eq!(
            handle_builtin(&["printf".into(), "%s".into(), "hello".into()], state),
            Some(0)
        );
        assert_eq!(
            handle_builtin(&["printf".into(), "%d %s".into(), "42".into(), "ok".into()], state),
            Some(0)
        );
        assert_eq!(
            handle_builtin(&["printf".into()], state),
            Some(1)
        );
    }

    #[test]
    fn test_eval_builtin() {
        let mut state = ShellState::new();
        assert_eq!(
            handle_builtin(&["eval".into(), "true".into()], &mut state),
            Some(0)
        );
        assert_eq!(
            handle_builtin(&["eval".into(), "false".into()], &mut state),
            Some(1)
        );
    }

    #[test]
    fn test_command_builtin_runs_external() {
        let mut state = ShellState::new();
        assert_eq!(
            handle_builtin(&["command".into(), "true".into()], &mut state),
            Some(0)
        );
    }

    #[test]
    fn test_command_builtin_not_found() {
        let mut state = ShellState::new();
        assert_eq!(
            handle_builtin(&["command".into(), "__jesh_noop_xyz__".into()], &mut state),
            Some(127)
        );
    }
}
