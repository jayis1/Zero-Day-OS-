// loot-tui — ZERO-DAY OS Loot Browser
// Browse /opt/cardputer/loot/ with file preview, cred highlighting, size gauges
// Keys: j/k=nav  l/Enter=open  h/BS=back  p=preview  Tab=categories  q=quit

use std::{
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
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
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, Clear, Gauge, List, ListItem, ListState,
        Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Table, Tabs, Wrap,
    },
    Frame, Terminal,
};

const LOOT_ROOT: &str = "/opt/cardputer/loot";
const CATEGORIES: &[&str] = &["wifi", "bt", "nfc", "ir", "rf", "cam", "recon", "creds", "general", "exfil"];

struct LootState {
    category_idx:  usize,
    entries:       Vec<LootEntry>,
    selected:      usize,
    preview_lines: Vec<String>,
    preview_scroll: usize,
    show_preview:  bool,
    cwd:           PathBuf,
    status:        String,
}

#[derive(Debug, Clone)]
struct LootEntry {
    name:    String,
    path:    PathBuf,
    size:    u64,
    is_dir:  bool,
    is_cred: bool,   // name pattern suggests credentials
}

impl LootState {
    fn new() -> Self {
        let mut s = Self {
            category_idx:  0,
            entries:       vec![],
            selected:      0,
            preview_lines: vec![],
            preview_scroll: 0,
            show_preview:  false,
            cwd:           PathBuf::from(LOOT_ROOT),
            status:        String::new(),
        };
        s.load_category(0);
        s
    }

    fn load_category(&mut self, idx: usize) {
        self.category_idx = idx;
        let dir = PathBuf::from(LOOT_ROOT).join(CATEGORIES[idx]);
        self.navigate_to(&dir.clone());
    }

    fn navigate_to(&mut self, dir: &Path) {
        self.cwd = dir.to_path_buf();
        self.selected = 0;
        self.entries = read_dir_entries(dir);
        let total_size: u64 = self.entries.iter().map(|e| e.size).sum();
        self.status = format!("{} items  {}",
            self.entries.len(), fmt_size(total_size));
    }

    fn selected_entry(&self) -> Option<&LootEntry> {
        self.entries.get(self.selected)
    }

    fn load_preview(&mut self) {
        self.preview_lines.clear();
        self.preview_scroll = 0;
        let Some(entry) = self.selected_entry() else { return };
        if entry.is_dir { return; }

        let path = &entry.path;
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        match ext {
            "json" | "txt" | "log" | "csv" | "xml" | "nmap" | "gnmap" | "md" => {
                if let Ok(mut f) = fs::File::open(path) {
                    let mut content = String::new();
                    let _ = f.read_to_string(&mut content);
                    self.preview_lines = content.lines().take(500)
                        .map(|l| l.chars().take(200).collect())
                        .collect();
                }
            }
            "cap" | "pcap" | "hccapx" | "hc22000" => {
                self.preview_lines = vec![
                    format!("Binary: {} ({})", entry.name, fmt_size(entry.size)),
                    String::new(),
                    "  WiFi capture file".into(),
                    "  Crack with: wifi-autocrack <file>".into(),
                    "  or: wifi-autocrack list".into(),
                ];
            }
            "gz" | "zip" | "tar" => {
                self.preview_lines = vec![
                    format!("Archive: {} ({})", entry.name, fmt_size(entry.size)),
                ];
            }
            _ => {
                // Try to read as text, show hex if binary
                let mut buf = [0u8; 512];
                if let Ok(mut f) = fs::File::open(path) {
                    let n = f.read(&mut buf).unwrap_or(0);
                    let is_binary = buf[..n].iter().any(|&b| b == 0);
                    if is_binary {
                        self.preview_lines = vec![
                            format!("Binary file: {} ({})", entry.name, fmt_size(entry.size)),
                            String::new(),
                        ];
                        // Hex preview of first bytes
                        for chunk in buf[..n].chunks(16) {
                            let hex: String = chunk.iter().map(|b| format!("{:02x} ", b)).collect();
                            let asc: String = chunk.iter().map(|&b| if b >= 0x20 && b < 0x7f { b as char } else { '.' }).collect();
                            self.preview_lines.push(format!("  {}  {}", hex, asc));
                        }
                    } else {
                        let s = String::from_utf8_lossy(&buf[..n]).to_string();
                        self.preview_lines = s.lines().take(50).map(|l| l.to_string()).collect();
                    }
                }
            }
        }
    }
}

fn read_dir_entries(dir: &Path) -> Vec<LootEntry> {
    let mut entries = vec![];
    if !dir.exists() {
        return entries;
    }
    if let Ok(rd) = fs::read_dir(dir) {
        for e in rd.flatten() {
            let path = e.path();
            let name = e.file_name().to_string_lossy().to_string();
            let meta = fs::metadata(&path);
            let (size, is_dir) = match &meta {
                Ok(m) => (if m.is_dir() { dir_size(&path) } else { m.len() }, m.is_dir()),
                Err(_) => (0, false),
            };
            let is_cred = name.contains("pass") || name.contains("cred") || name.contains("hash")
                       || name.contains("ntlm") || name.contains("secret") || name.ends_with(".pot");
            entries.push(LootEntry { name, path, size, is_dir, is_cred });
        }
    }
    entries.sort_by(|a, b| {
        if a.is_dir != b.is_dir { b.is_dir.cmp(&a.is_dir) }
        else { b.size.cmp(&a.size) }
    });
    entries
}

fn dir_size(path: &Path) -> u64 {
    let mut total = 0u64;
    if let Ok(rd) = fs::read_dir(path) {
        for e in rd.flatten() {
            if let Ok(m) = fs::metadata(e.path()) {
                total += m.len();
            }
        }
    }
    total
}

fn fmt_size(b: u64) -> String {
    if b > 1_073_741_824 { format!("{:.1}GB", b as f64 / 1073741824.0) }
    else if b > 1_048_576 { format!("{:.1}MB", b as f64 / 1048576.0) }
    else if b > 1024      { format!("{:.0}KB", b as f64 / 1024.0) }
    else                  { format!("{}B", b) }
}

// ── Draw ──────────────────────────────────────────────────────────────────────

fn draw(frame: &mut Frame, state: &LootState) {
    let area = frame.area();

    let layout = Layout::vertical([
        Constraint::Length(1), // category tabs
        Constraint::Fill(1),   // main
        Constraint::Length(1), // help
    ]).split(area);

    // Category tabs
    let tab_labels: Vec<String> = CATEGORIES.iter().map(|c| format!(" {} ", c)).collect();
    let tabs = Tabs::new(tab_labels)
        .select(state.category_idx)
        .style(Style::new().fg(Color::DarkGray))
        .highlight_style(Style::new().fg(Color::Yellow).bold())
        .divider("│");
    frame.render_widget(tabs, layout[0]);

    // Main area: file list + preview
    let main = if state.show_preview {
        Layout::horizontal([
            Constraint::Percentage(45),
            Constraint::Percentage(55),
        ]).split(layout[1])
    } else {
        Layout::horizontal([Constraint::Percentage(100)]).split(layout[1])
    };

    // File list
    let items: Vec<ListItem> = state.entries.iter().map(|e| {
        let icon = if e.is_dir { "📁" } else { "  " };
        let size_s = fmt_size(e.size);
        let name_color = if e.is_cred { Color::Red }
                         else if e.is_dir { Color::Cyan }
                         else { Color::White };
        let cred_mark = if e.is_cred { "🔑" } else { "  " };
        ListItem::new(Line::from(vec![
            Span::styled(cred_mark, Style::new().fg(Color::Red)),
            Span::styled(format!("{:<6} ", size_s), Style::new().fg(Color::DarkGray)),
            Span::styled(e.name.clone(), Style::new().fg(name_color)),
        ]))
    }).collect();

    let cwd_short = state.cwd.strip_prefix(LOOT_ROOT)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| state.cwd.display().to_string());

    let mut list_state = ListState::default().with_selected(Some(state.selected));
    let list = List::new(items)
        .block(Block::bordered()
            .title(format!(" loot/{} — {} ", cwd_short, state.status))
            .title_style(Style::new().fg(Color::Yellow))
            .border_style(Style::new().fg(Color::DarkGray)))
        .highlight_style(Style::new().fg(Color::Yellow).bold())
        .highlight_symbol("▶ ");
    frame.render_stateful_widget(list, main[0], &mut list_state);

    // Scrollbar for file list
    if state.entries.len() > main[0].height as usize {
        let mut sb = ScrollbarState::new(state.entries.len()).position(state.selected);
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight).style(Style::new().fg(Color::DarkGray)),
            main[0], &mut sb,
        );
    }

    // Preview pane
    if state.show_preview && main.len() > 1 {
        let preview_text: Vec<Line> = state.preview_lines
            .iter()
            .skip(state.preview_scroll)
            .map(|line| {
                // Highlight credential patterns
                let color = if line.to_lowercase().contains("password")
                            || line.to_lowercase().contains("psk:")
                            || line.to_lowercase().contains("hash:")
                            || line.to_lowercase().contains("ntlm") {
                    Color::Red
                } else if line.starts_with("  ") {
                    Color::DarkGray
                } else {
                    Color::White
                };
                Line::from(Span::styled(line.clone(), Style::new().fg(color)))
            }).collect();

        let name = state.selected_entry().map(|e| e.name.as_str()).unwrap_or("");
        let preview = Paragraph::new(preview_text)
            .block(Block::bordered()
                .title(format!(" Preview: {} ", name))
                .title_style(Style::new().fg(Color::Cyan))
                .border_style(Style::new().fg(Color::DarkGray)))
            .wrap(Wrap { trim: false });
        frame.render_widget(preview, main[1]);
    }

    // Help bar
    let help = if state.show_preview {
        " j/k=nav  l=open  h=back  p=preview off  Tab=category  q=quit"
    } else {
        " j/k=nav  l/Enter=open  h/BS=back  p=preview  Tab=next category  q=quit"
    };
    frame.render_widget(
        Paragraph::new(help).style(Style::new().fg(Color::DarkGray).bg(Color::Rgb(15, 15, 20))),
        layout[2],
    );
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() -> Result<(), Box<dyn std::error::Error>> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = LootState::new();

    loop {
        terminal.draw(|f| draw(f, &state))?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,

                    KeyCode::Char('j') | KeyCode::Down => {
                        if state.selected + 1 < state.entries.len() {
                            state.selected += 1;
                            if state.show_preview { state.load_preview(); }
                        }
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        if state.selected > 0 {
                            state.selected -= 1;
                            if state.show_preview { state.load_preview(); }
                        }
                    }
                    KeyCode::Char('l') | KeyCode::Enter => {
                        if let Some(entry) = state.selected_entry().cloned() {
                            if entry.is_dir {
                                state.navigate_to(&entry.path.clone());
                            } else {
                                state.show_preview = true;
                                state.load_preview();
                            }
                        }
                    }
                    KeyCode::Char('h') | KeyCode::Backspace => {
                        let parent = state.cwd.parent().map(|p| p.to_path_buf());
                        if let Some(p) = parent {
                            if p.starts_with(LOOT_ROOT) {
                                state.navigate_to(&p.clone());
                            }
                        }
                    }
                    KeyCode::Char('p') => {
                        state.show_preview = !state.show_preview;
                        if state.show_preview { state.load_preview(); }
                    }
                    KeyCode::Tab => {
                        let next = (state.category_idx + 1) % CATEGORIES.len();
                        state.load_category(next);
                    }
                    KeyCode::BackTab => {
                        let prev = if state.category_idx == 0 { CATEGORIES.len() - 1 } else { state.category_idx - 1 };
                        state.load_category(prev);
                    }
                    // Preview scroll
                    KeyCode::PageDown => {
                        state.preview_scroll = (state.preview_scroll + 10)
                            .min(state.preview_lines.len().saturating_sub(1));
                    }
                    KeyCode::PageUp => {
                        state.preview_scroll = state.preview_scroll.saturating_sub(10);
                    }
                    _ => {}
                }
            }
        }
    }

    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
