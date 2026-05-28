use std::sync::atomic::Ordering;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{
    Block, BorderType, Borders, Gauge, List, ListItem, Paragraph, Tabs, Wrap,
};
use ratatui::{Frame, Terminal};

use crate::state::DynState;

#[derive(Clone, Debug)]
pub enum PartStatus {
    Idle,
    Downloading,
    Finished,
    Error,
}

#[derive(Clone, Debug)]
pub enum UiEvent {
    Log(usize, String),
    PartUpdate(usize, u32, PartStatus, f64),
    Progress(usize, u32, u32),
    FileStart(usize, String, u32),
    FileComplete(usize),
    AllDone,
}

pub struct TuiState {
    pub tabs: Vec<TabState>,
    pub current_tab: usize,
    pub total_files: usize,
    pub current_file: usize,
    pub dyn_state: DynState,
    pub total_proxies: usize,
    pub paused: bool,
    pub all_done: bool,
}

pub struct TabState {
    pub url: String,
    pub log: Vec<String>,
    pub part_statuses: Vec<PartStatus>,
    pub total_parts: u32,
    pub done_parts: u32,
    pub proxy_items: Vec<(String, f64, u64)>,
}

impl TabState {
    pub fn new(url: String) -> Self {
        Self {
            url,
            log: Vec::new(),
            part_statuses: Vec::new(),
            total_parts: 0,
            done_parts: 0,
            proxy_items: Vec::new(),
        }
    }

    fn part_grid_string(&self) -> String {
        let total = self.part_statuses.len();
        if total == 0 {
            return String::new();
        }
        let cols = 50.min(total.max(1));
        let mut out = String::new();
        for (i, status) in self.part_statuses.iter().enumerate() {
            match status {
                PartStatus::Finished => out.push('█'),
                PartStatus::Downloading => out.push('▓'),
                PartStatus::Error => out.push('▒'),
                PartStatus::Idle => out.push('░'),
            }
            if (i + 1) % cols == 0 && i + 1 < total {
                out.push('\n');
            }
        }
        out
    }
}

const BG: Color = Color::Rgb(43, 43, 43);
const SURFACE: Color = Color::Rgb(53, 53, 53);
const PANEL: Color = Color::Rgb(60, 60, 60);
const ACCENT: Color = Color::Rgb(61, 174, 233);
const BORDER: Color = Color::Rgb(74, 74, 74);
const TEXT: Color = Color::Rgb(240, 240, 240);
const TEXT_DIM: Color = Color::Rgb(170, 170, 170);
const HEADER_BG: Color = Color::Rgb(26, 26, 26);
const DARK_BG: Color = Color::Rgb(30, 30, 30);

fn header_block() -> Paragraph<'static> {
    Paragraph::new(Text::from(Line::from(vec![
        Span::styled(" \u{25cf} ", Style::default().fg(ACCENT)),
        Span::styled("Downloader", Style::default().fg(TEXT)),
        Span::raw(" "),
        Span::styled(
            "\u{2500}".repeat(80),
            Style::default().fg(BORDER),
        ),
        Span::raw("  "),
        Span::styled("\u{2500}", Style::default().fg(BORDER)),
        Span::styled(" \u{25a0} ", Style::default().fg(TEXT_DIM)),
        Span::styled("\u{2715}", Style::default().fg(Color::Rgb(200, 60, 60))),
    ])))
    .style(Style::default().bg(HEADER_BG))
    .alignment(Alignment::Left)
}

fn toolbar(state: &TuiState) -> Paragraph<'static> {
    let workers = state.dyn_state.max_connections.load(Ordering::Acquire);
    let timeout = state.dyn_state.timeout.load(Ordering::Acquire);

    let pause_label = if state.paused {
        " [P] Resume "
    } else {
        " [P] Pause "
    };
    let pause_style = if state.paused {
        Style::default().fg(Color::Rgb(255, 80, 80)).bold()
    } else {
        Style::default().fg(Color::Rgb(80, 220, 100)).bold()
    };

    Paragraph::new(Text::from(Line::from(vec![
        Span::styled(pause_label, pause_style),
        Span::styled(" \u{2502} ", Style::default().fg(BORDER)),
        Span::styled(" Workers: ", Style::default().fg(TEXT_DIM)),
        Span::styled("[-]", Style::default().fg(ACCENT)),
        Span::styled(format!(" {} ", workers), Style::default().fg(TEXT).bold()),
        Span::styled("[+]", Style::default().fg(ACCENT)),
        Span::styled(" \u{2502} ", Style::default().fg(BORDER)),
        Span::styled(" Timeout: ", Style::default().fg(TEXT_DIM)),
        Span::styled("[-]", Style::default().fg(ACCENT)),
        Span::styled(format!(" {}s ", timeout), Style::default().fg(TEXT).bold()),
        Span::styled("[+]", Style::default().fg(ACCENT)),
    ])))
    .style(Style::default().bg(SURFACE))
    .alignment(Alignment::Left)
}

fn status_bar(state: &TuiState) -> Paragraph<'static> {
    let workers = state.dyn_state.max_connections.load(Ordering::Acquire);
    let timeout = state.dyn_state.timeout.load(Ordering::Acquire);
    let active_tab = state.current_tab.min(state.tabs.len().saturating_sub(1));
    let file_name = if active_tab < state.tabs.len() {
        state.tabs[active_tab]
            .url
            .split('/')
            .last()
            .unwrap_or("")
            .to_string()
    } else {
        String::new()
    };
    let file_info = if file_name.len() > 28 {
        format!("{}...", &file_name[..25])
    } else {
        file_name
    };
    let status_span: Span = if state.all_done {
        Span::styled(" Done ", Style::default().fg(TEXT_DIM))
    } else if state.paused {
        Span::styled(" Paused ", Style::default().fg(TEXT_DIM))
    } else {
        Span::styled(
            format!(" {}/{} ", state.current_file, state.total_files),
            Style::default().fg(TEXT_DIM),
        )
    };

    Paragraph::new(Text::from(Line::from(vec![
        Span::styled(format!(" {} ", file_info), Style::default().fg(TEXT_DIM)),
        Span::styled(" \u{2502} ", Style::default().fg(BORDER)),
        Span::styled(" Workers ", Style::default().fg(TEXT_DIM)),
        Span::styled(format!("{}", workers), Style::default().fg(ACCENT)),
        Span::styled(" \u{2502} ", Style::default().fg(BORDER)),
        Span::styled(" Proxies ", Style::default().fg(TEXT_DIM)),
        Span::styled(
            format!("{}", state.total_proxies),
            Style::default().fg(ACCENT),
        ),
        Span::styled(" \u{2502} ", Style::default().fg(BORDER)),
        Span::styled(" Timeout ", Style::default().fg(TEXT_DIM)),
        Span::styled(format!("{}s", timeout), Style::default().fg(ACCENT)),
        Span::styled(" \u{2502} ", Style::default().fg(BORDER)),
        status_span,
    ])))
    .style(Style::default().bg(HEADER_BG))
    .alignment(Alignment::Left)
}

fn log_panel(tab: &TabState, area: Rect, f: &mut Frame) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(BORDER))
        .style(Style::default().bg(PANEL))
        .title(Line::from(Span::styled(
            " Log ",
            Style::default().fg(ACCENT).bold(),
        )))
        .title_alignment(Alignment::Left);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let max_lines = inner.height as usize;
    let start = tab.log.len().saturating_sub(max_lines);
    let visible: Vec<&str> = tab.log[start..].iter().map(|s| s.as_str()).collect();

    let text = Text::from(visible.join("\n"));
    let para = Paragraph::new(text)
        .style(Style::default().bg(DARK_BG).fg(TEXT_DIM))
        .wrap(Wrap { trim: false });
    f.render_widget(para, inner);
}

fn proxy_panel(tab: &TabState, area: Rect, f: &mut Frame) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(BORDER))
        .style(Style::default().bg(PANEL))
        .title(Line::from(Span::styled(
            " Proxies ",
            Style::default().fg(ACCENT).bold(),
        )))
        .title_alignment(Alignment::Left);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let items: Vec<ListItem> = tab
        .proxy_items
        .iter()
        .map(|(addr, avg, count)| {
            let content = Line::from(vec![
                Span::styled(
                    format!(" {:<24}", addr),
                    Style::default().fg(TEXT_DIM),
                ),
                Span::styled(
                    format!(" {:.1}s ", avg),
                    Style::default().fg(ACCENT),
                ),
                Span::styled(
                    format!("#{}", count),
                    Style::default().fg(Color::Rgb(180, 180, 100)),
                ),
            ]);
            ListItem::new(content)
        })
        .collect();

    let list = List::new(items).style(Style::default().bg(DARK_BG));
    f.render_widget(list, inner);
}

fn parts_panel(tab: &TabState, area: Rect, f: &mut Frame) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(BORDER))
        .style(Style::default().bg(PANEL))
        .title(Line::from(Span::styled(
            " Parts ",
            Style::default().fg(ACCENT).bold(),
        )))
        .title_alignment(Alignment::Left);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let parts = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner);

    // Grid
    let grid_str = tab.part_grid_string();
    let filtered: Vec<Span> = grid_str
        .chars()
        .map(|c| match c {
            '█' => Span::styled("█", Style::default().fg(Color::Rgb(80, 220, 100)).bold()),
            '▓' => Span::styled("▓", Style::default().fg(Color::Rgb(0, 200, 255)).bold()),
            '▒' => Span::styled("▒", Style::default().fg(Color::Rgb(255, 80, 80)).bold()),
            '░' => Span::styled("░", Style::default().fg(Color::Rgb(60, 60, 60))),
            '\n' => Span::raw("\n"),
            _ => Span::raw(c.to_string()),
        })
        .collect();

    let text = Text::from(Line::from(filtered));
    let grid_para = Paragraph::new(text)
        .style(Style::default().bg(PANEL))
        .wrap(Wrap { trim: false });
    f.render_widget(grid_para, parts[0]);

    // Progress bar
    let pct = if tab.total_parts > 0 {
        (tab.done_parts as f64 / tab.total_parts as f64 * 100.0) as u16
    } else {
        0
    };
    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::NONE)
                .style(Style::default().bg(BG)),
        )
        .gauge_style(
            Style::default()
                .fg(ACCENT)
                .bg(SURFACE)
                .add_modifier(Modifier::BOLD),
        )
        .percent(pct)
        .label(format!(" {} / {}  ({}%) ", tab.done_parts, tab.total_parts, pct));
    f.render_widget(gauge, parts[1]);
}

pub fn ui(f: &mut Frame, state: &TuiState, tab_names: &[String], active_tab: usize) {
    let area = f.area();

    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),   // header
            Constraint::Length(1),   // toolbar
            Constraint::Min(1),      // main content
            Constraint::Length(1),   // status bar
        ])
        .split(area);

    // Header
    f.render_widget(header_block(), main_layout[0]);

    // Toolbar
    f.render_widget(toolbar(state), main_layout[1]);

    // Status bar
    f.render_widget(status_bar(state), main_layout[3]);

    // Main content
    let main = main_layout[2];

    let content_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(main);

    // Tabs
    let tabs_widget = Tabs::new(tab_names.to_vec())
        .select(active_tab)
        .divider(Span::styled(" \u{2502} ", Style::default().fg(BORDER)))
        .style(Style::default().bg(SURFACE).fg(TEXT_DIM))
        .highlight_style(Style::default().fg(ACCENT).bold());
    f.render_widget(tabs_widget, content_layout[0]);

    // Active tab content
    if active_tab < state.tabs.len() {
        let tab = &state.tabs[active_tab];
        let content = content_layout[1];

        let h_parts = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(67), Constraint::Percentage(33)])
            .split(content);

        // Left column: Log on top, Parts below
        let left = h_parts[0];
        let left_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(left);

        log_panel(tab, left_layout[0], f);
        parts_panel(tab, left_layout[1], f);
        proxy_panel(tab, h_parts[1], f);
    }
}

pub fn run_tui(
    mut state: TuiState,
    event_rx: crossbeam_channel::Receiver<UiEvent>,
) -> std::io::Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    let stdout = std::io::stdout();
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut all_done_time: Option<std::time::Instant> = None;

    let tab_names: Vec<String> = state
        .tabs
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let fallback = format!("URL {}", i);
            let n = t.url.split('/').last().unwrap_or(&fallback);
            if n.len() > 20 {
                format!("{}...", &n[..17])
            } else {
                n.to_string()
            }
        })
        .collect();

    let result: std::io::Result<()> = loop {
        // Drain events
        while let Ok(event) = event_rx.try_recv() {
            match event {
                UiEvent::Log(tab_idx, msg) => {
                    if tab_idx < state.tabs.len() {
                        state.tabs[tab_idx].log.push(msg);
                    }
                }
                UiEvent::PartUpdate(tab_idx, part_num, status, _elapsed) => {
                    if tab_idx < state.tabs.len() {
                        let tab = &mut state.tabs[tab_idx];
                        if (part_num as usize) < tab.part_statuses.len() {
                            tab.part_statuses[part_num as usize] = status;
                        }
                    }
                }
                UiEvent::Progress(tab_idx, done, total) => {
                    if tab_idx < state.tabs.len() {
                        let tab = &mut state.tabs[tab_idx];
                        tab.done_parts = done;
                        tab.total_parts = total;
                    }
                }
                UiEvent::FileStart(tab_idx, _url, total_parts) => {
                    state.current_tab = tab_idx;
                    state.current_file = tab_idx + 1;
                    if tab_idx < state.tabs.len() {
                        let tab = &mut state.tabs[tab_idx];
                        tab.total_parts = total_parts;
                        tab.part_statuses =
                            vec![PartStatus::Idle; total_parts as usize];
                    }
                }
                UiEvent::FileComplete(_tab_idx) => {}
                UiEvent::AllDone => {
                    state.all_done = true;
                    all_done_time = Some(std::time::Instant::now());
                }
            }
        }

        // Render
        terminal.draw(|f| {
            ui(f, &state, &tab_names, state.current_tab);
        })?;

        // Keyboard
        if crossterm::event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            break Ok(());
                        }
                        KeyCode::Char('p') | KeyCode::Char('P') => {
                            let paused = state.dyn_state.toggle_pause();
                            state.paused = paused;
                        }
                        KeyCode::Char('+') | KeyCode::Char('=') => {
                            state
                                .dyn_state
                                .max_connections
                                .fetch_add(1, Ordering::Release);
                        }
                        KeyCode::Char('-') | KeyCode::Char('_') => {
                            state
                                .dyn_state
                                .max_connections
                                .fetch_update(
                                    Ordering::AcqRel,
                                    Ordering::Acquire,
                                    |x| Some(x.saturating_sub(1).max(1)),
                                )
                                .ok();
                        }
                        KeyCode::Char(']') => {
                            state
                                .dyn_state
                                .timeout
                                .fetch_add(10, Ordering::Release);
                        }
                        KeyCode::Char('[') => {
                            state
                                .dyn_state
                                .timeout
                                .fetch_update(
                                    Ordering::AcqRel,
                                    Ordering::Acquire,
                                    |x| Some(x.saturating_sub(10).max(10)),
                                )
                                .ok();
                        }
                        KeyCode::Left => {
                            if state.current_tab > 0 {
                                state.current_tab -= 1;
                            }
                        }
                        KeyCode::Right => {
                            if state.current_tab + 1 < state.tabs.len() {
                                state.current_tab += 1;
                            }
                        }
                        _ => {}
                    }
                }
                Event::Resize(_, _) => {
                    terminal.clear()?;
                }
                _ => {}
            }
        }

        if let Some(t) = all_done_time {
            if t.elapsed() > Duration::from_secs(2) {
                break Ok(());
            }
        }
    };

    // Cleanup
    crossterm::terminal::disable_raw_mode()?;
    terminal.clear()?;
    terminal.show_cursor()?;

    // Print final state to stdout
    for (i, tab) in state.tabs.iter().enumerate() {
        let fallback = format!("tab {}", i);
        let name = tab.url.split('/').last().unwrap_or(&fallback);
        if tab.log.is_empty() {
            continue;
        }
        println!("\n--- {} ---", name);
        for msg in &tab.log {
            println!("{}", msg);
        }
    }

    result
}
