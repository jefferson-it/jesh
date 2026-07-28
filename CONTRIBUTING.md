# Contributing to jesh

## Build

```bash
git clone https://github.com/anomalyco/jesh
cd jesh
cargo build --release
```

## Test

```bash
cargo test
```

## Code Style

- Follow Rust standard formatting: `cargo fmt`
- Run clippy: `cargo clippy -- -D warnings`
- No `unsafe` unless absolutely necessary and documented
- Use idiomatic Rust patterns (Option, Result, iterators)

## Architecture

- `src/main.rs` — REPL loop, readline integration, signal handling
- `src/parser/` — lexer, parser, AST types
- `src/executor/` — command execution, pipelines, process management
- `src/builtin/` — builtin command implementations
- `src/shell/` — shell state, variables, history, prompt, globbing, functions
- `src/completion/` — tab completion engine
- `src/utils/` — arithmetic eval, ANSI sequences, smart paste, terminal protocols
- `src/semantic/` — structured data pipeline (Nushell-style)

## Pull Request Process

1. Create a feature branch from `main`
2. Run `cargo test` and ensure all tests pass
3. Run `cargo fmt` to format code
4. Update `CHANGELOG.md` with your changes
5. Open a PR with a clear description of the changes

## Adding a Builtin

1. Add the command name to `is_builtin()` in `src/builtin/mod.rs`
2. Add a match arm in `handle_builtin()` in the same file
3. Add tests at the bottom of `src/builtin/mod.rs`
