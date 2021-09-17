mod api;
mod app;
mod draw;
mod input;
mod listener;
mod lists;
mod utils;

//use std::{borrow::Borrow, io};
use std::error::Error;
use std::io;
use std::sync::mpsc;
use std::thread;

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};

use tui::{backend::CrosstermBackend, Terminal};

use app::*;
use draw::*;
use input::*;
use listener::listener;

fn main() -> Result<(), Box<dyn Error>> {
    let config = utils::config::get_configs().unwrap();
    let (tx, rx) = mpsc::channel();

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    execute!(io::stdout(), EnterAlternateScreen)?;

    let t_width: u16 = (terminal.size().unwrap().width as f64 * 0.98 * 0.75) as u16;

    let mut app = App::new(config.secret.to_string(), t_width);
    let user_id = app.user_id.clone();

    let notify_thread = thread::spawn(move || listener(rx, &user_id, &config.secret));

    crossterm::terminal::enable_raw_mode()?;
    loop {
        if !poll_input(&mut app, &mut terminal)? {
            break;
        }
        draw_term(&mut terminal, &mut app);
    }

    // Send shutdown to listener
    tx.send(true).unwrap();
    notify_thread.join().unwrap();

    // Restore terminal state before exiting
    crossterm::terminal::disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}
