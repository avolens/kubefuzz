use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io,
    sync::{atomic::Ordering, Arc},
};
use tui::{
    backend::CrosstermBackend,
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};

use crate::{error_exit, runtime::mode_fuzz::FuzzingStats};

fn createterm() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

pub fn tui_restore() {
    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture).unwrap();
    disable_raw_mode().expect("could not disable raw mode of terminal");
}

fn buildui(stats: Arc<FuzzingStats>) -> Vec<Span<'static>> {
    let mut ui = vec![];

    ui.push(Span::raw(format!(
        "Time: {}",
        stats.starttime.elapsed().as_secs()
    )));

    ui.push(Span::raw(format!(
        "Generated: {}",
        stats.generated.load(Ordering::Relaxed)
    )));

    ui.push(Span::raw(format!(
        "Mutated: {}",
        stats.mutated.load(Ordering::Relaxed)
    )));

    ui.push(Span::raw(format!(
        "Errors: {}",
        stats.errors.load(Ordering::Relaxed)
    )));

    ui.push(Span::raw(format!(
        "Accepted: {}",
        stats.accepted.load(Ordering::Relaxed)
    )));

    ui.push(Span::raw(format!(
        "Rejected: {}",
        stats.rejected.load(Ordering::Relaxed)
    )));

    ui.push(Span::raw(format!(
        "Corpus Size: {}",
        stats.corpus_size.load(Ordering::Relaxed)
    )));

    ui.push(Span::raw(format!(
        "New Coverage: {}",
        stats.newcov.load(Ordering::Relaxed)
    )));

    ui
}

pub fn tui_loop(stats: Arc<FuzzingStats>) -> io::Result<()> {
    enable_raw_mode()?;
    let mut terminal = createterm()?;
    let title = concat!("KubeFuzz ", env!("VERSION"));

    loop {
        if stats.exit.load(Ordering::Relaxed) {
            break Ok(());
        }

        terminal.draw(|f| {
            let size = f.size();

            let block = Block::default()
                .title(title)
                .title_alignment(tui::layout::Alignment::Center)
                .borders(Borders::ALL);

            let info = Spans::from(buildui(stats.clone()));

            let paragraph = Paragraph::new(info).block(block).wrap(Wrap { trim: true });

            f.render_widget(paragraph, size);
        });
    }
}
