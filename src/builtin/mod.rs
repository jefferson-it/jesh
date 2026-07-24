use crate::shell::ShellState;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

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
bjesh — shell interativo

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
                println!("bjesh");
            }
            Some(0)
        }
        "jesh-info" => {
            let exe_str = env::current_exe()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| "bjesh".to_string());
            println!("jesh v{} ({})", env!("CARGO_PKG_VERSION"), exe_str);
            Some(0)
        }
        "jesh-version" => {
            let exe_str = env::current_exe()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| "bjesh".to_string());
            println!("jesh v{} ({})", env!("CARGO_PKG_VERSION"), exe_str);
            Some(0)
        }
        "version" => {
            let exe_str = env::current_exe()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| "bjesh".to_string());
            println!("bjesh {} ({})", env!("CARGO_PKG_VERSION"), exe_str);
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
                    _ => {
                        eprintln!("Subcomando desconhecido. Subcomandos válidos: pin, unpin, clear");
                        Some(1)
                    }
                }
            } else {
                state.history_mgr.print_history();
                Some(0)
            }
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
        _ => None,
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
}
