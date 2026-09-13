// trail-tui — ZERO-DAY OS Breadcrumb Navigation Dashboard
// Ratatui live overlay for zeroday-trail: AP fingerprints, exit guidance,
// Overwatch threat alerts, breadcrumb timeline, GPS coords.
//
// Keys: q=quit  r=reload  m=mark-waypoint  c=clear-trail  Tab=view
//
// Layout (320x170):
//   ┌─ EXIT GUIDANCE ──────────────────────┐
//   │ ← BACK  82%  [entrance]  4m ago      │
//   ├─ OVERWATCH ──────┬─ GPS ─────────────┤
//   │ OK               │ 51.5074 -0.1278   │
//   ├─ CURRENT APs ────┴───────────────────┤
//   │ Table: SSID | BSSID | ch | signal ▓▓ │
//   ├─ TRAIL TIMELINE ─────────────────────┤
//   │ List: [tag] timestamp  APs  match%   │
//   └──────────────────────────────────────┘

use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::fs;
use std::io::BufRead;
use std::collections::HashMap;

use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, Clear, Gauge, List, ListItem, ListState,
        Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Table, TableState, Tabs, Wrap,
    },
};
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use serde::{Deserialize, Serialize};

// ── data types (mirrors trail/src/breadcrumb.rs) ────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Breadcrumb {
    timestamp: u64,
    fingerprints: Vec<AccessPoint>,
    altitude_hint: Option<i32>,
    tag: Option<String>,
    gps_lat: Option<f64>,
    gps_lon: Option<f64>,
    gps_alt: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AccessPoint {
    bssid: String,
    ssid: String,
    signal_dbm: i32,
    channel: u32,
    frequency_mhz: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ThreatLevel { None, Watch, Warn, Crit }

#[derive(Debug, Clone, Copy, PartialEq)]
enum ExitDirection { Forward, Back, Left, Right, Here, Lost }

#[derive(Debug, Clone)]
struct ExitGuidance {
    direction: ExitDirection,
    match_pct: u32,
    waypoint_tag: Option<String>,
    elapsed_secs: u64,
    distance_hint: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum View { Overview, APs, Timeline }

// ── state ────────────────────────────────────────────────────────────────────

struct App {
    trail: Vec<Breadcrumb>,
    data_dir: PathBuf,
    last_reload: Instant,
    reload_interval: Duration,
    guidance: Option<ExitGuidance>,
    threat: ThreatLevel,
    threat_reason: String,
    baseline_bssids: Vec<String>,
    view: View,
    ap_table_state: TableState,
    timeline_state: ListState,
    ap_scroll: ScrollbarState,
    timeline_scroll: ScrollbarState,
    mark_input: Option<String>,  // Some = input dialog open
    status_msg: Option<(String, Instant)>,
}

impl App {
    fn new() -> Self {
        let mut s = Self {
            trail: Vec::new(),
            data_dir: PathBuf::from("/opt/cardputer/trail/breadcrumbs"),
            last_reload: Instant::now() - Duration::from_secs(10),
            reload_interval: Duration::from_secs(3),
            guidance: None,
            threat: ThreatLevel::None,
            threat_reason: String::new(),
            baseline_bssids: Vec::new(),
            view: View::Overview,
            ap_table_state: TableState::default(),
            timeline_state: ListState::default(),
            ap_scroll: ScrollbarState::default(),
            timeline_scroll: ScrollbarState::default(),
            mark_input: None,
            status_msg: None,
        };
        s.reload();
        s
    }

    fn reload(&mut self) {
        self.trail = load_trail(&self.data_dir);
        self.compute_guidance();
        self.compute_threat();
        self.last_reload = Instant::now();
        // learn baseline from first 10 breadcrumbs
        if self.baseline_bssids.is_empty() && self.trail.len() >= 3 {
            for bc in self.trail.iter().take(10) {
                for ap in &bc.fingerprints {
                    if !self.baseline_bssids.contains(&ap.bssid) {
                        self.baseline_bssids.push(ap.bssid.clone());
                    }
                }
            }
        }
    }

    fn should_reload(&self) -> bool {
        self.last_reload.elapsed() >= self.reload_interval
    }

    fn compute_guidance(&mut self) {
        let now = now_epoch();
        if self.trail.len() < 2 {
            self.guidance = None;
            return;
        }
        let current = self.trail.last().unwrap().clone();
        // find exit/entrance tags, else use first breadcrumb
        let targets: Vec<&Breadcrumb> = {
            let tagged: Vec<&Breadcrumb> = self.trail.iter()
                .filter(|b| matches!(b.tag.as_deref(), Some("exit") | Some("entrance")))
                .collect();
            if tagged.is_empty() { vec![&self.trail[0]] } else { tagged }
        };

        let best = targets.iter().filter_map(|target| {
            let pct = similarity(&current, target, now);
            if pct < 5 { return None; }
            let elapsed = now.saturating_sub(target.timestamp);
            let dir = infer_direction(&current, target);
            Some(ExitGuidance {
                direction: dir,
                match_pct: pct,
                waypoint_tag: target.tag.clone(),
                elapsed_secs: elapsed,
                distance_hint: format_elapsed(elapsed),
            })
        }).max_by_key(|g| g.match_pct);

        self.guidance = best;
    }

    fn compute_threat(&mut self) {
        if self.trail.is_empty() {
            self.threat = ThreatLevel::None;
            return;
        }
        let current = self.trail.last().unwrap();
        // Evil twin: same SSID, different BSSID than baseline
        let ssid_bssid_map: HashMap<&str, Vec<&str>> = {
            let mut m: HashMap<&str, Vec<&str>> = HashMap::new();
            for ap in &current.fingerprints {
                m.entry(ap.ssid.as_str()).or_default().push(ap.bssid.as_str());
            }
            m
        };
        for (ssid, bssids) in &ssid_bssid_map {
            if bssids.len() > 1 {
                self.threat = ThreatLevel::Crit;
                self.threat_reason = format!("EVIL TWIN: {} ({} BSSIDs)", ssid, bssids.len());
                return;
            }
        }
        // New APs not in baseline
        if !self.baseline_bssids.is_empty() {
            let new: Vec<&str> = current.fingerprints.iter()
                .filter(|ap| !self.baseline_bssids.contains(&ap.bssid))
                .map(|ap| ap.ssid.as_str())
                .collect();
            if new.len() > 3 {
                self.threat = ThreatLevel::Watch;
                self.threat_reason = format!("{} new APs vs baseline", new.len());
                return;
            }
        }
        self.threat = ThreatLevel::None;
        self.threat_reason = String::new();
    }

    fn current_aps(&self) -> &[AccessPoint] {
        self.trail.last().map(|b| b.fingerprints.as_slice()).unwrap_or(&[])
    }

    fn current_gps(&self) -> Option<(f64, f64, Option<f64>)> {
        self.trail.last().and_then(|b| {
            b.gps_lat.zip(b.gps_lon).map(|(lat, lon)| (lat, lon, b.gps_alt))
        })
    }

    fn write_mark(&self, tag: &str) {
        // Write mark file that trail daemon picks up
        let _ = std::fs::write("/tmp/trail-mark", tag);
        let _ = std::process::Command::new("pkill")
            .args(["-USR1", "zeroday-trail"])
            .output();
    }

    fn set_status(&mut self, msg: impl Into<String>) {
        self.status_msg = Some((msg.into(), Instant::now()));
    }
}

// ── data loading ─────────────────────────────────────────────────────────────

fn load_trail(dir: &Path) -> Vec<Breadcrumb> {
    let Ok(entries) = fs::read_dir(dir) else { return Vec::new() };
    let mut all = Vec::new();
    let mut paths: Vec<PathBuf> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().map(|e| e == "jsonl").unwrap_or(false))
        .collect();
    paths.sort();
    for path in &paths {
        if let Ok(f) = fs::File::open(path) {
            for line in std::io::BufReader::new(f).lines().flatten() {
                if let Ok(bc) = serde_json::from_str::<Breadcrumb>(&line) {
                    all.push(bc);
                }
            }
        }
    }
    // keep only last 500
    if all.len() > 500 { all.drain(..all.len()-500); }
    all
}

// ── similarity (mirrors trail/src/breadcrumb.rs) ─────────────────────────────

fn similarity(a: &Breadcrumb, b: &Breadcrumb, now_epoch: u64) -> u32 {
    let a_map: HashMap<&str, i32> = a.fingerprints.iter()
        .map(|ap| (ap.bssid.as_str(), ap.signal_dbm)).collect();
    let b_map: HashMap<&str, i32> = b.fingerprints.iter()
        .map(|ap| (ap.bssid.as_str(), ap.signal_dbm)).collect();
    let shared: Vec<_> = a_map.keys()
        .filter_map(|k| Some((*a_map.get(k)?, *b_map.get(k)?)))
        .collect();
    if shared.is_empty() { return 0; }
    let (mut tw, mut ts) = (0.0f64, 0.0f64);
    for (sa, sb) in &shared {
        let w = ((*sa+100).max(1) as f64/100.0)*((*sb+100).max(1) as f64/100.0);
        let score = 1.0/(1.0+(sa-sb).abs() as f64/10.0);
        tw += w; ts += w*score;
    }
    let ratio = shared.len() as f64 / a_map.len().max(b_map.len()) as f64;
    let age = now_epoch.saturating_sub(a.timestamp.max(b.timestamp));
    let decay = if age < 28800 { 1.0 } else { 0.5f64.powf(age as f64/28800.0-1.0) };
    ((ts/tw.max(0.001)*ratio*decay)*100.0).min(100.0) as u32
}

fn infer_direction(current: &Breadcrumb, target: &Breadcrumb) -> ExitDirection {
    if current.fingerprints.is_empty() || target.fingerprints.is_empty() {
        return ExitDirection::Lost;
    }
    let cur_avg = avg_signal(&current.fingerprints);
    let tgt_avg = avg_signal(&target.fingerprints);
    let shared_delta: f64 = current.fingerprints.iter().filter_map(|ap| {
        target.fingerprints.iter().find(|t| t.bssid == ap.bssid)
            .map(|t| (t.signal_dbm - ap.signal_dbm) as f64)
    }).sum();
    let ratio = cur_avg / tgt_avg.max(0.001);
    if ratio > 1.1 && shared_delta > 3.0  { ExitDirection::Forward }
    else if ratio < 0.9 && shared_delta < -3.0 { ExitDirection::Back }
    else if shared_delta.abs() < 2.0 && ratio > 0.95 { ExitDirection::Here }
    else if shared_delta > 0.0 { ExitDirection::Forward }
    else { ExitDirection::Back }
}

fn avg_signal(aps: &[AccessPoint]) -> f64 {
    if aps.is_empty() { return -50.0; }
    aps.iter().map(|a| a.signal_dbm as f64).sum::<f64>() / aps.len() as f64
}

fn format_elapsed(secs: u64) -> String {
    if secs < 60 { format!("{}s ago", secs) }
    else if secs < 3600 { format!("{}m ago", secs/60) }
    else { format!("{}h{}m ago", secs/3600, (secs%3600)/60) }
}

fn now_epoch() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

fn signal_bar(dbm: i32) -> &'static str {
    if dbm >= -50 { "▓▓▓▓▓" }
    else if dbm >= -60 { "▓▓▓▓░" }
    else if dbm >= -70 { "▓▓▓░░" }
    else if dbm >= -80 { "▓▓░░░" }
    else               { "▓░░░░" }
}

fn signal_color(dbm: i32) -> Color {
    if dbm >= -60 { Color::Green }
    else if dbm >= -75 { Color::Yellow }
    else { Color::Red }
}

fn threat_color(t: ThreatLevel) -> Color {
    match t {
        ThreatLevel::None  => Color::Green,
        ThreatLevel::Watch => Color::Yellow,
        ThreatLevel::Warn  => Color::LightRed,
        ThreatLevel::Crit  => Color::Red,
    }
}

fn dir_symbol(d: ExitDirection) -> &'static str {
    match d {
        ExitDirection::Forward => "↑ FORWARD",
        ExitDirection::Back    => "↓ BACK",
        ExitDirection::Left    => "← LEFT",
        ExitDirection::Right   => "→ RIGHT",
        ExitDirection::Here    => "✓ HERE",
        ExitDirection::Lost    => "✗ LOST",
    }
}

fn dir_color(d: ExitDirection) -> Color {
    match d {
        ExitDirection::Here    => Color::Green,
        ExitDirection::Lost    => Color::Red,
        ExitDirection::Forward | ExitDirection::Back => Color::Cyan,
        _                      => Color::Yellow,
    }
}

// ── UI ───────────────────────────────────────────────────────────────────────

fn draw(f: &mut ratatui::Frame, app: &mut App) {
    let area = f.area();

    // top nav bar (1 line)
    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ]).split(area);

    // tab bar
    let tab_titles = vec!["Overview", "APs", "Timeline"];
    let tab_idx = match app.view { View::Overview => 0, View::APs => 1, View::Timeline => 2 };
    let tabs = Tabs::new(tab_titles)
        .select(tab_idx)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .divider("│");
    f.render_widget(tabs, chunks[0]);

    match app.view {
        View::Overview => draw_overview(f, app, chunks[1]),
        View::APs      => draw_aps(f, app, chunks[1]),
        View::Timeline => draw_timeline(f, app, chunks[1]),
    }

    // status bar
    let status_text = if let Some((msg, t)) = &app.status_msg {
        if t.elapsed() < Duration::from_secs(3) {
            msg.clone()
        } else {
            format!("{} breadcrumbs  │  reload in {}s  │  q=quit m=mark r=reload Tab=view",
                app.trail.len(),
                app.reload_interval.as_secs().saturating_sub(app.last_reload.elapsed().as_secs()))
        }
    } else {
        format!("{} breadcrumbs  │  reload in {}s  │  q=quit m=mark r=reload Tab=view",
            app.trail.len(),
            app.reload_interval.as_secs().saturating_sub(app.last_reload.elapsed().as_secs()))
    };
    let status = Paragraph::new(status_text)
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(status, chunks[2]);

    // mark input dialog
    if let Some(ref input) = app.mark_input.clone() {
        let popup = Rect {
            x: area.width.saturating_sub(32)/2,
            y: area.height/2 - 2,
            width: 32,
            height: 4,
        };
        f.render_widget(Clear, popup);
        let dialog = Paragraph::new(format!("Tag: {}_", input))
            .block(Block::default().borders(Borders::ALL).title(" Mark Waypoint ")
                .border_style(Style::default().fg(Color::Cyan)));
        f.render_widget(dialog, popup);
    }
}

fn draw_overview(f: &mut ratatui::Frame, app: &mut App, area: Rect) {
    let rows = Layout::vertical([
        Constraint::Length(3),   // exit guidance
        Constraint::Length(3),   // threat + GPS
        Constraint::Min(4),      // current APs (compact)
        Constraint::Length(6),   // match gauge + trail summary
    ]).split(area);

    // ── exit guidance ──
    let (dir_text, dir_col, pct, tag_str, time_str) = match &app.guidance {
        Some(g) => (
            dir_symbol(g.direction),
            dir_color(g.direction),
            g.match_pct,
            g.waypoint_tag.as_deref().unwrap_or("waypoint").to_string(),
            g.distance_hint.clone(),
        ),
        None => ("NO DATA", Color::DarkGray, 0, String::new(), String::new()),
    };
    let guide_line = Line::from(vec![
        Span::styled(format!("{:<12}", dir_text), Style::default().fg(dir_col).add_modifier(Modifier::BOLD)),
        Span::styled(format!(" {:>3}%", pct), Style::default().fg(Color::White)),
        Span::styled(if !tag_str.is_empty() { format!("  [{}]", tag_str) } else { String::new() },
            Style::default().fg(Color::Yellow)),
        Span::styled(if !time_str.is_empty() { format!("  {}", time_str) } else { String::new() },
            Style::default().fg(Color::DarkGray)),
    ]);
    let guide = Paragraph::new(guide_line)
        .block(Block::default().borders(Borders::ALL).title(" EXIT GUIDANCE ")
            .border_style(Style::default().fg(Color::Cyan)));
    f.render_widget(guide, rows[0]);

    // ── threat + GPS ──
    let cols = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(rows[1]);

    let threat_label = match app.threat {
        ThreatLevel::None  => "  OK",
        ThreatLevel::Watch => "  WATCH",
        ThreatLevel::Warn  => "  WARN",
        ThreatLevel::Crit  => "  !! CRIT",
    };
    let threat_text = if app.threat_reason.is_empty() {
        Line::from(Span::styled(threat_label, Style::default().fg(threat_color(app.threat)).add_modifier(Modifier::BOLD)))
    } else {
        Line::from(vec![
            Span::styled(threat_label, Style::default().fg(threat_color(app.threat)).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" {}", app.threat_reason), Style::default().fg(Color::Yellow)),
        ])
    };
    let threat_widget = Paragraph::new(threat_text)
        .block(Block::default().borders(Borders::ALL).title(" OVERWATCH ")
            .border_style(Style::default().fg(threat_color(app.threat))));
    f.render_widget(threat_widget, cols[0]);

    let gps_text = match app.current_gps() {
        Some((lat, lon, alt)) => {
            let alt_str = alt.map(|a| format!("  {:.0}m", a)).unwrap_or_default();
            format!("{:.4}  {:.4}{}", lat, lon, alt_str)
        }
        None => "No GPS fix".into(),
    };
    let gps = Paragraph::new(gps_text)
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL).title(" GPS ")
            .border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(gps, cols[1]);

    // ── current APs (compact list) ──
    let aps = app.current_aps();
    let items: Vec<ListItem> = aps.iter().take(8).map(|ap| {
        let bar = signal_bar(ap.signal_dbm);
        let ssid = if ap.ssid.is_empty() { "<hidden>" } else { ap.ssid.as_str() };
        ListItem::new(Line::from(vec![
            Span::styled(format!("{:<18}", truncate(ssid, 18)), Style::default().fg(Color::White)),
            Span::styled(format!(" ch{:>2} ", ap.channel), Style::default().fg(Color::DarkGray)),
            Span::styled(bar, Style::default().fg(signal_color(ap.signal_dbm))),
            Span::styled(format!(" {:>4}dBm", ap.signal_dbm), Style::default().fg(Color::DarkGray)),
        ]))
    }).collect();
    let ap_count = aps.len();
    let ap_list = List::new(items)
        .block(Block::default().borders(Borders::ALL)
            .title(format!(" CURRENT APs ({}) ", ap_count))
            .border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(ap_list, rows[2]);

    // ── match gauge + trail summary ──
    let gauge_cols = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(rows[3]);

    let gauge_pct = pct.min(100) as u16;
    let gauge_col = if pct >= 75 { Color::Green } else if pct >= 40 { Color::Yellow } else { Color::Red };
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" MATCH CONFIDENCE "))
        .gauge_style(Style::default().fg(gauge_col))
        .percent(gauge_pct)
        .label(format!("{}%", pct));
    f.render_widget(gauge, gauge_cols[0]);

    let summary = format!(
        "Trail: {} breadcrumbs  |  Tagged exits: {}  |  Baseline APs: {}",
        app.trail.len(),
        app.trail.iter().filter(|b| matches!(b.tag.as_deref(), Some("exit")|Some("entrance"))).count(),
        app.baseline_bssids.len(),
    );
    let summary_widget = Paragraph::new(summary)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL).title(" TRAIL "));
    f.render_widget(summary_widget, gauge_cols[1]);
}

fn draw_aps(f: &mut ratatui::Frame, app: &mut App, area: Rect) {
    let aps = app.current_aps().to_vec();
    let header = Row::new(vec!["SSID", "BSSID", "Ch", "Freq", "Signal", "Bar"])
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    let rows: Vec<Row> = aps.iter().map(|ap| {
        let ssid = if ap.ssid.is_empty() { "<hidden>".to_string() } else { ap.ssid.clone() };
        Row::new(vec![
            Cell::from(truncate(&ssid, 20)),
            Cell::from(ap.bssid.clone()),
            Cell::from(ap.channel.to_string()),
            Cell::from(format!("{}MHz", ap.frequency_mhz)),
            Cell::from(format!("{}dBm", ap.signal_dbm))
                .style(Style::default().fg(signal_color(ap.signal_dbm))),
            Cell::from(signal_bar(ap.signal_dbm))
                .style(Style::default().fg(signal_color(ap.signal_dbm))),
        ])
    }).collect();
    let n = rows.len();
    let widths = [
        Constraint::Length(20), Constraint::Length(18),
        Constraint::Length(4), Constraint::Length(8),
        Constraint::Length(8), Constraint::Length(6),
    ];
    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL)
            .title(format!(" ACCESS POINTS ({}) — current fingerprint ", n))
            .border_style(Style::default().fg(Color::Cyan)))
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    let mut state = app.ap_table_state.clone();
    f.render_stateful_widget(table, area, &mut state);
    app.ap_table_state = state;

    let mut scroll = app.ap_scroll.content_length(n);
    f.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight),
        area,
        &mut scroll,
    );
    app.ap_scroll = scroll;
}

fn draw_timeline(f: &mut ratatui::Frame, app: &mut App, area: Rect) {
    let now = now_epoch();
    let items: Vec<ListItem> = app.trail.iter().rev().enumerate().map(|(i, bc)| {
        let tag = bc.tag.as_deref().unwrap_or("");
        let elapsed = format_elapsed(now.saturating_sub(bc.timestamp));
        let ap_count = bc.fingerprints.len();
        let gps = if bc.gps_lat.is_some() { " GPS" } else { "" };
        let tag_span = if !tag.is_empty() {
            Span::styled(format!("[{:<8}] ", truncate(tag, 8)), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        } else {
            Span::styled(format!("[{:>3}]       ", app.trail.len().saturating_sub(i)), Style::default().fg(Color::DarkGray))
        };
        ListItem::new(Line::from(vec![
            tag_span,
            Span::styled(format!("{:<12}", elapsed), Style::default().fg(Color::White)),
            Span::styled(format!(" {:>2}APs", ap_count), Style::default().fg(Color::Cyan)),
            Span::styled(gps, Style::default().fg(Color::Green)),
        ]))
    }).collect();

    let n = items.len();
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL)
            .title(format!(" BREADCRUMB TIMELINE ({} entries, newest first) ", n))
            .border_style(Style::default().fg(Color::DarkGray)))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("▶ ");

    let mut state = app.timeline_state.clone();
    f.render_stateful_widget(list, area, &mut state);
    app.timeline_state = state;

    let mut scroll = app.timeline_scroll.content_length(n);
    f.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight),
        area,
        &mut scroll,
    );
    app.timeline_scroll = scroll;
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
        if app.should_reload() { app.reload(); }

        terminal.draw(|f| draw(f, &mut app))?;

        if event::poll(Duration::from_millis(500))? {
            if let Event::Key(key) = event::read()? {
                // mark input dialog
                if let Some(ref mut input) = app.mark_input.clone() {
                    match key.code {
                        KeyCode::Enter => {
                            let tag = input.clone();
                            app.write_mark(&tag);
                            app.set_status(format!("Waypoint marked: {}", tag));
                            app.mark_input = None;
                        }
                        KeyCode::Esc => { app.mark_input = None; }
                        KeyCode::Backspace => {
                            let mut s = input.clone();
                            s.pop();
                            app.mark_input = Some(s);
                        }
                        KeyCode::Char(c) => {
                            let mut s = input.clone();
                            s.push(c);
                            app.mark_input = Some(s);
                        }
                        _ => {}
                    }
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => break,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                    KeyCode::Tab => {
                        app.view = match app.view {
                            View::Overview => View::APs,
                            View::APs      => View::Timeline,
                            View::Timeline => View::Overview,
                        };
                    }
                    KeyCode::Char('r') => {
                        app.reload();
                        app.set_status("Reloaded.".into());
                    }
                    KeyCode::Char('m') => {
                        app.mark_input = Some(String::new());
                    }
                    KeyCode::Char('j') | KeyCode::Down => match app.view {
                        View::APs => {
                            let i = app.ap_table_state.selected().map(|i| i+1).unwrap_or(0);
                            app.ap_table_state.select(Some(i));
                        }
                        View::Timeline => {
                            let i = app.timeline_state.selected().map(|i| i+1).unwrap_or(0);
                            app.timeline_state.select(Some(i));
                        }
                        _ => {}
                    }
                    KeyCode::Char('k') | KeyCode::Up => match app.view {
                        View::APs => {
                            let i = app.ap_table_state.selected().unwrap_or(1).saturating_sub(1);
                            app.ap_table_state.select(Some(i));
                        }
                        View::Timeline => {
                            let i = app.timeline_state.selected().unwrap_or(1).saturating_sub(1);
                            app.timeline_state.select(Some(i));
                        }
                        _ => {}
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
        eprintln!("trail-tui error: {}", e);
        std::process::exit(1);
    }
}
