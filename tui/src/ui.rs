use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table, TableState, Wrap},
    Frame,
};
use crate::app::{App, Focus};
use crate::types::{FileStatus, Preset, CompressionBackend};

const HEADER_STYLE: Style = Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD);
const SELECTED_STYLE: Style = Style::new().bg(Color::DarkGray).fg(Color::White).add_modifier(Modifier::BOLD);
const NORMAL_STYLE: Style = Style::new().fg(Color::White);
const PROGRESS_STYLE: Style = Style::new().fg(Color::Yellow);
const COMPLETED_STYLE: Style = Style::new().fg(Color::Green);
const FAILED_STYLE: Style = Style::new().fg(Color::Red);
const HELP_STYLE: Style = Style::new().fg(Color::Gray);
const STATUS_RUNNING_STYLE: Style = Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD);
const STATUS_DONE_STYLE: Style = Style::new().fg(Color::Green).add_modifier(Modifier::BOLD);
const STATUS_READY_STYLE: Style = Style::new().fg(Color::Cyan);

pub fn draw(f: &mut Frame, app: &mut App) {
    let size = f.area();

    // Main layout: header, body, footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),   // Header
            Constraint::Min(0),      // Body
            Constraint::Length(3),   // Footer/Help bar
        ])
        .split(size);

    draw_header(f, chunks[0], app);
    draw_body(f, chunks[1], app);
    draw_footer(f, chunks[2], app);

    // Draw help overlay if shown
    if app.show_help {
        draw_help_overlay(f, size);
    }
}

fn draw_header(f: &mut Frame, area: Rect, _app: &App) {
    let title = " PDFcmp TUI Prototype ";
    let version = " v0.1.0 ";

    let header = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(title, HEADER_STYLE),
            Span::styled(version, Style::new().fg(Color::DarkGray)),
        ]),
    ])
    .block(Block::default().borders(Borders::BOTTOM).border_style(Style::new().fg(Color::DarkGray)))
    .alignment(Alignment::Center);

    f.render_widget(header, area);
}

fn draw_body(f: &mut Frame, area: Rect, app: &mut App) {
    // Split body into left (file list) and right (options/output)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60),
            Constraint::Percentage(40),
        ])
        .split(area);

    draw_file_list(f, chunks[0], app);
    draw_options_panel(f, chunks[1], app);
}

fn draw_file_list(f: &mut Frame, area: Rect, app: &mut App) {
    let is_focused = matches!(app.focus, Focus::FileList);
    let border_style = if is_focused {
        Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(" Files ")
        .title_style(Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Create table rows
    let rows: Vec<Row> = app.files.iter().enumerate().map(|(i, file)| {
        let is_selected = i == app.selected_file && is_focused;

        let (status_text, status_style) = match &file.status {
            FileStatus::Pending => ("⏳ Pending".to_string(), NORMAL_STYLE),
            FileStatus::Compressing(_) => {
                const SPINNER: [&str; 8] = ["⠁","⠂","⠄","⡀","⢀","⠠","⠐","⠈"];
                let idx = (app.progress_animation_frame as usize) % SPINNER.len();
                (format!("{} Compressing", SPINNER[idx]), PROGRESS_STYLE)
            },
            FileStatus::Completed => ("✓ Done".to_string(), COMPLETED_STYLE),
            FileStatus::Failed(e) => (format!("✗ {}", e), FAILED_STYLE),
            FileStatus::Skipped(r) => (format!("⊘ {}", r), Style::new().fg(Color::DarkGray)),
        };

        let row_style = if is_selected { SELECTED_STYLE } else { NORMAL_STYLE };

        Row::new(vec![
            Cell::from(file.file_name()).style(row_style),
            Cell::from(Text::from(file.original_size_formatted()).alignment(Alignment::Right)).style(row_style),
            Cell::from(Text::from(file.compressed_size_formatted()).alignment(Alignment::Right)).style(row_style),
            Cell::from(Text::from(file.compression_ratio().unwrap_or_else(|| "—".to_string())).alignment(Alignment::Right)).style(row_style),
            Cell::from(status_text).style(status_style),
        ])
    }).collect();

    let widths = [
        Constraint::Percentage(40),  // Name
        Constraint::Percentage(15),  // Original
        Constraint::Percentage(15),  // Compressed
        Constraint::Percentage(10),  // Ratio
        Constraint::Percentage(20),  // Status
    ];

    let table = Table::new(rows, widths)
        .header(
            Row::new(vec!["Name", "Original", "Compressed", "Ratio", "Status"])
                .style(Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        )
        .column_spacing(1);

    let mut state = TableState::default();
    state.select(Some(app.selected_file));

    f.render_stateful_widget(table, inner, &mut state);
}

fn draw_options_panel(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),   // Presets
            Constraint::Length(6),   // Backend
            Constraint::Length(6),   // Output path
            Constraint::Min(0),      // Status
        ])
        .split(area);

    draw_presets(f, chunks[0], app);
    draw_backend(f, chunks[1], app);
    draw_output_path(f, chunks[2], app);
    draw_status_area(f, chunks[3], app);
}

fn draw_presets(f: &mut Frame, area: Rect, app: &App) {
    let is_focused = matches!(app.focus, Focus::Preset);
    let border_style = if is_focused {
        Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(" Preset ")
        .title_style(Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let presets = Preset::all();
    let items: Vec<ListItem> = presets.iter().map(|p| {
        let is_selected = *p == app.preset;
        let style = if is_selected {
            Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            NORMAL_STYLE
        };
        let prefix = if is_selected { "▸ " } else { "  " };
        ListItem::new(Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(p.label(), style),
        ]))
    }).collect();

    let list = List::new(items)
        .highlight_style(SELECTED_STYLE);

    f.render_widget(list, inner);
}

fn draw_backend(f: &mut Frame, area: Rect, app: &App) {
    let is_focused = matches!(app.focus, Focus::Backend);
    let border_style = if is_focused {
        Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(" Backend ")
        .title_style(Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let backends = CompressionBackend::all();
    let items: Vec<ListItem> = backends.iter().map(|b| {
        let is_selected = *b == app.backend;
        let style = if is_selected {
            Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            NORMAL_STYLE
        };
        let prefix = if is_selected { "▸ " } else { "  " };
        ListItem::new(Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(b.label(), style),
        ]))
    }).collect();

    let list = List::new(items);
    f.render_widget(list, inner);
}

fn draw_output_path(f: &mut Frame, area: Rect, app: &App) {
    let is_focused = matches!(app.focus, Focus::OutputPath);
    let border_style = if is_focused {
        Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::new().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(" Output Path ")
        .title_style(Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let path_display = if is_focused && app.compression_running == false {
        format!("{}█", app.output_path)
    } else {
        app.output_path.clone()
    };

    let paragraph = Paragraph::new(path_display)
        .style(if is_focused && !app.compression_running {
            Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            NORMAL_STYLE
        })
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, inner);
}

fn draw_status_area(f: &mut Frame, area: Rect, app: &App) {
    let border_style = Style::new().fg(Color::DarkGray);

    let block = Block::default()
        .title(" Status ")
        .title_style(Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let status_style = if app.compression_running {
        STATUS_RUNNING_STYLE
    } else if app.overall_progress >= 100.0 {
        STATUS_DONE_STYLE
    } else {
        STATUS_READY_STYLE
    };

    let text = if app.compression_running {
        const SPINNER: [&str; 8] = ["⠁","⠂","⠄","⡀","⢀","⠠","⠐","⠈"];
        let idx = (app.progress_animation_frame as usize) % SPINNER.len();
        format!("{} {}", SPINNER[idx], app.status_message)
    } else {
        app.status_message.clone()
    };

    let msg = Paragraph::new(text)
        .style(status_style)
        .wrap(Wrap { trim: true });

    f.render_widget(msg, inner);
}

fn draw_footer(f: &mut Frame, area: Rect, app: &App) {
    let help_text = if app.show_help {
        " Press ? to close help ".to_string()
    } else if app.compression_running {
        " [o] Open files  [Tab] Switch panel  [Enter] Cancel  [?] Help  [q] Quit ".to_string()
    } else {
        " [o] Open files  [Tab] Switch panel  [Enter] Start  [?] Help  [q] Quit ".to_string()
    };

    let footer = Paragraph::new(help_text)
        .style(HELP_STYLE)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP).border_style(Style::new().fg(Color::DarkGray)));

    f.render_widget(footer, area);
}

fn draw_help_overlay(f: &mut Frame, area: Rect) {
    // Semi-transparent background
    let overlay = Block::default()
        .style(Style::new().bg(Color::Black));
    f.render_widget(overlay, area);

    // Help content centered
    let help_area = centered_rect(60, 70, area);

    let help_text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(" PDFcmp TUI - Key Bindings ", HEADER_STYLE)]),
        Line::from(""),
        Line::from(vec![Span::styled(" Navigation ", Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD))]),
        Line::from("  ↑/k, ↓/j          Navigate file list"),
        Line::from("  o                  Open file picker (add PDF files)"),
        Line::from("  Tab / Shift+Tab    Switch focus (Files → Preset → Backend → Output)"),
        Line::from(""),
        Line::from(vec![Span::styled(" File Operations ", Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD))]),
        Line::from("  o                  Open file picker (add PDF files)"),
        Line::from("  Enter              Start compression (when focused on Files)"),
        Line::from("  Enter              Cancel compression (when running)"),
        Line::from("  Space              Toggle file selection (future)"),
        Line::from(""),
        Line::from(vec![Span::styled(" Options ", Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD))]),
        Line::from("  ↑/k, ↓/j           Change preset (when focused on Preset)"),
        Line::from("  ↑/k, ↓/j           Change backend (when focused on Backend)"),
        Line::from("  a                  Toggle Auto backend (when focused on Backend)"),
        Line::from(""),
        Line::from(vec![Span::styled(" Output Path ", Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD))]),
        Line::from("  Type               Edit output path (when focused)"),
        Line::from("  Backspace          Delete character"),
        Line::from(""),
        Line::from(vec![Span::styled(" General ", Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD))]),
        Line::from("  ?                  Toggle this help"),
        Line::from("  q / Ctrl+C         Quit application"),
        Line::from(""),
        Line::from(vec![Span::styled(" Presets ", Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD))]),
        Line::from("  Screen (72 dpi)    Smallest size, for screen viewing"),
        Line::from("  Ebook (150 dpi)    Good balance for e-readers"),
        Line::from("  Printer (300 dpi)  High quality for printing"),
        Line::from("  Prepress (300 dpi) Maximum quality for professional use"),
        Line::from(""),
        Line::from(vec![Span::styled(" Backends ", Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD))]),
        Line::from("  Auto               Auto-detect best available backend"),
        Line::from("  MuPDF              Fast, lightweight"),
        Line::from("  Ghostscript        More features, better compatibility"),
        Line::from(""),
        Line::from(vec![Span::styled(" Status Indicators ", Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD))]),
        Line::from("  ⏳ Pending         Waiting to be compressed"),
        Line::from("  ⟳ XX%              Currently compressing"),
        Line::from("  ✓ Done             Completed successfully"),
        Line::from("  ✗ Error            Failed with error"),
        Line::from("  ⊘ Skipped          Skipped or cancelled"),
    ];

    let block = Block::default()
        .title(" Help ")
        .title_style(Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::new().fg(Color::Cyan))
        .style(Style::new().bg(Color::Black));

    let paragraph = Paragraph::new(help_text)
        .block(block)
        .wrap(Wrap { trim: true })
        .style(Style::new().fg(Color::White).bg(Color::Black));

    f.render_widget(paragraph, help_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}