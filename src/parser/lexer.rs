use super::{Word, WordSegment};

#[derive(Debug, Clone)]
pub enum RedirectTarget {
    File(String),
    Fd(i32),
    /// Heredoc: (delimiter, strip_leading_tabs).
    Heredoc(String, bool),
    /// Here-string: the string is the literal content fed to stdin (`<<<`).
    HereString(String),
}

#[derive(Debug, Clone)]
pub struct Redirect {
    /// File descriptor: 0 = stdin, 1 = stdout, 2 = stderr, -1 = both (from `&>`).
    pub fd: i32,
    pub append: bool,
    pub target: RedirectTarget,
}

#[derive(Debug, Clone)]
pub enum Token {
    Word(Word),
    Pipe,
    Redirect(Redirect),
    /// `;`
    Semi,
    /// `&&`
    And,
    /// `||`
    Or,
    /// trailing `&` (background)
    Background,
}

struct Lexer<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    tokens: Vec<Token>,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Lexer {
            chars: input.chars().peekable(),
            tokens: Vec::new(),
        }
    }

    fn run(mut self) -> Vec<Token> {
        loop {
            self.skip_spaces();
            let Some(&c) = self.chars.peek() else { break };

            match c {
                '#' => {
                    // Comment: skip to end of line.
                    while let Some(&c) = self.chars.peek() {
                        if c == '\n' {
                            break;
                        }
                        self.chars.next();
                    }
                    continue;
                }
                '|' => {
                    self.chars.next();
                    if self.chars.peek() == Some(&'|') {
                        self.chars.next();
                        self.tokens.push(Token::Or);
                    } else {
                        self.tokens.push(Token::Pipe);
                    }
                }
                '&' => {
                    self.chars.next();
                    if self.chars.peek() == Some(&'&') {
                        self.chars.next();
                        self.tokens.push(Token::And);
                    } else if self.chars.peek() == Some(&'>') {
                        self.chars.next();
                        let append = self.chars.peek() == Some(&'>');
                        if append {
                            self.chars.next();
                        }
                        self.skip_spaces();
                        let target = self.read_word();
                        self.tokens.push(Token::Redirect(Redirect {
                            fd: -1,
                            append,
                            target: RedirectTarget::File(flatten_literal(&target)),
                        }));
                    } else {
                        self.tokens.push(Token::Background);
                    }
                }
                '\n' => {
                    self.chars.next();
                    self.tokens.push(Token::Semi);
                }
                ';' => {
                    self.chars.next();
                    self.tokens.push(Token::Semi);
                }
                '<' => {
                    self.chars.next();
                    if self.chars.peek() == Some(&'<') {
                        self.chars.next();
                        if self.chars.peek() == Some(&'<') {
                            self.chars.next();
                            self.skip_spaces();
                            let target = self.read_word();
                            self.tokens.push(Token::Redirect(Redirect {
                                fd: 0,
                                append: false,
                                target: RedirectTarget::HereString(flatten_literal(&target)),
                            }));
                        } else {
                            let strip_tabs = self.chars.peek() == Some(&'-');
                            if strip_tabs {
                                self.chars.next();
                            }
                            self.skip_spaces();
                            let target = self.read_word();
                            let delim = flatten_literal(&target)
                                .trim_matches(|c| c == '\'' || c == '"')
                                .to_string();
                            self.tokens.push(Token::Redirect(Redirect {
                                fd: 0,
                                append: false,
                                target: RedirectTarget::Heredoc(delim, strip_tabs),
                            }));
                        }
                    } else {
                        self.skip_spaces();
                        let target = self.read_redirect_word();
                        self.tokens.push(Token::Redirect(Redirect {
                            fd: 0,
                            append: false,
                            target: RedirectTarget::File(flatten_literal(&target)),
                        }));
                    }
                }
                '>' => {
                    self.chars.next();
                    let append = self.chars.peek() == Some(&'>');
                    if append {
                        self.chars.next();
                    }
                    self.skip_spaces();
                    let target = self.read_redirect_word();
                    self.tokens.push(Token::Redirect(Redirect {
                        fd: 1,
                        append,
                        target: redirect_target_from_word(&target),
                    }));
                }
                '0'..='9' => {
                    // Might be `N>` / `N>>` / `N<`; otherwise it's a normal word.
                    if let Some((fd, append, is_input)) = self.peek_numeric_redirect() {
                        self.skip_spaces();
                        let target = self.read_redirect_word();
                        self.tokens.push(Token::Redirect(Redirect {
                            fd,
                            append,
                            target: if is_input {
                                RedirectTarget::File(flatten_literal(&target))
                            } else {
                                redirect_target_from_word(&target)
                            },
                        }));
                    } else {
                        let word = self.read_word();
                        self.tokens.push(Token::Word(word));
                    }
                }
                _ => {
                    let word = self.read_word();
                    if !word.segments.is_empty() {
                        self.tokens.push(Token::Word(word));
                    }
                }
            }
        }

        self.tokens
    }

    fn skip_spaces(&mut self) {
        while matches!(self.chars.peek(), Some(c) if c.is_whitespace() && *c != '\n') {
            self.chars.next();
        }
    }

    /// Looks ahead for a digit-prefixed redirect operator (`2>`, `2>>`, `0<`)
    /// without consuming input unless it matches. Returns `(fd, append, is_input)`.
    fn peek_numeric_redirect(&mut self) -> Option<(i32, bool, bool)> {
        let mut lookahead = self.chars.clone();
        let mut digits = String::new();
        while let Some(&c) = lookahead.peek() {
            if c.is_ascii_digit() {
                digits.push(c);
                lookahead.next();
            } else {
                break;
            }
        }
        if digits.is_empty() {
            return None;
        }
        match lookahead.peek() {
            Some('>') => {
                let fd: i32 = digits.parse().ok()?;
                for _ in 0..digits.len() {
                    self.chars.next();
                }
                self.chars.next(); // '>'
                let append = self.chars.peek() == Some(&'>');
                if append {
                    self.chars.next();
                }
                Some((fd, append, false))
            }
            Some('<') => {
                let fd: i32 = digits.parse().ok()?;
                for _ in 0..digits.len() {
                    self.chars.next();
                }
                self.chars.next(); // '<'
                Some((fd, false, true))
            }
            _ => None,
        }
    }

    /// Like `read_word`, but also allows `&` in the word (used for redirect
    /// targets so `>&1`, `N>&1`, etc. are parsed correctly).
    fn read_redirect_word(&mut self) -> Word {
        self.read_word_inner(false)
    }

    /// Reads one whitespace-delimited word, honoring quotes, backslash
    /// escapes, and `$VAR` / `${VAR}` / `$(...)` / backtick expansions.
    fn read_word(&mut self) -> Word {
        self.read_word_inner(true)
    }

    fn read_word_inner(&mut self, break_on_ampersand: bool) -> Word {
        let mut segments: Vec<WordSegment> = Vec::new();
        let mut current = String::new();
        let mut any_quotes = false;
        let mut at_word_start = true;

        macro_rules! flush_literal {
            () => {
                if !current.is_empty() {
                    segments.push(WordSegment::Literal(std::mem::take(&mut current)));
                }
            };
        }

        loop {
            let Some(&c) = self.chars.peek() else { break };

            if c.is_whitespace() {
                break;
            }
            match c {
                '|' | ';' | '<' | '>' => break,
                '&' if break_on_ampersand => break,
                '\'' => {
                    any_quotes = true;
                    self.chars.next();
                    while let Some(&sc) = self.chars.peek() {
                        if sc == '\'' {
                            self.chars.next();
                            break;
                        }
                        current.push(sc);
                        self.chars.next();
                    }
                }
                '"' => {
                    any_quotes = true;
                    self.chars.next();
                    self.read_double_quoted(&mut current, &mut segments);
                }
                '\\' => {
                    self.chars.next();
                    if let Some(nc) = self.chars.next() {
                        current.push(nc);
                    }
                }
                '~' if at_word_start => {
                    self.chars.next();
                    flush_literal!();
                    let mut rest = String::from("~");
                    while let Some(&tc) = self.chars.peek() {
                        if tc.is_whitespace() || matches!(tc, '|' | '&' | ';' | '<' | '>' | '/') {
                            if tc == '/' {
                                rest.push(tc);
                                self.chars.next();
                            }
                            break;
                        }
                        rest.push(tc);
                        self.chars.next();
                    }
                    segments.push(WordSegment::Tilde(rest));
                }
                '$' => {
                    let mut cloned = self.chars.clone();
                    cloned.next(); // Skip '$'
                    if cloned.peek() == Some(&'\'') {
                        self.chars.next(); // Consume '$'
                        self.chars.next(); // Consume '\''
                        any_quotes = true;
                        flush_literal!();
                        let mut ansi_body = String::new();
                        while let Some(&sc) = self.chars.peek() {
                            if sc == '\'' {
                                self.chars.next();
                                break;
                            }
                            ansi_body.push(sc);
                            self.chars.next();
                        }
                        let parsed = parse_ansi_c_string(&ansi_body);
                        segments.push(WordSegment::Literal(parsed));
                    } else {
                        self.chars.next();
                        self.read_dollar(&mut current, &mut segments);
                    }
                }
                '`' => {
                    self.chars.next();
                    flush_literal!();
                    let mut body = String::new();
                    while let Some(&bc) = self.chars.peek() {
                        if bc == '`' {
                            self.chars.next();
                            break;
                        }
                        body.push(bc);
                        self.chars.next();
                    }
                    segments.push(WordSegment::CommandSubst(body));
                }
                _ => {
                    current.push(c);
                    self.chars.next();
                }
            }
            at_word_start = false;
        }

        flush_literal!();

        Word {
            segments,
            quoted: any_quotes,
        }
    }

    /// Reads the body of a double-quoted string, expanding `$VAR`/`$(...)`
    /// but treating everything else literally.
    fn read_double_quoted(&mut self, current: &mut String, segments: &mut Vec<WordSegment>) {
        loop {
            let Some(&c) = self.chars.peek() else { break };
            match c {
                '"' => {
                    self.chars.next();
                    break;
                }
                '\\' => {
                    self.chars.next();
                    if let Some(nc) = self.chars.next() {
                        // Only these are "special" escapes inside double quotes.
                        if matches!(nc, '"' | '\\' | '$' | '`') {
                            current.push(nc);
                        } else {
                            current.push('\\');
                            current.push(nc);
                        }
                    }
                }
                '$' => {
                    self.chars.next();
                    self.read_dollar(current, segments);
                }
                _ => {
                    current.push(c);
                    self.chars.next();
                }
            }
        }
    }

    /// Handles everything after a `$` has been consumed: `$VAR`, `${VAR}`,
    /// `$(...)`.
    fn read_dollar(&mut self, current: &mut String, segments: &mut Vec<WordSegment>) {
        match self.chars.peek() {
            Some('(') => {
                self.chars.next();
                if self.chars.peek() == Some(&'(') {
                    self.chars.next();
                    if !current.is_empty() {
                        segments.push(WordSegment::Literal(std::mem::take(current)));
                    }
                    let mut depth = 1;
                    let mut body = String::new();
                    while let Some(&c) = self.chars.peek() {
                        self.chars.next();
                        if c == '(' {
                            depth += 1;
                            body.push(c);
                        } else if c == ')' {
                            if depth == 1 && self.chars.peek() == Some(&')') {
                                self.chars.next();
                                break;
                            } else {
                                depth -= 1;
                                body.push(c);
                            }
                        } else {
                            body.push(c);
                        }
                    }
                    segments.push(WordSegment::Arithmetic(body));
                } else {
                    if !current.is_empty() {
                        segments.push(WordSegment::Literal(std::mem::take(current)));
                    }
                    let mut depth = 1;
                    let mut body = String::new();
                    while let Some(&c) = self.chars.peek() {
                        self.chars.next();
                        if c == '(' {
                            depth += 1;
                            body.push(c);
                        } else if c == ')' {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                            body.push(c);
                        } else {
                            body.push(c);
                        }
                    }
                    segments.push(WordSegment::CommandSubst(body));
                }
            }
            Some('{') => {
                self.chars.next();
                if !current.is_empty() {
                    segments.push(WordSegment::Literal(std::mem::take(current)));
                }
                let mut body = String::new();
                let mut depth = 1;
                while let Some(&c) = self.chars.peek() {
                    self.chars.next();
                    if c == '{' {
                        depth += 1;
                        body.push(c);
                    } else if c == '}' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                        body.push(c);
                    } else {
                        body.push(c);
                    }
                }
                segments.push(parse_brace_body(&body));
            }
            Some(&c) if c.is_alphanumeric() || c == '_' || c == '?' || c == '$' || c == '@' || c == '#' => {
                if !current.is_empty() {
                    segments.push(WordSegment::Literal(std::mem::take(current)));
                }
                if matches!(c, '?' | '$' | '@' | '#') {
                    self.chars.next();
                    segments.push(WordSegment::VarExpand(c.to_string()));
                } else {
                    let mut name = String::new();
                    while let Some(&c) = self.chars.peek() {
                        if c.is_alphanumeric() || c == '_' {
                            name.push(c);
                            self.chars.next();
                        } else {
                            break;
                        }
                    }
                    segments.push(WordSegment::VarExpand(name));
                }
            }
            _ => {
                // Lone `$` with nothing recognizable after it.
                current.push('$');
            }
        }
    }
}

/// Parses the body of a `${...}` expansion: either a plain `NAME` (possibly
/// `@`/`#`/digits), or `NAME:+word` / `NAME:-word` parameter expansion.
fn parse_brace_body(body: &str) -> WordSegment {
    if let Some(pos) = body.find(":+").filter(|&p| is_valid_param_name(&body[..p])) {
        return WordSegment::ParamOp(body[..pos].to_string(), '+', body[pos + 2..].to_string());
    }
    if let Some(pos) = body.find(":-").filter(|&p| is_valid_param_name(&body[..p])) {
        return WordSegment::ParamOp(body[..pos].to_string(), '-', body[pos + 2..].to_string());
    }
    WordSegment::VarExpand(body.to_string())
}

fn is_valid_param_name(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_alphanumeric() || c == '_')
}

/// Collapses a `Word` produced for a redirect target/heredoc delimiter into
/// a plain string (no shell-var/glob expansion is performed here; that
/// happens later against `ShellState` for `$VAR` segments via a simple
/// literal join since redirect targets are commonly simple).
fn flatten_literal(word: &Word) -> String {
    let mut out = String::new();
    for seg in &word.segments {
        match seg {
            WordSegment::Literal(s) => out.push_str(s),
            WordSegment::Tilde(s) => out.push_str(s),
            WordSegment::VarExpand(name) => {
                out.push('$');
                out.push_str(name);
            }
            WordSegment::CommandSubst(s) => {
                out.push_str("$(");
                out.push_str(s);
                out.push(')');
            }
            WordSegment::ParamOp(name, op, word) => {
                out.push_str("${");
                out.push_str(name);
                out.push(':');
                out.push(*op);
                out.push_str(word);
                out.push('}');
            }
            WordSegment::Arithmetic(s) => {
                out.push_str("$((");
                out.push_str(s);
                out.push_str("))");
            }
        }
    }
    out
}

fn redirect_target_from_word(word: &Word) -> RedirectTarget {
    let flat = flatten_literal(word);
    if let Some(stripped) = flat.strip_prefix('&') {
        if let Ok(n) = stripped.parse::<i32>() {
            return RedirectTarget::Fd(n);
        }
    }
    RedirectTarget::File(flat)
}

/// Splits a command line into tokens, recognizing pipes, list operators
/// (`;`, `&&`, `||`, trailing `&`), quoting, escapes, variable/command
/// substitution, and shell-style redirections.
pub fn tokenize(input: &str) -> Vec<Token> {
    Lexer::new(input).run()
}

fn parse_ansi_c_string(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('a') => out.push('\x07'),
                Some('b') => out.push('\x08'),
                Some('e') | Some('E') => out.push('\x1B'),
                Some('f') => out.push('\x0C'),
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some('v') => out.push('\x0B'),
                Some('\\') => out.push('\\'),
                Some('\'') => out.push('\''),
                Some('"') => out.push('"'),
                Some('?') => out.push('?'),
                Some(oct) if oct.is_digit(8) => {
                    let mut oct_str = String::from(oct);
                    for _ in 0..2 {
                        if let Some(&nc) = chars.peek() {
                            if nc.is_digit(8) {
                                oct_str.push(nc);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                    }
                    if let Ok(val) = u32::from_str_radix(&oct_str, 8) {
                        if let Some(ch) = std::char::from_u32(val) {
                            out.push(ch);
                        }
                    }
                }
                Some('x') => {
                    let mut hex_str = String::new();
                    for _ in 0..2 {
                        if let Some(&nc) = chars.peek() {
                            if nc.is_ascii_hexdigit() {
                                hex_str.push(nc);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                    }
                    if !hex_str.is_empty() {
                        if let Ok(val) = u32::from_str_radix(&hex_str, 16) {
                            if let Some(ch) = std::char::from_u32(val) {
                                out.push(ch);
                            }
                        }
                    } else {
                        out.push('\\');
                        out.push('x');
                    }
                }
                Some('u') => {
                    let mut hex_str = String::new();
                    for _ in 0..4 {
                        if let Some(&nc) = chars.peek() {
                            if nc.is_ascii_hexdigit() {
                                hex_str.push(nc);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                    }
                    if hex_str.len() == 4 {
                        if let Ok(val) = u32::from_str_radix(&hex_str, 16) {
                            if let Some(ch) = std::char::from_u32(val) {
                                out.push(ch);
                            }
                        }
                    }
                }
                Some('U') => {
                    let mut hex_str = String::new();
                    for _ in 0..8 {
                        if let Some(&nc) = chars.peek() {
                            if nc.is_ascii_hexdigit() {
                                hex_str.push(nc);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                    }
                    if hex_str.len() == 8 {
                        if let Ok(val) = u32::from_str_radix(&hex_str, 16) {
                            if let Some(ch) = std::char::from_u32(val) {
                                out.push(ch);
                            }
                        }
                    }
                }
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => {
                    out.push('\\');
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}
