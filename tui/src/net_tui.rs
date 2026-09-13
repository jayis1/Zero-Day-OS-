// net-tui — ZERO-DAY OS Network Dashboard
// Live table of discovered hosts, open ports, services — reads from loot/recon
// Auto-refreshes as new nmap/ragnar-scan results appear.
// Keys: j/k=scroll  Tab=toggle hosts/ports  r=reload  q=quit

use std::{
    fs, io,
    path::PathBuf,
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
        Block, Borders, Cell, List, ListItem, ListState,
        Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Table, TableState, Tabs,
    },
    Frame, Terminal,
};

const LOOT_DIR: &str = "/opt/cardputer/loot/recon";
const TICK_MS:  u64 = 5000;

#[derive(Debug, Clone)]
struct HostEntry {
    ip:       String,
    hostname: String,
    os_hint: String,
    ports:   Vec<PortEntry>,
}

#[derive(Debug, Clone)]
struct PortEntry {
    port:     u16,
    protocol: String,
    state:    String,
    service:  String,
    version:  String,
}

struct NetState {
    hosts:        Vec<HostEntry>,
    selected_host: usize,
    selected_port: usize,
    tab:          usize,   // 0=hosts  1=ports  2=summary
    last_load:    Instant,
    scan_files:   Vec<PathBuf>,
    status:       String,
}

impl NetState {
    fn new() -> Self {
        let mut s = Self {
            hosts:         vec![],
            selected_host: 0,
            selected_port: 0,
            tab:           0,
            last_load:     Instant::now(),
            scan_files:    vec![],
            status:        "Loading...".into(),
        };
        s.reload();
        s
    }

    fn reload(&mut self) {
        self.scan_files = scan_xml_files(LOOT_DIR);
        let mut all_hosts: Vec<HostEntry> = vec![];
        for path in &self.scan_files {
            all_hosts.extend(parse_nmap_xml(path));
        }
        // Merge duplicate IPs
        let mut merged: Vec<HostEntry> = vec![];
        'outer: for h in all_hosts {
            for m in &mut merged {
                if m.ip == h.ip {
                    for p in h.ports {
                        if !m.ports.iter().any(|ep| ep.port == p.port) {
                            m.ports.push(p);
                        }
                    }
                    continue 'outer;
                }
            }
            merged.push(h);
        }
        merged.sort_by(|a, b| a.ip.cmp(&b.ip));
        let total_ports: usize = merged.iter().map(|h| h.ports.len()).sum();
        self.status = format!("{} hosts  {} open ports  {} scan files",
            merged.len(), total_ports, self.scan_files.len());
        self.hosts = merged;
        self.last_load = Instant::now();
    }

    fn selected_host(&self) -> Option<&HostEntry> {
        self.hosts.get(self.selected_host)
    }
}

fn scan_xml_files(dir: &str) -> Vec<PathBuf> {
    let mut files = vec![];
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "xml") {
                files.push(path);
            }
            // Also recurse one level (ragnar puts scans in subdirs)
            if entry.path().is_dir() {
                if let Ok(sub) = fs::read_dir(entry.path()) {
                    for s in sub.flatten() {
                        if s.path().extension().map_or(false, |e| e == "xml") {
                            files.push(s.path());
                        }
                    }
                }
            }
        }
    }
    files.sort();
    files
}

fn parse_nmap_xml(path: &PathBuf) -> Vec<HostEntry> {
    let content = fs::read_to_string(path).unwrap_or_default();
    let mut hosts = vec![];

    // Simple XML parsing without a full XML library to keep deps minimal
    // Parse <host> blocks
    for host_chunk in content.split("<host ").skip(1) {
        let end = host_chunk.find("</host>").unwrap_or(host_chunk.len());
        let chunk = &host_chunk[..end];

        // IP
        let ip = extract_attr(chunk, "<address addr=\"", "\"")
            .or_else(|| extract_attr(chunk, "addr=\"", "\""))
            .unwrap_or_default();
        if ip.is_empty() { continue; }

        // Only IPv4
        if !ip.contains('.') { continue; }

        // Hostname
        let hostname = extract_attr(chunk, "name=\"", "\"").unwrap_or_default();

        // OS
        let os_hint = extract_attr(chunk, "osclass type=\"", "\"")
            .or_else(|| extract_attr(chunk, "<osmatch name=\"", "\""))
            .unwrap_or_default();

        // Ports
        let mut ports = vec![];
        for port_chunk in chunk.split("<port ").skip(1) {
            let port_end = port_chunk.find("</port>").unwrap_or(port_chunk.len());
            let pc = &port_chunk[..port_end];

            let portid: u16 = extract_attr(pc, "portid=\"", "\"")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);
            let proto = extract_attr(pc, "protocol=\"", "\"").unwrap_or_default();
            let state = extract_attr(pc, "<state state=\"", "\"").unwrap_or_default();
            if state != "open" { continue; }
            let service = extract_attr(pc, "<service name=\"", "\"").unwrap_or_default();
            let version = extract_attr(pc, "product=\"", "\"").unwrap_or_default();

            ports.push(PortEntry {
                port: portid, protocol: proto, state,
                service, version,
            });
        }
        ports.sort_by_key(|p| p.port);

        hosts.push(HostEntry { ip, hostname, os_hint, ports });
    }
    hosts
}

fn extract_attr<'a>(s: &'a str, start_pat: &str, end_pat: &str) -> Option<String> {
    let start = s.find(start_pat)? + start_pat.len();
    let end = s[start..].find(end_pat)?;
    Some(s[start..start + end].to_string())
}

// ── Draw ──────────────────────────────────────────────────────────────────────

fn draw(frame: &mut Frame, state: &NetState) {
    let area = frame.area();

    let chunks = Layout::vertical([
        Constraint::Length(1),  // tabs
        Constraint::Fill(1),    // content
        Constraint::Length(1),  // status
    ]).split(area);

    // Tabs
    let tab_titles = vec![" Hosts ", " Ports ", " Summary "];
    let tabs = Tabs::new(tab_titles)
        .select(state.tab)
        .style(Style::new().fg(Color::DarkGray))
        .highlight_style(Style::new().fg(Color::Cyan).bold())
        .divider(" │ ");
    frame.render_widget(tabs, chunks[0]);

    match state.tab {
        0 => draw_hosts(frame, state, chunks[1]),
        1 => draw_all_ports(frame, state, chunks[1]),
        _ => draw_summary(frame, state, chunks[1]),
    }

    // Status bar
    frame.render_widget(
        Paragraph::new(format!(" {}  │  j/k=scroll  Tab=views  r=reload  q=quit", state.status))
            .style(Style::new().fg(Color::DarkGray).bg(Color::Rgb(15, 15, 20))),
        chunks[2],
    );
}

fn draw_hosts(frame: &mut Frame, state: &NetState, area: ratatui::layout::Rect) {
    if state.hosts.is_empty() {
        frame.render_widget(
            Paragraph::new("\n  No scan results found.\n  Run: net-quickscan <target>  or  ragnar-scan auto quick\n  Scans are read from: /opt/cardputer/loot/recon/")
                .style(Style::new().fg(Color::DarkGray))
                .block(Block::bordered().title(" Hosts ").border_style(Style::new().fg(Color::DarkGray))),
            area,
        );
        return;
    }

    let split = Layout::horizontal([
        Constraint::Percentage(40),
        Constraint::Percentage(60),
    ]).split(area);

    // Host list
    let host_items: Vec<ListItem> = state.hosts.iter().map(|h| {
        let port_count = h.ports.len();
        let color = if port_count > 10 { Color::Red }
                    else if port_count > 3 { Color::Yellow }
                    else { Color::Cyan };
        ListItem::new(Line::from(vec![
            Span::styled(format!("{:<16}", h.ip), Style::new().fg(color)),
            Span::styled(format!("{:>3}p", port_count), Style::new().fg(Color::DarkGray)),
        ]))
    }).collect();

    let mut host_state = ListState::default().with_selected(Some(state.selected_host));
    let host_list = List::new(host_items)
        .block(Block::bordered().title(" Hosts ").border_style(Style::new().fg(Color::DarkGray)))
        .highlight_style(Style::new().fg(Color::Cyan).bold())
        .highlight_symbol("▶ ");
    frame.render_stateful_widget(host_list, split[0], &mut host_state);

    // Port detail for selected host
    if let Some(host) = state.selected_host() {
        let title = format!(" {} {} — {} ports ",
            host.ip,
            if host.hostname.is_empty() { String::new() } else { format!("({})", host.hostname) },
            host.ports.len()
        );

        let port_rows: Vec<Row> = host.ports.iter().map(|p| {
            let svc_color = match p.service.as_str() {
                "ssh"   => Color::Green,
                "http" | "https" | "http-alt" => Color::Cyan,
                "smb" | "microsoft-ds" | "netbios-ssn" => Color::Yellow,
                "ftp" | "telnet" => Color::Red,
                "mysql" | "postgresql" | "mssql" => Color::Magenta,
                _ => Color::White,
            };
            Row::new(vec![
                Cell::from(p.port.to_string()).style(Style::new().fg(Color::DarkGray)),
                Cell::from(p.protocol.clone()).style(Style::new().fg(Color::DarkGray)),
                Cell::from(p.service.clone()).style(Style::new().fg(svc_color)),
                Cell::from(p.version.chars().take(20).collect::<String>()).style(Style::new().fg(Color::White)),
            ])
        }).collect();

        let table = Table::new(port_rows, [
            Constraint::Length(6),
            Constraint::Length(4),
            Constraint::Length(14),
            Constraint::Fill(1),
        ])
        .header(Row::new(vec!["PORT", "PROTO", "SERVICE", "VERSION"])
            .style(Style::new().fg(Color::Cyan).bold()))
        .block(Block::bordered().title(title).border_style(Style::new().fg(Color::DarkGray)));

        let mut ts = TableState::default();
        frame.render_stateful_widget(table, split[1], &mut ts);
    }
}

fn draw_all_ports(frame: &mut Frame, state: &NetState, area: ratatui::layout::Rect) {
    // Flat list of all open ports across all hosts — sorted by port number
    let mut all_ports: Vec<(String, &PortEntry)> = state.hosts.iter()
        .flat_map(|h| h.ports.iter().map(move |p| (h.ip.clone(), p)))
        .collect();
    all_ports.sort_by_key(|(_, p)| p.port);

    let rows: Vec<Row> = all_ports.iter().map(|(ip, p)| {
        let svc_color = match p.service.as_str() {
            "ssh"   => Color::Green,
            "http" | "https" | "http-alt" => Color::Cyan,
            "smb" | "microsoft-ds" => Color::Yellow,
            "ftp" | "telnet" => Color::Red,
            "mysql" | "postgresql" => Color::Magenta,
            _ => Color::White,
        };
        Row::new(vec![
            Cell::from(p.port.to_string()).style(Style::new().fg(Color::DarkGray)),
            Cell::from(ip.clone()).style(Style::new().fg(Color::Cyan)),
            Cell::from(p.service.clone()).style(Style::new().fg(svc_color)),
            Cell::from(p.version.chars().take(30).collect::<String>()).style(Style::new().fg(Color::White)),
        ])
    }).collect();

    let table = Table::new(rows, [
        Constraint::Length(6),
        Constraint::Length(16),
        Constraint::Length(14),
        Constraint::Fill(1),
    ])
    .header(Row::new(vec!["PORT", "HOST", "SERVICE", "VERSION"])
        .style(Style::new().fg(Color::Cyan).bold()))
    .block(Block::bordered().title(format!(" All Open Ports ({}) ", all_ports.len()))
        .border_style(Style::new().fg(Color::DarkGray)));

    let mut ts = TableState::default().with_selected(Some(state.selected_port));
    frame.render_stateful_widget(table, area, &mut ts);
}

fn draw_summary(frame: &mut Frame, state: &NetState, area: ratatui::layout::Rect) {
    let mut lines = vec![];
    lines.push(Line::from(Span::styled("  Network Scan Summary", Style::new().fg(Color::Cyan).bold())));
    lines.push(Line::default());
    lines.push(Line::from(vec![
        Span::styled("  Hosts discovered: ", Style::new().fg(Color::DarkGray)),
        Span::styled(state.hosts.len().to_string(), Style::new().fg(Color::White)),
    ]));
    let total_ports: usize = state.hosts.iter().map(|h| h.ports.len()).sum();
    lines.push(Line::from(vec![
        Span::styled("  Total open ports: ", Style::new().fg(Color::DarkGray)),
        Span::styled(total_ports.to_string(), Style::new().fg(Color::White)),
    ]));
    lines.push(Line::default());
    lines.push(Line::from(Span::styled("  Juicy ports found:", Style::new().fg(Color::Yellow))));

    let juicy_ports = [22u16, 80, 443, 445, 3389, 3306, 5432, 8080, 8443, 6379, 27017];
    for port in juicy_ports {
        let hosts_with: Vec<&str> = state.hosts.iter()
            .filter(|h| h.ports.iter().any(|p| p.port == port))
            .map(|h| h.ip.as_str())
            .collect();
        if !hosts_with.is_empty() {
            lines.push(Line::from(vec![
                Span::styled(format!("  :{:<5}", port), Style::new().fg(Color::Red)),
                Span::styled(hosts_with.join(", "), Style::new().fg(Color::White)),
            ]));
        }
    }

    frame.render_widget(
        Paragraph::new(lines)
            .block(Block::bordered().title(" Summary ").border_style(Style::new().fg(Color::DarkGray))),
        area,
    );
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() -> Result<(), Box<dyn std::error::Error>> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = NetState::new();
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| draw(f, &state))?;

        let timeout = Duration::from_millis(200).min(
            Duration::from_millis(TICK_MS).saturating_sub(last_tick.elapsed())
        );

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                    KeyCode::Char('r') => state.reload(),
                    KeyCode::Tab => state.tab = (state.tab + 1) % 3,
                    KeyCode::Char('j') | KeyCode::Down => {
                        match state.tab {
                            0 => if state.selected_host + 1 < state.hosts.len() { state.selected_host += 1; },
                            1 => {
                                let total: usize = state.hosts.iter().map(|h| h.ports.len()).sum();
                                if state.selected_port + 1 < total { state.selected_port += 1; }
                            }
                            _ => {}
                        }
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        match state.tab {
                            0 => if state.selected_host > 0 { state.selected_host -= 1; },
                            1 => if state.selected_port > 0 { state.selected_port -= 1; },
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= Duration::from_millis(TICK_MS) {
            state.reload();
            last_tick = Instant::now();
        }
    }

    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
