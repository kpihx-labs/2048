use std::io::{stdout, IsTerminal};
use terminal_2048::cli::{help, parse_args, CliAction};
use terminal_2048::game::ui::GameUI;

fn main() -> std::io::Result<()> {
    let action = match parse_args(std::env::args()) {
        Ok(action) => action,
        Err(error) => {
            eprintln!("error: {error}\n\n{}", help());
            std::process::exit(2);
        }
    };

    let layout = match action {
        CliAction::Help => {
            println!("{}", help());
            return Ok(());
        }
        CliAction::Version => {
            println!("2048 {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        CliAction::Run(layout) => layout,
    };

    if !stdout().is_terminal() {
        eprintln!("Not running in an interactive terminal.");
        eprintln!("This game requires an interactive terminal to run.");
        return Ok(());
    }
    let mut game = GameUI::with_layout(layout);
    game.run()
}
