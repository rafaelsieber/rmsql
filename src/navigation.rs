use ratatui::widgets::{ListState, TableState};

#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    Databases,
    Tables,
    TableData,
    SqlEditor,
}

pub struct NavigationState {
    pub mode: ViewMode,
    pub current_database: Option<String>,
    pub current_table: Option<String>,
    
    // Data storage
    pub databases: Vec<String>,
    pub tables: Vec<String>,
    pub table_columns: Vec<String>,
    pub table_rows: Vec<Vec<String>>,
    
    // Table display settings
    pub expanded_columns: bool,
    pub horizontal_scroll: usize,
    pub visible_columns: usize,
    
    // SQL Editor
    pub sql_input: String,
    pub sql_history: Vec<String>,
    pub sql_history_index: Option<usize>,
    pub sql_result: Option<SqlResult>,
    
    // List states for UI
    pub database_list_state: ListState,
    pub table_list_state: ListState,
    pub data_table_state: TableState,
}

#[derive(Debug, Clone)]
pub struct SqlResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub message: String,
}

impl NavigationState {
    pub fn new() -> Self {
        let mut nav = NavigationState {
            mode: ViewMode::Databases,
            current_database: None,
            current_table: None,
            databases: Vec::new(),
            tables: Vec::new(),
            table_columns: Vec::new(),
            table_rows: Vec::new(),
            expanded_columns: false,
            horizontal_scroll: 0,
            visible_columns: 3, // Default number of visible columns when expanded
            sql_input: String::new(),
            sql_history: Vec::new(),
            sql_history_index: None,
            sql_result: None,
            database_list_state: ListState::default(),
            table_list_state: ListState::default(),
            data_table_state: TableState::default(),
        };
        
        // Initialize first item selected
        nav.database_list_state.select(Some(0));
        nav.table_list_state.select(Some(0));
        nav.data_table_state.select(Some(0));
        
        nav
    }
    
    pub fn move_up(&mut self) {
        match self.mode {
            ViewMode::Databases => {
                let current = self.database_list_state.selected().unwrap_or(0);
                if current > 0 {
                    self.database_list_state.select(Some(current - 1));
                }
            },
            ViewMode::Tables => {
                let current = self.table_list_state.selected().unwrap_or(0);
                if current > 0 {
                    self.table_list_state.select(Some(current - 1));
                }
            },
            ViewMode::TableData => {
                let current = self.data_table_state.selected().unwrap_or(0);
                if current > 0 {
                    self.data_table_state.select(Some(current - 1));
                }
            },
            ViewMode::SqlEditor => {
                // No movement in SQL editor mode
            },
        }
    }
    
    pub fn move_down(&mut self) {
        match self.mode {
            ViewMode::Databases => {
                let current = self.database_list_state.selected().unwrap_or(0);
                if current < self.databases.len().saturating_sub(1) {
                    self.database_list_state.select(Some(current + 1));
                }
            },
            ViewMode::Tables => {
                let current = self.table_list_state.selected().unwrap_or(0);
                if current < self.tables.len().saturating_sub(1) {
                    self.table_list_state.select(Some(current + 1));
                }
            },
            ViewMode::TableData => {
                let current = self.data_table_state.selected().unwrap_or(0);
                if current < self.table_rows.len().saturating_sub(1) {
                    self.data_table_state.select(Some(current + 1));
                }
            },
            ViewMode::SqlEditor => {
                // No movement in SQL editor mode
            },
        }
    }
    
    pub fn move_to_top(&mut self) {
        match self.mode {
            ViewMode::Databases => self.database_list_state.select(Some(0)),
            ViewMode::Tables => self.table_list_state.select(Some(0)),
            ViewMode::TableData => self.data_table_state.select(Some(0)),
            ViewMode::SqlEditor => {} // No action needed
        }
    }
    
    pub fn move_to_bottom(&mut self) {
        match self.mode {
            ViewMode::Databases => {
                if !self.databases.is_empty() {
                    self.database_list_state.select(Some(self.databases.len() - 1));
                }
            },
            ViewMode::Tables => {
                if !self.tables.is_empty() {
                    self.table_list_state.select(Some(self.tables.len() - 1));
                }
            },
            ViewMode::TableData => {
                if !self.table_rows.is_empty() {
                    self.data_table_state.select(Some(self.table_rows.len() - 1));
                }
            },
            ViewMode::SqlEditor => {} // No action needed
        }
    }
    
    pub fn set_mode(&mut self, mode: ViewMode) {
        self.mode = mode;
    }
    
    pub fn set_current_database(&mut self, database: String) {
        self.current_database = Some(database);
        self.current_table = None; // Reset table when changing database
        self.tables.clear();
        self.table_rows.clear();
        self.table_columns.clear();
        self.table_list_state.select(Some(0));
        self.data_table_state.select(Some(0));
    }
    
    pub fn set_current_table(&mut self, table: String) {
        self.current_table = Some(table);
        self.table_rows.clear();
        self.table_columns.clear();
        self.data_table_state.select(Some(0));
    }
    
    pub fn set_databases(&mut self, databases: Vec<String>) {
        self.databases = databases;
        if !self.databases.is_empty() && self.database_list_state.selected().is_none() {
            self.database_list_state.select(Some(0));
        }
    }
    
    pub fn set_tables(&mut self, tables: Vec<String>) {
        self.tables = tables;
        if !self.tables.is_empty() && self.table_list_state.selected().is_none() {
            self.table_list_state.select(Some(0));
        }
    }
    
    pub fn set_table_data(&mut self, columns: Vec<String>, rows: Vec<Vec<String>>) {
        self.table_columns = columns;
        self.table_rows = rows;
        if !self.table_rows.is_empty() && self.data_table_state.selected().is_none() {
            self.data_table_state.select(Some(0));
        }
    }
    
    pub fn get_selected_database(&self) -> Option<&String> {
        self.database_list_state
            .selected()
            .and_then(|i| self.databases.get(i))
    }
    
    pub fn get_selected_table(&self) -> Option<&String> {
        self.table_list_state
            .selected()
            .and_then(|i| self.tables.get(i))
    }
    
    pub fn get_current_path(&self) -> String {
        match (&self.current_database, &self.current_table) {
            (Some(db), Some(table)) => format!("{}/{}", db, table),
            (Some(db), None) => db.clone(),
            _ => "/".to_string(),
        }
    }
    
    pub fn add_to_sql_input(&mut self, ch: char) {
        self.sql_input.push(ch);
    }
    
    pub fn backspace_sql_input(&mut self) {
        self.sql_input.pop();
    }
    
    pub fn execute_sql(&mut self) -> String {
        if !self.sql_input.trim().is_empty() {
            let sql = self.sql_input.trim().to_string();
            self.sql_history.push(sql.clone());
            self.sql_history_index = None;
            self.sql_input.clear();
            return sql;
        }
        String::new()
    }
    
    pub fn navigate_history_up(&mut self) {
        if !self.sql_history.is_empty() {
            match self.sql_history_index {
                None => {
                    self.sql_history_index = Some(self.sql_history.len() - 1);
                    self.sql_input = self.sql_history[self.sql_history.len() - 1].clone();
                },
                Some(index) if index > 0 => {
                    self.sql_history_index = Some(index - 1);
                    self.sql_input = self.sql_history[index - 1].clone();
                },
                _ => {}
            }
        }
    }
    
    pub fn navigate_history_down(&mut self) {
        if let Some(index) = self.sql_history_index {
            if index < self.sql_history.len() - 1 {
                self.sql_history_index = Some(index + 1);
                self.sql_input = self.sql_history[index + 1].clone();
            } else {
                self.sql_history_index = None;
                self.sql_input.clear();
            }
        }
    }
    
    pub fn set_sql_result(&mut self, result: SqlResult) {
        self.sql_result = Some(result);
    }
    
    pub fn clear_sql_result(&mut self) {
        self.sql_result = None;
    }
    
    pub fn set_sql_history(&mut self, history: Vec<String>) {
        self.sql_history = history;
    }
    
    pub fn toggle_expanded_columns(&mut self) {
        self.expanded_columns = !self.expanded_columns;
        // Reset horizontal scroll when toggling
        self.horizontal_scroll = 0;
    }
    
    pub fn scroll_right(&mut self) {
        if self.expanded_columns && !self.table_columns.is_empty() {
            let max_scroll = self.table_columns.len().saturating_sub(self.visible_columns);
            if self.horizontal_scroll < max_scroll {
                self.horizontal_scroll += 1;
            }
        }
    }
    
    pub fn scroll_left(&mut self) {
        if self.expanded_columns && self.horizontal_scroll > 0 {
            self.horizontal_scroll -= 1;
        }
    }
    
    pub fn get_visible_columns(&self) -> (usize, usize) {
        if !self.expanded_columns || self.table_columns.is_empty() {
            return (0, self.table_columns.len());
        }
        
        let start = self.horizontal_scroll;
        let end = (start + self.visible_columns).min(self.table_columns.len());
        (start, end)
    }
    
    pub fn set_visible_columns(&mut self, count: usize) {
        self.visible_columns = count.max(1); // At least 1 column visible
    }
}
