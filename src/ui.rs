use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Table, Row, Cell, Clear},
    Frame,
};
use crate::app::{App, Screen};

pub fn ui(f: &mut Frame, app: &mut App) {
    match app.screen {
        Screen::Connection => draw_connection_screen(f, app),
        Screen::Main => draw_main_screen(f, app),
    }

    if let Some(ref err) = app.error_msg {
        draw_error_popup(f, err);
    }
}

fn draw_connection_screen(f: &mut Frame, app: &mut App) {
    let size = f.area();
    
    // We need 22 lines for our form.
    let form_height = 22;
    // Calculate vertical offset to center it, or 0 if terminal is too small
    let y_offset = size.height.saturating_sub(form_height) / 2;
    // Calculate horizontal offset
    let form_width = 50; 
    let x_offset = size.width.saturating_sub(form_width) / 2;
    
    let area = Rect {
        x: x_offset,
        y: y_offset,
        width: form_width.min(size.width),
        height: form_height.min(size.height),
    };

    let form_block = Block::default().borders(Borders::ALL).title(" MySQL Remote TUI ");
    f.render_widget(form_block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1) // smaller margin to fit
        .constraints(
            [
                Constraint::Length(2), // Title
                Constraint::Length(3), // Host
                Constraint::Length(3), // Port
                Constraint::Length(3), // User
                Constraint::Length(3), // Password
                Constraint::Length(3), // Database
                Constraint::Length(3), // Connect Button
                Constraint::Min(0),
            ]
            .as_ref(),
        )
        .split(area);

    let title = Paragraph::new("Nhập thông tin kết nối")
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    let active_style = Style::default().fg(Color::LightGreen);
    let inactive_style = Style::default().fg(Color::White);

    use crate::app::ConnectionField;

    // Host
    app.host_input.set_block(Block::default().borders(Borders::ALL).title(" Host "));
    app.host_input.set_style(if app.active_field == ConnectionField::Host { active_style } else { inactive_style });
    f.render_widget(&app.host_input, chunks[1]);

    // Port
    app.port_input.set_block(Block::default().borders(Borders::ALL).title(" Port "));
    app.port_input.set_style(if app.active_field == ConnectionField::Port { active_style } else { inactive_style });
    f.render_widget(&app.port_input, chunks[2]);

    // User
    app.user_input.set_block(Block::default().borders(Borders::ALL).title(" User "));
    app.user_input.set_style(if app.active_field == ConnectionField::User { active_style } else { inactive_style });
    f.render_widget(&app.user_input, chunks[3]);

    // Password
    app.password_input.set_block(Block::default().borders(Borders::ALL).title(" Password "));
    app.password_input.set_style(if app.active_field == ConnectionField::Password { active_style } else { inactive_style });
    f.render_widget(&app.password_input, chunks[4]);

    // Database
    app.db_input.set_block(Block::default().borders(Borders::ALL).title(" Database "));
    app.db_input.set_style(if app.active_field == ConnectionField::Database { active_style } else { inactive_style });
    f.render_widget(&app.db_input, chunks[5]);

    // Connect button
    let active = app.active_field == ConnectionField::ConnectButton;
    let connect_btn = Paragraph::new(if active { "[ Kết nối (Enter) ]" } else { "Kết nối (Enter)" })
        .style(if active { Style::default().fg(Color::Black).bg(Color::LightGreen) } else { Style::default().fg(Color::White) })
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).style(if active { Style::default().fg(Color::LightGreen) } else { Style::default().fg(Color::DarkGray) }));
    f.render_widget(connect_btn, chunks[6]);
}

fn draw_main_screen(f: &mut Frame, app: &mut App) {
    let size = f.area();
    
    // Split screen horizontally: Left sidebar (20%), Right content (80%)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
        .split(size);

    // -------------------------------------------------------------
    // LEFT SIDEBAR: Database Tables / Databases Explorer
    // -------------------------------------------------------------
    use ratatui::widgets::{List, ListItem, Scrollbar, ScrollbarOrientation, ScrollbarState};
    use crate::app::SidebarMode;

    let (title, items_len, emoji, help_text) = match app.sidebar_mode {
        SidebarMode::Databases => (" Databases", app.databases.len(), "🗄️", " [Alt+Enter] Chọn"),
        SidebarMode::Tables => (" Tables", app.tables.len(), "🗃️", " [Alt+Back] Trở lại"),
    };

    let tables_block = Block::default()
        .borders(Borders::ALL)
        .title(format!("{} ({}) ", title, items_len))
        .title_bottom(help_text);

    let list_data = match app.sidebar_mode {
        SidebarMode::Databases => &app.databases,
        SidebarMode::Tables => &app.tables,
    };

    let items: Vec<ListItem> = list_data.iter().map(|t| ListItem::new(format!("{} {}", emoji, t))).collect();
    let list = List::new(items)
        .block(tables_block)
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, main_chunks[0], &mut app.table_list_state);

    let mut scrollbar_state = ScrollbarState::default()
        .content_length(list_data.len())
        .position(app.table_list_state.selected().unwrap_or(0));

    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("▲"))
        .end_symbol(Some("▼"));

    let scrollbar_area = Rect {
        x: main_chunks[0].right().saturating_sub(1),
        y: main_chunks[0].y + 1,
        width: 1,
        height: main_chunks[0].height.saturating_sub(2),
    };

    f.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);

    // -------------------------------------------------------------
    // RIGHT CONTENT: Header, Editor, Results
    // -------------------------------------------------------------
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3), // Header
                Constraint::Percentage(40), // Editor
                Constraint::Percentage(60), // Results table
            ]
            .as_ref(),
        )
        .split(main_chunks[1]);

    // Header
    let header = Paragraph::new(" F5: Thực thi | ESC: Thoát | Mũi tên: Editor | Ctrl+Up/Down: Cuộn bảng | Alt+Up/Down: Cuộn Sidebar ")
        .style(Style::default().fg(Color::White).bg(Color::Blue))
        .alignment(Alignment::Center);
    f.render_widget(header, chunks[0]);

    // Editor Area
    let editor_block = Block::default().borders(Borders::ALL).title(" Trình biên tập SQL ");
    let text_spans = app.query_editor.highlight();
    let pad_cursor_y = app.query_editor.cursor_y as u16;
    let pad_cursor_x = app.query_editor.cursor_x as u16;

    let paragraph = Paragraph::new(text_spans)
        .block(editor_block);
    f.render_widget(paragraph, chunks[1]);
    
    // Render Cursor natively since we use Paragraph
    f.set_cursor_position((
        chunks[1].x + 1 + pad_cursor_x, 
        chunks[1].y + 1 + pad_cursor_y
    ));

    // Results Table Area
    if app.columns.is_empty() {
        let block = Block::default().borders(Borders::ALL).title(" Kết quả ");
        f.render_widget(block, chunks[2]);
    } else {
        let header_cells = app.columns.iter().map(|h| Cell::from(h.as_str()).style(Style::default().fg(Color::Yellow)));
        let table_header = Row::new(header_cells).style(Style::default().bg(Color::DarkGray)).height(1).bottom_margin(1);

        let rows: Vec<Row> = app.rows.iter().map(|r| {
            let mut max_lines = 1;
            let cells: Vec<Cell> = r.iter().map(|c| {
                let lines_in_c = c.lines().count();
                if lines_in_c > max_lines { max_lines = lines_in_c; }

                let char_count = c.chars().count();
                let text = if char_count > 250 && lines_in_c == 1 {
                    let mut s = c.chars().take(247).collect::<String>();
                    s.push_str("...");
                    s
                } else {
                    c.to_string()
                };
                Cell::from(text)
            }).collect();
            Row::new(cells).height(max_lines as u16)
        }).collect();

        // Calculate maximum visual widths dynamically
        let mut computed_widths = vec![0; app.columns.len()];
        
        for (i, col) in app.columns.iter().enumerate() {
            computed_widths[i] = computed_widths[i].max(col.chars().count());
        }
        for row in app.rows.iter() {
            for (i, col) in row.iter().enumerate() {
                if i < computed_widths.len() {
                    let max_line_w = col.lines().map(|l| l.chars().count()).max().unwrap_or(0);
                    computed_widths[i] = computed_widths[i].max(max_line_w);
                }
            }
        }
        
        // We use Min(width) to allow columns to naturally fit their content unless they exceed horizontal space
        let widths: Vec<Constraint> = computed_widths
            .into_iter()
            .map(|w| Constraint::Min(w.min(250) as u16 + 2))
            .collect();

        let table = Table::new(rows, widths)
            .header(table_header)
            .block(Block::default().borders(Borders::ALL).title(format!(" Kết quả ({} dòng) ", app.rows.len())))
            .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED).fg(Color::LightCyan))
            .highlight_symbol(">> ");

        f.render_stateful_widget(table, chunks[2], &mut app.table_state);
    }

    // Suggestion Popup Area (drawn last to float above everything else)
    if app.query_editor.suggestion_active && !app.query_editor.suggestions.is_empty() {
        use ratatui::widgets::{List, ListItem};
        
        let mut items = vec![];
        for (i, sug) in app.query_editor.suggestions.iter().enumerate() {
            let style = if i == app.query_editor.selected_suggestion {
                Style::default().bg(Color::DarkGray).fg(Color::Yellow)
            } else {
                Style::default().fg(Color::White)
            };
            items.push(ListItem::new(sug.as_str()).style(style));
        }

        let max_width = app.query_editor.suggestions.iter().map(|s| s.len() as u16).max().unwrap_or(10).max(10) + 2;
        let popup_height = items.len() as u16 + 2;
        
        let p_x = chunks[1].x + 1 + pad_cursor_x;
        let mut p_y = chunks[1].y + 2 + pad_cursor_y; 
        
        if p_y + popup_height > size.height {
            p_y = (chunks[1].y + pad_cursor_y).saturating_sub(popup_height).max(0);
        }
        
        let adjusted_width = max_width.min(size.width.saturating_sub(p_x));

        let popup_area = Rect {
            x: p_x,
            y: p_y,
            width: adjusted_width,
            height: popup_height,
        };

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).style(Style::default().bg(Color::Black)));

        f.render_widget(Clear, popup_area); // Clear background to prevent overlap glitches
        f.render_widget(list, popup_area);
    }
}

fn draw_error_popup(f: &mut Frame, msg: &str) {
    let area = centered_rect(60, 20, f.area());
    let paragraph = Paragraph::new(msg)
        .block(Block::default().title(" Lỗi ").borders(Borders::ALL))
        .style(Style::default().fg(Color::Red).bg(Color::Black))
        .wrap(ratatui::widgets::Wrap { trim: true })
        .alignment(Alignment::Center);

    f.render_widget(Clear, area); // clear background
    f.render_widget(paragraph, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
