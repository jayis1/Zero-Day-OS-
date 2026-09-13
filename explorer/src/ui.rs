use std::io;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Cell, Clear, List, ListItem, ListState,
        Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Table, TableState, Wrap,
    },
    Frame,
};
use crossterm::{
    event,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};
use crate::app::{App, Mode, InputKind, SortBy};
use crate::fsops;

// Colours matching ZERO-DAY OS dark theme
const C_ACCENT:    Color = Color::Cyan;
const C_DIM:       Color = Color::DarkGray;
const C_DANGER:    Color = Color::Red;
const C_WARN:      Color = Color::Yellow;
const C_OK:        Color = Color::Green;
const C_SYMLINK:   Color = Color::Magenta;
const C_SEL_BG:    Color = Color::Rgb(0, 60, 120);
const C_BAR_BG:    Color = Color::Rgb(20, 20, 30);

pub fn run(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(app, &mut terminal);

    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    result
}

fn run_app(
    app: &mut App,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        terminal.draw(|frame| draw(frame, app))?;
        if !crate::input::handle_input(app)? {
            break;
        }
    }
    Ok(())
}

// ── Main draw dispatcher ──────────────────────────────────────────────────────

fn draw(frame: &mut Frame, app: &App) {
    match app.mode {
        Mode::HexView   => draw_hex(frame, app),
        Mode::Metadata  => draw_metadata(frame, app),
        Mode::BookmarkList => draw_bookmarks(frame, app),
        _ => draw_navigate(frame, app),
    }
}

// ── File navigator ────────────────────────────────────────────────────────────

fn draw_navigate(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Layout: path bar | file list | status | help
    let chunks = Layout::vertical([
        Constraint::Length(1),  // path bar
        Constraint::Fill(1),    // file list
        Constraint::Length(1),  // status
        Constraint::Length(1),  // help / input
    ]).split(area);

    // ── Path bar ─────────────────────────────────────────────────────────────
    let cwd = app.cwd.display().to_string();
    let path_bar = Paragraph::new(format!(" {}", cwd))
        .style(Style::new().fg(C_ACCENT).bg(C_BAR_BG).bold());
    frame.render_widget(path_bar, chunks[0]);

    // ── File list ─────────────────────────────────────────────────────────────
    let list_height = chunks[1].height as usize;
    let selected = app.selected;

    let items: Vec<ListItem> = app.entries.iter().enumerate().map(|(i, e)| {
        let marked  = app.marks.contains(&i);
        let icon    = if e.is_dir { "/" } else if e.is_symlink { "@" } else { " " };
        let mark    = if marked { "*" } else { " " };
        let size_s  = if e.is_dir {
            "     ".to_string()
        } else {
            format!("{:>5}", fsops::format_size(e.size))
        };

        let name_color = if e.is_dir {
            C_ACCENT
        } else if e.is_symlink {
            C_SYMLINK
        } else if e.permissions & 0o111 != 0 {
            C_OK
        } else {
            Color::White
        };

        let line = Line::from(vec![
            Span::styled(format!("{}{}", mark, icon), Style::new().fg(C_DIM)),
            Span::styled(format!("{} ", size_s), Style::new().fg(C_DIM)),
            Span::styled(e.name.clone(), Style::new().fg(name_color)),
        ]);

        ListItem::new(line)
    }).collect();

    let mut list_state = ListState::default().with_selected(Some(selected));

    let sort_label = match app.sort_by {
        SortBy::Type     => "Type",
        SortBy::Name     => "Name",
        SortBy::Size     => "Size",
        SortBy::Modified => "Date",
    };

    let list = List::new(items)
        .block(
            Block::new()
                .title(Line::from(vec![
                    Span::styled(" zeroday-fm ", Style::new().fg(C_ACCENT).bold()),
                    Span::styled(format!("[{}]", sort_label), Style::new().fg(C_DIM)),
                ]))
                .borders(Borders::TOP | Borders::BOTTOM)
                .border_style(Style::new().fg(C_DIM))
        )
        .highlight_style(Style::new().bg(C_SEL_BG).bold())
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, chunks[1], &mut list_state);

    // Scrollbar
    if app.entries.len() > list_height {
        let mut sb_state = ScrollbarState::new(app.entries.len())
            .position(selected);
        let sb = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .style(Style::new().fg(C_DIM));
        frame.render_stateful_widget(sb, chunks[1], &mut sb_state);
    }

    // ── Status bar ────────────────────────────────────────────────────────────
    let status_style = if app.status_is_error {
        Style::new().fg(C_DANGER).bg(C_BAR_BG)
    } else {
        Style::new().fg(C_WARN).bg(C_BAR_BG)
    };
    let status_text = if app.status_message.is_empty() {
        format!(" {} items   ^Y=copy  ^X=cut  ^V=paste  ^D=del  ^R=ren  ^Z=zip  Alt+H=hex", app.entries.len())
    } else {
        format!(" {}", app.status_message)
    };
    frame.render_widget(
        Paragraph::new(status_text).style(status_style),
        chunks[2],
    );

    // ── Help / Input bar ──────────────────────────────────────────────────────
    let help_widget = match &app.mode {
        Mode::Input(InputKind::Rename) =>
            Paragraph::new(format!(" Rename → {}_", app.input_buffer))
                .style(Style::new().fg(Color::White).bg(C_BAR_BG)),
        Mode::Input(InputKind::Mkdir) =>
            Paragraph::new(format!(" New dir → {}_", app.input_buffer))
                .style(Style::new().fg(Color::White).bg(C_BAR_BG)),
        Mode::Input(InputKind::SearchQuery) =>
            Paragraph::new(format!(" Search → {}_", app.input_buffer))
                .style(Style::new().fg(C_ACCENT).bg(C_BAR_BG)),
        Mode::Input(InputKind::ZipArchive) =>
            Paragraph::new(format!(" Zip name → {}_", app.input_buffer))
                .style(Style::new().fg(C_OK).bg(C_BAR_BG)),
        Mode::ConfirmDelete =>
            Paragraph::new(" DELETE? Y=yes  N/Esc=no")
                .style(Style::new().fg(C_DANGER).bg(C_BAR_BG).bold()),
        _ =>
            Paragraph::new(" ↑↓=nav  Enter=open  BS=back  ^F=find  Tab=marks  Alt+B=bookmarks")
                .style(Style::new().fg(C_DIM).bg(C_BAR_BG)),
    };
    frame.render_widget(help_widget, chunks[3]);
}

// ── Hex viewer ────────────────────────────────────────────────────────────────

fn draw_hex(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let path = match &app.hex_view_path {
        Some(p) => p.clone(),
        None => return,
    };

    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ]).split(area);

    // Title
    let title = Paragraph::new(format!(" HEX ▶ {}  q/Esc=back  j/k/PgUp/PgDn", path.display()))
        .style(Style::new().fg(C_ACCENT).bg(C_BAR_BG).bold());
    frame.render_widget(title, chunks[0]);

    // Hex table
    if let Ok(hv) = crate::hexview::HexView::from_file(&path, 65536) {
        let rows_available = chunks[1].height as usize;
        let mut view = hv;
        view.offset = app.hex_offset;
        let lines = view.lines(rows_available);

        // Build rows: address | hex bytes | ascii
        let rows: Vec<Row> = lines.iter().map(|line| {
            // line format: "00001234  41 42 43 ...  |ABC...|"
            let parts: Vec<&str> = line.splitn(3, "  ").collect();
            let addr  = parts.get(0).copied().unwrap_or("");
            let hex   = parts.get(1).copied().unwrap_or("");
            let ascii = parts.get(2).copied().unwrap_or("");
            Row::new(vec![
                Cell::from(addr).style(Style::new().fg(C_DIM)),
                Cell::from(hex).style(Style::new().fg(Color::White)),
                Cell::from(ascii).style(Style::new().fg(C_OK)),
            ])
        }).collect();

        let widths = [
            Constraint::Length(10),
            Constraint::Fill(1),
            Constraint::Length(18),
        ];

        let table = Table::new(rows, widths)
            .block(Block::new().borders(Borders::NONE))
            .header(Row::new(vec!["Addr", "Hex", "ASCII"])
                .style(Style::new().fg(C_ACCENT).bold()));

        let mut ts = TableState::default().with_selected(None);
        frame.render_stateful_widget(table, chunks[1], &mut ts);

        // Footer
        let footer = Paragraph::new(format!(
            " offset 0x{:08x} / {} bytes   j/k=line  PgUp/PgDn=page  Home/End",
            app.hex_offset, view.data.len()
        )).style(Style::new().fg(C_DIM).bg(C_BAR_BG));
        frame.render_widget(footer, chunks[2]);
    }
}

// ── Metadata view ─────────────────────────────────────────────────────────────

fn draw_metadata(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let entry = match app.selected_entry() {
        Some(e) => e,
        None => return,
    };

    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ]).split(area);

    frame.render_widget(
        Paragraph::new(format!(" META ▶ {}  Esc=back  Alt+H=hex", entry.name))
            .style(Style::new().fg(C_ACCENT).bg(C_BAR_BG).bold()),
        chunks[0],
    );

    let meta = std::fs::symlink_metadata(&entry.path).ok();
    let link_target = if entry.is_symlink {
        std::fs::read_link(&entry.path).ok().map(|p| p.display().to_string())
    } else { None };

    let mut lines: Vec<Line> = vec![
        labeled_line("Name", &entry.name),
        labeled_line("Path", &entry.path.display().to_string()),
        labeled_line("Type", if entry.is_dir { "Directory" } else if entry.is_symlink { "Symlink" } else { "File" }),
    ];

    if let Some(ref t) = link_target {
        lines.push(labeled_line("Target", t));
    }

    lines.push(labeled_line("Size", &format!("{} ({} bytes)", fsops::format_size(entry.size), entry.size)));

    if let Some(ref m) = meta {
        lines.push(labeled_line("Perms", &format_perms(fsops::get_permissions_mode(m))));
        lines.push(labeled_line("UID:GID", &format!("{}:{}", entry.owner_uid, entry.group_gid)));
        if let Ok(mtime) = m.modified() {
            let dt: chrono::DateTime<chrono::Local> = mtime.into();
            lines.push(labeled_line("Modified", &dt.format("%Y-%m-%d %H:%M:%S").to_string()));
        }
    }

    frame.render_widget(
        Paragraph::new(Text::from(lines))
            .block(Block::new().borders(Borders::NONE))
            .wrap(Wrap { trim: false }),
        chunks[1],
    );

    frame.render_widget(
        Paragraph::new(" Esc=back  Alt+H=hex view")
            .style(Style::new().fg(C_DIM).bg(C_BAR_BG)),
        chunks[2],
    );
}

fn labeled_line<'a>(label: &'a str, value: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!("{:<10} ", label), Style::new().fg(C_ACCENT)),
        Span::styled(value.to_string(), Style::new().fg(Color::White)),
    ])
}

// ── Bookmark list ─────────────────────────────────────────────────────────────

fn draw_bookmarks(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let popup = centered_rect(50, 60, area);
    frame.render_widget(Clear, popup);

    let items: Vec<ListItem> = app.bookmarks.iter().map(|bm| {
        ListItem::new(Line::from(vec![
            Span::styled(format!("{:<12} ", bm.name), Style::new().fg(C_WARN).bold()),
            Span::styled(bm.path.display().to_string(), Style::new().fg(C_DIM)),
        ]))
    }).collect();

    let mut state = ListState::default().with_selected(Some(app.selected));

    let list = List::new(items)
        .block(
            Block::bordered()
                .title(" Bookmarks ")
                .title_style(Style::new().fg(C_ACCENT).bold())
                .border_style(Style::new().fg(C_ACCENT))
        )
        .highlight_style(Style::new().bg(C_SEL_BG).bold())
        .highlight_symbol("▶ ")
        .footer(Line::from(" Enter=go  Esc=back  j/k=select ").fg(C_DIM));

    frame.render_stateful_widget(list, popup, &mut state);
}

// ── Layout helpers ────────────────────────────────────────────────────────────

/// Centre a popup of given % width/height within `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ]).split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ]).split(popup_layout[1])[1]
}

fn format_perms(mode: u32) -> String {
    let bits = [
        (0o400,'r'),(0o200,'w'),(0o100,'x'),
        (0o040,'r'),(0o020,'w'),(0o010,'x'),
        (0o004,'r'),(0o002,'w'),(0o001,'x'),
    ];
    let mut s = String::from("-");
    for &(bit, ch) in &bits {
        s.push(if mode & bit != 0 { ch } else { '-' });
    }
    s
}
