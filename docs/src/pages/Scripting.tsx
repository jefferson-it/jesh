import { DocPage } from '../components/DocPage'

export default function Scripting() {
  return (
    <DocPage title="Scripting">
      <p>
        jesh aims for broad POSIX/Bash compatibility, making it suitable for interactive
        use and shell scripting. This page covers the scripting features available in
        jesh.
      </p>

      <h2>Variables</h2>
      <p>
        Variables are assigned with <code>name=value</code> (no spaces around
        <code>=</code>). By default variables are strings:
      </p>
      <pre><code>{`name="world"
echo "Hello, $name"`}</code></pre>

      <h3>Local Variables</h3>
      <p>
        Inside a function, use <code>local</code> to restrict a variable's scope:
      </p>
      <pre><code>{`myfunc() {
  local x=42
  echo "$x"
}`}</code></pre>

      <h3>Environment Variables</h3>
      <p>
        Use <code>export</code> to pass a variable to child processes:
      </p>
      <pre><code>{`export PATH="$HOME/bin:$PATH"`}</code></pre>

      <h3>Special Variables</h3>
      <table>
        <thead><tr><th>Variable</th><th>Description</th></tr></thead>
        <tbody>
          <tr><td><code>$?</code></td><td>Exit status of the last foreground command.</td></tr>
          <tr><td><code>$$</code></td><td>PID of the current shell.</td></tr>
          <tr><td><code>$!</code></td><td>PID of the last background command.</td></tr>
          <tr><td><code>$0</code> – <code>$9</code></td><td>Positional parameters.</td></tr>
          <tr><td><code>$@</code></td><td>All positional parameters, individually quoted.</td></tr>
          <tr><td><code>$#</code></td><td>Number of positional parameters.</td></tr>
          <tr><td><code>$*</code></td><td>All positional parameters as a single string.</td></tr>
          <tr><td><code>$PWD</code></td><td>Current working directory.</td></tr>
          <tr><td><code>$OLDPWD</code></td><td>Previous working directory.</td></tr>
          <tr><td><code>$PIPESTATUS</code></td><td>Array of exit statuses from the last pipeline.</td></tr>
          <tr><td><code>$IFS</code></td><td>Internal field separator (default: space, tab, newline).</td></tr>
          <tr><td><code>$LINENO</code></td><td>Current line number in script or function.</td></tr>
          <tr><td><code>$BASH_SOURCE</code></td><td>Array of source file paths for the current call stack.</td></tr>
          <tr><td><code>$FUNCNAME</code></td><td>Array of function names in the call stack.</td></tr>
        </tbody>
      </table>

      <h2>Expansions</h2>

      <h3>Command Substitution: <code>$()</code></h3>
      <p>
        Capture the output of a command:
      </p>
      <pre><code>{`files=$(ls -la)
echo "Today is $(date)"`}</code></pre>

      <h3>Parameter Expansion: <code>${}</code></h3>
      <p>
        Brace-delimited parameter expansion supports default values, substring
        extraction, and pattern replacement:
      </p>
      <pre><code>{`echo "\${name:-default}"    # default if unset
echo "\${name:?error msg}"  # error if unset
echo "\${#name}"            # length
echo "\${name:offset:len}"  # substring
echo "\${name/old/new}"     # replace first match
echo "\${name//old/new}"    # replace all matches
echo "\${name#prefix}"      # strip shortest prefix
echo "\${name##prefix}"     # strip longest prefix
echo "\${name%suffix}"      # strip shortest suffix
echo "\${name%%suffix}"     # strip longest suffix`}</code></pre>

      <h3>Arithmetic Expansion: <code>$((...))</code></h3>
      <p>
        Integer arithmetic with C-like operators:
      </p>
      <pre><code>{`echo $((5 + 3 * 2))
echo $(( (x > y) ? x : y ))`}</code></pre>

      <h3>Brace Expansion: <code>{'{a,b,c}'}</code></h3>
      <p>
        Generate arbitrary strings:
      </p>
      <pre><code>{`echo {a,b,c}{1,2}   # a1 a2 b1 b2 c1 c2
echo {1..5}         # 1 2 3 4 5
echo {1..10..2}     # 1 3 5 7 9`}</code></pre>

      <h2>Functions</h2>
      <p>
        Define a function with the POSIX <code>name() compound</code> syntax or the
        Bash-compatible <code>function name { }</code>:
      </p>
      <pre><code>{`greet() {
  local name="$1"
  echo "Hello, $name"
}

function add {
  echo $(($1 + $2))
}`}</code></pre>
      <p>
        Arguments are accessed via <code>$1</code>, <code>$2</code>, etc.
        <code>$@</code> expands to all arguments. Use <code>local</code> for
        function-scoped variables. Functions can return an integer status with
        <code>return n</code>.
      </p>

      <h2>declare / typeset</h2>
      <p>
        The <code>declare</code> and <code>typeset</code> builtins set variable
        attributes:
      </p>
      <ul>
        <li><code>-i</code> — Integer attribute; arithmetic evaluation on assignment.</li>
        <li><code>-a</code> — Indexed array.</li>
        <li><code>-A</code> — Associative array (hash map).</li>
        <li><code>-r</code> — Read-only.</li>
        <li><code>-x</code> — Export to environment.</li>
        <li><code>-l</code> — Convert value to lowercase on assignment.</li>
        <li><code>-u</code> — Convert value to uppercase on assignment.</li>
      </ul>
      <pre><code>{`declare -i num=42
declare -a arr=(one two three)
declare -A map=([key]=value [foo]=bar)
declare -r CONST=100
declare -x MY_VAR="visible to children"`}</code></pre>

      <h2>Shell Options: <code>set</code></h2>
      <p>
        The <code>set</code> builtin controls shell execution behaviour:
      </p>
      <ul>
        <li><code>set -e</code> — Exit immediately on error (errexit).</li>
        <li><code>set -u</code> — Treat unset variables as an error (nounset).</li>
        <li><code>set -x</code> — Print commands and their arguments (xtrace).</li>
        <li><code>set -o pipefail</code> — Pipeline returns the rightmost non-zero status.</li>
        <li><code>set -o noglob</code> — Disable pathname expansion (globbing).</li>
        <li><code>set -o allexport</code> — Automatically export all variables.</li>
        <li><code>set -o notify</code> — Report job status immediately.</li>
      </ul>

      <h2>Control Flow</h2>

      <h3>Conditionals</h3>
      <pre><code>{`if [[ -f "$file" ]]; then
  echo "File exists"
elif [[ -d "$file" ]]; then
  echo "Is a directory"
else
  echo "Not found"
fi`}</code></pre>

      <h3>case</h3>
      <pre><code>{`case "$os" in
  linux) echo "Linux" ;;
  darwin|macos) echo "macOS" ;;
  *) echo "Unknown: $os" ;;
esac`}</code></pre>

      <h3>Loops</h3>
      <pre><code>{`# while
while [[ $i -lt 10 ]]; do
  echo "$i"
  ((i++))
done

# until
until ping -c1 example.com &>/dev/null; do
  sleep 1
done

# for (list)
for file in *.txt; do
  echo "Found: $file"
done

# for (C-style)
for ((i=0; i<10; i++)); do
  echo "$i"
done`}</code></pre>

      <h2>getopts</h2>
      <p>
        Parse script options with <code>getopts</code>. It uses the standard POSIX
        option-string syntax:
      </p>
      <pre><code>{`#!/usr/bin/env jesh
while getopts "ab:o:" opt; do
  case $opt in
    a) flag_a=true ;;
    b) arg_b="$OPTARG" ;;
    o) output="$OPTARG" ;;
    ?) echo "Usage: $0 [-a] [-b arg] [-o file]" >&2; exit 1 ;;
  esac
done
shift $((OPTIND - 1))
echo "Remaining args: $@"`}</code></pre>
      <p>
        Each character in the option string is a valid flag. A colon after a character
        means that flag requires an argument, stored in <code>$OPTARG</code>.
        <code>$OPTIND</code> tracks the current option index and can be reset for
        multiple rounds of parsing. The <code>?</code> case handles unknown options.
      </p>
    </DocPage>
  )
}
