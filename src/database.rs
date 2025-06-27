use anyhow::Result;
use mysql::prelude::*;
use mysql::{Pool, Row};

pub struct DatabaseManager {
    pool: Pool,
}

impl DatabaseManager {
    pub fn new(pool: Pool) -> Result<Self> {
        // Test connection and set charset
        {
            let mut conn = pool.get_conn()?;
            conn.query_drop("SET NAMES utf8mb4 COLLATE utf8mb4_unicode_ci")?;
        }
        Ok(DatabaseManager { pool })
    }
    
    pub fn get_databases(&self) -> Result<Vec<String>> {
        let mut conn = self.pool.get_conn()?;
        conn.query_drop("SET NAMES utf8mb4 COLLATE utf8mb4_unicode_ci")?;
        
        let databases: Vec<String> = conn
            .query_map(
                "SHOW DATABASES",
                |database: String| database,
            )?
            .into_iter()
            .filter(|db| !["information_schema", "performance_schema", "sys"].contains(&db.as_str()))
            .collect();
        
        Ok(databases)
    }
    
    pub fn get_tables(&self, database: &str) -> Result<Vec<String>> {
        let mut conn = self.pool.get_conn()?;
        conn.query_drop("SET NAMES utf8mb4 COLLATE utf8mb4_unicode_ci")?;
        
        // Switch to the specified database
        conn.query_drop(format!("USE `{}`", database))?;
        
        let tables: Vec<String> = conn
            .query_map(
                "SHOW TABLES",
                |table: String| table,
            )?;
        
        Ok(tables)
    }
    
    pub fn get_table_data(&self, database: &str, table: &str) -> Result<(Vec<String>, Vec<Vec<String>>)> {
        let mut conn = self.pool.get_conn()?;
        conn.query_drop("SET NAMES utf8mb4 COLLATE utf8mb4_unicode_ci")?;
        
        // Switch to the specified database
        conn.query_drop(format!("USE `{}`", database))?;
        
        // Get column information
        let columns: Vec<String> = conn
            .query_map(
                format!("DESCRIBE `{}`", table),
                |row: Row| {
                    let field: String = row.get("Field").unwrap_or_default();
                    let type_info: String = row.get("Type").unwrap_or_default();
                    format!("{} ({})", field, type_info)
                },
            )?;
        
        // Get table data (limit to first 100 rows for performance)
        let query = format!("SELECT * FROM `{}` LIMIT 100", table);
        let result = conn.query_iter(query)?;
        
        let mut rows = Vec::new();
        for row_result in result {
            let row = row_result?;
            let mut row_data = Vec::new();
            
            // Convert each column value to string, handling NULL values properly
            for i in 0..row.len() {
                let value = row.get_opt::<String, usize>(i);
                let string_value = match value {
                    Some(Ok(s)) => s,
                    Some(Err(_)) => {
                        // Try to get as bytes and convert to string for better encoding handling
                        let bytes_value = row.get_opt::<Vec<u8>, usize>(i);
                        match bytes_value {
                            Some(Ok(bytes)) => {
                                match String::from_utf8(bytes) {
                                    Ok(utf8_string) => utf8_string,
                                    Err(_) => "(binary data)".to_string(),
                                }
                            },
                            _ => "NULL".to_string(),
                        }
                    },
                    None => "NULL".to_string(),
                };
                
                row_data.push(string_value);
            }
            
            rows.push(row_data);
        }
        
        Ok((columns, rows))
    }
    
    pub fn execute_sql(&self, sql: &str, database: Option<&str>) -> Result<(Vec<String>, Vec<Vec<String>>, String)> {
        let mut conn = self.pool.get_conn()?;
        conn.query_drop("SET NAMES utf8mb4 COLLATE utf8mb4_unicode_ci")?;
        
        // Switch to database if specified
        if let Some(db) = database {
            conn.query_drop(format!("USE `{}`", db))?;
        }
        
        // Determine if this is a SELECT query or other type
        let sql_trimmed = sql.trim().to_uppercase();
        
        if sql_trimmed.starts_with("SELECT") || sql_trimmed.starts_with("SHOW") || sql_trimmed.starts_with("DESCRIBE") || sql_trimmed.starts_with("EXPLAIN") {
            // Execute SELECT-like query
            let result = conn.query_iter(sql)?;
            let mut columns = Vec::new();
            let mut rows = Vec::new();
            let mut first_row = true;
            
            for row_result in result {
                let row = row_result?;
                
                // Get column names from the first row
                if first_row {
                    for i in 0..row.len() {
                        if let Some(column_name) = row.columns().get(i) {
                            columns.push(column_name.name_str().to_string());
                        } else {
                            columns.push(format!("Column_{}", i));
                        }
                    }
                    first_row = false;
                }
                
                let mut row_data = Vec::new();
                for i in 0..row.len() {
                    let value = row.get_opt::<String, usize>(i);
                    let string_value = match value {
                        Some(Ok(s)) => s,
                        Some(Err(_)) => {
                            let bytes_value = row.get_opt::<Vec<u8>, usize>(i);
                            match bytes_value {
                                Some(Ok(bytes)) => {
                                    match String::from_utf8(bytes) {
                                        Ok(utf8_string) => utf8_string,
                                        Err(_) => "(binary data)".to_string(),
                                    }
                                },
                                _ => "NULL".to_string(),
                            }
                        },
                        None => "NULL".to_string(),
                    };
                    
                    row_data.push(string_value);
                }
                rows.push(row_data);
            }
            
            let message = format!("Query executed successfully. {} rows returned.", rows.len());
            Ok((columns, rows, message))
        } else {
            // Execute non-SELECT query
            let result = conn.query_drop(sql);
            match result {
                Ok(()) => {
                    let affected_rows = conn.affected_rows();
                    let message = format!("Query executed successfully. {} rows affected.", affected_rows);
                    Ok((Vec::new(), Vec::new(), message))
                },
                Err(e) => {
                    let message = format!("Error: {}", e);
                    Ok((Vec::new(), Vec::new(), message))
                }
            }
        }
    }
}
