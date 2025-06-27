use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table},
    Frame,
};

use crate::navigation::{NavigationState, ViewMode};

// Helper function to truncate UTF-8 strings safely
fn truncate_utf8(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    
    // Find the last valid UTF-8 char boundary at or before max_bytes
    for i in (0..=max_bytes).rev() {
        if s.is_char_boundary(i) {
            return &s[..i];
        }
    }
    
    // Fallback (shouldn't happen with valid UTF-8)
    ""
}

pub struct AppUI;

impl AppUI {
    pub fn new() -> Self {
        AppUI
    }
    
    pub fn draw(
        &self,
        f: &mut Frame,
        navigation: &NavigationState,
        status_message: &str,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Main content
                Constraint::Length(3), // Status bar
            ])
            .split(f.area());
        
        // Draw header
        self.draw_header(f, chunks[0], navigation);
        
        // Draw main content based on current mode
        match navigation.mode {
            ViewMode::Databases => self.draw_databases(f, chunks[1], navigation),
            ViewMode::Tables => self.draw_tables(f, chunks[1], navigation),
            ViewMode::TableData => self.draw_table_data(f, chunks[1], navigation),
            ViewMode::SqlEditor => self.draw_sql_editor(f, chunks[1], navigation),
        }
        
        // Draw status bar
        self.draw_status_bar(f, chunks[2], status_message, navigation);
    }
    
    fn draw_header(&self, f: &mut Frame, area: Rect, navigation: &NavigationState) {
        let title = match navigation.mode {
            ViewMode::Databases => "RMSQL - Databases",
            ViewMode::Tables => "RMSQL - Tables",
            ViewMode::TableData => "RMSQL - Table Data",
            ViewMode::SqlEditor => "RMSQL - SQL Editor",
        };
        
        let path = navigation.get_current_path();
        let header_text = format!("{} [{}]", title, path);
        
        let header = Paragraph::new(header_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::Cyan))
            )
            .style(Style::default().fg(Color::White));
        
        f.render_widget(header, area);
    }
    
    fn draw_databases(&self, f: &mut Frame, area: Rect, navigation: &NavigationState) {
        let items: Vec<ListItem> = navigation
            .databases
            .iter()
            .map(|db| {
                ListItem::new(Line::from(Span::styled(
                    format!("📁 {}", db),
                    Style::default().fg(Color::Yellow),
                )))
            })
            .collect();
        
        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Databases (j/k to navigate, l/Enter to open)")
            )
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            )
            .highlight_symbol("► ");
        
        f.render_stateful_widget(list, area, &mut navigation.database_list_state.clone());
    }
    
    fn draw_tables(&self, f: &mut Frame, area: Rect, navigation: &NavigationState) {
        let items: Vec<ListItem> = navigation
            .tables
            .iter()
            .map(|table| {
                ListItem::new(Line::from(Span::styled(
                    format!("📋 {}", table),
                    Style::default().fg(Color::Green),
                )))
            })
            .collect();
        
        let database_name = navigation
            .current_database
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("None");
        
        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Tables in '{}' (h to go back, l/Enter to view data)", database_name))
            )
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            )
            .highlight_symbol("► ");
        
        f.render_stateful_widget(list, area, &mut navigation.table_list_state.clone());
    }
    
    fn draw_table_data(&self, f: &mut Frame, area: Rect, navigation: &NavigationState) {
        if navigation.table_columns.is_empty() || navigation.table_rows.is_empty() {
            let empty_msg = Paragraph::new("No data available or table is empty")
                .block(Block::default().borders(Borders::ALL).title("Table Data"))
                .style(Style::default().fg(Color::Gray));
            f.render_widget(empty_msg, area);
            return;
        }
        
        // Split area for columns info and table data
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5), // Column info
                Constraint::Min(0),    // Table data
            ])
            .split(area);
        
        // Get visible column range based on expanded mode and horizontal scroll
        let (start_col, end_col) = if navigation.expanded_columns {
            navigation.get_visible_columns()
        } else {
            (0, navigation.table_columns.len())
        };
        
        // Draw column info - show only visible columns in expanded mode
        let column_info = if navigation.expanded_columns {
            let visible_cols = &navigation.table_columns[start_col..end_col];
            let info = visible_cols.join(" | ");
            format!("Columns {}-{} of {}: {}", start_col + 1, end_col, navigation.table_columns.len(), info)
        } else {
            navigation.table_columns.join(" | ")
        };
        
        let columns_widget = Paragraph::new(column_info)
            .block(Block::default().borders(Borders::ALL).title("Columns"))
            .style(Style::default().fg(Color::Cyan));
        f.render_widget(columns_widget, chunks[0]);
        
        // Prepare table headers - only visible columns
        let header = navigation
            .table_columns
            .iter()
            .skip(start_col)
            .take(end_col - start_col)
            .map(|col| {
                // Extract just the column name (before the type info in parentheses)
                let name = col.split(" (").next().unwrap_or(col);
                name.to_string()
            })
            .collect::<Vec<_>>();
        
        // Prepare table rows - only visible columns
        let rows: Vec<Row> = navigation
            .table_rows
            .iter()
            .map(|row| {
                Row::new(
                    row.iter()
                        .skip(start_col)
                        .take(end_col - start_col)
                        .map(|cell| {
                            // Truncate long values based on expansion mode
                            let max_len = if navigation.expanded_columns { 100 } else { 30 };
                            if cell.len() > max_len {
                                let truncated = truncate_utf8(cell, max_len.saturating_sub(3));
                                format!("{}...", truncated)
                            } else {
                                cell.clone()
                            }
                        })
                        .collect::<Vec<_>>()
                )
            })
            .collect();
        
        // Calculate column widths based on expansion mode
        let num_visible_cols = header.len().max(1);
        let available_width = chunks[1].width.saturating_sub(2); // Account for borders
        
        let constraints = if navigation.expanded_columns {
            // In expanded mode, give more space to columns (minimum 20 chars each)
            let min_col_width = 20u16;
            let total_min_width = min_col_width * num_visible_cols as u16;
            
            if total_min_width <= available_width {
                let extra_width = available_width - total_min_width;
                let extra_per_col = extra_width / num_visible_cols as u16;
                vec![Constraint::Length(min_col_width + extra_per_col); num_visible_cols]
            } else {
                vec![Constraint::Length(min_col_width); num_visible_cols]
            }
        } else {
            // In normal mode, distribute space evenly
            let col_width = available_width / num_visible_cols as u16;
            vec![Constraint::Length(col_width); num_visible_cols]
        };
        
        let table_name = navigation
            .current_table
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("Unknown");
        
        let title = if navigation.expanded_columns {
            format!(
                "Data from '{}' [EXPANDED {}-{}/{}] (←→ navigate, Space compress, h back)", 
                table_name,
                start_col + 1,
                end_col,
                navigation.table_columns.len()
            )
        } else {
            format!(
                "Data from '{}' (h to go back, Space to expand, showing first 100 rows)", 
                table_name
            )
        };
        
        let table = Table::new(rows, constraints)
            .header(
                Row::new(header)
                    .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                    .bottom_margin(1)
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
            )
            .style(Style::default().fg(Color::White))
            .row_highlight_style(
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            );
        
        f.render_stateful_widget(table, chunks[1], &mut navigation.data_table_state.clone());
    }
    
    fn draw_sql_editor(&self, f: &mut Frame, area: Rect, navigation: &NavigationState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5), // SQL input
                Constraint::Length(3), // History info
                Constraint::Min(0),    // Results
            ])
            .split(area);
        
        // Draw SQL input
        let current_db = navigation.current_database
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("none");
        
        let sql_input = Paragraph::new(navigation.sql_input.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("SQL Editor - Database: {} (Enter to execute, Esc to exit, Up/Down for history)", current_db))
            )
            .style(Style::default().fg(Color::White))
            .wrap(ratatui::widgets::Wrap { trim: false });
        
        f.render_widget(sql_input, chunks[0]);
        
        // Draw history info
        let history_info = if navigation.sql_history.is_empty() {
            "No SQL history yet".to_string()
        } else {
            format!("History: {} queries saved", navigation.sql_history.len())
        };
        
        let history_widget = Paragraph::new(history_info)
            .block(Block::default().borders(Borders::ALL).title("History"))
            .style(Style::default().fg(Color::Gray));
        
        f.render_widget(history_widget, chunks[1]);
        
        // Draw results
        if let Some(result) = &navigation.sql_result {
            if result.columns.is_empty() {
                // Non-SELECT query result
                let result_widget = Paragraph::new(result.message.as_str())
                    .block(Block::default().borders(Borders::ALL).title("Result"))
                    .style(Style::default().fg(Color::Green));
                
                f.render_widget(result_widget, chunks[2]);
            } else {
                // SELECT query result
                let rows: Vec<Row> = result.rows
                    .iter()
                    .map(|row| {
                        Row::new(
                            row.iter()
                                .map(|cell| {
                                    if cell.len() > 50 {
                                        let truncated = truncate_utf8(cell, 47);
                                        format!("{}...", truncated)
                                    } else {
                                        cell.clone()
                                    }
                                })
                                .collect::<Vec<_>>()
                        )
                    })
                    .collect();
                
                let num_cols = result.columns.len().max(1);
                let available_width = chunks[2].width.saturating_sub(2);
                let col_width = available_width / num_cols as u16;
                let constraints = vec![Constraint::Length(col_width); num_cols];
                
                let table = Table::new(rows, constraints)
                    .header(
                        Row::new(result.columns.clone())
                            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                            .bottom_margin(1)
                    )
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(format!("Result - {}", result.message))
                    )
                    .style(Style::default().fg(Color::White));
                
                f.render_widget(table, chunks[2]);
            }
        } else {
            let placeholder = Paragraph::new("Enter SQL query above and press Enter to execute")
                .block(Block::default().borders(Borders::ALL).title("Results"))
                .style(Style::default().fg(Color::Gray));
            
            f.render_widget(placeholder, chunks[2]);
        }
    }
    
    fn draw_status_bar(
        &self,
        f: &mut Frame,
        area: Rect,
        status_message: &str,
        navigation: &NavigationState,
    ) {
        let mode_text = match navigation.mode {
            ViewMode::Databases => "[1] Databases",
            ViewMode::Tables => "[2] Tables", 
            ViewMode::TableData => "[3] Data",
            ViewMode::SqlEditor => "[i] SQL Editor",
        };
        
        let help_text = "Press '?' for help | q: quit | r: refresh | 1/2/3: switch modes | i: SQL editor | Space: expand columns";
        let status_text = format!("{} | {} | {}", mode_text, status_message, help_text);
        
        let status = Paragraph::new(status_text)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::White).bg(Color::DarkGray));
        
        f.render_widget(status, area);
    }
}
