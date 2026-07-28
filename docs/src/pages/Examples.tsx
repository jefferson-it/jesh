import { DocPage } from '../components/DocPage'

export default function Examples() {
  return (
    <DocPage title="Examples">
      <p>
        This page collects practical examples of jesh configuration, custom completions,
        tool integrations, and shell scripting patterns. Use these as starting points
        for your own <code>~/.jeshrc</code> and completion scripts.
      </p>

      <h2>Migrating from <code>.bashrc</code> to <code>.jeshrc</code></h2>
      <p>
        Most Bash configuration transfers directly to jesh. Here is a side-by-side
        comparison of common patterns:
      </p>
      <table>
        <thead><tr><th>Bash (<code>.bashrc</code>)</th><th>jesh (<code>.jeshrc</code>)</th></tr></thead>
        <tbody>
          <tr>
            <td><pre><code>{`export EDITOR=vim
export PATH="$HOME/bin:$PATH"`}</code></pre></td>
            <td><pre><code>{`export EDITOR=vim
export PATH="$HOME/bin:$PATH"`}</code></pre></td>
          </tr>
          <tr>
            <td><pre><code>{`alias ll='ls -alF'
alias gs='git status'`}</code></pre></td>
            <td><pre><code>{`alias ll='ls -alF'
alias gs='git status'`}</code></pre></td>
          </tr>
          <tr>
            <td><pre><code>{`PS1='\\u@\\h:\\w\\$ '`}</code></pre></td>
            <td><pre><code>{`PROMPT='%n@%m:%~%# '`}</code></pre></td>
          </tr>
          <tr>
            <td><pre><code>{`source "$HOME/.cargo/env"`}</code></pre></td>
            <td><pre><code>{`source "$HOME/.cargo/env"`}</code></pre></td>
          </tr>
          <tr>
            <td><pre><code>{`[[ -f /etc/bash_completion ]] && source /etc/bash_completion`}</code></pre></td>
            <td><pre><code>{`# jesh has built-in completions;
# Bash completions fall back automatically`}</code></pre></td>
          </tr>
        </tbody>
      </table>

      <h2>Custom Completions</h2>

      <h3>Static Word List</h3>
      <p>
        Provide a fixed set of options for a custom command. The words are presented
        in the completion menu when you press <kbd>Tab</kbd>:
      </p>
      <pre><code>{`complete -W "start stop restart status reload logs" myservice`}</code></pre>
      <p>
        After adding this to <code>~/.jeshrc</code>, typing <code>myservice </code> and
        pressing <kbd>Tab</kbd> shows the available subcommands. The first word list
        applies to the first argument. You can add multiple <code>complete</code> rules
        for different argument positions.
      </p>

      <h3>Dynamic Completion with a Function</h3>
      <p>
        For commands where the available options depend on runtime state, use a completer
        function. The function writes candidates to the <code>COMPREPLY</code> array:
      </p>
      <pre><code>{`_myapp_complete() {
  local word="\${COMP_WORDS[COMP_CWORD]}"
  local prev="\${COMP_WORDS[COMP_CWORD-1]}"
  case "$prev" in
    --project)
      COMPREPLY=($(compgen -W "$(myapp list-projects)" "$word"))
      ;;
    --env)
      COMPREPLY=($(compgen -W "production staging development" "$word"))
      ;;
    *)
      COMPREPLY=($(compgen -W "--project --env --verbose --help start stop" "$word"))
      ;;
  esac
}
complete -F _myapp_complete myapp`}</code></pre>
      <p>
        The <code>compgen</code> builtin filters the word list against the current word,
        returning only matching candidates. The function is re-evaluated every time
        completion is triggered, so the results are always current.
      </p>

      <h2>Tool Integrations</h2>

      <h3>NVM (Node Version Manager)</h3>
      <p>
        NVM works with jesh through the Bash fallback mechanism. Add this to
        <code>~/.jeshrc</code>:
      </p>
      <pre><code>{`export NVM_DIR="$HOME/.nvm"
source "$NVM_DIR/nvm.sh"`}</code></pre>
      <p>
        When jesh encounters Bash-specific constructs inside <code>nvm.sh</code>, it
        transparently delegates execution to <code>bash -ic</code> and imports the
        resulting environment. NVM commands like <code>nvm use</code>, <code>nvm install</code>,
        and <code>nvm list</code> work as expected.
      </p>

      <h3>Rust Toolchain</h3>
      <p>
        The Rust toolchain provides completions for <code>cargo</code>, <code>rustup</code>,
        and <code>rustc</code>. Source the completions directly:
      </p>
      <pre><code>{`source "$HOME/.cargo/env"
# If rustup completions are installed:
source "$(rustup completions jesh)"`}</code></pre>
      <p>
        jesh's built-in completion engine also infers completions from CLI frameworks
        like Clap and StructOpt when the binary exposes completion hints. Cargo
        subcommands from installed crates (e.g. <code>cargo add</code>, <code>cargo watch</code>)
        are completed automatically.
      </p>

      <h3>Python Virtual Environments</h3>
      <p>
        Activate Python virtual environments in jesh the same way as in Bash. jesh
        handles the <code>deactivate</code> function and prompt changes correctly:
      </p>
      <pre><code>{`$ source .venv/bin/activate
(.venv) $ python -m flask run`}</code></pre>
      <p>
        jesh integrates with <code>virtualenv</code>, <code>poetry</code>, and
        <code>pipenv</code>. The activated environment name is automatically shown in
        the prompt via the <code>$JSH_VENV</code> variable.
      </p>

      <h3>Deno Completions</h3>
      <p>
        Deno supports generating shell completions. Generate and source them in your
        <code>~/.jeshrc</code>:
      </p>
      <pre><code>{`deno completions jesh &gt; ~/.local/jesh/completions/deno.sh
source ~/.local/jesh/completions/deno.sh`}</code></pre>
      <p>
        Alternatively, you can source the Bash completions and let the fallback handle
        them. The Deno completions include subcommands, permissions flags (<code>--allow-read</code>,
        <code>--allow-net</code>, <code>--allow-env</code>), and file paths for script
        arguments.
      </p>

      <h2>Scripting Examples</h2>
      <p>
        jesh supports the same scripting constructs as Bash. Here are common patterns:
      </p>

      <h3>Loops</h3>
      <pre><code>{`for file in *.rs; do
  echo "Compiling $file..."
  rustc "$file"
done`}</code></pre>

      <h3>Conditionals</h3>
      <pre><code>{`if [[ -f "config.toml" ]] &amp;&amp; [[ -r "config.toml" ]]; then
  echo "Config file exists and is readable"
elif [[ -f "config.json" ]]; then
  echo "Using JSON config instead"
else
  echo "No configuration found"
  exit 1
fi`}</code></pre>

      <h3>Pipes and Functions</h3>
      <pre><code>{`function count_lines() {
  local pattern="$1"
  grep -r "$pattern" src/ | wc -l
}

# Use the function in a pipeline
count_lines "fn main" | xargs echo "Occurrences:"`}</code></pre>

      <h3>Argument Parsing with <code>getopts</code></h3>
      <pre><code>{`while getopts "o:v" opt; do
  case "$opt" in
    o) output="$OPTARG" ;;
    v) verbose=true ;;
    ?) echo "Usage: $0 [-o output] [-v] file" &gt;&amp;2; exit 1 ;;
  esac
done
shift $((OPTIND - 1))`}</code></pre>
    </DocPage>
  )
}
