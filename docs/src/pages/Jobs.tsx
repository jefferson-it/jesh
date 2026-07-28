import { DocPage } from '../components/DocPage'

export default function Jobs() {
  return (
    <DocPage title="Jobs">
      <p>
        jesh provides full job control similar to Bash and Zsh. You can run commands
        in the background, suspend running processes, bring jobs to the foreground,
        and detach jobs from the shell entirely. The shell tracks each child process
        in a job table and provides builtin commands for managing them.
      </p>

      <h2>Background Jobs with <code>&amp;</code></h2>
      <p>
        Append an ampersand (<code>&amp;</code>) to a command to run it in the
        background. The shell immediately returns control to you and displays the job
        number and process ID:
      </p>
      <pre><code>$ sleep 100 &amp;
[1] 12345
$</code></pre>
      <p>
        Background jobs continue running even while you type and execute other
        commands. Their output is printed to the terminal interleaved with your work
        unless you redirect it.
      </p>

      <h2>Managing Jobs</h2>
      <p>Job control builtins use the <code>%n</code> syntax to reference a specific job by its job number.</p>

      <h3><code>jobs</code> — List Jobs</h3>
      <p>
        The <code>jobs</code> builtin lists all active jobs with their job number,
        status, and command line:
      </p>
      <pre><code>$ jobs
[1]  + running    sleep 100
[2]  - suspended  vim
[3]    running    tail -f /var/log/syslog</code></pre>
      <p>
        The <code>+</code> indicates the current (default) job, and <code>-</code>
        indicates the previous job. Options:
      </p>
      <ul>
        <li><code>-l</code> — Show process IDs.</li>
        <li><code>-p</code> — Show only process IDs.</li>
        <li><code>-r</code> — Show only running jobs.</li>
        <li><code>-s</code> — Show only suspended (stopped) jobs.</li>
      </ul>

      <h3><code>fg</code> — Foreground</h3>
      <p>
        Bring a job to the foreground with <code>fg %n</code>. Without a job
        specifier, the current job (<code>%+</code>) is used:
      </p>
      <pre><code>$ fg %1
sleep 100</code></pre>
      <p>
        The job resumes execution and the shell waits for it to complete or be
        suspended again. You can also abbreviate job references as <code>%1</code>
        directly.
      </p>

      <h3><code>bg</code> — Background</h3>
      <p>
        Resume a stopped (suspended) job in the background with <code>bg %n</code>:
      </p>
      <pre><code>$ bg %2
[2]  + continued  vim</code></pre>
      <p>
        The job continues running in the background. This is equivalent to sending
        <code>SIGCONT</code> to the process group.
      </p>

      <h3><code>disown</code> — Detach</h3>
      <p>
        Remove a job from the shell's job table with <code>disown %n</code>. The
        process continues running but the shell no longer tracks it. This means the
        process will not receive <code>SIGHUP</code> when the shell exits:
      </p>
      <pre><code>$ sleep 1000 &amp;
[1] 12345
$ disown %1
$ exit</code></pre>
      <p>
        The <code>disown</code> command is useful for launching long-running processes
        that should outlive the shell session. Options include <code>-h</code> (mark
        job so it does not receive SIGHUP, but keep it in the table) and
        <code>-a</code> (act on all jobs).
      </p>

      <h2>Keyboard Shortcuts</h2>
      <table>
        <thead><tr><th>Shortcut</th><th>Action</th><th>Signal</th></tr></thead>
        <tbody>
          <tr><td><kbd>Ctrl+Z</kbd></td><td>Suspend the foreground process.</td><td><code>SIGTSTP</code></td></tr>
          <tr><td><kbd>Ctrl+C</kbd></td><td>Interrupt the foreground process.</td><td><code>SIGINT</code></td></tr>
          <tr><td><kbd>Ctrl+D</kbd></td><td>Send EOF; exit the shell if line is empty.</td><td>—</td></tr>
        </tbody>
      </table>

      <h3><kbd>Ctrl+Z</kbd> — Suspend</h3>
      <p>
        Press <kbd>Ctrl+Z</kbd> to suspend the foreground process and return to the
        shell. The process receives <code>SIGTSTP</code> and is stopped. You can then
        use <code>bg</code> to resume it in the background or <code>fg</code> to bring
        it back to the foreground.
      </p>

      <h3><kbd>Ctrl+C</kbd> — Interrupt</h3>
      <p>
        Press <kbd>Ctrl+C</kbd> to send <code>SIGINT</code> to the foreground process
        group. The shell itself is protected from <code>SIGINT</code> by its process
        group isolation so that accidental <kbd>Ctrl+C</kbd> does not terminate the
        shell.
      </p>

      <h3><kbd>Ctrl+D</kbd> — EOF</h3>
      <p>
        Press <kbd>Ctrl+D</kbd> on an empty line to exit the shell. If pressed in the
        middle of typing, it sends EOF to the current read operation. This is the same
        behaviour as in Bash and other POSIX shells.
      </p>

      <h2>Process Group Isolation</h2>
      <p>
        jesh places each command pipeline into its own process group (PGID). This
        provides critical isolation between the shell and its child processes. When you
        press <kbd>Ctrl+C</kbd>, the <code>SIGINT</code> signal is delivered only to
        the foreground process group, not to the shell itself. This prevents accidental
        termination of the shell and ensures reliable job control.
      </p>
      <p>
        The shell also restores terminal control to the foreground process group when
        a job is brought to the foreground, and reclaims it when the job is suspended
        or completed.
      </p>

      <h2>Async Job Notification</h2>
      <p>
        When a background job finishes, jesh notifies you asynchronously. A message is
        printed to the terminal immediately before the next prompt, without interrupting
        your typing:
      </p>
      <pre><code>$ sleep 5 &amp;
[1] 12345
$ echo hello
[1]  + done       sleep 5
hello</code></pre>
      <p>
        Notifications are triggered by <code>SIGCHLD</code> and are processed
        incrementally. The notification includes the job number, status (done, exited,
        signalled, stopped), and the command line of the completed job.
      </p>
    </DocPage>
  )
}
