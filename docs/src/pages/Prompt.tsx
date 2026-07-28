import { DocPage } from '../components/DocPage'

export default function Prompt() {
  return (
    <DocPage title="Prompt">
      <p>
        jesh offers extensive prompt customisation through variables similar to
        <code>PS1</code> in Bash. You can define the primary prompt, a right-aligned
        prompt, and a transient prompt that simplifies previous command lines. The
        prompt can display exit status, user, host, working directory, git branch,
        time, and even an OS logo.
      </p>

      <h2>Prompt Variables</h2>
      <p>
        Set the following variables in <code>~/.jeshrc</code> to control prompt
        appearance. Escape sequences and special placeholders are expanded at each
        prompt.
      </p>
      <table>
        <thead><tr><th>Variable</th><th>Description</th><th>Example Value</th></tr></thead>
        <tbody>
          <tr><td><code>PROMPT</code> or <code>PS1</code></td><td>Primary prompt string.</td><td><code>'%n@%m %~ %# '</code></td></tr>
          <tr><td><code>RPROMPT</code></td><td>Right-aligned prompt, fixed to the right edge.</td><td><code>{'%? %B%F{red}%*'}</code></td></tr>
          <tr><td><code>PROMPT2</code></td><td>Secondary prompt for multi-line input.</td><td><code>'&gt; '</code></td></tr>
          <tr><td><code>SPROMPT</code></td><td>Correction prompt for spell checking.</td><td><code>'zsh: correct %R to %r? '</code></td></tr>
        </tbody>
      </table>

      <h3>Escape Sequences</h3>
      <p>
        The following escape sequences are recognised in prompt variables:
      </p>
      <table>
        <thead><tr><th>Sequence</th><th>Expands To</th></tr></thead>
        <tbody>
          <tr><td><code>%n</code></td><td>Current username.</td></tr>
          <tr><td><code>%m</code></td><td>Hostname up to the first dot.</td></tr>
          <tr><td><code>%M</code></td><td>Fully qualified hostname.</td></tr>
          <tr><td><code>%~</code></td><td>Current directory with <code>~</code> abbreviation.</td></tr>
          <tr><td><code>%/</code> or <code>%d</code></td><td>Full current directory path.</td></tr>
          <tr><td><code>%?</code></td><td>Exit status of the last command.</td></tr>
          <tr><td><code>%#</code></td><td><code>#</code> for root, <code>%</code> for normal users.</td></tr>
          <tr><td><code>%*</code></td><td>Current time in 24-hour format (HH:MM:SS).</td></tr>
          <tr><td><code>%T</code></td><td>Current time in 24-hour format (HH:MM).</td></tr>
          <tr><td><code>%t</code> or <code>%@</code></td><td>Current time in 12-hour format (HH:MM AM/PM).</td></tr>
          <tr><td><code>%D</code></td><td>Current date in <code>yy-mm-dd</code> format.</td></tr>
          <tr><td><code>{'%F{color}'}</code></td><td>Set foreground color (named or hex).</td></tr>
          <tr><td><code>{'%K{color}'}</code></td><td>Set background color (named or hex).</td></tr>
          <tr><td><code>%B</code></td><td>Bold text.</td></tr>
          <tr><td><code>%b</code></td><td>Disable bold text.</td></tr>
          <tr><td><code>%E</code></td><td>Clear to end of line.</td></tr>
          <tr><td><code>{'%{...%}'}</code></td><td>Literal escape sequence (not counted for prompt width).</td></tr>
        </tbody>
      </table>

      <h3>Info Placeholders</h3>
      <p>
        Beyond standard escape sequences, jesh provides expanded placeholders for
        dynamic information:
      </p>
      <ul>
        <li><code>$JSH_GIT_BRANCH</code> — Current git branch name, or empty if not in a git repo.</li>
        <li><code>$JSH_GIT_DIRTY</code> — <code>*</code> if the working tree has uncommitted changes.</li>
        <li><code>$JSH_OS_LOGO</code> — A small ASCII or Nerd Font icon representing the OS (e.g. <code></code> for Linux, <code></code> for macOS).</li>
        <li><code>$JSH_CONTAINER</code> — Container or VM indicator when running inside a container.</li>
        <li><code>$JSH_SSH</code> — <code>SSH</code> indicator when connected over SSH.</li>
      </ul>

      <h2>RPROMPT — Right-Aligned Prompt</h2>
      <p>
        The <code>RPROMPT</code> variable defines a prompt that is right-aligned on the
        terminal line. This is useful for less critical information that you want visible
        but not in the way. Typical elements placed in RPROMPT include:
      </p>
      <ul>
        <li>Exit code of the last command (shown only when non-zero).</li>
        <li>SSH connection indicator.</li>
        <li>Current git branch and dirty status.</li>
        <li>Elapsed time of the last command.</li>
      </ul>
      <pre><code>{`RPROMPT='%(?..%F{red}%?%f )$JSH_GIT_BRANCH'`}</code></pre>
      <p>
        The <code>%(condition.true-text.false-text)</code> syntax provides conditional
        display. In the example above, the exit code is only shown when it is non-zero.
      </p>

      <h2>Transient Prompt</h2>
      <p>
        Set <code>JSH_TRANSIENT_PROMPT=true</code> to enable the transient prompt
        feature. When enabled, after a command executes, the prompt on that line is
        replaced with a simplified version. This keeps the terminal scrollback clean
        and readable.
      </p>
      <p>
        The default transient prompt is <code>'$ '</code>, but you can customise it by
        defining a <code>TRANSIENT_PROMPT</code> variable:
      </p>
      <pre><code>{`JSH_TRANSIENT_PROMPT=true
TRANSIENT_PROMPT='%# '`}</code></pre>
      <p>
        The right prompt (<code>RPROMPT</code>) is also hidden on completed lines when
        transient mode is active.
      </p>

      <h2>Async Git Branch Rendering</h2>
      <p>
        Git operations like status checks can be slow in large repositories. jesh
        renders the git branch and status information asynchronously so that the prompt
        appears immediately after a command finishes. A placeholder (by default
        <code>...</code>) is shown while the git data is being fetched in a background
        thread. Once the data is ready, the prompt line is updated in place without
        delaying keyboard input.
      </p>
      <p>
        This means you never experience lag when typing commands after <code>git checkout</code>,
        <code>git merge</code>, or other repository operations. The feature is enabled
        by default and can be disabled by setting <code>JSH_ASYNC_GIT=false</code>.
      </p>

      <h2>Theme System</h2>
      <p>
        Set the <code>THEME</code> variable to load a theme from
        <code>~/.local/jesh/themes/&lt;name&gt;.sh</code>:
      </p>
      <pre><code>{`THEME="jesh-dracula"`}</code></pre>
      <p>Built-in themes include:</p>
      <ul>
        <li><code>jesh-default</code> — Simple dark theme with green accents.</li>
        <li><code>jesh-dark</code> — Dark background with cyan and yellow highlights.</li>
        <li><code>jesh-dracula</code> — Dracula colour palette with pink and purple accents.</li>
        <li><code>jesh-light</code> — Light background for bright terminals.</li>
        <li><code>jesh-nord</code> — Arctic bluish palette inspired by Nord.</li>
        <li><code>jesh-solarized</code> — Solarized dark palette.</li>
      </ul>
      <p>
        Theme scripts set <code>JSH_THEME_*</code> variables to define colour values:
      </p>
      <pre><code>{`JSH_THEME_PRIMARY="#bd93f9"
JSH_THEME_SECONDARY="#6272a4"
JSH_THEME_ACCENT="#50fa7b"
JSH_THEME_ERROR="#ff5555"
JSH_THEME_PROMPT="#f8f8f2"`}</code></pre>
      <p>
        Themes can also emit OSC escape sequences to change the terminal background
        colour, set the cursor style, or configure terminal title. This is done by
        printing the appropriate escape sequences from the theme script:
      </p>
      <pre><code>{`# Set terminal background via OSC 11
printf '\e]11;#282a36\e\\'
# Set cursor colour via OSC 12
printf '\e]12;#bd93f9\e\\'
# Set cursor shape to beam via OSC 50
printf '\e[2 q'`}</code></pre>

      <h2>Nerd Font Icons</h2>
      <p>
        If you have a Nerd Font installed in your terminal, jesh can use Nerd Font
        icons in the prompt. Install a Nerd Font (e.g. FiraCode Nerd Font, JetBrains
        Mono Nerd Font) and set it as your terminal font. Then include icon glyphs
        directly in your prompt string:
      </p>
      <pre><code>{`PROMPT='  %n@%m  %~ $JSH_GIT_BRANCH  %# '`}</code></pre>
      <p>
        The <code>$JSH_OS_LOGO</code> variable returns a Nerd Font icon when a Nerd
        Font is detected automatically.
      </p>
    </DocPage>
  )
}
