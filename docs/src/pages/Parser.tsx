import { DocPage } from '../components/DocPage'

export default function Parser() {
  return (
    <DocPage title="Parser">
      <p>
        The jesh parser handles command-line parsing, quoting, expansions, and
        substitutions. This page documents how the parser interprets your input.
      </p>

      <h2>Quoting</h2>

      <h3>Single Quotes</h3>
      <p>
        Text enclosed in single quotes preserves every character literally. No
        expansion (variables, history, globbing) is performed inside single quotes:
      </p>
      <pre><code>{`echo 'The $HOME variable is not expanded here'
echo 'Even backticks \` are literal'`}</code></pre>
      <p>
        The only character that cannot appear inside single quotes is a single quote
        itself. There is no escape sequence for it.
      </p>

      <h3>Double Quotes</h3>
      <p>
        Double quotes preserve most characters literally but still allow variable
        expansion, command substitution, and arithmetic expansion:
      </p>
      <pre><code>{`echo "Home is $HOME"
echo "Today is $(date)"
echo "Result: $((2 + 2))"`}</code></pre>
      <p>
        Backslash retains its escape meaning inside double quotes for
        <code>$</code>, <code>`</code>, <code>"</code>, <code>\</code>, and newline.
        All other backslashes are literal.
      </p>

      <h3>Escape Characters</h3>
      <p>
        Outside quotes, a backslash preserves the next character literally:
      </p>
      <pre><code>{`echo \$HOME   # prints $HOME literally
echo \\       # prints a single backslash
echo hello\ world  # prints "hello world" (escaped space)`}</code></pre>

      <h3>ANSI-C Quoting: <code>$'...'</code></h3>
      <p>
        The <code>$'...'</code> syntax interprets ANSI C escape sequences inside the
        quoted string:
      </p>
      <pre><code>{`echo $'Line one\nLine two'
echo $'Tab\tseparated'
echo $'Bell: \a'
echo $'Unicode: \u2764'  # ❤️
echo $'Hex byte: \x41'   # A`}</code></pre>
      <p>
        Supported escape sequences include <code>\n</code>, <code>\t</code>,
        <code>\r</code>, <code>\a</code>, <code>\b</code>, <code>\e</code> (escape),
        <code>\0nnn</code> (octal), <code>\xHH</code> (hex), <code>\uHHHH</code>
        (Unicode BMP), and <code>\UHHHHHH</code> (full Unicode).
      </p>

      <h2>Line Continuation</h2>
      <p>
        A backslash at the end of a line continues the command on the next line. The
        backslash and the newline are removed from the input:
      </p>
      <pre><code>{`echo "This is a \
very long command \
spread across lines"`}</code></pre>

      <h2>Process Substitution</h2>
      <p>
        Process substitution feeds the output (or input) of a command as a file
        argument. It is a powerful feature for diff, grep, and other tools that expect
        file paths:
      </p>
      <pre><code>{`diff &lt;(sort file1) &lt;(sort file2)
grep pattern &gt;(tee output.txt)`}</code></pre>
      <p>
        <code>&lt;(command)</code> provides the command's stdout as a readable file.
        <code>&gt;(command)</code> provides a writable file that feeds into the
        command's stdin. jesh implements this via <code>/dev/fd</code> where possible.
      </p>

      <h2>History Expansion</h2>
      <p>
        jesh supports interactive history expansion, similar to Bash's <code>!histchars</code>:
      </p>
      <table>
        <thead><tr><th>Expression</th><th>Description</th><th>Example</th></tr></thead>
        <tbody>
          <tr>
            <td><code>!!</code></td>
            <td>Repeat the last command.</td>
            <td><code>!!</code> re-runs the previous command.</td>
          </tr>
          <tr>
            <td><code>!$</code></td>
            <td>Last argument of the previous command.</td>
            <td><code>mkdir dir; cd !$</code> — cd into the newly created dir.</td>
          </tr>
          <tr>
            <td><code>!^</code></td>
            <td>First argument of the previous command.</td>
            <td><code>!^</code></td>
          </tr>
          <tr>
            <td><code>!n</code></td>
            <td>The <em>n</em>-th command in history.</td>
            <td><code>!42</code></td>
          </tr>
          <tr>
            <td><code>!-n</code></td>
            <td>The command <em>n</em> lines back.</td>
            <td><code>!-3</code></td>
          </tr>
          <tr>
            <td><code>!prefix</code></td>
            <td>The most recent command starting with <em>prefix</em>.</td>
            <td><code>!git</code> re-runs the last <code>git</code> command.</td>
          </tr>
          <tr>
            <td><code>!?text</code></td>
            <td>The most recent command containing <em>text</em>.</td>
            <td><code>!?commit</code></td>
          </tr>
          <tr>
            <td><code>!string:s/old/new/</code></td>
            <td>Substitute text in the matched command.</td>
            <td><code>!git:s/push/pull/</code></td>
          </tr>
        </tbody>
      </table>
      <p>
        History expansion is disabled by default when <code>set +H</code> is active or
        within non-interactive scripts.
      </p>

      <h2>Arithmetic Expansion</h2>
      <p>
        Integer arithmetic is performed with <code>$((expression))</code>. Supported
        operators include all C arithmetic, bitwise, and logical operators:
      </p>
      <pre><code>{`echo $((5 + 3 * 2))        # 11
echo $(( (x > y) ? x : y ))  # ternary
echo $((1 << 8))            # 256
echo $((RANDOM % 100))      # random number 0-99
((counter++))               # post-increment variable
((sum += value))            # compound assignment`}</code></pre>

      <h2>Comments</h2>
      <p>
        Everything from an unquoted <code>#</code> to the end of the line is a
        comment. Comments can appear at the start of a line or after a command:
      </p>
      <pre><code>{`# This entire line is a comment
echo "hello"  # inline comment`}</code></pre>
      <p>
        A <code>#</code> inside quotes is literal and does not start a comment.
      </p>
    </DocPage>
  )
}
