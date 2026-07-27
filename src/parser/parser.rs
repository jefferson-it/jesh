use super::*;
use super::lexer::{Redirect, Token};

/// Builds a `Command` from the buffered words/redirects and pushes it onto `commands`.
/// Also extracts leading KEY=VALUE prefixes for environment variables scoped to this command.
pub fn finalize(words: &mut Vec<Word>, redirects: &mut Vec<Redirect>, commands: &mut Vec<Command>) {
    // Extract leading KEY=VALUE assignments as environment variables for the command
    let mut env_vars = Vec::new();
    while let Some(word) = words.first() {
        if let Some((name, value)) = crate::shell::ShellState::as_assignment(&word) {
            env_vars.push((name.clone(), value));
            words.remove(0);
        } else {
            break;
        }
    }

    if words.is_empty() {
        if !env_vars.is_empty() {
            commands.push(Command {
                program: Word::literal(""),
                args: Vec::new(),
                env_vars,
                redirects: Vec::new(),
            });
        }
        return;
    }
    let program = words.remove(0);

    // `&>` / `&>>` target both streams: expand into two explicit redirects.
    // (`-1` is the sentinel produced by the lexer for "both"; fd 0 is stdin.)
    let mut expanded: Vec<Redirect> = Vec::new();
    for r in redirects.drain(..) {
        if r.fd == -1 {
            // `&>` / `&>>` target both streams
            expanded.push(Redirect {
                fd: 1,
                append: r.append,
                target: r.target.clone(),
                dyn_var: None,
            });
            expanded.push(Redirect {
                fd: 2,
                append: r.append,
                target: r.target.clone(),
                dyn_var: None,
            });
        } else if r.fd == -2 {
            // `{VAR}>` / `{VAR}>>` / `{VAR}<` dynamic fd allocation.
            // The fd value will be resolved at exec time;
            // for now keep dyn_var and fd=-2 as a marker.
            expanded.push(r);
        } else {
            expanded.push(r);
        }
    }

    commands.push(Command {
        program,
        args: std::mem::take(words),
        env_vars,
        redirects: expanded,
    });
}

fn finalize_pipeline(
    words: &mut Vec<Word>,
    redirects: &mut Vec<Redirect>,
    commands: &mut Vec<Command>,
) -> Option<Pipeline> {
    finalize(words, redirects, commands);
    if commands.is_empty() {
        None
    } else {
        Some(Pipeline {
            commands: std::mem::take(commands),
        })
    }
}

/// Parses a full line (possibly containing `;`, `&&`, `||`, `|`, and a
/// trailing `&`) into a `CommandList`.
pub fn parse(tokens: Vec<Token>) -> CommandList {
    let mut items: Vec<(AndOrList, Option<ListOp>)> = Vec::new();

    let mut commands: Vec<Command> = Vec::new();
    let mut words: Vec<Word> = Vec::new();
    let mut redirects: Vec<Redirect> = Vec::new();
    let mut background = false;
    let mut negated = false;

    for token in tokens {
        match token {
            Token::Word(w) => words.push(w),
            Token::Redirect(r) => redirects.push(r),
            Token::Not => {
                // `!` at the start of a pipeline negates its exit status.
                // Multiple `!` cancel out (like bash): `!! cmd` is not negated.
                negated = !negated;
            }
            Token::Pipe => finalize(&mut words, &mut redirects, &mut commands),
            Token::Semi => {
                close_item(&mut words, &mut redirects, &mut commands, &mut background, &mut negated, Some(ListOp::Seq), &mut items);
            }
            Token::And => {
                close_item(&mut words, &mut redirects, &mut commands, &mut background, &mut negated, Some(ListOp::And), &mut items);
            }
            Token::Or => {
                close_item(&mut words, &mut redirects, &mut commands, &mut background, &mut negated, Some(ListOp::Or), &mut items);
            }
            Token::Background => {
                background = true;
                close_item(&mut words, &mut redirects, &mut commands, &mut background, &mut negated, None, &mut items);
            }
        }
    }
    // Trailing pipeline with no following operator.
    close_item(&mut words, &mut redirects, &mut commands, &mut background, &mut negated, None, &mut items);

    CommandList { items }
}

fn close_item(
    words: &mut Vec<Word>,
    redirects: &mut Vec<Redirect>,
    commands: &mut Vec<Command>,
    background: &mut bool,
    negated: &mut bool,
    op: Option<ListOp>,
    items: &mut Vec<(AndOrList, Option<ListOp>)>,
) {
    if let Some(pipeline) = finalize_pipeline(words, redirects, commands) {
        items.push((
            AndOrList {
                pipeline,
                background: *background,
                negated: *negated,
            },
            op,
        ));
    }
    *background = false;
    *negated = false;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_env_vars_prefix() {
        let tokens = lexer::tokenize("UBUNTU_MENUPROXY=1 GTK_MODULES=unity-gtk-module gedit");
        let list = parse(tokens);
        assert_eq!(list.items.len(), 1);
        let cmd = &list.items[0].0.pipeline.commands[0];
        assert_eq!(cmd.env_vars, vec![
            ("UBUNTU_MENUPROXY".to_string(), Word::literal("1")),
            ("GTK_MODULES".to_string(), Word::literal("unity-gtk-module")),
        ]);
        assert_eq!(cmd.program.segments.len(), 1);
        if let WordSegment::Literal(ref s) = cmd.program.segments[0] {
            assert_eq!(s, "gedit");
        } else {
            panic!("Expected literal program name");
        }
    }
}
