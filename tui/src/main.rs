mod app;
mod types;
mod ui;

use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use anyhow::Result;
use clap::Parser;
use app::{App, Focus};

#[derive(Parser, Debug)]
#[command(name = "pdfcmp-tui", version, about = "PDFcmp TUI Prototype")]
struct Args {
    /// PDF files to compress
    #[arg(value_name = "FILE")]
    files: Vec<PathBuf>,
}

fn main() -> Result<()> {
    // Set up panic hook for safe terminal restoration
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = restore_terminal();
        original_hook(panic_info);
    }));

    // Initialize terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let args = Args::parse();

// Run the app
    let res = run_app(&mut terminal, args);

    // Restore terminal
    restore_terminal()?;

    // Handle any errors
    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(
        io::stdout(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, args: Args) -> Result<()> {
    let mut app = if args.files.is_empty() {
        // No files provided, use mock data
        App::new()
    } else {
        // Use provided files
        App::from_files(args.files)
    };
    let tick_rate = Duration::from_millis(100);
    let mut last_tick = Instant::now();

    loop {
        // Process any compression events from background thread
        app.process_events();

        terminal.draw(|f| ui::draw(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if handle_input(&mut app, key.code, key.modifiers) {
                    break;
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            // Tick to update UI (animation, etc.)
            app.tick();
            last_tick = Instant::now();
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn handle_input(app: &mut App, key: KeyCode, modifiers: KeyModifiers) -> bool {
    // Global shortcuts
    match key {
        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
            return true; // Quit
        }
        KeyCode::Char('q') if !matches!(app.focus, Focus::OutputPath) => {
            return true; // Quit
        }
        KeyCode::Char('o') if !matches!(app.focus, Focus::OutputPath) => {
            app.open_file_picker();
            return false;
        }
        KeyCode::Char('?') => {
            app.toggle_help();
            return false;
        }
        _ => {}
    }

    // If help is shown, only process ? to close it
    if app.show_help {
        return false;
    }

    // Focus-specific handling
    match app.focus {
        Focus::FileList => handle_file_list_input(app, key),
        Focus::Preset => handle_preset_input(app, key),
        Focus::Backend => handle_backend_input(app, key),
        Focus::OutputPath => handle_output_path_input(app, key),
        Focus::Help => false,
    }
}

fn handle_file_list_input(app: &mut App, key: KeyCode) -> bool {
    match key {
        KeyCode::Up | KeyCode::Char('k') => {
            app.prev_file();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.next_file();
        }
        KeyCode::Tab => {
            app.next_focus();
        }
        KeyCode::BackTab => {
            app.prev_focus();
        }
        KeyCode::Enter => {
            if app.compression_running {
                app.cancel_compression();
            } else {
                app.start_compression();
            }
        }
        _ => {}
    }
    false
}

fn handle_preset_input(app: &mut App, key: KeyCode) -> bool {
    match key {
        KeyCode::Up | KeyCode::Char('k') => {
            app.prev_preset();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.next_preset();
        }
        KeyCode::Tab => {
            app.next_focus();
        }
        KeyCode::BackTab => {
            app.prev_focus();
        }
        KeyCode::Enter => {
            if app.compression_running {
                app.cancel_compression();
            } else {
                app.start_compression();
            }
        }
        _ => {}
    }
    false
}

fn handle_backend_input(app: &mut App, key: KeyCode) -> bool {
    match key {
        KeyCode::Char('a') => {
            app.toggle_auto_backend();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.prev_backend();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.next_backend();
        }
        KeyCode::Tab => {
            app.next_focus();
        }
        KeyCode::BackTab => {
            app.prev_focus();
        }
        KeyCode::Enter => {
            if app.compression_running {
                app.cancel_compression();
            } else {
                app.start_compression();
            }
        }
        _ => {}
    }
    false
}

fn handle_output_path_input(app: &mut App, key: KeyCode) -> bool {
    if app.compression_running {
        // Only allow focus switching when running
        match key {
            KeyCode::Tab => app.next_focus(),
            KeyCode::BackTab => app.prev_focus(),
            _ => {}
        }
        return false;
    }

    match key {
        KeyCode::Char(c) => {
            app.edit_output_path(c);
        }
        KeyCode::Backspace => {
            app.backspace_output_path();
        }
        KeyCode::Tab => {
            app.next_focus();
        }
        KeyCode::BackTab => {
            app.prev_focus();
        }
        KeyCode::Enter => {
            app.start_compression();
        }
        _ => {}
    }
    false
}
