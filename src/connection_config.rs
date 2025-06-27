use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub default_database: Option<String>,
    #[serde(default = "default_use_ssl")]
    pub use_ssl: bool,
}

fn default_use_ssl() -> bool {
    true
}

impl ConnectionConfig {
    pub fn new(name: String, host: String, port: u16, username: String, password: String, default_database: Option<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            host,
            port,
            username,
            password,
            default_database,
            use_ssl: true, // Default to SSL enabled for security
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionManager {
    pub connections: HashMap<String, ConnectionConfig>,
    pub last_used: Option<String>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
            last_used: None,
        }
    }

    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;
        
        if !config_path.exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(&config_path)
            .context("Failed to read connection config file")?;
        
        let manager: ConnectionManager = serde_json::from_str(&content)
            .context("Failed to parse connection config file")?;
            
        Ok(manager)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;
        
        // Create config directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }

        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize connection config")?;
            
        fs::write(&config_path, content)
            .context("Failed to write connection config file")?;
            
        Ok(())
    }

    pub fn add_connection(&mut self, config: ConnectionConfig) -> Result<()> {
        self.connections.insert(config.id.clone(), config);
        self.save()
    }

    pub fn remove_connection(&mut self, id: &str) -> Result<bool> {
        let removed = self.connections.remove(id).is_some();
        if removed {
            if self.last_used.as_ref() == Some(&id.to_string()) {
                self.last_used = None;
            }
            self.save()?;
        }
        Ok(removed)
    }

    pub fn list_connections(&self) -> Vec<&ConnectionConfig> {
        let mut connections: Vec<&ConnectionConfig> = self.connections.values().collect();
        connections.sort_by(|a, b| a.name.cmp(&b.name));
        connections
    }

    pub fn set_last_used(&mut self, id: &str) -> Result<()> {
        if self.connections.contains_key(id) {
            self.last_used = Some(id.to_string());
            self.save()?;
        }
        Ok(())
    }

    pub fn get_last_used(&self) -> Option<&ConnectionConfig> {
        self.last_used.as_ref()
            .and_then(|id| self.connections.get(id))
    }

    fn get_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to get config directory")?;
        Ok(config_dir.join("rmsql").join("connections.json"))
    }

    pub fn create_root_connection() -> ConnectionConfig {
        ConnectionConfig {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Root (Local)".to_string(),
            host: "localhost".to_string(),
            port: 3306,
            username: "root".to_string(),
            password: String::new(),
            default_database: None,
            use_ssl: true, // Default to SSL enabled
        }
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}
