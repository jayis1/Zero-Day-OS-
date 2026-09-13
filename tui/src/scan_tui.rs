// scan-tui — ZERO-DAY OS Live Scan Dashboard
// Interactive nmap/ragnar-scan wrapper with ratatui progress + live results
// Keys: Enter=start scan  j/k=scroll results  Tab=mode  s=stealth  q=quit

use std::{
    io::{self, BufRead, BufReader},
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    style::{Color, Style, Stylize},
    symbols,
    text::{Line, Span},
    widgets::{
        Block, Borders, Gauge, List, ListItem, ListState,
        Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Sparkline, Tabs,
    },
    Frame, Terminal,
};

#[derive(Debug, Clone, PartialEq)]
enum ScanMode { Quick, Full, Stealth, Vuln }

#[derive(Debug, Clone, PartialEq)]
enum ScanState { Idle, Running, Done, Error(String) }

struct AppState {
    mode:         ScanMode,
    scan_state:   ScanState,
    target:       String,
    iface:        String,
    output_lines: Arc<Mutex<Vec<String>>>,
    scroll:       usize,
    progress:     f64,
    elapsed:      Duration,
    start_time:   Option<Instant>,
    find_count:   u64,
    // sparkline of findings over time
    find_history: Vec<u64>,
    input_mode:   bool,
    input_buf:    String,
}

impl AppState {
    fn new() -> Self {
        Self {
            mode:         ScanMode::Quick,
            scan_state:   ScanState::Idle,
            target:       "auto".into(),
            iface:        "auto".into(),
            output_lines: Arc::new(Mutex::new(vec![])),
            scroll:       0,
            progress:     0.0,
            elapsed:      Duration::ZERO,
            start_time:   None,
            find_count:   0,
            find_history: vec![0u64; 30],
            input_mode:   false,
            input_buf:    String::new(),
        }
    }

    fn start_scan(&mut self) {
        let target = self.target.clone();
        let iface  = self.iface.clone();
        let mode   = match self.mode {
            ScanMode::Quick   => "quick",
            ScanMode::Full    => "full",
            ScanMode::Stealth => "stealth",
            ScanMode::Vuln    => "vuln",
        };

        self.output_lines.lock().unwrap().clear();
        self.scroll = 0;
        self.progress = 0.0;
        self.find_count = 0;
        self.start_time = Some(Instant::now());
        self.scan_state = ScanState::Running;

        let lines = Arc::clone(&self.output_lines);
        let mode_s = mode.to_string();

        thread::spawn(move || {
            // Build command: try ragnar-scan first, fallback to nmap
            let (prog, args): (&str, Vec<String>) = if std::path::Path::new("/usr/local/bin/ragnar-scan").exists() {
                ("/usr/local/bin/ragnar-scan", vec![iface.clone(), mode_s.clone()])
            } else if std::path::Path::new("/usr/bin/nmap").exists() || std::path::Path::new("/bin/nmap").exists() {
                let nmap_flags: Vec<String> = match mode_s.as_str() {
                    "quick"   => vec!["-sV".into(), "--top-ports".into(), "1000".into(), target.clone()],
                    "full"    => vec!["-sV".into(), "-sC".into(), "-p-".into(), target.clone()],
                    "stealth" => vec!["-sS".into(), "-T1".into(), "--top-ports".into(), "100".into(), target.clone()],
                    "vuln"    => vec!["-sV".into(), "--script=vuln".into(), target.clone()],
                    _         => vec![target.clone()],
                };
                ("nmap", nmap_flags)
            } else {
                // Fallback: ping sweep
                ("ping", vec!["-c".into(), "4".into(), target.clone()])
            };

            let child = Command::new(prog)
                .args(&args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn();

            match child {
                Err(e) => {
                    lines.lock().unwrap().push(format!("[!] Failed to start {}: {}", prog, e));
                }
                Ok(mut child) => {
                    if let Some(stdout) = child.stdout.take() {
                        let reader = BufReader::new(stdout);
                        for line in reader.lines() {
                            if let Ok(l) = line {
                                lines.lock().unwrap().push(l);
                            }
                        }
                    }
                    let _ = child.wait();
                    lines.lock().unwrap().push("[+] Scan complete".into());
                }
            }
        });
    }
}

// ── Draw ──────────────────────────────────────────────────────────────────────

fn draw(frame: &mut Frame, app: &AppState) {
    let area = frame.area();

    let layout = Layout::vertical([
        Constraint::Length(1), // mode tabs
        Constraint::Length(3), // target + progress
        Constraint::Fill(1),   // output
        Constraint::Length(3), // sparkline
        Constraint::Length(1), // help
    ]).split(area);

    // Mode tabs
    let modes = vec![" Quick ", " Full ", " Stealth ", " Vuln "];
    let mode_idx = match app.mode {
        ScanMode::Quick   => 0,
        ScanMode::Full    => 1,
        ScanMode::Stealth => 2,
        ScanMode::Vuln    => 3,
    };
    let tabs = Tabs::new(modes)
        .select(mode_idx)
        .style(Style::new().fg(Color::DarkGray))
        .highlight_style(Style::new().fg(Color::Cyan).bold())
        .divider("│");
    frame.render_widget(tabs, layout[0]);

    // Target + progress row
    let info_cols = Layout::horizontal([
        Constraint::Percentage(50),
        Constraint::Percentage(50),
    ]).split(layout[1]);

    let target_text = if app.input_mode {
        format!(" Target: {}_", app.input_buf)
    } else {
        format!(" Target: {}  Iface: {}", app.target, app.iface)
    };
    let target_style = if app.input_mode {
        Style::new().fg(Color::White).bg(Color::Rgb(20, 30, 50))
    } else {
        Style::new().fg(Color::Cyan)
    };

    frame.render_widget(
        Paragraph::new(target_text)
            .style(target_style)
            .block(Block::bordered().border_style(Style::new().fg(Color::DarkGray))),
        info_cols[0],
    );

    let (prog_label, prog_color, prog_ratio) = match &app.scan_state {
        ScanState::Idle    => ("Ready — Enter to scan", Color::DarkGray, 0.0),
        ScanState::Running => {
            let elapsed = app.elapsed.as_secs();
            // Fake progress — we don't know total scan time
            let p = (elapsed as f64 / 120.0).min(0.95);
            ("Scanning...", Color::Cyan, p)
        }
        ScanState::Done    => ("Done", Color::Green, 1.0),
        ScanState::Error(_) => ("Error", Color::Red, 0.0),
    };

    let elapsed_s = app.elapsed.as_secs();
    let gauge = Gauge::default()
        .block(Block::bordered()
            .title(format!(" {}  {}s  {} findings ", prog_label, elapsed_s, app.find_count))
            .border_style(Style::new().fg(Color::DarkGray)))
        .gauge_style(Style::new().fg(prog_color))
        .ratio(prog_ratio);
    frame.render_widget(gauge, info_cols[1]);

    // Output lines
    let lines = app.output_lines.lock().unwrap();
    let output_items: Vec<ListItem> = lines.iter().map(|line| {
        let color = if line.starts_with("[+]") || line.contains("open") || line.contains("FOUND") {
            Color::Green
        } else if line.starts_with("[!]") || line.contains("VULN") || line.contains("CVE-") {
            Color::Red
        } else if line.starts_with("[*]") || line.starts_with("Nmap") {
            Color::Cyan
        } else if line.contains("filtered") || line.contains("closed") {
            Color::DarkGray
        } else {
            Color::White
        };
        ListItem::new(Line::from(Span::styled(line.clone(), Style::new().fg(color))))
    }).collect();
    drop(lines);

    let line_count = output_items.len();
    let mut list_state = ListState::default().with_selected(
        if line_count > 0 { Some(app.scroll.min(line_count - 1)) } else { None }
    );

    let list = List::new(output_items)
        .block(Block::bordered()
            .title(" Scan Output ")
            .border_style(Style::new().fg(Color::DarkGray)))
        .highlight_style(Style::new().fg(Color::Cyan));
    frame.render_stateful_widget(list, layout[2], &mut list_state);

    if line_count > layout[2].height as usize {
        let mut sb = ScrollbarState::new(line_count).position(app.scroll);
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight).style(Style::new().fg(Color::DarkGray)),
            layout[2], &mut sb,
        );
    }

    // Findings sparkline
    let spark = Sparkline::default()
        .block(Block::bordered()
            .title(" Findings timeline ")
            .border_style(Style::new().fg(Color::DarkGray)))
        .data(&app.find_history)
        .style(Style::new().fg(Color::Green))
        .bar_set(symbols::bar::NINE_LEVELS);
    frame.render_widget(spark, layout[3]);

    // Help bar
    let help = if app.input_mode {
        " Type target IP/subnet — Enter=confirm  Esc=cancel"
    } else {
        " Enter=scan  t=set target  Tab=mode  j/k=scroll  s=stealth  q=quit"
    };
    frame.render_widget(
        Paragraph::new(help).style(Style::new().fg(Color::DarkGray).bg(Color::Rgb(15, 15, 20))),
        layout[4],
    );
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() -> Result<(), Box<dyn std::error::Error>> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = AppState::new();
    let mut last_tick = Instant::now();

    loop {
        // Update elapsed + progress + sparkline
        if let Some(start) = app.start_time {
            app.elapsed = start.elapsed();
        }

        // Count "open" / "FOUND" lines as findings
        if app.scan_state == ScanState::Running {
            let lines = app.output_lines.lock().unwrap();
            let new_finds = lines.iter().filter(|l| l.contains("open") || l.contains("FOUND")).count() as u64;
            if new_finds != app.find_count {
                app.find_count = new_finds;
                app.find_history.push(new_finds);
                if app.find_history.len() > 30 { app.find_history.remove(0); }
            }
            // Check if scan finished
            if lines.last().map(|l| l.contains("complete") || l.contains("done")).unwrap_or(false) {
                app.scan_state = ScanState::Done;
                app.start_time = None;
            }
            // Auto-scroll to bottom
            app.scroll = lines.len().saturating_sub(1);
        }

        terminal.draw(|f| draw(f, &app))?;

        let timeout = Duration::from_millis(200);
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if app.input_mode {
                    match key.code {
                        KeyCode::Enter => {
                            if !app.input_buf.is_empty() {
                                app.target = app.input_buf.clone();
                            }
                            app.input_buf.clear();
                            app.input_mode = false;
                        }
                        KeyCode::Esc => {
                            app.input_buf.clear();
                            app.input_mode = false;
                        }
                        KeyCode::Backspace => { app.input_buf.pop(); }
                        KeyCode::Char(c)   => { app.input_buf.push(c); }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                        KeyCode::Enter => {
                            if app.scan_state != ScanState::Running {
                                app.start_scan();
                            }
                        }
                        KeyCode::Char('t') => {
                            app.input_mode = true;
                            app.input_buf = app.target.clone();
                        }
                        KeyCode::Tab => {
                            app.mode = match app.mode {
                                ScanMode::Quick   => ScanMode::Full,
                                ScanMode::Full    => ScanMode::Stealth,
                                ScanMode::Stealth => ScanMode::Vuln,
                                ScanMode::Vuln    => ScanMode::Quick,
                            };
                        }
                        KeyCode::Char('s') => app.mode = ScanMode::Stealth,
                        KeyCode::Char('j') | KeyCode::Down => {
                            let n = app.output_lines.lock().unwrap().len();
                            if app.scroll + 1 < n { app.scroll += 1; }
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            if app.scroll > 0 { app.scroll -= 1; }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
