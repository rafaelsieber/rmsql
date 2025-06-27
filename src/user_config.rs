use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseInfo {
    pub name: String,
    pub connection_id: String,
    pub last_accessed: Option<chrono::DateTime<chrono::Utc>>,
    pub favorite: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqlHistoryEntry {
    pub sql: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub database: Option<String>,
    pub connection_id: String,
    pub execution_time_ms: Option<u64>,
    pub success: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserConfig {
    pub databases: HashMap<String, DatabaseInfo>,
    pub last_selected_database: Option<String>,
    pub last_connection_id: Option<String>,
    pub preferences: UserPreferences,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserPreferences {
    pub auto_save_history: bool,
    pub max_history_entries: usize,
    pub show_execution_time: bool,
    pub confirm_dangerous_queries: bool,
    pub default_limit: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SqlHistory {
    pub entries: Vec<SqlHistoryEntry>,
    pub max_entries: usize,
}

pub struct UserConfigManager {
    config: UserConfig,
    history: SqlHistory,
    config_path: PathBuf,
    history_path: PathBuf,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            auto_save_history: true,
            max_history_entries: 1000,
            show_execution_time: true,
            confirm_dangerous_queries: true,
            default_limit: Some(100),
        }
    }
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            databases: HashMap::new(),
            last_selected_database: None,
            last_connection_id: None,
            preferences: UserPreferences::default(),
        }
    }
}

impl Default for SqlHistory {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            max_entries: 1000,
        }
    }
}

impl UserConfigManager {
    pub fn new() -> Result<Self> {
        let config_path = Self::get_config_path()?;
        let history_path = Self::get_history_path()?;
        
        let config = Self::load_config(&config_path)?;
        let history = Self::load_history(&history_path)?;
        
        Ok(Self {
            config,
            history,
            config_path,
            history_path,
        })
    }

    #[allow(dead_code)]
    pub fn get_config(&self) -> &UserConfig {
        &self.config
    }

    #[allow(dead_code)]
    pub fn get_config_mut(&mut self) -> &mut UserConfig {
        &mut self.config
    }

    #[allow(dead_code)]
    pub fn get_history(&self) -> &SqlHistory {
        &self.history
    }

    pub fn add_database(&mut self, connection_id: String, database_name: String) -> Result<()> {
        let db_key = format!("{}:{}", connection_id, database_name);
        let db_info = DatabaseInfo {
            name: database_name,
            connection_id,
            last_accessed: Some(chrono::Utc::now()),
            favorite: false,
        };
        
        self.config.databases.insert(db_key, db_info);
        self.save_config()
    }

    #[allow(dead_code)]
    pub fn remove_database(&mut self, connection_id: &str, database_name: &str) -> Result<bool> {
        let db_key = format!("{}:{}", connection_id, database_name);
        let removed = self.config.databases.remove(&db_key).is_some();
        if removed {
            self.save_config()?;
        }
        Ok(removed)
    }

    #[allow(dead_code)]
    pub fn set_database_favorite(&mut self, connection_id: &str, database_name: &str, favorite: bool) -> Result<()> {
        let db_key = format!("{}:{}", connection_id, database_name);
        if let Some(db_info) = self.config.databases.get_mut(&db_key) {
            db_info.favorite = favorite;
            self.save_config()?;
        }
        Ok(())
    }

    pub fn update_database_access(&mut self, connection_id: &str, database_name: &str) -> Result<()> {
        let db_key = format!("{}:{}", connection_id, database_name);
        if let Some(db_info) = self.config.databases.get_mut(&db_key) {
            db_info.last_accessed = Some(chrono::Utc::now());
            self.save_config()?;
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_databases_for_connection(&self, connection_id: &str) -> Vec<&DatabaseInfo> {
        self.config.databases
            .values()
            .filter(|db| db.connection_id == connection_id)
            .collect()
    }

    #[allow(dead_code)]
    pub fn get_recent_databases(&self, limit: usize) -> Vec<&DatabaseInfo> {
        let mut databases: Vec<&DatabaseInfo> = self.config.databases.values().collect();
        databases.sort_by(|a, b| {
            b.last_accessed.unwrap_or_default().cmp(&a.last_accessed.unwrap_or_default())
        });
        databases.into_iter().take(limit).collect()
    }

    #[allow(dead_code)]
    pub fn get_favorite_databases(&self) -> Vec<&DatabaseInfo> {
        self.config.databases
            .values()
            .filter(|db| db.favorite)
            .collect()
    }

    pub fn add_sql_history(&mut self, entry: SqlHistoryEntry) -> Result<()> {
        if !self.config.preferences.auto_save_history {
            return Ok(());
        }

        self.history.entries.push(entry);
        
        // Limit the number of entries
        if self.history.entries.len() > self.history.max_entries {
            let excess = self.history.entries.len() - self.history.max_entries;
            self.history.entries.drain(0..excess);
        }
        
        self.save_history()
    }

    #[allow(dead_code)]
    pub fn get_sql_history(&self) -> &Vec<SqlHistoryEntry> {
        &self.history.entries
    }

    #[allow(dead_code)]
    pub fn get_sql_history_for_connection(&self, connection_id: &str) -> Vec<&SqlHistoryEntry> {
        self.history.entries
            .iter()
            .filter(|entry| entry.connection_id == connection_id)
            .collect()
    }

    #[allow(dead_code)]
    pub fn get_sql_history_for_database(&self, connection_id: &str, database: &str) -> Vec<&SqlHistoryEntry> {
        self.history.entries
            .iter()
            .filter(|entry| {
                entry.connection_id == connection_id && 
                entry.database.as_deref() == Some(database)
            })
            .collect()
    }

    pub fn get_recent_sql_commands(&self, limit: usize) -> Vec<String> {
        self.history.entries
            .iter()
            .rev()
            .take(limit)
            .map(|entry| entry.sql.clone())
            .collect()
    }

    #[allow(dead_code)]
    pub fn clear_history(&mut self) -> Result<()> {
        self.history.entries.clear();
        self.save_history()
    }

    #[allow(dead_code)]
    pub fn clear_history_for_connection(&mut self, connection_id: &str) -> Result<()> {
        self.history.entries.retain(|entry| entry.connection_id != connection_id);
        self.save_history()
    }

    pub fn set_last_database(&mut self, connection_id: String, database: String) -> Result<()> {
        self.config.last_connection_id = Some(connection_id);
        self.config.last_selected_database = Some(database);
        self.save_config()
    }

    #[allow(dead_code)]
    pub fn get_last_database(&self) -> Option<(String, String)> {
        match (&self.config.last_connection_id, &self.config.last_selected_database) {
            (Some(conn_id), Some(db)) => Some((conn_id.clone(), db.clone())),
            _ => None,
        }
    }

    pub fn save_config(&self) -> Result<()> {
        // Create config directory if it doesn't exist
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }

        let content = serde_json::to_string_pretty(&self.config)
            .context("Failed to serialize user config")?;
            
        fs::write(&self.config_path, content)
            .context("Failed to write user config file")?;
            
        Ok(())
    }

    pub fn save_history(&self) -> Result<()> {
        // Create cache directory if it doesn't exist
        if let Some(parent) = self.history_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create cache directory")?;
        }

        let content = serde_json::to_string_pretty(&self.history)
            .context("Failed to serialize SQL history")?;
            
        fs::write(&self.history_path, content)
            .context("Failed to write SQL history file")?;
            
        Ok(())
    }

    fn load_config(config_path: &PathBuf) -> Result<UserConfig> {
        if !config_path.exists() {
            return Ok(UserConfig::default());
        }

        let content = fs::read_to_string(config_path)
            .context("Failed to read user config file")?;
        
        let config: UserConfig = serde_json::from_str(&content)
            .context("Failed to parse user config file")?;
            
        Ok(config)
    }

    fn load_history(history_path: &PathBuf) -> Result<SqlHistory> {
        if !history_path.exists() {
            return Ok(SqlHistory::default());
        }

        let content = fs::read_to_string(history_path)
            .context("Failed to read SQL history file")?;
        
        let history: SqlHistory = serde_json::from_str(&content)
            .context("Failed to parse SQL history file")?;
            
        Ok(history)
    }

    fn get_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to get config directory")?;
        Ok(config_dir.join("rmsql").join("user_config.json"))
    }

    fn get_history_path() -> Result<PathBuf> {
        let cache_dir = dirs::cache_dir()
            .context("Failed to get cache directory")?;
        Ok(cache_dir.join("rmsql").join("sql_history.json"))
    }
}

impl Default for UserConfigManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            // Fallback in case of error
            let config_path = PathBuf::from("user_config.json");
            let history_path = PathBuf::from("sql_history.json");
            Self {
                config: UserConfig::default(),
                history: SqlHistory::default(),
                config_path,
                history_path,
            }
        })
    }
}
