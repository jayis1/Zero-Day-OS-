// sys-tui — ZERO-DAY OS System Monitor
// Ratatui dashboard: CPU, RAM, battery, temp, interfaces, processes
// Usage: sys-tui
//
// Layout (320x170 optimised):
//   ┌──────────────────────────────────────────┐
//   │ CPU ████████░░ 43%   TEMP 52°C   BATT 67% │  ← gauges row
//   ├────────────┬─────────────────────────────-┤
//   │ INTERFACES │ TOP PROCESSES               │  ← split
//   │ wlan0 ↑↓   │ PID  CPU  MEM  CMD          │
//   │ eth0  ↑↓   │ ...                         │
//   ├────────────┴────────────────────────────-┤
//   │ CPU sparkline (last 60s)                 │
//   ├──────────────────────────────────────────┤
//   │ status / help bar                        │
//   └──────────────────────────────────────────┘

use std::{
    fs,
    io::{self},
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
    style::{Color, Modifier, Style, Stylize},
    symbols,
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, Gauge, List, ListItem, Paragraph,
        Row, Sparkline, Table, TableState,
    },
    Frame, Terminal,
};

const TICK_MS: u64 = 1000;
const HISTORY: usize = 60;

// ── System data ───────────────────────────────────────────────────────────────

struct SysState {
    cpu_pct:      f64,
    cpu_history:  Vec<u64>,
    mem_total_kb: u64,
    mem_used_kb:  u64,
    temp_c:       Option<f64>,
    battery_pct:  Option<u64>,
    battery_chg:  bool,
    interfaces:   Vec<(String, u64, u64)>,  // name, rx_bytes, tx_bytes
    processes:    Vec<ProcInfo>,
    uptime_secs:  u64,
    // for delta CPU calc
    prev_idle:    u64,
    prev_total:   u64,
}

struct ProcInfo {
    pid: u32,
    name: String,
    cpu_pct: f64,
    mem_kb: u64,
}

impl SysState {
    fn new() -> Self {
        let mut s = Self {
            cpu_pct: 0.0,
            cpu_history: vec![0u64; HISTORY],
            mem_total_kb: 0,
            mem_used_kb: 0,
            temp_c: None,
            battery_pct: None,
            battery_chg: false,
            interfaces: vec![],
            processes: vec![],
            uptime_secs: 0,
            prev_idle: 0,
            prev_total: 0,
        };
        s.refresh();
        s
    }

    fn refresh(&mut self) {
        self.cpu_pct = self.read_cpu();
        self.cpu_history.push((self.cpu_pct * 100.0) as u64);
        if self.cpu_history.len() > HISTORY { self.cpu_history.remove(0); }
        self.read_mem();
        self.temp_c = read_temp();
        self.battery_pct = read_battery_pct();
        self.battery_chg = read_battery_charging();
        self.interfaces = read_interfaces();
        self.processes = read_top_procs(8);
        self.uptime_secs = read_uptime();
    }

    fn read_cpu(&mut self) -> f64 {
        let stat = fs::read_to_string("/proc/stat").unwrap_or_default();
        let line = stat.lines().next().unwrap_or("");
        let nums: Vec<u64> = line.split_whitespace().skip(1)
            .filter_map(|s| s.parse().ok()).collect();
        if nums.len() < 4 { return self.cpu_pct; }
        let idle  = nums.get(3).copied().unwrap_or(0);
        let total: u64 = nums.iter().sum();
        let d_total = total.saturating_sub(self.prev_total);
        let d_idle  = idle.saturating_sub(self.prev_idle);
        self.prev_total = total;
        self.prev_idle  = idle;
        if d_total == 0 { return self.cpu_pct; }
        ((d_total - d_idle) as f64 / d_total as f64 * 100.0).clamp(0.0, 100.0)
    }

    fn read_mem(&mut self) {
        let s = fs::read_to_string("/proc/meminfo").unwrap_or_default();
        for line in s.lines() {
            if line.starts_with("MemTotal:") {
                self.mem_total_kb = line.split_whitespace().nth(1)
                    .and_then(|v| v.parse().ok()).unwrap_or(0);
            }
            if line.starts_with("MemAvailable:") {
                let avail: u64 = line.split_whitespace().nth(1)
                    .and_then(|v| v.parse().ok()).unwrap_or(0);
                self.mem_used_kb = self.mem_total_kb.saturating_sub(avail);
            }
        }
    }
}

fn read_temp() -> Option<f64> {
    for entry in fs::read_dir("/sys/class/thermal").ok()? {
        let path = entry.ok()?.path().join("temp");
        if let Ok(s) = fs::read_to_string(&path) {
            if let Ok(v) = s.trim().parse::<i64>() {
                return Some(v as f64 / 1000.0);
            }
        }
    }
    None
}

fn read_battery_pct() -> Option<u64> {
    for entry in fs::read_dir("/sys/class/power_supply").ok()? {
        let path = entry.ok()?.path();
        let cap = path.join("capacity");
        if cap.exists() {
            return fs::read_to_string(&cap).ok()?.trim().parse().ok();
        }
    }
    None
}

fn read_battery_charging() -> bool {
    if let Ok(entries) = fs::read_dir("/sys/class/power_supply") {
        for entry in entries.flatten() {
            let p = entry.path().join("status");
            if let Ok(s) = fs::read_to_string(&p) {
                return s.trim() == "Charging";
            }
        }
    }
    false
}

fn read_interfaces() -> Vec<(String, u64, u64)> {
    let s = fs::read_to_string("/proc/net/dev").unwrap_or_default();
    s.lines().skip(2).filter_map(|line| {
        let mut it = line.split_whitespace();
        let name = it.next()?.trim_end_matches(':').to_string();
        if name == "lo" { return None; }
        let rx: u64 = it.next()?.parse().ok()?;
        let tx: u64 = it.nth(7)?.parse().ok()?;
        Some((name, rx, tx))
    }).collect()
}

fn read_top_procs(n: usize) -> Vec<ProcInfo> {
    let mut procs = vec![];
    if let Ok(entries) = fs::read_dir("/proc") {
        for entry in entries.flatten() {
            let pid_str = entry.file_name().to_string_lossy().to_string();
            let Ok(pid) = pid_str.parse::<u32>() else { continue };
            let stat_path = entry.path().join("stat");
            let stat = fs::read_to_string(&stat_path).unwrap_or_default();
            let fields: Vec<&str> = stat.split_whitespace().collect();
            if fields.len() < 24 { continue }
            let name = fields[1].trim_matches(|c| c == '(' || c == ')').to_string();
            let utime: u64 = fields[13].parse().unwrap_or(0);
            let stime: u64 = fields[14].parse().unwrap_or(0);
            let cpu = (utime + stime) as f64 / 100.0;
            let mem_kb: u64 = fs::read_to_string(entry.path().join("status"))
                .unwrap_or_default()
                .lines()
                .find(|l| l.starts_with("VmRSS:"))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);
            procs.push(ProcInfo { pid, name, cpu_pct: cpu, mem_kb });
        }
    }
    procs.sort_by(|a, b| b.mem_kb.cmp(&a.mem_kb));
    procs.truncate(n);
    procs
}

fn read_uptime() -> u64 {
    fs::read_to_string("/proc/uptime").ok()
        .and_then(|s| s.split_whitespace().next().and_then(|v| v.parse::<f64>().ok()))
        .map(|f| f as u64)
        .unwrap_or(0)
}

fn fmt_uptime(s: u64) -> String {
    let h = s / 3600;
    let m = (s % 3600) / 60;
    let sec = s % 60;
    format!("{:02}:{:02}:{:02}", h, m, sec)
}

fn fmt_bytes(b: u64) -> String {
    if b > 1_000_000_000 { format!("{:.1}G", b as f64 / 1e9) }
    else if b > 1_000_000 { format!("{:.1}M", b as f64 / 1e6) }
    else if b > 1_000     { format!("{:.0}K", b as f64 / 1e3) }
    else                  { format!("{}B", b) }
}

// ── Draw ──────────────────────────────────────────────────────────────────────

fn draw(frame: &mut Frame, state: &SysState) {
    let area = frame.area();

    let rows = Layout::vertical([
        Constraint::Length(3),  // gauges
        Constraint::Fill(1),    // middle split
        Constraint::Length(4),  // sparkline
        Constraint::Length(1),  // help
    ]).split(area);

    // ── Gauges row ────────────────────────────────────────────────────────────
    let gauge_cols = Layout::horizontal([
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
    ]).split(rows[0]);

    // CPU gauge
    let cpu_color = if state.cpu_pct > 80.0 { Color::Red }
                    else if state.cpu_pct > 50.0 { Color::Yellow }
                    else { Color::Green };
    let cpu_gauge = Gauge::default()
        .block(Block::bordered().title(" CPU ").border_style(Style::new().fg(Color::DarkGray)))
        .gauge_style(Style::new().fg(cpu_color))
        .ratio(state.cpu_pct / 100.0)
        .label(format!("{:.0}%", state.cpu_pct));
    frame.render_widget(cpu_gauge, gauge_cols[0]);

    // RAM gauge
    let mem_pct = if state.mem_total_kb > 0 {
        state.mem_used_kb as f64 / state.mem_total_kb as f64
    } else { 0.0 };
    let mem_color = if mem_pct > 0.85 { Color::Red }
                    else if mem_pct > 0.6 { Color::Yellow }
                    else { Color::Cyan };
    let mem_gauge = Gauge::default()
        .block(Block::bordered().title(" MEM ").border_style(Style::new().fg(Color::DarkGray)))
        .gauge_style(Style::new().fg(mem_color))
        .ratio(mem_pct)
        .label(format!("{}/{}M",
            state.mem_used_kb / 1024, state.mem_total_kb / 1024));
    frame.render_widget(mem_gauge, gauge_cols[1]);

    // Battery / Temp
    let batt_title = match (state.battery_pct, state.temp_c) {
        (Some(b), Some(t)) => format!(" BATT {}%{}  {:.0}°C ", b, if state.battery_chg { "⚡" } else { "" }, t),
        (Some(b), None)    => format!(" BATT {}{}% ", b, if state.battery_chg { "⚡" } else { "" }),
        (None, Some(t))    => format!(" TEMP {:.0}°C ", t),
        _                  => " BATT/TEMP ".to_string(),
    };
    let batt_pct = state.battery_pct.unwrap_or(100) as f64 / 100.0;
    let batt_color = if batt_pct < 0.2 { Color::Red }
                     else if batt_pct < 0.4 { Color::Yellow }
                     else { Color::Green };
    let batt_gauge = Gauge::default()
        .block(Block::bordered().title(batt_title).border_style(Style::new().fg(Color::DarkGray)))
        .gauge_style(Style::new().fg(batt_color))
        .ratio(batt_pct)
        .label(format!("up {}", fmt_uptime(state.uptime_secs)));
    frame.render_widget(batt_gauge, gauge_cols[2]);

    // ── Middle: interfaces + processes ────────────────────────────────────────
    let mid = Layout::horizontal([
        Constraint::Length(22),
        Constraint::Fill(1),
    ]).split(rows[1]);

    // Interfaces list
    let iface_items: Vec<ListItem> = state.interfaces.iter().map(|(name, rx, tx)| {
        ListItem::new(Line::from(vec![
            Span::styled(format!("{:<8}", name), Style::new().fg(Color::Cyan)),
            Span::styled(format!("↑{:<6}", fmt_bytes(*tx)), Style::new().fg(Color::Green)),
            Span::styled(format!("↓{}", fmt_bytes(*rx)), Style::new().fg(Color::Yellow)),
        ]))
    }).collect();

    let iface_list = List::new(iface_items)
        .block(Block::bordered().title(" NET ").border_style(Style::new().fg(Color::DarkGray)));
    frame.render_widget(iface_list, mid[0]);

    // Process table
    let header = Row::new(vec!["PID", "CPU%", "MEM", "NAME"])
        .style(Style::new().fg(Color::Cyan).bold())
        .height(1);

    let proc_rows: Vec<Row> = state.processes.iter().map(|p| {
        let cpu_style = if p.cpu_pct > 50.0 { Style::new().fg(Color::Red) }
                        else if p.cpu_pct > 20.0 { Style::new().fg(Color::Yellow) }
                        else { Style::new().fg(Color::White) };
        Row::new(vec![
            Cell::from(p.pid.to_string()).style(Style::new().fg(Color::DarkGray)),
            Cell::from(format!("{:.0}", p.cpu_pct)).style(cpu_style),
            Cell::from(fmt_bytes(p.mem_kb * 1024)).style(Style::new().fg(Color::Cyan)),
            Cell::from(p.name.clone()).style(Style::new().fg(Color::White)),
        ])
    }).collect();

    let proc_table = Table::new(proc_rows, [
        Constraint::Length(6),
        Constraint::Length(5),
        Constraint::Length(7),
        Constraint::Fill(1),
    ])
    .header(header)
    .block(Block::bordered().title(" PROCESSES ").border_style(Style::new().fg(Color::DarkGray)));

    let mut ts = TableState::default();
    frame.render_stateful_widget(proc_table, mid[1], &mut ts);

    // ── CPU Sparkline ─────────────────────────────────────────────────────────
    let spark = Sparkline::default()
        .block(Block::bordered().title(" CPU history (1min) ").border_style(Style::new().fg(Color::DarkGray)))
        .data(&state.cpu_history)
        .max(100)
        .style(Style::new().fg(Color::Cyan))
        .bar_set(symbols::bar::NINE_LEVELS);
    frame.render_widget(spark, rows[2]);

    // ── Help bar ──────────────────────────────────────────────────────────────
    frame.render_widget(
        Paragraph::new(" q=quit  r=refresh  ZERO-DAY OS sys-tui")
            .style(Style::new().fg(Color::DarkGray).bg(Color::Rgb(15, 15, 20))),
        rows[3],
    );
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() -> Result<(), Box<dyn std::error::Error>> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = SysState::new();
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| draw(f, &state))?;

        let timeout = Duration::from_millis(TICK_MS)
            .saturating_sub(last_tick.elapsed());

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                    KeyCode::Char('r') => state.refresh(),
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= Duration::from_millis(TICK_MS) {
            state.refresh();
            last_tick = Instant::now();
        }
    }

    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
