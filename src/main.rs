use anyhow::{Context, Result};
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use mysql::{Pool, OptsBuilder, SslOpts};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;

mod database;
mod ui;
mod navigation;
mod connection_config;
mod connection_ui;
mod user_config;

use database::DatabaseManager;
use navigation::{NavigationState, ViewMode, SqlResult};
use ui::AppUI;
use connection_config::{ConnectionConfig, ConnectionManager};
use connection_ui::ConnectionUI;
use user_config::{UserConfigManager, SqlHistoryEntry};

#[derive(Parser)]
#[command(name = "rmsql")]
#[command(about = "A vim-inspired MySQL client for navigating databases")]
#[command(about = "A vim-like MySQL client for navigating databases")]
struct Args {
    /// MySQL host
    #[arg(short = 'h', long, default_value = "localhost")]
    host: String,
    
    /// MySQL port
    #[arg(short = 'P', long, default_value = "3306")]
    port: u16,
    
    /// MySQL username (default: root when running with sudo)
    #[arg(short = 'u', long)]
    username: Option<String>,
    
    /// MySQL password
    #[arg(short = 'p', long)]
    password: Option<String>,
    
    /// Initial database to connect to
    #[arg(short = 'd', long)]
    database: Option<String>,
}

pub struct App {
    db_manager: DatabaseManager,
    navigation: NavigationState,
    ui: AppUI,
    user_config: UserConfigManager,
    connection_config: ConnectionConfig,
    should_quit: bool,
    status_message: String,
}

impl App {
    pub fn new(pool: Pool, connection_config: ConnectionConfig) -> Result<Self> {
        let db_manager = DatabaseManager::new(pool)?;
        let navigation = NavigationState::new();
        let ui = AppUI::new();
        let user_config = UserConfigManager::new()?;
        
        Ok(App {
            db_manager,
            navigation,
            ui,
            user_config,
            connection_config,
            should_quit: false,
            status_message: "Welcome to RMSQL - Press 'q' to quit, 'h' for help".to_string(),
        })
    }

    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        // Load initial data
        self.refresh_current_view()?;
        
        loop {
            terminal.draw(|f| self.ui.draw(f, &self.navigation, &self.status_message))?;
            
            if self.should_quit {
                break;
            }
            
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    self.handle_key_event(key.code)?;
                }
            }
        }
        
        Ok(())
    }
    
    fn handle_key_event(&mut self, key_code: KeyCode) -> Result<()> {
        // Handle SQL editor mode separately
        if self.navigation.mode == ViewMode::SqlEditor {
            return self.handle_sql_editor_key(key_code);
        }
        
        match key_code {
            KeyCode::Char('q') => self.should_quit = true,
            
            // Vim-like navigation
            KeyCode::Char('j') | KeyCode::Down => self.navigation.move_down(),
            KeyCode::Char('k') | KeyCode::Up => self.navigation.move_up(),
            
            // Navigation controls
            KeyCode::Enter => self.navigate_forward()?,
            KeyCode::Esc => self.navigate_back()?,
            
            // Horizontal navigation (only in expanded table mode)
            KeyCode::Char('h') | KeyCode::Left => {
                if self.navigation.mode == ViewMode::TableData && self.navigation.expanded_columns {
                    self.navigation.scroll_left();
                    self.update_scroll_status();
                } else {
                    self.navigate_back()?;
                }
            },
            KeyCode::Char('l') | KeyCode::Right => {
                if self.navigation.mode == ViewMode::TableData && self.navigation.expanded_columns {
                    self.navigation.scroll_right();
                    self.update_scroll_status();
                } else {
                    self.navigate_forward()?;
                }
            },
            
            // Page navigation
            KeyCode::Char('g') => self.navigation.move_to_top(),
            KeyCode::Char('G') => self.navigation.move_to_bottom(),
            
            // Refresh
            KeyCode::Char('r') => self.refresh_current_view()?,
            
            // Help
            KeyCode::Char('?') => self.show_help(),
            
            // Toggle column expansion (only in TableData mode)
            KeyCode::Char(' ') => {
                if self.navigation.mode == ViewMode::TableData {
                    self.navigation.toggle_expanded_columns();
                    if self.navigation.expanded_columns {
                        // Calculate visible columns based on terminal width
                        // For now, use a default of 3-4 columns
                        self.navigation.set_visible_columns(3);
                        self.status_message = "Expanded mode: Use ←→ arrows to navigate columns, Esc to go back".to_string();
                    } else {
                        self.status_message = "Normal mode: Press Space to expand columns".to_string();
                    }
                }
            },
            
            // SQL Editor
            KeyCode::Char('i') => {
                self.navigation.set_mode(ViewMode::SqlEditor);
                self.navigation.clear_sql_result();
                self.status_message = "Entered SQL Editor mode - Type SQL and press Enter to execute".to_string();
            },
            
            // Mode switching
            KeyCode::Char('1') => {
                self.navigation.set_mode(ViewMode::Databases);
                self.refresh_current_view()?;
            },
            KeyCode::Char('2') => {
                if self.navigation.current_database.is_some() {
                    self.navigation.set_mode(ViewMode::Tables);
                    self.refresh_current_view()?;
                }
            },
            KeyCode::Char('3') => {
                if self.navigation.current_table.is_some() {
                    self.navigation.set_mode(ViewMode::TableData);
                    self.refresh_current_view()?;
                }
            },
            
            _ => {}
        }
        
        Ok(())
    }
    
    fn navigate_forward(&mut self) -> Result<()> {
        match self.navigation.mode {
            ViewMode::Databases => {
                if let Some(selected) = self.navigation.get_selected_database() {
                    let selected = selected.clone(); // Clone to avoid borrow issues
                    self.navigation.set_current_database(selected.clone());
                    self.navigation.set_mode(ViewMode::Tables);
                    self.refresh_current_view()?;
                    self.status_message = format!("Switched to database: {}", selected);
                }
            },
            ViewMode::Tables => {
                if let Some(selected) = self.navigation.get_selected_table() {
                    let selected = selected.clone(); // Clone to avoid borrow issues
                    self.navigation.set_current_table(selected.clone());
                    self.navigation.set_mode(ViewMode::TableData);
                    self.refresh_current_view()?;
                    self.status_message = format!("Viewing table: {}", selected);
                }
            },
            ViewMode::TableData => {
                // Could implement row details view here
            },
            ViewMode::SqlEditor => {
                // No forward navigation in SQL editor
            },
        }
        
        Ok(())
    }
    
    fn navigate_back(&mut self) -> Result<()> {
        match self.navigation.mode {
            ViewMode::Tables => {
                self.navigation.set_mode(ViewMode::Databases);
                self.refresh_current_view()?;
                self.status_message = "Switched to databases view".to_string();
            },
            ViewMode::TableData => {
                self.navigation.set_mode(ViewMode::Tables);
                self.refresh_current_view()?;
                self.status_message = "Switched to tables view".to_string();
            },
            ViewMode::SqlEditor => {
                // Exit SQL editor, go back to appropriate view
                if self.navigation.current_table.is_some() {
                    self.navigation.set_mode(ViewMode::TableData);
                    self.refresh_current_view()?;
                    self.status_message = "Exited SQL Editor, back to table data".to_string();
                } else if self.navigation.current_database.is_some() {
                    self.navigation.set_mode(ViewMode::Tables);
                    self.refresh_current_view()?;
                    self.status_message = "Exited SQL Editor, back to tables".to_string();
                } else {
                    self.navigation.set_mode(ViewMode::Databases);
                    self.refresh_current_view()?;
                    self.status_message = "Exited SQL Editor, back to databases".to_string();
                }
            },
            _ => {}
        }
        
        Ok(())
    }
    
    fn refresh_current_view(&mut self) -> Result<()> {
        match self.navigation.mode {
            ViewMode::Databases => {
                let databases = self.db_manager.get_databases()?;
                
                // Save discovered databases to user config
                for db_name in &databases {
                    let _ = self.user_config.add_database(
                        self.connection_config.id.clone(),
                        db_name.clone()
                    );
                }
                
                self.navigation.set_databases(databases);
                self.status_message = "Databases loaded".to_string();
            },
            ViewMode::Tables => {
                if let Some(db_name) = &self.navigation.current_database {
                    let db_name = db_name.clone(); // Clone to avoid borrow issues
                    
                    // Update last accessed database
                    let _ = self.user_config.update_database_access(&self.connection_config.id, &db_name);
                    let _ = self.user_config.set_last_database(self.connection_config.id.clone(), db_name.clone());
                    
                    let tables = self.db_manager.get_tables(&db_name)?;
                    self.navigation.set_tables(tables);
                    self.status_message = format!("Tables loaded for database: {}", db_name);
                }
            },
            ViewMode::TableData => {
                if let (Some(db_name), Some(table_name)) = (
                    &self.navigation.current_database,
                    &self.navigation.current_table,
                ) {
                    let db_name = db_name.clone(); // Clone to avoid borrow issues
                    let table_name = table_name.clone(); // Clone to avoid borrow issues
                    let (columns, rows) = self.db_manager.get_table_data(&db_name, &table_name)?;
                    self.navigation.set_table_data(columns, rows);
                    self.status_message = format!("Data loaded for table: {}.{}", db_name, table_name);
                }
            },
            ViewMode::SqlEditor => {
                // Load recent SQL commands when entering SQL editor
                let recent_commands = self.user_config.get_recent_sql_commands(10);
                self.navigation.set_sql_history(recent_commands);
                // No other refresh needed for SQL editor
            },
        }
        
        Ok(())
    }
    
    fn show_help(&mut self) {
        self.status_message = "Help: j/k=up/down, h/l=back/forward, r=refresh, 1/2/3=modes, i=SQL editor, Space=expand, q=quit".to_string();
    }
    
    fn handle_sql_editor_key(&mut self, key_code: KeyCode) -> Result<()> {
        match key_code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Esc => {
                // Exit SQL editor mode, go back to previous mode
                if self.navigation.current_table.is_some() {
                    self.navigation.set_mode(ViewMode::TableData);
                    self.refresh_current_view()?;
                } else if self.navigation.current_database.is_some() {
                    self.navigation.set_mode(ViewMode::Tables);
                    self.refresh_current_view()?;
                } else {
                    self.navigation.set_mode(ViewMode::Databases);
                    self.refresh_current_view()?;
                }
                self.status_message = "Exited SQL Editor mode".to_string();
            },
            KeyCode::Enter => {
                // Execute SQL
                let sql = self.navigation.execute_sql();
                if !sql.is_empty() {
                    self.execute_sql_query(&sql)?;
                }
            },
            KeyCode::Up => {
                // Navigate history up
                self.navigation.navigate_history_up();
            },
            KeyCode::Down => {
                // Navigate history down
                self.navigation.navigate_history_down();
            },
            KeyCode::Backspace => {
                self.navigation.backspace_sql_input();
            },
            KeyCode::Char(c) => {
                self.navigation.add_to_sql_input(c);
            },
            _ => {}
        }
        
        Ok(())
    }
    
    fn execute_sql_query(&mut self, sql: &str) -> Result<()> {
        let start_time = std::time::Instant::now();
        
        match self.db_manager.execute_sql(sql, self.navigation.current_database.as_deref()) {
            Ok((columns, rows, message)) => {
                let execution_time = start_time.elapsed().as_millis() as u64;
                
                let result = SqlResult {
                    columns,
                    rows,
                    message: message.clone(),
                };
                self.navigation.set_sql_result(result);
                self.status_message = message;
                
                // Save to history
                let history_entry = SqlHistoryEntry {
                    sql: sql.to_string(),
                    timestamp: chrono::Utc::now(),
                    database: self.navigation.current_database.clone(),
                    connection_id: self.connection_config.id.clone(),
                    execution_time_ms: Some(execution_time),
                    success: true,
                    error_message: None,
                };
                let _ = self.user_config.add_sql_history(history_entry);
            },
            Err(e) => {
                let execution_time = start_time.elapsed().as_millis() as u64;
                
                let result = SqlResult {
                    columns: Vec::new(),
                    rows: Vec::new(),
                    message: format!("Error: {}", e),
                };
                self.navigation.set_sql_result(result);
                self.status_message = format!("SQL Error: {}", e);
                
                // Save error to history
                let history_entry = SqlHistoryEntry {
                    sql: sql.to_string(),
                    timestamp: chrono::Utc::now(),
                    database: self.navigation.current_database.clone(),
                    connection_id: self.connection_config.id.clone(),
                    execution_time_ms: Some(execution_time),
                    success: false,
                    error_message: Some(e.to_string()),
                };
                let _ = self.user_config.add_sql_history(history_entry);
            }
        }
        Ok(())
    }
    
    fn update_scroll_status(&mut self) {
        if self.navigation.expanded_columns {
            let (start, end) = self.navigation.get_visible_columns();
            let total = self.navigation.table_columns.len();
            self.status_message = format!(
                "Columns {}-{} of {} | Use ←→ to scroll, Space to exit expanded mode", 
                start + 1, 
                end, 
                total
            );
        }
    }
}

fn show_connection_selector() -> Result<ConnectionConfig> {
    let mut connection_manager = ConnectionManager::load()?;
    let mut connection_ui = ConnectionUI::new();
    
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = loop {
        terminal.draw(|f| connection_ui.draw(f, &connection_manager))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                // Check if we should handle 'q' for quitting or let the form handle it
                if key.code == KeyCode::Char('q') && connection_ui.mode == connection_ui::ConnectionUIMode::List {
                    // Only quit when in list mode, not in forms
                    disable_raw_mode()?;
                    execute!(
                        terminal.backend_mut(),
                        LeaveAlternateScreen,
                        DisableMouseCapture
                    )?;
                    terminal.show_cursor()?;
                    return Err(anyhow::anyhow!("User quit connection selection"));
                } else {
                    // Let the connection UI handle all other keys, including 'q' in forms
                    if let Some(config) = connection_ui.handle_key(key, &mut connection_manager)? {
                        break config;
                    }
                }
            }
        }
    };

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Save the selected connection as last used
    connection_manager.set_last_used(&result.id)?;

    Ok(result)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Check if connection parameters were provided via command line
    let connection_config = if args.host != "localhost" || args.port != 3306 || args.username.is_some() || args.password.is_some() {
        // Use command line parameters
        let username = match args.username {
            Some(user) => user,
            None => {
                if std::env::var("SUDO_USER").is_ok() || std::env::var("USER").unwrap_or_default() == "root" {
                    "root".to_string()
                } else {
                    return Err(anyhow::anyhow!("Username is required. Use -u flag or run with sudo to use root"));
                }
            }
        };
        
        ConnectionConfig::new(
            "Command Line".to_string(),
            args.host,
            args.port,
            username,
            args.password.unwrap_or_default(),
            args.database,
        )
    } else {
        // Show connection selector
        show_connection_selector()?
    };

    // Build connection options with UTF-8 charset
    let password = connection_config.password.clone();
    let mut opts_builder = OptsBuilder::new()
        .ip_or_hostname(Some(connection_config.host.clone()))
        .tcp_port(connection_config.port)
        .user(Some(connection_config.username.clone()))
        .pass(if password.is_empty() { None } else { Some(password) })
        .init(vec!["SET NAMES utf8mb4 COLLATE utf8mb4_unicode_ci".to_string()]);
    
    // Configure SSL based on connection settings
    if !connection_config.use_ssl {
        // Disable SSL by setting empty SSL options
        opts_builder = opts_builder.ssl_opts(None::<SslOpts>);
    }
    
    let opts = opts_builder;
    
    // Create connection pool
    let pool = Pool::new(opts)
        .context("Failed to create MySQL connection pool")?;
    
    // Test connection
    {
        let mut _conn = pool.get_conn()
            .context("Failed to establish MySQL connection. Make sure MySQL is running and accessible.")?;
    }
    
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // Create and run app
    let mut app = App::new(pool, connection_config)?;
    let result = app.run(&mut terminal);
    
    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    
    result
}
