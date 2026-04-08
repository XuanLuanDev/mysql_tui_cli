use crate::db::{DatabaseExecutor, MysqlExecutor};
use crate::editor::SqlEditor;
use ratatui::widgets::{TableState, ListState};
use tui_textarea::TextArea;

pub enum Screen {
    Connection,
    Main,
}

#[derive(PartialEq)]
pub enum SidebarMode {
    Databases,
    Tables,
}

#[derive(PartialEq)]
pub enum ConnectionField {
    Host,
    Port,
    User,
    Password,
    Database,
    ConnectButton,
}

pub struct App<'a> {
    pub screen: Screen,
    pub host_input: TextArea<'a>,
    pub port_input: TextArea<'a>,
    pub user_input: TextArea<'a>,
    pub password_input: TextArea<'a>,
    pub db_input: TextArea<'a>,
    pub active_field: ConnectionField,
    pub query_editor: SqlEditor,
    pub executor: Box<dyn DatabaseExecutor>,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub error_msg: Option<String>,
    pub should_quit: bool,
    pub vertical_scroll: usize,
    pub horizontal_scroll: usize,
    pub table_state: TableState,
    pub table_list_state: ListState,
    pub tables: Vec<String>,
    pub sidebar_mode: SidebarMode,
    pub databases: Vec<String>,
    pub info_msg: Option<String>,
    pub export_menu_active: bool,
    pub export_menu_selected: usize,
}

impl<'a> App<'a> {
    pub fn new() -> App<'a> {
        let mut host_input = TextArea::default();
        host_input.insert_str("localhost");

        let mut port_input = TextArea::default();
        port_input.insert_str("3306");

        let mut user_input = TextArea::default();
        user_input.insert_str("root");

        let mut password_input = TextArea::default();
        password_input.set_mask_char('*'); // visually mask

        let mut db_input = TextArea::default();
        db_input.insert_str("mysql");
        
        App {
            screen: Screen::Connection,
            host_input,
            port_input,
            user_input,
            password_input,
            db_input,
            active_field: ConnectionField::Host,
            query_editor: SqlEditor::new(),
            executor: Box::new(MysqlExecutor::new()),
            columns: vec![],
            rows: vec![],
            error_msg: None,
            should_quit: false,
            vertical_scroll: 0,
            horizontal_scroll: 0,
            table_state: TableState::default(),
            table_list_state: ListState::default(),
            tables: vec![],
            sidebar_mode: SidebarMode::Tables,
            databases: vec![],
            info_msg: None,
            export_menu_active: false,
            export_menu_selected: 0,
        }
    }

    pub fn try_connect(&mut self) {
        let host = self.host_input.lines().join("");
        let port = self.port_input.lines().join("");
        let user = self.user_input.lines().join("");
        let password = self.password_input.lines().join("");
        let db = self.db_input.lines().join("");
        
        let pass_part = if password.is_empty() {
            String::new()
        } else {
            format!(":{}", password)
        };

        let db_part = if db.is_empty() {
            String::new()
        } else {
            format!("/{}", db)
        };

        // Correcting formatting string placeholder format
        let conn_str = format!("mysql://{}{}[at]{}:{}{}", user, pass_part, host, port, db_part).replace("[at]", "@");
        match self.executor.connect(&conn_str) {
            Ok(_) => {
                self.error_msg = None;
                self.screen = Screen::Main;
                
                if db_part.is_empty() {
                    self.sidebar_mode = SidebarMode::Databases;
                    if let Ok(fetched_dbs) = self.executor.get_databases() {
                        self.databases = fetched_dbs;
                    }
                } else {
                    self.sidebar_mode = SidebarMode::Tables;
                    if let Ok(fetched_tables) = self.executor.get_tables() {
                        self.tables = fetched_tables.clone();
                        self.query_editor.dynamic_keywords = fetched_tables;
                    }
                }
            }
            Err(e) => {
                self.error_msg = Some(format!("Connection error: {}", e));
            }
        }
    }

    pub fn execute_query(&mut self) {
        let text = self.query_editor.get_executable_text();
        let queries: Vec<&str> = text.split(';').map(|q| q.trim()).filter(|q| !q.is_empty()).collect();
        
        if queries.is_empty() {
            return;
        }

        self.error_msg = None;

        for (i, q) in queries.iter().enumerate() {
            let mut query = q.to_string();
            let upper_query = query.to_uppercase();
            if upper_query.starts_with("SELECT") && !upper_query.contains("LIMIT") {
                query = format!("{} LIMIT 1000", query);
            }

            match self.executor.execute(&query) {
                Ok((cols, rows)) => {
                    // We keep results from the last query that returned columns, or the last query executed.
                    // To avoid erasing a previous SELECT result with a subsequent UPDATE, we can just update if it's the last one OR check if cols are not empty.
                    // For simplicity, just update UI with the latest result
                    self.columns = cols;
                    self.rows = rows;
                    self.vertical_scroll = 0;
                    self.horizontal_scroll = 0;
                    self.table_state.select(if self.rows.is_empty() { None } else { Some(0) });
                }
                Err(e) => {
                    self.error_msg = Some(if queries.len() > 1 {
                        format!("Error in query {}: {}", i + 1, e)
                    } else {
                        e
                    });
                    break; // Stop executing on first error
                }
            }
        }
    }

    pub fn scroll_table(&mut self, offset: isize) {
        if self.rows.is_empty() { return; }
        let current = self.table_state.selected().unwrap_or(0);
        let max_idx = self.rows.len().saturating_sub(1) as isize;
        let new_idx = (current as isize + offset).clamp(0, max_idx) as usize;
        self.table_state.select(Some(new_idx));
    }

    pub fn scroll_table_list(&mut self, offset: isize) {
        let max_len = if self.sidebar_mode == SidebarMode::Databases { self.databases.len() } else { self.tables.len() };
        if max_len == 0 { return; }
        
        let current = self.table_list_state.selected().unwrap_or(0);
        let max_idx = max_len.saturating_sub(1) as isize;
        let new_idx = (current as isize + offset).clamp(0, max_idx) as usize;
        self.table_list_state.select(Some(new_idx));
    }

    pub fn select_database_from_sidebar(&mut self) {
        if self.sidebar_mode == SidebarMode::Databases && !self.databases.is_empty() {
            let idx = self.table_list_state.selected().unwrap_or(0);
            if idx < self.databases.len() {
                let db_name = &self.databases[idx];
                let query = format!("USE {};", db_name);
                match self.executor.execute(&query) {
                    Ok(_) => {
                        self.sidebar_mode = SidebarMode::Tables;
                        self.error_msg = None;
                        if let Ok(fetched_tables) = self.executor.get_tables() {
                            self.tables = fetched_tables.clone();
                            self.query_editor.dynamic_keywords = fetched_tables;
                            self.table_list_state.select(Some(0)); 
                        } else {
                            self.tables.clear();
                            self.query_editor.dynamic_keywords.clear();
                        }
                    }
                    Err(e) => {
                        self.error_msg = Some(e);
                    }
                }
            }
        }
    }

    pub fn back_to_databases(&mut self) {
        if self.sidebar_mode == SidebarMode::Tables {
            self.sidebar_mode = SidebarMode::Databases;
            if let Ok(fetched_dbs) = self.executor.get_databases() {
                self.databases = fetched_dbs;
                self.table_list_state.select(Some(0));
            }
        }
    }

    pub fn export_to_csv(&mut self) {
        if self.columns.is_empty() {
            self.error_msg = Some("No data to export.".to_string());
            return;
        }

        if let Some(path) = rfd::FileDialog::new()
            .add_filter("CSV", &["csv"])
            .set_file_name("export.csv")
            .save_file() {
            
            match csv::Writer::from_path(&path) {
                Ok(mut wtr) => {
                    let _ = wtr.write_record(&self.columns);
                    for row in &self.rows {
                        let _ = wtr.write_record(row);
                    }
                    if let Err(e) = wtr.flush() {
                        self.error_msg = Some(format!("Failed to write CSV: {}", e));
                    } else {
                        self.info_msg = Some(format!("Successfully exported to {:?}", path));
                    }
                }
                Err(e) => {
                    self.error_msg = Some(format!("Failed to create CSV: {}", e));
                }
            }
        }
    }

    pub fn export_to_json(&mut self) {
        if self.columns.is_empty() {
            self.error_msg = Some("No data to export.".to_string());
            return;
        }

        if let Some(path) = rfd::FileDialog::new()
            .add_filter("JSON", &["json"])
            .set_file_name("export.json")
            .save_file() {
            
            let mut data = Vec::new();
            for row in &self.rows {
                let mut map = std::collections::HashMap::new();
                for (i, col_name) in self.columns.iter().enumerate() {
                    let val = row.get(i).cloned().unwrap_or_default();
                    map.insert(col_name.clone(), val);
                }
                data.push(map);
            }

            match std::fs::write(&path, serde_json::to_string_pretty(&data).unwrap_or_default()) {
                Ok(_) => self.info_msg = Some(format!("Successfully exported to {:?}", path)),
                Err(e) => self.error_msg = Some(format!("Failed to write JSON: {}", e)),
            }
        }
    }

    pub fn export_to_excel(&mut self) {
        if self.columns.is_empty() {
            self.error_msg = Some("No data to export.".to_string());
            return;
        }

        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Excel", &["xlsx"])
            .set_file_name("export.xlsx")
            .save_file() {
            
            let mut workbook = rust_xlsxwriter::Workbook::new();
            let worksheet = workbook.add_worksheet();

            // Write headers
            for (col_num, col_name) in self.columns.iter().enumerate() {
                let _ = worksheet.write_string(0, col_num as u16, col_name);
            }

            // Write rows
            for (row_num, row_data) in self.rows.iter().enumerate() {
                for (col_num, cell_data) in row_data.iter().enumerate() {
                    let _ = worksheet.write_string((row_num + 1) as u32, col_num as u16, cell_data);
                }
            }

            match workbook.save(&path) {
                Ok(_) => self.info_msg = Some(format!("Successfully exported to {:?}", path)),
                Err(e) => self.error_msg = Some(format!("Failed to write Excel: {}", e)),
            }
        }
    }
}
