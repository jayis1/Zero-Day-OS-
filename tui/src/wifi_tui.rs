// wifi-tui — ZERO-DAY OS WiFi Dashboard
// Ratatui wrapper for wifi-scan, wardrive, wifi-handshake, wifi-crack
// Reads live from /opt/cardputer/loot/recon/ CSVs + iw scan output
//
// Tabs: Scan | Wardrive | Attacks | Saved
// Keys: Tab=switch  j/k=scroll  Enter=select  s=scan  w=wardrive
//       h=handshake  c=crack  d=deauth  q=quit

use std::io;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::fs;

use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, Clear, List, ListItem, ListState,
        Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Sparkline, Table, TableState, Tabs,
    },
};
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

// ── data ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct WifiAp {
    ssid: String,
    bssid: String,
    channel: u32,
    encryption: String,
    signal_dbm: i32,
    frequency_mhz: u32,
    handshake: bool,   // .cap file exists in loot/wifi/
    cracked: bool,     // .pot file exists in loot/wifi/
}

#[derive(Debug, Clone)]
struct WardriveEntry {
    ssid: String,
    bssid: String,
    channel: u32,
    encryption: String,
    signal_dbm: i32,
    session: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Tab { Scan, Wardrive, Attacks, Saved }

struct App {
    tab: Tab,
    aps: Vec<WifiAp>,
    wardrive_entries: Vec<WardriveEntry>,
    saved_caps: Vec<String>,
    scan_table_state: TableState,
    wardrive_table_state: TableState,
    saved_list_state: ListState,
    scan_scroll: ScrollbarState,
    wardrive_scroll: ScrollbarState,
    log: Vec<String>,
    log_scroll: ScrollbarState,
    scanning: bool,
    scan_started: Option<Instant>,
    channel_counts: HashMap<u32, u64>,  // channel -> AP count for sparkline
    signal_history: Vec<u64>,           // rolling signal avg for sparkline
    iface: String,
    last_scan: Option<Instant>,
    status: String,
}

impl App {
    fn new() -> Self {
        let mut app = Self {
            tab: Tab::Scan,
            aps: Vec::new(),
            wardrive_entries: Vec::new(),
            saved_caps: Vec::new(),
            scan_table_state: TableState::default(),
            wardrive_table_state: TableState::default(),
            saved_list_state: ListState::default(),
            scan_scroll: ScrollbarState::default(),
            wardrive_scroll: ScrollbarState::default(),
            log: Vec::new(),
            log_scroll: ScrollbarState::default(),
            scanning: false,
            scan_started: None,
            channel_counts: HashMap::new(),
            signal_history: vec![0; 60],
            iface: detect_iface(),
            last_scan: None,
            status: "Press s to scan".into(),
        };
        app.load_loot();
        app.load_wardrive();
        app
    }

    fn load_loot(&mut self) {
        // Load saved .cap files
        let cap_dir = "/opt/cardputer/loot/wifi";
        self.saved_caps.clear();
        if let Ok(entries) = fs::read_dir(cap_dir) {
            let mut caps: Vec<String> = entries
                .filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().to_string())
                .filter(|n| n.ends_with(".cap") || n.ends_with(".pcap"))
                .collect();
            caps.sort();
            self.saved_caps = caps;
        }
        // Cross-reference with .pot files to mark cracked
        if let Ok(entries) = fs::read_dir("/opt/cardputer/loot/wifi") {
            let cracked: Vec<String> = entries
                .filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().to_string())
                .filter(|n| n.ends_with(".pot") || n.ends_with(".cracked"))
                .collect();
            for ap in &mut self.aps {
                let bssid_clean = ap.bssid.replace(':', "");
                ap.cracked = cracked.iter().any(|c| c.contains(&bssid_clean));
                ap.handshake = self.saved_caps.iter().any(|c| c.contains(&bssid_clean));
            }
        }
    }

    fn load_wardrive(&mut self) {
        let recon_dir = "/opt/cardputer/loot/recon";
        self.wardrive_entries.clear();
        let Ok(entries) = fs::read_dir(recon_dir) else { return };
        let mut csvs: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name().to_string_lossy().starts_with("wifi_scan_")
                    && e.file_name().to_string_lossy().ends_with(".csv")
            })
            .collect();
        csvs.sort_by_key(|e| e.file_name());
        for entry in csvs.iter().rev().take(5) {
            let session = entry.file_name().to_string_lossy()
                .trim_start_matches("wifi_scan_").trim_end_matches(".csv").to_string();
            if let Ok(content) = fs::read_to_string(entry.path()) {
                for line in content.lines() {
                    let parts: Vec<&str> = line.split(',').collect();
                    if parts.len() >= 5 {
                        self.wardrive_entries.push(WardriveEntry {
                            ssid: parts[0].trim().to_string(),
                            bssid: parts[1].trim().to_string(),
                            channel: parts[2].trim().parse().unwrap_or(0),
                            encryption: parts[3].trim().to_string(),
                            signal_dbm: parts[4].trim().parse().unwrap_or(-100),
                            session: session.clone(),
                        });
                    }
                }
            }
        }
        // dedup by BSSID keeping strongest signal
        let mut seen: HashMap<String, usize> = HashMap::new();
        let mut deduped: Vec<WardriveEntry> = Vec::new();
        for entry in &self.wardrive_entries {
            if let Some(&i) = seen.get(&entry.bssid) {
                if entry.signal_dbm > deduped[i].signal_dbm {
                    deduped[i] = entry.clone();
                }
            } else {
                seen.insert(entry.bssid.clone(), deduped.len());
                deduped.push(entry.clone());
            }
        }
        deduped.sort_by(|a, b| b.signal_dbm.cmp(&a.signal_dbm));
        self.wardrive_entries = deduped;
    }

    fn run_scan(&mut self) {
        self.scanning = true;
        self.scan_started = Some(Instant::now());
        self.log.push(format!("[*] Starting scan on {}...", self.iface));
        self.status = format!("Scanning {}...", self.iface);

        // Run iw scan and parse inline (synchronous, ~3s)
        let output = Command::new("sudo")
            .args(["iw", "dev", &self.iface, "scan"])
            .output();

        self.aps.clear();
        self.channel_counts.clear();

        match output {
            Ok(out) => {
                let text = String::from_utf8_lossy(&out.stdout);
                self.aps = parse_iw_scan(&text);
                self.aps.sort_by(|a, b| b.signal_dbm.cmp(&a.signal_dbm));
                for ap in &self.aps {
                    *self.channel_counts.entry(ap.channel).or_insert(0) += 1;
                }
                let avg = if self.aps.is_empty() { 0 }
                    else { (-self.aps.iter().map(|a| a.signal_dbm as i64).sum::<i64>() / self.aps.len() as i64) as u64 };
                self.signal_history.push(avg);
                self.signal_history.drain(..self.signal_history.len().saturating_sub(60));
                self.log.push(format!("[+] Found {} APs", self.aps.len()));
                self.status = format!("{} APs found — s=rescan", self.aps.len());
                self.load_loot();  // refresh handshake/cracked status
            }
            Err(e) => {
                self.log.push(format!("[!] Scan failed: {}", e));
                self.status = "Scan failed — root required?".into();
            }
        }
        self.scanning = false;
        self.last_scan = Some(Instant::now());
    }

    fn selected_ap(&self) -> Option<&WifiAp> {
        self.scan_table_state.selected().and_then(|i| self.aps.get(i))
    }

    fn run_handshake(&mut self) {
        if let Some(ap) = self.selected_ap() {
            let cmd = format!("sudo wifi-handshake {} {} {}", self.iface, ap.bssid, ap.channel);
            self.log.push(format!("[*] {}", cmd));
            self.status = format!("Capturing handshake from {}...", ap.ssid);
        } else {
            self.log.push("[!] No AP selected".into());
        }
    }

    fn run_deauth(&mut self) {
        if let Some(ap) = self.selected_ap() {
            let cmd = format!("sudo wifi-deauth {} {}", self.iface, ap.bssid);
            self.log.push(format!("[*] {}", cmd));
            self.status = format!("Deauthing {} — select AP to confirm", ap.ssid);
        }
    }
}

// ── parsing ──────────────────────────────────────────────────────────────────

fn parse_iw_scan(text: &str) -> Vec<WifiAp> {
    let mut aps = Vec::new();
    let mut cur = WifiAp {
        ssid: String::new(), bssid: String::new(),
        channel: 0, encryption: "OPEN".into(),
        signal_dbm: -100, frequency_mhz: 0,
        handshake: false, cracked: false,
    };
    let mut in_bss = false;

    for line in text.lines() {
        let t = line.trim();
        if t.starts_with("BSS ") {
            if in_bss && !cur.bssid.is_empty() {
                aps.push(cur.clone());
            }
            cur = WifiAp {
                ssid: String::new(),
                bssid: t.split_whitespace().nth(1).unwrap_or("").trim_end_matches('(').to_string(),
                channel: 0, encryption: "OPEN".into(),
                signal_dbm: -100, frequency_mhz: 0,
                handshake: false, cracked: false,
            };
            in_bss = true;
        } else if t.starts_with("SSID:") {
            cur.ssid = t.trim_start_matches("SSID:").trim().to_string();
        } else if t.starts_with("freq:") {
            cur.frequency_mhz = t.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
            cur.channel = freq_to_channel(cur.frequency_mhz);
        } else if t.starts_with("signal:") {
            cur.signal_dbm = t.split_whitespace().nth(1).unwrap_or("0")
                .parse::<f32>().unwrap_or(-100.0) as i32;
        } else if t.contains("RSN") || t.contains("WPA2") {
            cur.encryption = "WPA2".into();
        } else if t.contains("WPA") && cur.encryption != "WPA2" {
            cur.encryption = "WPA".into();
        } else if t.contains("Privacy") && cur.encryption == "OPEN" {
            cur.encryption = "WEP".into();
        }
    }
    if in_bss && !cur.bssid.is_empty() {
        aps.push(cur);
    }
    aps
}

fn freq_to_channel(freq: u32) -> u32 {
    if freq >= 2412 && freq <= 2484 { (freq - 2407) / 5 }
    else if freq >= 5180 { (freq - 5000) / 5 }
    else { 0 }
}

fn detect_iface() -> String {
    for iface in &["wlan0", "wlan1", "wlp2s0"] {
        if std::path::Path::new(&format!("/sys/class/net/{}", iface)).exists() {
            return iface.to_string();
        }
    }
    "wlan0".into()
}

fn signal_bar(dbm: i32) -> &'static str {
    if dbm >= -50 { "▓▓▓▓▓" } else if dbm >= -60 { "▓▓▓▓░" }
    else if dbm >= -70 { "▓▓▓░░" } else if dbm >= -80 { "▓▓░░░" }
    else { "▓░░░░" }
}

fn signal_color(dbm: i32) -> Color {
    if dbm >= -60 { Color::Green } else if dbm >= -75 { Color::Yellow } else { Color::Red }
}

fn enc_color(enc: &str) -> Color {
    match enc {
        "OPEN" => Color::Red,
        "WEP"  => Color::LightRed,
        "WPA"  => Color::Yellow,
        _      => Color::Green,
    }
}

// ── UI ───────────────────────────────────────────────────────────────────────

fn draw(f: &mut ratatui::Frame, app: &mut App) {
    let area = f.area();
    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ]).split(area);

    // tab bar
    let tab_idx = match app.tab { Tab::Scan => 0, Tab::Wardrive => 1, Tab::Attacks => 2, Tab::Saved => 3 };
    let tabs = Tabs::new(vec!["Scan", "Wardrive", "Attacks", "Saved"])
        .select(tab_idx)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .divider("│");
    f.render_widget(tabs, chunks[0]);

    match app.tab {
        Tab::Scan     => draw_scan(f, app, chunks[1]),
        Tab::Wardrive => draw_wardrive(f, app, chunks[1]),
        Tab::Attacks  => draw_attacks(f, app, chunks[1]),
        Tab::Saved    => draw_saved(f, app, chunks[1]),
    }

    // status bar
    let status = Paragraph::new(format!(
        "{}  │  iface:{}  │  Tab=switch  s=scan  h=handshake  d=deauth  c=crack  q=quit",
        app.status, app.iface
    )).style(Style::default().fg(Color::DarkGray));
    f.render_widget(status, chunks[2]);
}

fn draw_scan(f: &mut ratatui::Frame, app: &mut App, area: Rect) {
    let rows_layout = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(5),  // channel sparkline + signal history
    ]).split(area);

    // AP table
    let header = Row::new(vec!["SSID", "BSSID", "Ch", "Enc", "Signal", "Bar", "Cap"])
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

    let rows: Vec<Row> = app.aps.iter().map(|ap| {
        let ssid = if ap.ssid.is_empty() { "<hidden>" } else { ap.ssid.as_str() };
        let cap = if ap.cracked { "🔓" } else if ap.handshake { "📡" } else { "" };
        Row::new(vec![
            Cell::from(truncate(ssid, 20)),
            Cell::from(ap.bssid.clone()).style(Style::default().fg(Color::DarkGray)),
            Cell::from(format!("{:>2}", ap.channel)),
            Cell::from(ap.encryption.as_str()).style(Style::default().fg(enc_color(&ap.encryption))),
            Cell::from(format!("{:>4}dBm", ap.signal_dbm)).style(Style::default().fg(signal_color(ap.signal_dbm))),
            Cell::from(signal_bar(ap.signal_dbm)).style(Style::default().fg(signal_color(ap.signal_dbm))),
            Cell::from(cap).style(Style::default().fg(Color::Yellow)),
        ])
    }).collect();

    let n = rows.len();
    let widths = [
        Constraint::Length(20), Constraint::Length(18), Constraint::Length(3),
        Constraint::Length(5), Constraint::Length(8), Constraint::Length(6),
        Constraint::Length(3),
    ];
    let scan_status = if app.scanning { " SCANNING... " }
        else if n > 0 { "" } else { " (press s to scan) " };
    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL)
            .title(format!(" WiFi APs ({}){}  ", n, scan_status))
            .border_style(Style::default().fg(if app.scanning { Color::Yellow } else { Color::Cyan })))
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    let mut state = app.scan_table_state.clone();
    f.render_stateful_widget(table, rows_layout[0], &mut state);
    app.scan_table_state = state;

    let mut scroll = app.scan_scroll.content_length(n);
    f.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight),
        rows_layout[0], &mut scroll,
    );
    app.scan_scroll = scroll;

    // sparklines: signal history + channel heatmap
    let spark_cols = Layout::horizontal([
        Constraint::Percentage(50), Constraint::Percentage(50)
    ]).split(rows_layout[1]);

    let sig_data: Vec<u64> = app.signal_history.clone();
    let signal_spark = Sparkline::default()
        .block(Block::default().borders(Borders::ALL).title(" Avg Signal History "))
        .data(&sig_data)
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(signal_spark, spark_cols[0]);

    // channel heatmap as sparkline (channel 1-14)
    let ch_data: Vec<u64> = (1..=14).map(|ch| *app.channel_counts.get(&ch).unwrap_or(&0)).collect();
    let ch_spark = Sparkline::default()
        .block(Block::default().borders(Borders::ALL).title(" Channel Density (1-14) "))
        .data(&ch_data)
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(ch_spark, spark_cols[1]);
}

fn draw_wardrive(f: &mut ratatui::Frame, app: &mut App, area: Rect) {
    let header = Row::new(vec!["SSID", "BSSID", "Ch", "Enc", "Signal", "Bar", "Session"])
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    let rows: Vec<Row> = app.wardrive_entries.iter().map(|e| {
        let ssid = if e.ssid.is_empty() { "<hidden>" } else { e.ssid.as_str() };
        Row::new(vec![
            Cell::from(truncate(ssid, 18)),
            Cell::from(e.bssid.clone()).style(Style::default().fg(Color::DarkGray)),
            Cell::from(format!("{:>2}", e.channel)),
            Cell::from(e.encryption.as_str()).style(Style::default().fg(enc_color(&e.encryption))),
            Cell::from(format!("{:>4}dBm", e.signal_dbm)).style(Style::default().fg(signal_color(e.signal_dbm))),
            Cell::from(signal_bar(e.signal_dbm)).style(Style::default().fg(signal_color(e.signal_dbm))),
            Cell::from(truncate(&e.session, 16)).style(Style::default().fg(Color::DarkGray)),
        ])
    }).collect();
    let n = rows.len();
    let widths = [
        Constraint::Length(18), Constraint::Length(18), Constraint::Length(3),
        Constraint::Length(5), Constraint::Length(8), Constraint::Length(6),
        Constraint::Length(16),
    ];
    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL)
            .title(format!(" Wardrive History ({} unique APs from recent sessions) ", n))
            .border_style(Style::default().fg(Color::DarkGray)))
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    let mut state = app.wardrive_table_state.clone();
    f.render_stateful_widget(table, area, &mut state);
    app.wardrive_table_state = state;
    let mut scroll = app.wardrive_scroll.content_length(n);
    f.render_stateful_widget(Scrollbar::new(ScrollbarOrientation::VerticalRight), area, &mut scroll);
    app.wardrive_scroll = scroll;
}

fn draw_attacks(f: &mut ratatui::Frame, app: &mut App, area: Rect) {
    let cols = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(area);

    // Selected AP detail
    let detail_text = if let Some(ap) = app.selected_ap() {
        let ssid = if ap.ssid.is_empty() { "<hidden>" } else { &ap.ssid };
        vec![
            Line::from(vec![Span::styled("SSID:  ", Style::default().fg(Color::DarkGray)),
                Span::styled(ssid.to_string(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD))]),
            Line::from(vec![Span::styled("BSSID: ", Style::default().fg(Color::DarkGray)),
                Span::styled(ap.bssid.clone(), Style::default().fg(Color::Cyan))]),
            Line::from(vec![Span::styled("Ch:    ", Style::default().fg(Color::DarkGray)),
                Span::styled(ap.channel.to_string(), Style::default().fg(Color::White))]),
            Line::from(vec![Span::styled("Enc:   ", Style::default().fg(Color::DarkGray)),
                Span::styled(ap.encryption.clone(), Style::default().fg(enc_color(&ap.encryption)))]),
            Line::from(vec![Span::styled("Signal:", Style::default().fg(Color::DarkGray)),
                Span::styled(format!(" {}dBm {}", ap.signal_dbm, signal_bar(ap.signal_dbm)),
                    Style::default().fg(signal_color(ap.signal_dbm)))]),
            Line::from(""),
            Line::from(vec![Span::styled("Cap:   ", Style::default().fg(Color::DarkGray)),
                Span::styled(if ap.handshake { "YES (.cap)" } else { "no" },
                    Style::default().fg(if ap.handshake { Color::Green } else { Color::DarkGray }))]),
            Line::from(vec![Span::styled("Crack: ", Style::default().fg(Color::DarkGray)),
                Span::styled(if ap.cracked { "CRACKED" } else { "no" },
                    Style::default().fg(if ap.cracked { Color::LightRed } else { Color::DarkGray }))]),
        ]
    } else {
        vec![Line::from(Span::styled(
            "No AP selected — go to Scan tab, j/k to select",
            Style::default().fg(Color::DarkGray)
        ))]
    };
    let detail = Paragraph::new(detail_text)
        .block(Block::default().borders(Borders::ALL).title(" Selected AP ")
            .border_style(Style::default().fg(Color::Cyan)));
    f.render_widget(detail, cols[0]);

    // Attack menu + log
    let right = Layout::vertical([Constraint::Length(10), Constraint::Min(0)]).split(cols[1]);

    let attacks = vec![
        ListItem::new(Line::from(vec![
            Span::styled("h ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("Capture Handshake", Style::default().fg(Color::White)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("d ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled("Deauth Attack", Style::default().fg(Color::White)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("c ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::styled("Crack Handshake (wifi-crack)", Style::default().fg(Color::White)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("p ", Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD)),
            Span::styled("PMKID Attack (wifi-pmkid)", Style::default().fg(Color::White)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("e ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled("Evil Twin (wifi-evil-twin)", Style::default().fg(Color::White)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("P ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("Probe Dump (wifi-probe-dump)", Style::default().fg(Color::White)),
        ])),
    ];
    let attack_list = List::new(attacks)
        .block(Block::default().borders(Borders::ALL).title(" Attack Menu ")
            .border_style(Style::default().fg(Color::Red)));
    f.render_widget(attack_list, right[0]);

    // Activity log
    let log_items: Vec<ListItem> = app.log.iter().rev().take(20).map(|l| {
        let col = if l.contains("[+]") { Color::Green }
            else if l.contains("[!]") { Color::Red }
            else { Color::DarkGray };
        ListItem::new(Span::styled(l.as_str(), Style::default().fg(col)))
    }).collect();
    let log_widget = List::new(log_items)
        .block(Block::default().borders(Borders::ALL).title(" Activity Log ")
            .border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(log_widget, right[1]);
}

fn draw_saved(f: &mut ratatui::Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = if app.saved_caps.is_empty() {
        vec![ListItem::new(Span::styled("No .cap files in /opt/cardputer/loot/wifi/",
            Style::default().fg(Color::DarkGray)))]
    } else {
        app.saved_caps.iter().map(|name| {
            let col = if name.contains(".pot") || name.contains(".cracked") { Color::LightRed }
                else { Color::Cyan };
            ListItem::new(Span::styled(name.as_str(), Style::default().fg(col)))
        }).collect()
    };
    let n = items.len();
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL)
            .title(format!(" Saved Captures ({}) — c=crack selected  r=reload ", n))
            .border_style(Style::default().fg(Color::DarkGray)))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("▶ ");
    let mut state = app.saved_list_state.clone();
    f.render_stateful_widget(list, area, &mut state);
    app.saved_list_state = state;
}

fn truncate(s: &str, n: usize) -> String {
    if s.len() <= n { s.to_string() } else { format!("{}~", &s[..n-1]) }
}

// ── main ─────────────────────────────────────────────────────────────────────

pub fn run() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::new();

    loop {
        terminal.draw(|f| draw(f, &mut app))?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => break,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                    KeyCode::Tab => {
                        app.tab = match app.tab {
                            Tab::Scan => Tab::Wardrive,
                            Tab::Wardrive => Tab::Attacks,
                            Tab::Attacks => Tab::Saved,
                            Tab::Saved => Tab::Scan,
                        };
                    }
                    KeyCode::Char('s') => { app.run_scan(); }
                    KeyCode::Char('r') => { app.load_loot(); app.load_wardrive(); }
                    KeyCode::Char('h') => { app.tab = Tab::Attacks; app.run_handshake(); }
                    KeyCode::Char('d') => { app.tab = Tab::Attacks; app.run_deauth(); }
                    KeyCode::Char('j') | KeyCode::Down => {
                        match app.tab {
                            Tab::Scan => {
                                let i = app.scan_table_state.selected().map(|i| (i+1).min(app.aps.len().saturating_sub(1))).unwrap_or(0);
                                app.scan_table_state.select(Some(i));
                            }
                            Tab::Wardrive => {
                                let i = app.wardrive_table_state.selected().map(|i| (i+1).min(app.wardrive_entries.len().saturating_sub(1))).unwrap_or(0);
                                app.wardrive_table_state.select(Some(i));
                            }
                            Tab::Saved => {
                                let i = app.saved_list_state.selected().map(|i| (i+1).min(app.saved_caps.len().saturating_sub(1))).unwrap_or(0);
                                app.saved_list_state.select(Some(i));
                            }
                            _ => {}
                        }
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        match app.tab {
                            Tab::Scan => {
                                let i = app.scan_table_state.selected().unwrap_or(1).saturating_sub(1);
                                app.scan_table_state.select(Some(i));
                            }
                            Tab::Wardrive => {
                                let i = app.wardrive_table_state.selected().unwrap_or(1).saturating_sub(1);
                                app.wardrive_table_state.select(Some(i));
                            }
                            Tab::Saved => {
                                let i = app.saved_list_state.selected().unwrap_or(1).saturating_sub(1);
                                app.saved_list_state.select(Some(i));
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("wifi-tui error: {}", e);
        std::process::exit(1);
    }
}
