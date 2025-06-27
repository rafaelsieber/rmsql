use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::connection_config::{ConnectionConfig, ConnectionManager};

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionUIMode {
    List,
    NewConnection,
    EditConnection(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputField {
    Name,
    Host,
    Port,
    Username,
    Password,
    Database,
}

pub struct ConnectionUI {
    pub mode: ConnectionUIMode,
    pub list_state: ListState,
    pub input_field: InputField,
    pub temp_config: ConnectionConfig,
    pub show_password: bool,
    pub status_message: String,
}

impl ConnectionUI {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            mode: ConnectionUIMode::List,
            list_state,
            input_field: InputField::Name,
            temp_config: ConnectionConfig::new(
                String::new(),
                "localhost".to_string(),
                3306,
                String::new(),
                String::new(),
                None,
            ),
            show_password: false,
            status_message: "Select a connection or create a new one".to_string(),
        }
    }

    pub fn draw(&mut self, f: &mut Frame, manager: &ConnectionManager) {
        let size = f.area();
        
        match self.mode {
            ConnectionUIMode::List => self.draw_connection_list(f, size, manager),
            ConnectionUIMode::NewConnection | ConnectionUIMode::EditConnection(_) => {
                self.draw_connection_form(f, size)
            }
        }
    }

    fn draw_connection_list(&mut self, f: &mut Frame, area: Rect, manager: &ConnectionManager) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(3),
                Constraint::Length(4),
            ])
            .split(area);

        // Title
        let title = Paragraph::new("RMSQL - Connection Manager")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // Connection list
        let connections = manager.list_connections();
        let mut items = Vec::new();

        // Add root connection option if running as root
        if Self::is_running_as_root() {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("⚡ ", Style::default().fg(Color::Yellow)),
                Span::raw("Root (Auto-detect)"),
            ])));
        }

        // Add saved connections
        for config in &connections {
            let marker = if manager.get_last_used().map(|c| &c.id) == Some(&config.id) {
                "★ "
            } else {
                "  "
            };
            
            items.push(ListItem::new(Line::from(vec![
                Span::styled(marker, Style::default().fg(Color::Yellow)),
                Span::raw(&config.name),
                Span::styled(
                    format!(" ({}:{}@{}:{})", 
                        config.username, 
                        if config.password.is_empty() { "no-pass" } else { "***" },
                        config.host, 
                        config.port
                    ),
                    Style::default().fg(Color::Gray)
                ),
            ])));
        }

        if items.is_empty() {
            items.push(ListItem::new("No connections configured"));
        }

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Connections"))
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
            .highlight_symbol("> ");

        f.render_stateful_widget(list, chunks[1], &mut self.list_state);

        // Status message
        let status = Paragraph::new(self.status_message.clone())
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(status, chunks[2]);

        // Help
        let help_text = vec![
            Line::from(vec![
                Span::styled("Enter", Style::default().fg(Color::Green)),
                Span::raw(": Connect | "),
                Span::styled("n", Style::default().fg(Color::Green)),
                Span::raw(": New | "),
                Span::styled("e", Style::default().fg(Color::Green)),
                Span::raw(": Edit | "),
                Span::styled("d", Style::default().fg(Color::Green)),
                Span::raw(": Delete | "),
                Span::styled("q", Style::default().fg(Color::Green)),
                Span::raw(": Quit"),
            ]),
            Line::from(vec![
                Span::styled("↑↓", Style::default().fg(Color::Green)),
                Span::raw(": Navigate"),
            ]),
        ];

        let help = Paragraph::new(help_text)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Help"));
        f.render_widget(help, chunks[3]);
    }

    fn draw_connection_form(&mut self, f: &mut Frame, area: Rect) {
        let popup_area = Self::centered_rect(80, 70, area);
        f.render_widget(Clear, popup_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(3),
            ])
            .split(popup_area);

        // Title
        let title = match self.mode {
            ConnectionUIMode::NewConnection => "New Connection",
            ConnectionUIMode::EditConnection(_) => "Edit Connection",
            _ => "Connection Form",
        };

        let title_widget = Paragraph::new(title)
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title_widget, chunks[0]);

        // Form fields
        let form_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .split(chunks[1]);

        self.draw_input_field(f, form_chunks[0], "Name", &self.temp_config.name, &InputField::Name);
        self.draw_input_field(f, form_chunks[1], "Host", &self.temp_config.host, &InputField::Host);
        self.draw_input_field(f, form_chunks[2], "Port", &self.temp_config.port.to_string(), &InputField::Port);
        self.draw_input_field(f, form_chunks[3], "Username", &self.temp_config.username, &InputField::Username);
        
        let password_display = if self.show_password { 
            self.temp_config.password.clone() 
        } else { 
            "*".repeat(self.temp_config.password.len()) 
        };
        self.draw_input_field(f, form_chunks[4], "Password", &password_display, &InputField::Password);
        
        self.draw_input_field(
            f, 
            form_chunks[5], 
            "Database (optional)", 
            &self.temp_config.default_database.as_deref().unwrap_or(""),
            &InputField::Database
        );

        // Help
        let help_text = vec![
            Line::from(vec![
                Span::styled("Tab/Shift+Tab", Style::default().fg(Color::Green)),
                Span::raw(": Navigate fields | "),
                Span::styled("Ctrl+S", Style::default().fg(Color::Green)),
                Span::raw(": Save | "),
                Span::styled("Esc", Style::default().fg(Color::Green)),
                Span::raw(": Cancel"),
            ]),
            Line::from(vec![
                Span::styled("Ctrl+P", Style::default().fg(Color::Green)),
                Span::raw(": Toggle password visibility"),
            ]),
        ];

        let help = Paragraph::new(help_text)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Help"));
        f.render_widget(help, chunks[2]);
    }

    fn draw_input_field(&self, f: &mut Frame, area: Rect, label: &str, value: &str, field: &InputField) {
        let is_selected = &self.input_field == field;
        
        let style = if is_selected {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(label)
            .border_style(if is_selected { 
                Style::default().fg(Color::Yellow) 
            } else { 
                Style::default() 
            });

        let paragraph = Paragraph::new(value)
            .style(style)
            .block(block);

        f.render_widget(paragraph, area);
    }

    pub fn handle_key(&mut self, key: KeyEvent, manager: &mut ConnectionManager) -> Result<Option<ConnectionConfig>> {
        match self.mode {
            ConnectionUIMode::List => self.handle_list_key(key, manager),
            ConnectionUIMode::NewConnection | ConnectionUIMode::EditConnection(_) => {
                self.handle_form_key(key, manager)
            }
        }
    }

    fn handle_list_key(&mut self, key: KeyEvent, manager: &mut ConnectionManager) -> Result<Option<ConnectionConfig>> {
        match key.code {
            KeyCode::Up => {
                let i = match self.list_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            self.get_total_connections(manager).saturating_sub(1)
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.list_state.select(Some(i));
            }
            KeyCode::Down => {
                let i = match self.list_state.selected() {
                    Some(i) => {
                        if i >= self.get_total_connections(manager).saturating_sub(1) {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                self.list_state.select(Some(i));
            }
            KeyCode::Enter => {
                if let Some(selected) = self.list_state.selected() {
                    return Ok(self.get_selected_connection(selected, manager));
                }
            }
            KeyCode::Char('n') => {
                self.mode = ConnectionUIMode::NewConnection;
                self.reset_temp_config();
            }
            KeyCode::Char('e') => {
                if let Some(selected) = self.list_state.selected() {
                    if let Some(config) = self.get_connection_by_index(selected, manager) {
                        let config_id = config.id.clone();
                        let config_clone = config.clone();
                        self.mode = ConnectionUIMode::EditConnection(config_id);
                        self.temp_config = config_clone;
                    }
                }
            }
            KeyCode::Char('d') => {
                if let Some(selected) = self.list_state.selected() {
                    if let Some(config) = self.get_connection_by_index(selected, manager) {
                        let config_id = config.id.clone();
                        let config_name = config.name.clone();
                        manager.remove_connection(&config_id)?;
                        self.status_message = format!("Deleted connection '{}'", config_name);
                        
                        // Adjust selection after deletion
                        let total = self.get_total_connections(manager);
                        if total == 0 {
                            self.list_state.select(None);
                        } else if selected >= total {
                            self.list_state.select(Some(total - 1));
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(None)
    }

    fn handle_form_key(&mut self, key: KeyEvent, manager: &mut ConnectionManager) -> Result<Option<ConnectionConfig>> {
        match key.code {
            KeyCode::Esc => {
                self.mode = ConnectionUIMode::List;
            }
            KeyCode::Tab => {
                self.next_field();
            }
            KeyCode::BackTab => {
                self.prev_field();
            }
            KeyCode::Char(c) if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                match c {
                    's' => {
                        return self.save_connection(manager);
                    }
                    'p' => {
                        self.show_password = !self.show_password;
                    }
                    _ => {}
                }
            }
            KeyCode::Char(c) => {
                self.input_char(c);
            }
            KeyCode::Backspace => {
                self.delete_char();
            }
            _ => {}
        }
        Ok(None)
    }

    fn get_total_connections(&self, manager: &ConnectionManager) -> usize {
        let mut count = manager.list_connections().len();
        if Self::is_running_as_root() {
            count += 1;
        }
        count
    }

    fn get_selected_connection(&self, index: usize, manager: &ConnectionManager) -> Option<ConnectionConfig> {
        let mut current_index = 0;
        
        // Check root connection first
        if Self::is_running_as_root() {
            if index == current_index {
                return Some(ConnectionManager::create_root_connection());
            }
            current_index += 1;
        }

        // Check saved connections
        let connections = manager.list_connections();
        if let Some(config) = connections.get(index - current_index) {
            return Some((*config).clone());
        }

        None
    }

    fn get_connection_by_index<'a>(&self, index: usize, manager: &'a ConnectionManager) -> Option<&'a ConnectionConfig> {
        let mut current_index = 0;
        
        // Skip root connection
        if Self::is_running_as_root() {
            if index == current_index {
                return None; // Can't edit root connection
            }
            current_index += 1;
        }

        // Get saved connections
        let connections = manager.list_connections();
        connections.get(index - current_index).copied()
    }

    fn is_running_as_root() -> bool {
        std::env::var("SUDO_USER").is_ok() || 
        std::env::var("USER").unwrap_or_default() == "root" ||
        unsafe { libc::geteuid() } == 0
    }

    fn reset_temp_config(&mut self) {
        self.temp_config = ConnectionConfig::new(
            String::new(),
            "localhost".to_string(),
            3306,
            String::new(),
            String::new(),
            None,
        );
        self.input_field = InputField::Name;
    }

    fn save_connection(&mut self, manager: &mut ConnectionManager) -> Result<Option<ConnectionConfig>> {
        // Validate required fields
        if self.temp_config.name.trim().is_empty() {
            self.status_message = "Name is required".to_string();
            return Ok(None);
        }
        if self.temp_config.username.trim().is_empty() {
            self.status_message = "Username is required".to_string();
            return Ok(None);
        }

        match &self.mode {
            ConnectionUIMode::NewConnection => {
                let config = self.temp_config.clone();
                manager.add_connection(config.clone())?;
                self.mode = ConnectionUIMode::List;
                return Ok(Some(config));
            }
            ConnectionUIMode::EditConnection(id) => {
                self.temp_config.id = id.clone();
                let config = self.temp_config.clone();
                manager.add_connection(config.clone())?;
                self.mode = ConnectionUIMode::List;
                return Ok(Some(config));
            }
            _ => {}
        }

        Ok(None)
    }

    fn next_field(&mut self) {
        self.input_field = match self.input_field {
            InputField::Name => InputField::Host,
            InputField::Host => InputField::Port,
            InputField::Port => InputField::Username,
            InputField::Username => InputField::Password,
            InputField::Password => InputField::Database,
            InputField::Database => InputField::Name,
        };
    }

    fn prev_field(&mut self) {
        self.input_field = match self.input_field {
            InputField::Name => InputField::Database,
            InputField::Host => InputField::Name,
            InputField::Port => InputField::Host,
            InputField::Username => InputField::Port,
            InputField::Password => InputField::Username,
            InputField::Database => InputField::Password,
        };
    }

    fn input_char(&mut self, c: char) {
        match self.input_field {
            InputField::Name => self.temp_config.name.push(c),
            InputField::Host => self.temp_config.host.push(c),
            InputField::Port => {
                if c.is_ascii_digit() {
                    let mut port_str = self.temp_config.port.to_string();
                    port_str.push(c);
                    if let Ok(port) = port_str.parse::<u16>() {
                        self.temp_config.port = port;
                    }
                }
            }
            InputField::Username => self.temp_config.username.push(c),
            InputField::Password => self.temp_config.password.push(c),
            InputField::Database => {
                if self.temp_config.default_database.is_none() {
                    self.temp_config.default_database = Some(String::new());
                }
                if let Some(ref mut db) = self.temp_config.default_database {
                    db.push(c);
                }
            }
        }
    }

    fn delete_char(&mut self) {
        match self.input_field {
            InputField::Name => { self.temp_config.name.pop(); }
            InputField::Host => { self.temp_config.host.pop(); }
            InputField::Port => {
                let mut port_str = self.temp_config.port.to_string();
                port_str.pop();
                if port_str.is_empty() {
                    self.temp_config.port = 0;
                } else if let Ok(port) = port_str.parse::<u16>() {
                    self.temp_config.port = port;
                }
            }
            InputField::Username => { self.temp_config.username.pop(); }
            InputField::Password => { self.temp_config.password.pop(); }
            InputField::Database => {
                if let Some(ref mut db) = self.temp_config.default_database {
                    db.pop();
                    if db.is_empty() {
                        self.temp_config.default_database = None;
                    }
                }
            }
        }
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
}

impl Default for ConnectionUI {
    fn default() -> Self {
        Self::new()
    }
}
