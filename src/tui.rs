use crate::jobs::{FileRecord, JobStatus};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    layout::{Layout, Constraint, Direction},
    style::{Style, Modifier},
    text::{Line, Span},
};
use crossbeam_channel::Receiver;
use std::time::Duration;
use crossterm::event::{self, Event as CEvent, KeyCode};

pub fn run_tui(records: &mut Vec<FileRecord>, status_rx: Receiver<(std::path::PathBuf, JobStatus)>) -> anyhow::Result<()> {
    let stdout = std::io::stdout();
    crossterm::terminal::enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let tick_rate = Duration::from_millis(200);

    loop {
        while let Ok((path, status)) = status_rx.try_recv() {
            if let Some(rec) = records.iter_mut().find(|r| r.path == path) {
                match &status {
                    JobStatus::Working => rec.started_at = Some(std::time::Instant::now()),
                    JobStatus::Done(_) | JobStatus::Error(_) => rec.finished_at = Some(std::time::Instant::now()),
                    _ => {}
                }
                rec.status = status;
            }
        }

        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(2)].as_ref())
                .split(size);

            let header = Paragraph::new(Line::from(vec![
                Span::raw("file-hasher "), Span::styled("(q to quit)", Style::default().add_modifier(Modifier::DIM))
            ])).block(Block::default().borders(Borders::ALL).title("Header"));
            f.render_widget(header, chunks[0]);

            let items: Vec<ListItem> = records.iter().map(|r| {
                let status_str = match &r.status {
                    JobStatus::Pending => "PENDING".into(),
                    JobStatus::Working => "WORKING".into(),
                    JobStatus::Done(d) => format!("DONE {}", d),
                    JobStatus::Error(e) => format!("ERR {}", crate::utils::truncate_middle(e, 20)),
                };
                ListItem::new(vec![
                    Line::from(r.path.display().to_string()),
                    Line::from(Span::styled(status_str, Style::default().add_modifier(Modifier::BOLD))),
                ])
            }).collect();

            let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Files"));
            f.render_widget(list, chunks[1]);
        })?;

        if event::poll(tick_rate)? {
            if let CEvent::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('c') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                        crossterm::terminal::disable_raw_mode()?;
                        terminal.show_cursor()?;
                        return Ok(());
                    }
                    _ => {}
                }
            }
        }

        if records.iter().all(|r| matches!(r.status, JobStatus::Done(_) | JobStatus::Error(_))) {
            crossterm::terminal::disable_raw_mode()?;
            terminal.show_cursor()?;
            return Ok(());
        }
    }
}