use crate::ui::App;
use config::Config;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::error::Error;
use tui::{backend::CrosstermBackend, Terminal};

mod config;
mod input;
mod mpd;
mod ui;

fn main() -> Result<(), Box<dyn Error>> {
    // connect to mpd server and create an mpd data holder
    let client = ::mpd::Client::connect("127.0.0.1:6600").unwrap();
    let client = mpd::Mpd::new(client).unwrap();

    // parse config
    let config = Config::new().unwrap();

    // setup UI
    let app = App::build(&client, &config).unwrap();

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // handle input
    if let Err(e) = input::input(&mut terminal, app, client, config) {
        println!("Input Error: {:?}\r", e);
    }

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
