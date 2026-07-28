import { DocPage } from '../components/DocPage'

export default function Globbing() {
  return (
    <DocPage title="Globbing">
      <p>
        Globbing (pathname expansion) matches file paths using pattern syntax. When a
        word contains unquoted glob characters, jesh replaces it with the list of
        matching files. If no matches are found, the pattern is left unchanged (or an
        error is raised depending on shell options).
      </p>

      <h2>Basic Patterns</h2>
      <table>
        <thead><tr><th>Pattern</th><th>Matches</th><th>Example</th></tr></thead>
        <tbody>
          <tr>
            <td><code>*</code></td>
            <td>Any string of characters, including the empty string.</td>
            <td><code>*.txt</code> matches all text files.</td>
          </tr>
          <tr>
            <td><code>?</code></td>
            <td>Any single character.</td>
            <td><code>file.? </code> matches <code>file.a</code>, <code>file.b</code>.</td>
          </tr>
          <tr>
            <td><code>[abc]</code></td>
            <td>Any one character from the set.</td>
            <td><code>[Ff]ile</code> matches <code>File</code> or <code>file</code>.</td>
          </tr>
          <tr>
            <td><code>[a-z]</code></td>
            <td>Any character in the range.</td>
            <td><code>report[0-9].pdf</code> matches <code>report1.pdf</code>.</td>
          </tr>
          <tr>
            <td><code>[^abc]</code> / <code>[!abc]</code></td>
            <td>Any character not in the set.</td>
            <td><code>[^0-9]*</code> matches files not starting with a digit.</td>
          </tr>
        </tbody>
      </table>

      <h2>Recursive Matching: <code>**</code></h2>
      <p>
        The <code>**</code> pattern matches zero or more directory levels, allowing
        recursive globbing:
      </p>
      <pre><code>**/*.rs          # all Rust source files in the tree
src/**/*.ts      # all TypeScript files under src/
a/**/b           # any b file/dir nested under a/</code></pre>
      <p>
        Unlike a single <code>*</code>, <code>**</code> traverses subdirectories.
        Note that <code>**</code> is only special when it appears alone in a path
        component (i.e., between two slashes or at the start/end of a pattern).
        <code>file**</code> is equivalent to <code>file*</code>.
      </p>

      <h2>Extended Globbing</h2>
      <p>
        jesh supports extended glob patterns (similar to Bash's <code>extglob</code>).
        These must be explicitly enabled (they are on by default in interactive
        sessions):
      </p>
      <table>
        <thead><tr><th>Pattern</th><th>Description</th><th>Matches</th></tr></thead>
        <tbody>
          <tr>
            <td><code>@(pattern-list)</code></td>
            <td>Exactly one of the patterns.</td>
            <td><code>@(*.txt|*.md)</code> matches text or markdown files.</td>
          </tr>
          <tr>
            <td><code>*(pattern-list)</code></td>
            <td>Zero or more occurrences.</td>
            <td><code>*(a|b)</code> matches "", <code>a</code>, <code>b</code>,
            <code>abab</code>.</td>
          </tr>
          <tr>
            <td><code>+(pattern-list)</code></td>
            <td>One or more occurrences.</td>
            <td><code>+([0-9])</code> matches one or more digits.</td>
          </tr>
          <tr>
            <td><code>?(pattern-list)</code></td>
            <td>Zero or one occurrence.</td>
            <td><code>?(foo)</code> matches "" or <code>foo</code>.</td>
          </tr>
          <tr>
            <td><code>!(pattern-list)</code></td>
            <td>Anything except the patterns.</td>
            <td><code>!(*.txt)</code> matches everything except text files.</td>
          </tr>
        </tbody>
      </table>
      <pre><code># Find files that are NOT .txt or .md
echo !(*.txt|*.md)

# Match any filename with exactly one extension
echo +(*.)+([a-z])</code></pre>

      <h2>Glob Flags (shopt)</h2>
      <p>
        Several <code>shopt</code> options control globbing behaviour:
      </p>
      <table>
        <thead><tr><th>Flag</th><th>Description</th></tr></thead>
        <tbody>
          <tr>
            <td><code>nullglob</code></td>
            <td>If no matches are found, the pattern expands to nothing (empty),
            instead of being left as-is. Useful in scripts to avoid passing literal
            patterns to commands.</td>
          </tr>
          <tr>
            <td><code>failglob</code></td>
            <td>If no matches are found, jesh raises an error and the command is not
            executed.</td>
          </tr>
          <tr>
            <td><code>dotglob</code></td>
            <td>Include dotfiles (files beginning with <code>.</code>) in match
            results. By default, dotfiles are excluded unless the pattern starts with
            a <code>.</code>.</td>
          </tr>
          <tr>
            <td><code>nocaseglob</code></td>
            <td>Perform case-insensitive matching.</td>
          </tr>
          <tr>
            <td><code>globstar</code></td>
            <td>Enable <code>**</code> for recursive matching. This is on by default
            in jesh.</td>
          </tr>
          <tr>
            <td><code>extglob</code></td>
            <td>Enable extended glob patterns <code>@</code>, <code>*</code>,
            <code>+</code>, <code>?</code>, <code>!</code>. On by default.</td>
          </tr>
        </tbody>
      </table>
      <pre><code>shopt -s nullglob dotglob nocaseglob</code></pre>

      <h2>Zsh-style Qualifiers</h2>
      <p>
        jesh also supports Zsh-inspired glob qualifiers for filtering by file type.
        These are specified as <code>*(qualifier)</code> immediately after the pattern:
      </p>
      <table>
        <thead><tr><th>Qualifier</th><th>Description</th><th>Example</th></tr></thead>
        <tbody>
          <tr>
            <td><code>*(/)</code></td>
            <td>Match directories only.</td>
            <td><code>*(/)</code> — list all directories.</td>
          </tr>
          <tr>
            <td><code>*(.)</code></td>
            <td>Match regular files only.</td>
            <td><code>src/*(.)</code> — files directly in src/.</td>
          </tr>
          <tr>
            <td><code>*(\@)</code></td>
            <td>Match symbolic links only.</td>
            <td><code>*(@)</code> — list all symlinks.</td>
          </tr>
          <tr>
            <td><code>*(\*)</code></td>
            <td>Match executable files only.</td>
            <td><code>*(*)</code> — list all executables in PATH.</td>
          </tr>
          <tr>
            <td><code>*(^/)</code></td>
            <td>Negate: match everything <em>except</em> directories.</td>
            <td><code>*(^/)</code> — files, symlinks, sockets, etc.</td>
          </tr>
        </tbody>
      </table>
      <pre><code># List only directories
echo *(/)

# List only executable files (not directories)
echo *(*)

# List regular files larger than 1 KB (if size qualifier supported)
# echo *(Lk+1)</code></pre>
      <p>
        Qualifiers can be combined. They are evaluated after pathname expansion and
        filter the results in-memory. Unlike <code>find</code>, qualifiers run entirely
        in the shell process for maximum speed.
      </p>
    </DocPage>
  )
}
