// bt-tui — ZERO-DAY OS Bluetooth Dashboard
// Ratatui wrapper for bt-scan, bt-deep, ble-gatt, bt-attack
// Reads /opt/cardputer/loot/bt/ for historical scan data
//
// Tabs: Scan | Devices | GATT | Attacks
// Keys: Tab=switch  j/k=nav  s=scan  g=gatt  Enter=deep-scan  q=quit

use std::collections::HashMap;
use std::fs;
use std::io;
use std::process::Command;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Table, TableState, Tabs,
    },
    Terminal,
};

// ── data ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct BtDevice {
    mac: String,
    name: String,
    dev_type: BtType,
    rssi: Option<i32>,
    class: Option<String>,
    services: Vec<String>,
    vendor: String,
    first_seen: String,
    session: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum BtType {
    Classic,
    Ble,
    Unknown,
}

#[derive(Debug, Clone)]
struct GattService {
    uuid: String,
    name: String,
    handle_start: String,
    handle_end: String,
    characteristics: Vec<GattChar>,
}

#[derive(Debug, Clone)]
struct GattChar {
    uuid: String,
    handle: String,
    properties: String,
    value: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Tab {
    Scan,
    Devices,
    Gatt,
    Attacks,
}

struct App {
    tab: Tab,
    devices: Vec<BtDevice>,
    gatt_services: Vec<GattService>,
    gatt_target: Option<String>,
    scan_list_state: ListState,
    device_table_state: TableState,
    gatt_list_state: ListState,
    scan_scroll: ScrollbarState,
    device_scroll: ScrollbarState,
    gatt_scroll: ScrollbarState,
    log: Vec<String>,
    scanning: bool,
    scan_progress: u8,
    scan_duration: u32,
    status: String,
    last_scan: Option<Instant>,
}

impl App {
    fn new() -> Self {
        let mut app = Self {
            tab: Tab::Scan,
            devices: Vec::new(),
            gatt_services: Vec::new(),
            gatt_target: None,
            scan_list_state: ListState::default(),
            device_table_state: TableState::default(),
            gatt_list_state: ListState::default(),
            scan_scroll: ScrollbarState::default(),
            device_scroll: ScrollbarState::default(),
            gatt_scroll: ScrollbarState::default(),
            log: Vec::new(),
            scanning: false,
            scan_progress: 0,
            scan_duration: 15,
            status: "Press s to scan (needs root + BT adapter)".into(),
            last_scan: None,
        };
        app.load_loot();
        app
    }

    fn load_loot(&mut self) {
        let bt_dir = "/opt/cardputer/loot/bt";
        let Ok(entries) = fs::read_dir(bt_dir) else { return };
        let mut files: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                let n = e.file_name().to_string_lossy().to_string();
                n.starts_with("bt_scan_") && n.ends_with(".txt")
            })
            .collect();
        files.sort_by_key(|e| e.file_name());

        let mut seen: HashMap<String, usize> = HashMap::new();

        for entry in files.iter().rev().take(10) {
            let session = entry.file_name().to_string_lossy()
                .trim_start_matches("bt_scan_").trim_end_matches(".txt").to_string();
            let Ok(content) = fs::read_to_string(entry.path()) else { continue };
            let mut dev_type = BtType::Unknown;

            for line in content.lines() {
                let t = line.trim();
                if t.contains("classic BT") || t.contains("Classic Bluetooth") {
                    dev_type = BtType::Classic;
                } else if t.contains("BLE scan") || t.contains("lescan") {
                    dev_type = BtType::Ble;
                } else {
                    // Parse lines like: "AA:BB:CC:DD:EE:FF  Device Name"
                    let parts: Vec<&str> = t.splitn(2, "  ").collect();
                    if parts.len() == 2 {
                        let mac = parts[0].trim();
                        if is_mac(mac) {
                            let name = parts[1].trim().to_string();
                            let vendor = oui_lookup(mac);
                            if let Some(&i) = seen.get(mac) {
                                // update name if we have a better one
                                if !name.is_empty() && self.devices[i].name.is_empty() {
                                    self.devices[i].name = name;
                                }
                            } else {
                                seen.insert(mac.to_string(), self.devices.len());
                                self.devices.push(BtDevice {
                                    mac: mac.to_string(),
                                    name,
                                    dev_type,
                                    rssi: None,
                                    class: None,
                                    services: Vec::new(),
                                    vendor,
                                    first_seen: session.clone(),
                                    session: session.clone(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    fn run_scan(&mut self) {
        self.scanning = true;
        self.scan_progress = 0;
        self.log.push(format!("[*] Starting {}s BT scan...", self.scan_duration));
        self.status = format!("Scanning... ({}s)", self.scan_duration);

        let output = Command::new("sudo")
            .args(["bt-scan", &self.scan_duration.to_string()])
            .output();

        match output {
            Ok(out) => {
                let text = String::from_utf8_lossy(&out.stdout).to_string();
                let new_devices = parse_bt_scan_output(&text);
                let mut added = 0;
                for dev in new_devices {
                    if !self.devices.iter().any(|d| d.mac == dev.mac) {
                        self.devices.push(dev);
                        added += 1;
                    }
                }
                self.log.push(format!("[+] Scan complete. {} new devices (+{} total {})",
                    added, self.devices.len(), self.devices.len()));
                self.status = format!("{} devices — s=rescan  g=GATT  Enter=deep", self.devices.len());
            }
            Err(e) => {
                self.log.push(format!("[!] bt-scan failed: {}", e));
                self.status = "Scan failed — bt-scan not found or no adapter".into();
            }
        }
        self.scanning = false;
        self.last_scan = Some(Instant::now());
    }

    fn run_gatt(&mut self) {
        let mac = match self.selected_mac() {
            Some(m) => m,
            None => { self.log.push("[!] No device selected".into()); return; }
        };
        self.log.push(format!("[*] GATT enum: {}", mac));
        self.gatt_target = Some(mac.clone());
        self.status = format!("GATT scanning {}...", mac);

        let output = Command::new("sudo")
            .args(["ble-gatt", &mac])
            .output();

        match output {
            Ok(out) => {
                let text = String::from_utf8_lossy(&out.stdout).to_string();
                self.gatt_services = parse_gatt_output(&text);
                self.log.push(format!("[+] GATT: {} services found", self.gatt_services.len()));
                self.status = format!("{} GATT services — Tab to view", self.gatt_services.len());
                self.tab = Tab::Gatt;
            }
            Err(e) => {
                self.log.push(format!("[!] ble-gatt failed: {}", e));
                self.status = "GATT failed".into();
            }
        }
    }

    fn selected_mac(&self) -> Option<String> {
        match self.tab {
            Tab::Scan | Tab::Devices => {
                let i = self.scan_list_state.selected()
                    .or(self.device_table_state.selected())?;
                self.devices.get(i).map(|d| d.mac.clone())
            }
            _ => self.gatt_target.clone(),
        }
    }
}

// ── parsing ──────────────────────────────────────────────────────────────────

fn parse_bt_scan_output(text: &str) -> Vec<BtDevice> {
    let mut devices = Vec::new();
    let mut dev_type = BtType::Unknown;
    for line in text.lines() {
        let t = line.trim();
        if t.contains("classic BT") { dev_type = BtType::Classic; }
        else if t.contains("BLE") || t.contains("lescan") { dev_type = BtType::Ble; }
        let parts: Vec<&str> = t.splitn(2, "  ").collect();
        if parts.len() == 2 {
            let mac = parts[0].trim();
            if is_mac(mac) {
                let name = parts[1].trim().to_string();
                let vendor = oui_lookup(mac);
                devices.push(BtDevice {
                    mac: mac.to_string(), name,
                    dev_type, rssi: None, class: None,
                    services: Vec::new(), vendor,
                    first_seen: "now".into(),
                    session: "live".into(),
                });
            }
        }
    }
    devices
}

fn parse_gatt_output(text: &str) -> Vec<GattService> {
    let mut services = Vec::new();
    let mut cur: Option<GattService> = None;
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with("attr handle:") || t.starts_with("Primary Service") {
            if let Some(svc) = cur.take() { services.push(svc); }
            let uuid = extract_uuid(t).unwrap_or_else(|| "unknown".into());
            cur = Some(GattService {
                uuid: uuid.clone(),
                name: gatt_service_name(&uuid),
                handle_start: extract_handle(t, "handle:").unwrap_or_default(),
                handle_end: extract_handle(t, "end grp handle:").unwrap_or_default(),
                characteristics: Vec::new(),
            });
        } else if t.starts_with("handle:") || t.starts_with("characteristic") {
            let uuid = extract_uuid(t).unwrap_or_else(|| "unknown".into());
            let props = extract_field(t, "properties").unwrap_or_else(|| "0x00".into());
            if let Some(ref mut svc) = cur {
                svc.characteristics.push(GattChar {
                    uuid: uuid.clone(),
                    handle: extract_handle(t, "handle:").unwrap_or_default(),
                    properties: props,
                    value: None,
                });
            }
        }
    }
    if let Some(svc) = cur { services.push(svc); }
    services
}

fn is_mac(s: &str) -> bool {
    let parts: Vec<&str> = s.split(':').collect();
    parts.len() == 6 && parts.iter().all(|p| p.len() == 2 && p.chars().all(|c| c.is_ascii_hexdigit()))
}

fn extract_uuid(s: &str) -> Option<String> {
    let lower = s.to_lowercase();
    let idx = lower.find("uuid:")?;
    let rest = s[idx+5..].trim();
    Some(rest.split_whitespace().next()?.trim_end_matches(',').to_string())
}

fn extract_handle(s: &str, key: &str) -> Option<String> {
    let idx = s.to_lowercase().find(key)?;
    let rest = s[idx+key.len()..].trim();
    Some(rest.split_whitespace().next()?.trim_end_matches(',').to_string())
}

fn extract_field(s: &str, key: &str) -> Option<String> {
    let idx = s.to_lowercase().find(key)?;
    let rest = s[idx+key.len()..].trim().trim_start_matches(':').trim();
    Some(rest.split_whitespace().next()?.to_string())
}

fn gatt_service_name(uuid: &str) -> String {
    match uuid.to_lowercase().as_str() {
        "1800" | "00001800-0000-1000-8000-00805f9b34fb" => "Generic Access".into(),
        "1801" | "00001801-0000-1000-8000-00805f9b34fb" => "Generic Attribute".into(),
        "180a" | "0000180a-0000-1000-8000-00805f9b34fb" => "Device Information".into(),
        "180f" | "0000180f-0000-1000-8000-00805f9b34fb" => "Battery Service".into(),
        "1810" | "00001810-0000-1000-8000-00805f9b34fb" => "Blood Pressure".into(),
        "1812" | "00001812-0000-1000-8000-00805f9b34fb" => "HID (Keyboard/Mouse)".into(),
        "1816" | "00001816-0000-1000-8000-00805f9b34fb" => "Cycling Speed".into(),
        "181c" | "0000181c-0000-1000-8000-00805f9b34fb" => "User Data".into(),
        "ffe0" => "Custom (Nordic/TI UART?)".into(),
        _ => "Unknown Service".into(),
    }
}

fn oui_lookup(mac: &str) -> String {
    // Basic OUI prefix lookup for common manufacturers
    let prefix = mac[..8].to_uppercase();
    match prefix.as_str() {
        "00:17:88" => "Philips Hue".into(),
        "B8:27:EB" | "DC:A6:32" | "E4:5F:01" => "Raspberry Pi".into(),
        "00:1A:7D" | "00:1E:C2" => "Apple".into(),
        "FC:1D:43" | "54:60:09" => "Samsung".into(),
        "00:15:83" | "00:1D:43" => "Logitech".into(),
        _ => "Unknown".into(),
    }
}

fn bt_type_label(t: BtType) -> &'static str {
    match t { BtType::Classic => "BT", BtType::Ble => "BLE", BtType::Unknown => "?" }
}

fn bt_type_color(t: BtType) -> Color {
    match t { BtType::Classic => Color::Cyan, BtType::Ble => Color::Magenta, BtType::Unknown => Color::DarkGray }
}

fn truncate(s: &str, n: usize) -> String {
    if s.len() <= n { s.to_string() } else { format!("{}~", &s[..n-1]) }
}

// ── UI ───────────────────────────────────────────────────────────────────────

fn draw(f: &mut ratatui::Frame, app: &mut App) {
    let area = f.area();
    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ]).split(area);

    let tab_idx = match app.tab { Tab::Scan => 0, Tab::Devices => 1, Tab::Gatt => 2, Tab::Attacks => 3 };
    let tabs = Tabs::new(vec!["Scan", "Devices", "GATT", "Attacks"])
        .select(tab_idx)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .divider("│");
    f.render_widget(tabs, chunks[0]);

    match app.tab {
        Tab::Scan    => draw_scan(f, app, chunks[1]),
        Tab::Devices => draw_devices(f, app, chunks[1]),
        Tab::Gatt    => draw_gatt(f, app, chunks[1]),
        Tab::Attacks => draw_attacks(f, app, chunks[1]),
    }

    let status = Paragraph::new(format!(
        "{}  │  Tab=switch  s=scan  g=GATT  Enter=deep  a=attack  q=quit",
        app.status
    )).style(Style::default().fg(Color::DarkGray));
    f.render_widget(status, chunks[2]);
}

fn draw_scan(f: &mut ratatui::Frame, app: &mut App, area: Rect) {
    let layout = Layout::horizontal([Constraint::Percentage(65), Constraint::Percentage(35)]).split(area);

    // Device list
    let items: Vec<ListItem> = app.devices.iter().map(|d| {
        let name = if d.name.is_empty() { "<unknown>" } else { d.name.as_str() };
        ListItem::new(Line::from(vec![
            Span::styled(format!("{} ", bt_type_label(d.dev_type)),
                Style::default().fg(bt_type_color(d.dev_type)).add_modifier(Modifier::BOLD)),
            Span::styled(d.mac.clone(), Style::default().fg(Color::DarkGray)),
            Span::styled(format!("  {}", truncate(name, 20)), Style::default().fg(Color::White)),
            Span::styled(format!("  {}", d.vendor), Style::default().fg(Color::DarkGray)),
        ]))
    }).collect();
    let n = items.len();
    let scan_title = if app.scanning { " BT SCAN — SCANNING... " }
        else if n > 0 { " BT SCAN — s=rescan " }
        else { " BT SCAN — s=scan " };
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL)
            .title(format!("{} ({}) ", scan_title, n))
            .border_style(Style::default().fg(if app.scanning { Color::Yellow } else { Color::Cyan })))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("▶ ");
    let mut state = app.scan_list_state.clone();
    f.render_stateful_widget(list, layout[0], &mut state);
    app.scan_list_state = state;

    let mut scroll = app.scan_scroll.content_length(n);
    f.render_stateful_widget(Scrollbar::new(ScrollbarOrientation::VerticalRight), layout[0], &mut scroll);
    app.scan_scroll = scroll;

    // Detail + log panel
    let right = Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).split(layout[1]);

    // Selected device detail
    let detail_text = if let Some(i) = app.scan_list_state.selected().and_then(|i| app.devices.get(i).map(|_| i)) {
        let d = &app.devices[i];
        let name = if d.name.is_empty() { "<unknown>" } else { &d.name };
        vec![
            Line::from(vec![Span::styled("Type: ", Style::default().fg(Color::DarkGray)),
                Span::styled(bt_type_label(d.dev_type), Style::default().fg(bt_type_color(d.dev_type)))]),
            Line::from(vec![Span::styled("MAC:  ", Style::default().fg(Color::DarkGray)),
                Span::styled(d.mac.clone(), Style::default().fg(Color::Cyan))]),
            Line::from(vec![Span::styled("Name: ", Style::default().fg(Color::DarkGray)),
                Span::styled(name.to_string(), Style::default().fg(Color::White))]),
            Line::from(vec![Span::styled("OUI:  ", Style::default().fg(Color::DarkGray)),
                Span::styled(d.vendor.clone(), Style::default().fg(Color::Yellow))]),
            Line::from(""),
            Line::from(Span::styled("g=GATT  Enter=deep-scan", Style::default().fg(Color::DarkGray))),
        ]
    } else {
        vec![Line::from(Span::styled("j/k to select", Style::default().fg(Color::DarkGray)))]
    };
    let detail = Paragraph::new(detail_text)
        .block(Block::default().borders(Borders::ALL).title(" Device ")
            .border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(detail, right[0]);

    // Log
    let log_items: Vec<ListItem> = app.log.iter().rev().take(15).map(|l| {
        let col = if l.contains("[+]") { Color::Green }
            else if l.contains("[!]") { Color::Red }
            else { Color::DarkGray };
        ListItem::new(Span::styled(l.as_str(), Style::default().fg(col)))
    }).collect();
    let log_widget = List::new(log_items)
        .block(Block::default().borders(Borders::ALL).title(" Log ")
            .border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(log_widget, right[1]);
}

fn draw_devices(f: &mut ratatui::Frame, app: &mut App, area: Rect) {
    let header = Row::new(vec!["Type", "MAC", "Name", "Vendor", "Session"])
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    let rows: Vec<Row> = app.devices.iter().map(|d| {
        let name = if d.name.is_empty() { "<unknown>".to_string() } else { d.name.clone() };
        Row::new(vec![
            Cell::from(bt_type_label(d.dev_type)).style(Style::default().fg(bt_type_color(d.dev_type))),
            Cell::from(d.mac.clone()).style(Style::default().fg(Color::Cyan)),
            Cell::from(truncate(&name, 22)),
            Cell::from(truncate(&d.vendor, 16)).style(Style::default().fg(Color::Yellow)),
            Cell::from(truncate(&d.session, 14)).style(Style::default().fg(Color::DarkGray)),
        ])
    }).collect();
    let n = rows.len();
    let widths = [
        Constraint::Length(4), Constraint::Length(18),
        Constraint::Length(22), Constraint::Length(16), Constraint::Length(14),
    ];
    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL)
            .title(format!(" All Devices ({}) — historical + live scans ", n))
            .border_style(Style::default().fg(Color::DarkGray)))
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    let mut state = app.device_table_state.clone();
    f.render_stateful_widget(table, area, &mut state);
    app.device_table_state = state;
    let mut scroll = app.device_scroll.content_length(n);
    f.render_stateful_widget(Scrollbar::new(ScrollbarOrientation::VerticalRight), area, &mut scroll);
    app.device_scroll = scroll;
}

fn draw_gatt(f: &mut ratatui::Frame, app: &mut App, area: Rect) {
    let target = app.gatt_target.as_deref().unwrap_or("none");
    if app.gatt_services.is_empty() {
        let p = Paragraph::new(format!(
            "No GATT data for {}\n\nSelect a BLE device in Scan tab and press g",
            target
        )).style(Style::default().fg(Color::DarkGray))
          .block(Block::default().borders(Borders::ALL).title(" GATT Services "));
        f.render_widget(p, area);
        return;
    }

    let items: Vec<ListItem> = app.gatt_services.iter().flat_map(|svc| {
        let mut lines = vec![
            ListItem::new(Line::from(vec![
                Span::styled("SVC  ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(svc.uuid.clone(), Style::default().fg(Color::White)),
                Span::styled(format!("  {}", svc.name), Style::default().fg(Color::Yellow)),
            ]))
        ];
        for ch in &svc.characteristics {
            let val_str = ch.value.as_deref().map(|v| format!(" = {}", v)).unwrap_or_default();
            lines.push(ListItem::new(Line::from(vec![
                Span::styled("  CH ", Style::default().fg(Color::DarkGray)),
                Span::styled(ch.uuid.clone(), Style::default().fg(Color::DarkGray)),
                Span::styled(format!(" [{}]", ch.properties), Style::default().fg(Color::Magenta)),
                Span::styled(val_str, Style::default().fg(Color::Green)),
            ])));
        }
        lines
    }).collect();

    let n = items.len();
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL)
            .title(format!(" GATT Services — {} ({} services) ", target, app.gatt_services.len()))
            .border_style(Style::default().fg(Color::Magenta)))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("▶ ");
    let mut state = app.gatt_list_state.clone();
    f.render_stateful_widget(list, area, &mut state);
    app.gatt_list_state = state;
    let mut scroll = app.gatt_scroll.content_length(n);
    f.render_stateful_widget(Scrollbar::new(ScrollbarOrientation::VerticalRight), area, &mut scroll);
    app.gatt_scroll = scroll;
}

fn draw_attacks(f: &mut ratatui::Frame, app: &mut App, area: Rect) {
    let layout = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(area);

    // Target info
    let target_text = match app.scan_list_state.selected().and_then(|i| app.devices.get(i)) {
        Some(d) => {
            let name = if d.name.is_empty() { "<unknown>".to_string() } else { d.name.clone() };
            vec![
                Line::from(vec![Span::styled("Target: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(d.mac.clone(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]),
                Line::from(vec![Span::styled("Name:   ", Style::default().fg(Color::DarkGray)),
                    Span::styled(name, Style::default().fg(Color::White))]),
                Line::from(vec![Span::styled("Type:   ", Style::default().fg(Color::DarkGray)),
                    Span::styled(bt_type_label(d.dev_type), Style::default().fg(bt_type_color(d.dev_type)))]),
                Line::from(vec![Span::styled("Vendor: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(d.vendor.clone(), Style::default().fg(Color::Yellow))]),
            ]
        }
        None => vec![Line::from(Span::styled(
            "Go to Scan tab, j/k to select a device",
            Style::default().fg(Color::DarkGray)
        ))]
    };
    let target = Paragraph::new(target_text)
        .block(Block::default().borders(Borders::ALL).title(" Target ")
            .border_style(Style::default().fg(Color::Cyan)));
    f.render_widget(target, layout[0]);

    // Attack menu
    let attacks = vec![
        ListItem::new(Line::from(vec![
            Span::styled("g ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::styled("GATT Enumerate (ble-gatt)", Style::default().fg(Color::White)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("d ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("Deep Scan (bt-deep)", Style::default().fg(Color::White)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("a ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled("BT Attack (bt-attack)", Style::default().fg(Color::White)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("S ", Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD)),
            Span::styled("BLE SPAM (ble-spam)", Style::default().fg(Color::White)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("b ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("Bettercap BT scan", Style::default().fg(Color::White)),
        ])),
    ];
    let attack_list = List::new(attacks)
        .block(Block::default().borders(Borders::ALL).title(" Attack Menu ")
            .border_style(Style::default().fg(Color::Red)));
    f.render_widget(attack_list, layout[1]);
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
                            Tab::Scan    => Tab::Devices,
                            Tab::Devices => Tab::Gatt,
                            Tab::Gatt    => Tab::Attacks,
                            Tab::Attacks => Tab::Scan,
                        };
                    }
                    KeyCode::Char('s') => { app.run_scan(); }
                    KeyCode::Char('g') => { app.run_gatt(); }
                    KeyCode::Char('j') | KeyCode::Down => {
                        let n = app.devices.len();
                        match app.tab {
                            Tab::Scan | Tab::Attacks => {
                                let i = app.scan_list_state.selected().map(|i| (i+1).min(n.saturating_sub(1))).unwrap_or(0);
                                app.scan_list_state.select(Some(i));
                            }
                            Tab::Devices => {
                                let i = app.device_table_state.selected().map(|i| (i+1).min(n.saturating_sub(1))).unwrap_or(0);
                                app.device_table_state.select(Some(i));
                            }
                            Tab::Gatt => {
                                let n2 = app.gatt_services.len();
                                let i = app.gatt_list_state.selected().map(|i| (i+1).min(n2.saturating_sub(1))).unwrap_or(0);
                                app.gatt_list_state.select(Some(i));
                            }
                        }
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        match app.tab {
                            Tab::Scan | Tab::Attacks => {
                                let i = app.scan_list_state.selected().unwrap_or(1).saturating_sub(1);
                                app.scan_list_state.select(Some(i));
                            }
                            Tab::Devices => {
                                let i = app.device_table_state.selected().unwrap_or(1).saturating_sub(1);
                                app.device_table_state.select(Some(i));
                            }
                            Tab::Gatt => {
                                let i = app.gatt_list_state.selected().unwrap_or(1).saturating_sub(1);
                                app.gatt_list_state.select(Some(i));
                            }
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
        eprintln!("bt-tui error: {}", e);
        std::process::exit(1);
    }
}
