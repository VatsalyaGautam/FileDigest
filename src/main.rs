use std::fs::File;
use std::io::{self, Read};
use std::path::{PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use blake3::Hasher;
use crossbeam_channel::{unbounded, Receiver, Sender};
use walkdir::WalkDir;

use clap::Parser;
use thiserror::Error;
use anyhow::Context;

// RATATUI imports
use crossterm::event::{self, Event as CEvent, KeyCode};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    layout::{Layout, Constraint, Direction},
    style::{Style, Modifier},
    text::{Span, Line},
};

/// CLI definition using clap derive
#[derive(Parser, Debug)]
#[command(name = "file-hasher", about = "Multithreaded file hasher with TUI")]
struct Cli {
    /// Files or directories to hash
    #[arg(required = true)]
    inputs: Vec<PathBuf>,

    /// Number of worker threads to use
    #[arg(short, long, default_value_t = num_cpus::get())]
    threads: usize,

    /// Follow directories recursively (default true)
    #[arg(short, long, default_value_t = true)]
    recursive: bool,
}

#[derive(Debug, Clone)]
enum Job {
    Hash(PathBuf),
}

#[derive(Debug, Clone)]
enum JobStatus {
    Pending,
    Working,
    Done(String), // hex digest
    Error(String),
}

/// Errors local to hashing pipeline
#[derive(Error, Debug)]
enum AppError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),

    #[error("walkdir error: {0}")]
    WalkDir(#[from] walkdir::Error),

    // #[error("hashing error: {0}")]
    // Hashing(String),
}

/// A simple record for UI and state
#[derive(Debug)]
struct FileRecord {
    path: PathBuf,
    status: JobStatus,
    started_at: Option<Instant>,
    finished_at: Option<Instant>,
}

fn main() -> anyhow::Result<()> {
    env_logger::init(); // optional logger

    let cli = Cli::parse();

    if cli.threads == 0 {
        eprintln!("threads must be >= 1");
        std::process::exit(2);
    }

    // Build list of files
    let mut files: Vec<PathBuf> = Vec::new();
    for input in &cli.inputs {
        collect_paths(input, cli.recursive, &mut files)
            .with_context(|| format!("Collecting files from {:?}", input))?;
    }

    if files.is_empty() {
        println!("No files found to hash.");
        return Ok(());
    }

    // Build file records shared between UI loop and workers via channels
    let (job_tx, job_rx): (Sender<Job>, Receiver<Job>) = unbounded();
    let (status_tx, status_rx): (Sender<(PathBuf, JobStatus)>, Receiver<(PathBuf, JobStatus)>) = unbounded();

    // Insert jobs
    for p in files.iter() {
        job_tx.send(Job::Hash(p.clone())).expect("job channel closed");
    }
    drop(job_tx); // close sender so workers exit when done

    // Spawn worker threads
    let worker_count = cli.threads.min(files.len()).max(1);
    let mut workers = Vec::with_capacity(worker_count);
    for i in 0..worker_count {
        let rx = job_rx.clone();
        let stx = status_tx.clone();
        let worker = thread::Builder::new()
            .name(format!("worker-{}", i))
            .spawn(move || worker_loop(rx, stx))
            .expect("failed to spawn worker");
        workers.push(worker);
    }
    drop(status_tx); // UI will own receiving side

    // Build initial records for UI
    let mut records: Vec<FileRecord> = files.into_iter().map(|p| FileRecord {
        path: p,
        status: JobStatus::Pending,
        started_at: None,
        finished_at: None,
    }).collect();

    // Start TUI and event loop
    run_tui(&mut records, status_rx)?;

    // Wait for workers to finish (join threads)
    for worker in workers {
        let _ = worker.join();
    }

    Ok(())
}

/// Recursively (or not) collect regular files from `path`.
fn collect_paths(path: &PathBuf, recursive: bool, out: &mut Vec<PathBuf>) -> Result<(), AppError> {
    if path.is_file() {
        out.push(path.clone());
        return Ok(());
    }
    if path.is_dir() {
        let walker = WalkDir::new(path).max_depth(if recursive {usize::MAX} else {1});
        for entry_result in walker {
            let entry = entry_result?;
            let ft = entry.file_type();
            if ft.is_file() {
                out.push(entry.path().to_path_buf());
            }
        }
        return Ok(());
    }
    // If path does not exist or is not a file/dir, error
    Err(AppError::Io(io::Error::new(io::ErrorKind::NotFound, format!("{} not found", path.display()))))
}

/// Worker thread loop: receive jobs, compute hash, send status updates.
fn worker_loop(rx: Receiver<Job>, status_tx: Sender<(PathBuf, JobStatus)>) {
    for job in rx.iter() {
        match job {
            Job::Hash(path) => {
                // mark as working
                let _ = status_tx.send((path.clone(), JobStatus::Working));

                // read & hash
                match hash_file(&path) {
                    Ok(digest) => {
                        let _ = status_tx.send((path.clone(), JobStatus::Done(digest)));
                    }
                    Err(e) => {
                        let _ = status_tx.send((path.clone(), JobStatus::Error(format!("{:?}", e))));
                    }
                }
            }
        }
    }
}

/// Incrementally reads a file and returns hex digest. Reports IO errors.
fn hash_file(path: &PathBuf) -> Result<String, AppError> {
    let mut f = File::open(path)?;
    let mut hasher = Hasher::new();
    let mut buffer = [0u8; 8 * 1024];
    loop {
        let n = f.read(&mut buffer)?;
        if n == 0 { break; }
        hasher.update(&buffer[..n]);
    }
    let out = hasher.finalize();
    Ok(hex::encode(out.as_bytes()))
}

/// TUI: render loop that also consumes status updates from worker threads.
/// Press 'q' or Ctrl-C to quit.
fn run_tui(records: &mut Vec<FileRecord>, status_rx: Receiver<(PathBuf, JobStatus)>) -> anyhow::Result<()> {
    // Setup terminal
    let  stdout = std::io::stdout();
    crossterm::terminal::enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // Event polling tick
    let tick_rate = Duration::from_millis(200);

    // Main loop: poll for keys or status updates
    loop {
        // Drain status updates non-blocking
        while let Ok((path, status)) = status_rx.try_recv() {
            if let Some(rec) = records.iter_mut().find(|r| r.path == path) {
                match &status {
                    JobStatus::Working => rec.started_at = Some(Instant::now()),
                    JobStatus::Done(_) | JobStatus::Error(_) => rec.finished_at = Some(Instant::now()),
                    _ => {}
                }
                rec.status = status;
            }
        }

        // Render UI
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(2)].as_ref())
                .split(size);

            let header = Paragraph::new(Line::from(vec![
    Span::raw("file-hasher "),
    Span::styled("(q to quit)", Style::default().add_modifier(Modifier::DIM)),
])).block(Block::default().borders(Borders::ALL).title("Header"));
            f.render_widget(header, chunks[0]);

           let items: Vec<ListItem> = records.iter().map(|r| {
    let status_str = match &r.status {
        JobStatus::Pending => "PENDING".to_string(),
        JobStatus::Working => "WORKING".to_string(),
        JobStatus::Done(d) => format!("DONE {}", d),
        JobStatus::Error(e) => format!("ERR {}", truncate_middle(e, 20)),
    };

    let lines = vec![
        Line::from(Span::raw(r.path.display().to_string())),
        Line::from(Span::styled(status_str, Style::default().add_modifier(Modifier::BOLD))),
    ];

    ListItem::new(lines)
}).collect();

            let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Files"));
            f.render_widget(list, chunks[1]);
        })?;

        // Input handling with timeout
        if event::poll(tick_rate)? {
            if let CEvent::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        // restore terminal
                        crossterm::terminal::disable_raw_mode()?;
                        terminal.show_cursor()?;
                        return Ok(());
                    }
                    KeyCode::Char('c') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                        crossterm::terminal::disable_raw_mode()?;
                        terminal.show_cursor()?;
                        return Ok(());
                    }
                    _ => {}
                }
            }
        }

        // If all records are Done or Error, we can exit
        if records.iter().all(|r| matches!(r.status, JobStatus::Done(_) | JobStatus::Error(_))) {
            crossterm::terminal::disable_raw_mode()?;
            terminal.show_cursor()?;
            return Ok(());
        }
    }
}

/// Utility: truncate long strings in the middle to fit UI
fn truncate_middle(s: &str, keep: usize) -> String {
    if s.len() <= keep { return s.to_string(); }
    let half = keep / 2;
    format!("{}...{}", &s[..half], &s[s.len()-half..])
}