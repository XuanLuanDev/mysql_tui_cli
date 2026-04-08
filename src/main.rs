mod app;
mod db;
mod editor;
mod ui;

use app::{App, Screen};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{error::Error, io};
use ui::ui;

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            app.error_msg = None; // clear error upon any key press
            app.info_msg = None;
            
            if key.kind == KeyEventKind::Press {
                // Global quit
                if key.code == KeyCode::Esc {
                    if app.export_menu_active {
                        app.export_menu_active = false;
                        continue;
                    }
                    if app.query_editor.suggestion_active {
                        app.query_editor.suggestion_active = false;
                        app.query_editor.suggestions.clear();
                        continue;
                    }
                    return Ok(()); // Always quit on ESC
                }
                
                if app.export_menu_active {
                    match key.code {
                        KeyCode::Up => {
                            if app.export_menu_selected > 0 { app.export_menu_selected -= 1; }
                            else { app.export_menu_selected = 2; }
                        }
                        KeyCode::Down => {
                            if app.export_menu_selected < 2 { app.export_menu_selected += 1; }
                            else { app.export_menu_selected = 0; }
                        }
                        KeyCode::Enter => {
                            app.export_menu_active = false;
                            match app.export_menu_selected {
                                0 => app.export_to_csv(),
                                1 => app.export_to_json(),
                                2 => app.export_to_excel(),
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                    continue;
                }

                match app.screen {
                    Screen::Connection => {
                        match key.code {
                            KeyCode::Enter => {
                                app.try_connect();
                            }
                            KeyCode::Tab | KeyCode::Down => {
                                app.active_field = match app.active_field {
                                    app::ConnectionField::Host => app::ConnectionField::Port,
                                    app::ConnectionField::Port => app::ConnectionField::User,
                                    app::ConnectionField::User => app::ConnectionField::Password,
                                    app::ConnectionField::Password => app::ConnectionField::Database,
                                    app::ConnectionField::Database => app::ConnectionField::ConnectButton,
                                    app::ConnectionField::ConnectButton => app::ConnectionField::Host,
                                };
                            }
                            KeyCode::BackTab | KeyCode::Up => {
                                app.active_field = match app.active_field {
                                    app::ConnectionField::Host => app::ConnectionField::ConnectButton,
                                    app::ConnectionField::Port => app::ConnectionField::Host,
                                    app::ConnectionField::User => app::ConnectionField::Port,
                                    app::ConnectionField::Password => app::ConnectionField::User,
                                    app::ConnectionField::Database => app::ConnectionField::Password,
                                    app::ConnectionField::ConnectButton => app::ConnectionField::Database,
                                };
                            }
                            _ => {
                                match app.active_field {
                                    app::ConnectionField::Host => { app.host_input.input(key); }
                                    app::ConnectionField::Port => { app.port_input.input(key); }
                                    app::ConnectionField::User => { app.user_input.input(key); }
                                    app::ConnectionField::Password => { app.password_input.input(key); }
                                    app::ConnectionField::Database => { app.db_input.input(key); }
                                    app::ConnectionField::ConnectButton => {}
                                }
                            }
                        }
                    }
                    Screen::Main => {
                        let mut handled = false;
                        if app.query_editor.suggestion_active {
                            match key.code {
                                KeyCode::Up => {
                                    if app.query_editor.selected_suggestion > 0 {
                                        app.query_editor.selected_suggestion -= 1;
                                    } else {
                                        app.query_editor.selected_suggestion = app.query_editor.suggestions.len().saturating_sub(1);
                                    }
                                    handled = true;
                                }
                                KeyCode::Down => {
                                    if app.query_editor.selected_suggestion < app.query_editor.suggestions.len().saturating_sub(1) {
                                        app.query_editor.selected_suggestion += 1;
                                    } else {
                                        app.query_editor.selected_suggestion = 0;
                                    }
                                    handled = true;
                                }
                                KeyCode::Tab => {
                                    app.query_editor.apply_suggestion();
                                    handled = true;
                                }
                                KeyCode::Esc => {
                                    app.query_editor.suggestion_active = false;
                                    app.query_editor.suggestions.clear();
                                    handled = true;
                                }
                                _ => {}
                            }
                        }
                        
                        if !handled {
                            match key.code {
                                KeyCode::F(5) => {
                                    app.execute_query();
                                    app.query_editor.suggestion_active = false;
                                }
                                KeyCode::F(6) => app.export_to_csv(),
                                KeyCode::F(7) => app.export_to_json(),
                                KeyCode::F(8) => app.export_to_excel(),
                                KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                    app.export_menu_active = true;
                                    app.export_menu_selected = 0;
                                }
                                KeyCode::Char(c) => {
                                    app.query_editor.insert_char(c);
                                }
                                KeyCode::Enter => {
                                    if key.modifiers.contains(KeyModifiers::ALT) {
                                        app.select_database_from_sidebar();
                                    } else {
                                        app.query_editor.insert_newline();
                                        app.query_editor.suggestion_active = false;
                                    }
                                }
                                KeyCode::Backspace => {
                                    if key.modifiers.contains(KeyModifiers::ALT) {
                                        app.back_to_databases();
                                    } else {
                                        app.query_editor.backspace();
                                    }
                                }
                                KeyCode::Left => {
                                    let shift = key.modifiers.contains(KeyModifiers::SHIFT);
                                    app.query_editor.move_left(shift);
                                    app.query_editor.update_suggestions();
                                }
                                KeyCode::Right => {
                                    let shift = key.modifiers.contains(KeyModifiers::SHIFT);
                                    app.query_editor.move_right(shift);
                                    app.query_editor.update_suggestions();
                                }
                                KeyCode::Up => {
                                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                                        app.scroll_table(-1);
                                    } else if key.modifiers.contains(KeyModifiers::ALT) {
                                        app.scroll_table_list(-1);
                                    } else {
                                        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
                                        app.query_editor.move_up(shift);
                                        app.query_editor.suggestion_active = false;
                                        app.query_editor.suggestions.clear();
                                    }
                                }
                                KeyCode::Down => {
                                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                                        app.scroll_table(1);
                                    } else if key.modifiers.contains(KeyModifiers::ALT) {
                                        app.scroll_table_list(1);
                                    } else {
                                        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
                                        app.query_editor.move_down(shift);
                                        app.query_editor.suggestion_active = false;
                                        app.query_editor.suggestions.clear();
                                    }
                                }
                                KeyCode::PageUp => {
                                    app.scroll_table(-10);
                                }
                                KeyCode::PageDown => {
                                    app.scroll_table(10);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }
}
