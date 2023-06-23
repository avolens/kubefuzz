use core::sync::atomic::AtomicU64;
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

fn seconds_to_formatted(s: u64) -> String {
    let days = s / (24 * 60 * 60);
    let hours = (s % (24 * 60 * 60)) / (60 * 60);
    let minutes = (s % (60 * 60)) / 60;
    let seconds = s % 60;

    let mut formatted = String::new();

    if days > 0 {
        formatted += &format!("{} days, ", days);
    }
    if hours > 0 {
        formatted += &format!("{} hours, ", hours);
    }
    if minutes > 0 {
        formatted += &format!("{} minutes, ", minutes);
    }

    formatted += &format!("{} seconds, ", seconds);

    if formatted.ends_with(", ") {
        formatted.truncate(formatted.len() - 2);
    }

    formatted
}
fn format_last_time(stats: &FuzzingStats, last: &AtomicU64) -> String {
    match last.load(Ordering::Relaxed) {
        0 => "(none seen)".to_string(),
        x => format!(
            "(last: {} ago)",
            seconds_to_formatted(stats.starttime.elapsed().as_secs() - x)
        ),
    }
}

fn buildui(stats: Arc<FuzzingStats>) -> Vec<Spans<'static>> {
    let mut ui = vec![];

    let mut total_seconds = stats.starttime.elapsed().as_secs() as usize;

    // we dont want to risk a division by zero
    if total_seconds == 0 {
        total_seconds = 1;
    }

    let runtime = seconds_to_formatted(total_seconds as u64);

    let generated = stats.generated.load(Ordering::Relaxed);
    let generated_per_second = generated / total_seconds;

    let mutated = stats.mutated.load(Ordering::Relaxed);
    let mutated_per_second = mutated / total_seconds;

    let lasterror = format_last_time(&stats, &stats.last_error);
    let lastaccepted = format_last_time(&stats, &stats.last_accepted);
    let lastnewcov = format_last_time(&stats, &stats.last_newcov);

    ui.push(Spans::from(Span::raw(format!("Running for {} ", runtime))));

    ui.push(Spans::from(Span::raw(format!(
        "Generated: {} ({}/s)",
        generated, generated_per_second
    ))));

    ui.push(Spans::from(Span::raw(format!(
        "Mutated: {} ({}/s)",
        mutated, mutated_per_second
    ))));

    ui.push(Spans::from(Span::raw(format!(
        "Errors: {} {}",
        stats.errors.load(Ordering::Relaxed),
        lasterror
    ))));

    ui.push(Spans::from(Span::raw(format!(
        "Accepted: {} {}",
        stats.accepted.load(Ordering::Relaxed),
        lastaccepted
    ))));

    ui.push(Spans::from(Span::raw(format!(
        "Corpus Size: {}",
        stats.corpus_size.load(Ordering::Relaxed)
    ))));

    ui.push(Spans::from(Span::raw(format!(
        "New Coverage hits: {} {}",
        stats.newcov.load(Ordering::Relaxed),
        lastnewcov
    ))));

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

            let info = buildui(stats.clone());
            let paragraph = Paragraph::new(info).block(block).wrap(Wrap { trim: true });

            f.render_widget(paragraph, size);
        });

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('c')
                    && key.modifiers == crossterm::event::KeyModifiers::CONTROL
                {
                    stats.exit.store(true, Ordering::Relaxed);
                    break Ok(());
                }
            }
        }
    }
}
