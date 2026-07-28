use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
#[cfg(unix)]
use std::os::unix::io::{FromRawFd, OwnedFd};
#[cfg(unix)]
use std::os::unix::net::UnixStream;
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};

use crate::parser::lexer::RedirectTarget;
use crate::parser::{ExpandedCommand, ExpandedPipeline};
use crate::utils::expand_target;

/// Opens (creating if needed) the file used by an output redirection (`>`, `>>`).
fn open_output_file(path: &str, append: bool) -> File {
    let path = expand_target(path);
    let mut opts = OpenOptions::new();
    opts.write(true).create(true);
    if append {
        opts.append(true);
    } else {
        opts.truncate(true);
    }

    match opts.open(&path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("jesh: {}: {}", path, e);
            File::open("/dev/null").unwrap_or_else(|_| {
                eprintln!("jesh: erro crítico: não foi possível abrir /dev/null");
                std::process::exit(1);
            })
        }
    }
}

/// Opens the file used by an input redirection (`<`).
fn open_input_file(path: &str) -> File {
    let path = expand_target(path);
    match OpenOptions::new().read(true).open(&path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("jesh: {}: {}", path, e);
            File::open("/dev/null").unwrap_or_else(|_| {
                eprintln!("jesh: erro crítico: não foi possível abrir /dev/null");
                std::process::exit(1);
            })
        }
    }
}

/// Duplicates an existing file descriptor (used by `2>&1`, `0<&3`, ...).
/// Uses `libc::dup()` to create a real copy of the fd.
#[cfg(unix)]
fn dup_fd(fd: i32, _writable: bool) -> Stdio {
    let new_fd = unsafe { libc::dup(fd) };
    if new_fd < 0 {
        return Stdio::inherit();
    }
    unsafe {
        Stdio::from(OwnedFd::from_raw_fd(new_fd))
    }
}

#[cfg(windows)]
fn dup_fd(_fd: i32, _writable: bool) -> Stdio {
    Stdio::inherit()
}

/// Builds a `Stdio` that feeds `content` to a child's stdin via a pipe.
#[cfg(unix)]
fn string_to_stdio(content: &str) -> Option<Stdio> {
    let (read, mut write) = UnixStream::pair().ok()?;
    let _ = write.write_all(content.as_bytes());
    drop(write);
    Some(Stdio::from(OwnedFd::from(read)))
}

#[cfg(windows)]
fn string_to_stdio(_content: &str) -> Option<Stdio> {
    None
}

/// Builds the stdin/stdout/stderr `Stdio` for one command in a pipeline.
/// `capture_stdout` routes this command's stdout into a captured pipe
/// (`Command::stdout(Stdio::piped())`) instead of inheriting it, used for
/// the last stage of `$(...)` command substitution.
fn spawn_one(
    cmd: &ExpandedCommand,
    piped: bool,
    next_stdin: &mut Option<Stdio>,
    capture_stdout: bool,
) -> Command {
    let mut process = Command::new(&cmd.program);
    process.args(&cmd.args);
    for (k, v) in &cmd.env_vars {
        process.env(k, v);
    }

    let mut stdin_r = None;
    let mut stdout_r = None;
    let mut stderr_r = None;
    for r in &cmd.redirects {
        match r.fd {
            0 => stdin_r = Some(r),
            1 => stdout_r = Some(r),
            2 => stderr_r = Some(r),
            _ => {}
        }
    }

    // ---- stdin ----
    let stdin = if let Some(r) = stdin_r {
        match &r.target {
            RedirectTarget::File(p) => Stdio::from(open_input_file(p)),
            RedirectTarget::Fd(fd) => dup_fd(*fd, false),
            RedirectTarget::HereString(s) => string_to_stdio(&format!("{}\n", s)).unwrap_or_else(Stdio::inherit),
            RedirectTarget::Heredoc(..) => match &cmd.heredoc {
                Some(body) => string_to_stdio(body).unwrap_or_else(Stdio::inherit),
                None => Stdio::inherit(),
            },
            RedirectTarget::ProcessSubst(cmd, is_input) => {
                use std::process::{Command, Stdio};
                if *is_input {
                    {
                        let mut __cmd = Command::new("sh");
                        __cmd.arg("-c").arg(cmd).stdout(Stdio::piped()).stderr(Stdio::null());
                        if let Ok(mut c) = __cmd.spawn() {
                            match c.stdout { Some(o) => Stdio::from(o), None => Stdio::inherit() }
                        } else {
                            Stdio::inherit()
                        }
                    }
                } else {
                    Stdio::inherit()
                }
            },
            RedirectTarget::Close(fd) => {
                if *fd == 0 {
                    // Closing stdin = pipe from /dev/null
                    Stdio::null()
                } else {
                    Stdio::inherit()
                }
            }
            RedirectTarget::Dynamic(_) | RedirectTarget::LazyWord(_) => {
                // Dynamic FD allocation: not applicable to stdin.
                // LazyWord should have been resolved during expand_pipeline.
                Stdio::inherit()
            }
        }
    } else if let Some(s) = next_stdin.take() {
        s
    } else {
        Stdio::inherit()
    };
    process.stdin(stdin);

    // ---- stdout ----
    let (stdout, pipe_write) = if let Some(r) = stdout_r {
        match &r.target {
            RedirectTarget::File(p) => (Stdio::from(open_output_file(p, r.append)), None),
            RedirectTarget::Fd(fd) => (dup_fd(*fd, true), None),
            RedirectTarget::ProcessSubst(cmd, is_input) => {
                use std::process::{Command, Stdio};
                let stdio = if !*is_input {
                    let mut __cmd = Command::new("sh");
                    __cmd.arg("-c").arg(cmd).stdin(Stdio::piped()).stderr(Stdio::null());
                    if let Ok(mut c) = __cmd.spawn() {
                        match c.stdin { Some(o) => Stdio::from(o), None => Stdio::inherit() }
                    } else {
                        Stdio::inherit()
                    }
                } else {
                    Stdio::inherit()
                };
                (stdio, None)
            }
            RedirectTarget::Close(fd) => {
                if *fd == 1 || *fd == 0 {
                    // Closing stdout = route to /dev/null
                    (Stdio::null(), None)
                } else {
                    (Stdio::inherit(), None)
                }
            }
            RedirectTarget::Dynamic(_) | RedirectTarget::LazyWord(_) => {
                // Should have been resolved during expand_pipeline.
                (Stdio::inherit(), None)
            }
            _ => (Stdio::inherit(), None),
        }
    } else if piped {
        #[cfg(unix)]
        {
            if let Ok((read_end, write_end)) = UnixStream::pair() {
                match write_end.try_clone() {
                    Ok(write_clone) => {
                        *next_stdin = Some(Stdio::from(OwnedFd::from(read_end)));
                        (
                            Stdio::from(OwnedFd::from(write_clone)),
                            Some(write_end),
                        )
                    }
                    Err(e) => {
                        eprintln!("jesh: erro ao criar pipe: {}", e);
                        (Stdio::inherit(), None)
                    }
                }
            } else {
                (Stdio::inherit(), None)
            }
        }
        #[cfg(windows)]
        {
            (Stdio::inherit(), None)
        }
    } else if capture_stdout {
        (Stdio::piped(), None)
    } else {
        (Stdio::inherit(), None)
    };
    process.stdout(stdout);

    // ---- stderr ----
    let stderr = if let Some(r) = stderr_r {
        match &r.target {
            RedirectTarget::File(p) => Stdio::from(open_output_file(p, r.append)),
            RedirectTarget::Fd(target_fd) => {
                if *target_fd == 1 && pipe_write.is_some() {
                    // `2>&1` inside a pipeline: join the stdout pipe.
                    pipe_write.as_ref()
                        .and_then(|pw| pw.try_clone().ok())
                        .map(|cloned| Stdio::from(OwnedFd::from(cloned)))
                        .unwrap_or_else(|| dup_fd(*target_fd, true))
                } else {
                    dup_fd(*target_fd, true)
                }
            }
            RedirectTarget::ProcessSubst(cmd, is_input) => {
                use std::process::{Command, Stdio};
                let stdio = if !*is_input {
                    let mut __cmd = Command::new("sh");
                    __cmd.arg("-c").arg(cmd).stdin(Stdio::piped()).stderr(Stdio::null());
                    if let Ok(mut c) = __cmd.spawn() {
                        match c.stdin { Some(o) => Stdio::from(o), None => Stdio::inherit() }
                    } else {
                        Stdio::inherit()
                    }
                } else {
                    Stdio::inherit()
                };
                stdio
            }
            RedirectTarget::Close(fd) => {
                if *fd == 2 || *fd == 0 {
                    // Closing stderr = route to /dev/null
                    Stdio::null()
                } else {
                    Stdio::inherit()
                }
            }
            _ => Stdio::inherit(),
        }
    } else {
        Stdio::inherit()
    };
    process.stderr(stderr);

    process
}

pub fn execute_with(pipe: ExpandedPipeline, state: &crate::shell::ShellState) -> (i32, Vec<i32>) {
    let quiet = state.quiet_errors;
    let n = pipe.commands.len();
    if n == 0 {
        return (0, Vec::new());
    }

    #[cfg(unix)]
    let is_tty = state.is_interactive && unsafe { libc::isatty(libc::STDIN_FILENO) != 0 };
    #[cfg(windows)]
    let is_tty = false;
    #[cfg(unix)]
    let mut old_sigint: usize = 0;
    #[cfg(unix)]
    if is_tty {
        unsafe {
            old_sigint = libc::signal(libc::SIGINT, libc::SIG_IGN);
            libc::signal(libc::SIGTTOU, libc::SIG_IGN);
            libc::signal(libc::SIGTTIN, libc::SIG_IGN);
        }
        crate::utils::save_shell_termios();
    }

    let mut children = Vec::new();
    let mut next_stdin: Option<Stdio> = None;
    let mut pgid = 0;

    for i in 0..n {
        let cmd = &pipe.commands[i];
        let piped = i < n - 1;
        let mut process = spawn_one(cmd, piped, &mut next_stdin, false);

        #[cfg(unix)]
        unsafe {
            let is_first = i == 0;
            let first_pgid = pgid;
            process.pre_exec(move || {
                let pid = libc::getpid();
                let pgrp = if is_first { pid } else { first_pgid };
                let _ = libc::setpgid(0, pgrp);
                if is_tty {
                    let _ = libc::tcsetpgrp(libc::STDIN_FILENO, pgrp);
                }
                libc::signal(libc::SIGTTOU, libc::SIG_DFL);
                libc::signal(libc::SIGTTIN, libc::SIG_DFL);
                libc::signal(libc::SIGTSTP, libc::SIG_DFL);
                libc::signal(libc::SIGQUIT, libc::SIG_DFL);
                libc::signal(libc::SIGINT, libc::SIG_DFL);
                Ok(())
            });
        }

        match process.spawn() {
            Ok(child) => {
                #[cfg(unix)]
                let child_id = child.id() as libc::pid_t;
                if i == 0 {
                    #[cfg(unix)]
                    { pgid = child_id; }
                    #[cfg(windows)]
                    { pgid = child.id(); }
                }
                #[cfg(unix)]
                unsafe {
                    let target = if i == 0 { child_id } else { pgid };
                    let _ = libc::setpgid(child_id, target);
                    if is_tty && i == 0 {
                        let _ = libc::tcsetpgrp(libc::STDIN_FILENO, target);
                    }
                }
                children.push(child);
            }
            Err(e) => {
                if !quiet {
                    eprintln!("jesh: {}: {}", cmd.program, e);
                    if e.kind() == std::io::ErrorKind::NotFound {
                        if let Some(suggestion) = crate::utils::suggest_command(&cmd.program, state) {
                            eprintln!("Você quis dizer '{}'?", suggestion);
                        }
                    }
                }
                for mut child in children {
                    let _ = child.kill();
                    let _ = child.wait();
                }
                #[cfg(unix)]
                if is_tty {
                    unsafe {
                        let shell_pgid = libc::getpgrp();
                        libc::tcsetpgrp(libc::STDIN_FILENO, shell_pgid);
                        libc::signal(libc::SIGINT, old_sigint);
                        libc::signal(libc::SIGTTOU, libc::SIG_IGN);
                        libc::signal(libc::SIGTTIN, libc::SIG_IGN);
                    }
                    crate::utils::restore_shell_termios();
                    crate::utils::reset_terminal_and_flush_stdin();
                }
                return (127, vec![127]);
            }
        }
    }

    #[cfg(unix)]
    if is_tty && pgid != 0 {
        unsafe {
            libc::tcsetpgrp(libc::STDIN_FILENO, pgid);
        }
    }

    let mut last_status = 0;
    let mut statuses = Vec::new();
    for mut child in children {
        match child.wait() {
            Ok(status) => {
                last_status = crate::utils::exit_code_from_status(status);
                statuses.push(last_status);
            }
            Err(_) => last_status = 1,
        }
    }

    #[cfg(unix)]
    if is_tty {
        unsafe {
            let shell_pgid = libc::getpgrp();
            libc::tcsetpgrp(libc::STDIN_FILENO, shell_pgid);
            libc::signal(libc::SIGINT, old_sigint);
            libc::signal(libc::SIGTTOU, libc::SIG_IGN);
            libc::signal(libc::SIGTTIN, libc::SIG_IGN);
        }
        crate::utils::restore_shell_termios();
        crate::utils::reset_terminal_and_flush_stdin();
    }

    (last_status, statuses)
}

/// Spawns a pipeline in the background without waiting for it (`cmd &`).
/// Returns the PID of the last stage, if it started successfully. Not full
/// job control (no `jobs`/`fg`/`bg`) — just fire-and-forget, like a
/// disowned background job.
pub fn spawn_detached(pipe: ExpandedPipeline) -> Option<u32> {
    let n = pipe.commands.len();
    if n == 0 {
        return None;
    }
    let mut children: Vec<std::process::Child> = Vec::new();
    let mut next_stdin: Option<Stdio> = None;
    let mut last_pid = None;

    for i in 0..n {
        let cmd = &pipe.commands[i];
        let piped = i < n - 1;
        let mut process = spawn_one(cmd, piped, &mut next_stdin, false);
        if next_stdin.is_none() && i == 0 {
            process.stdin(Stdio::null());
        }
        match process.spawn() {
            Ok(child) => {
                let pid = child.id();
                children.push(child);
                last_pid = Some(pid);
            }
            Err(e) => {
                eprintln!("jesh: {}: {}", cmd.program, e);
                for mut child in children {
                    let _ = child.kill();
                    let _ = child.wait();
                }
                return None;
            }
        }
    }

    last_pid
}

/// Like `execute`, but captures the final command's stdout and returns it
/// instead of printing it — used for `$(...)` command substitution.
pub fn execute_capture(pipe: ExpandedPipeline) -> Vec<u8> {
    let n = pipe.commands.len();
    if n == 0 {
        return Vec::new();
    }
    let mut children = Vec::new();
    let mut next_stdin: Option<Stdio> = None;

    for i in 0..n {
        let cmd = &pipe.commands[i];
        let piped = i < n - 1;
        let is_last = i == n - 1;
        let mut process = spawn_one(cmd, piped, &mut next_stdin, is_last);

        match process.spawn() {
            Ok(child) => children.push(child),
            Err(e) => {
                eprintln!("jesh: {}: {}", cmd.program, e);
                return Vec::new();
            }
        }
    }

    let mut last_child = children.pop();
    let mut output = Vec::new();
    if let Some(child) = last_child.as_mut() {
        if let Some(mut stdout) = child.stdout.take() {
            use std::io::Read;
            let _ = stdout.read_to_end(&mut output);
        }
    }

    for mut child in children {
        let _ = child.wait();
    }
    if let Some(mut child) = last_child {
        let _ = child.wait();
    }

    output
}
