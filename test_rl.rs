use rustyline::{Cmd, Editor, KeyCode, KeyEvent, Modifiers, Result};
use rustyline::history::DefaultHistory;

fn main() -> Result<()> {
    let mut rl = Editor::<(), DefaultHistory>::new()?;
    rl.add_history_entry("ls -l")?;
    rl.add_history_entry("echo hello")?;
    rl.add_history_entry("bash -c 'exit 1'")?;
    
    // Bind Up to HistorySearchBackward
    rl.bind_sequence(KeyEvent(KeyCode::Up, Modifiers::empty()), Cmd::HistorySearchBackward);
    
    // Try it
    println!("Try pressing Up");
    let line = rl.readline("> ")?;
    println!("Got: {}", line);
    Ok(())
}
