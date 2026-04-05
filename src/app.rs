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
                self.error_msg = Some(format!("Lỗi kết nối: {}", e));
            }
        }
    }

    pub fn execute_query(&mut self) {
        let query = self.query_editor.get_text();
        match self.executor.execute(&query) {
            Ok((cols, rows)) => {
                self.columns = cols;
                self.rows = rows;
                self.error_msg = None;
                self.vertical_scroll = 0;
                self.horizontal_scroll = 0;
                self.table_state.select(if self.rows.is_empty() { None } else { Some(0) });
            }
            Err(e) => {
                self.error_msg = Some(e);
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
}
