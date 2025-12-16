use crate::config;
use crate::error::{MirrorError, Result};
use crate::traits::SourceManager;
use crate::types::Mirror;
use crate::utils;
use async_trait::async_trait;
use directories::BaseDirs;
use std::path::PathBuf;
use tokio::fs;

pub struct UvManager {
    custom_path: Option<PathBuf>,
}

impl UvManager {
    pub fn new() -> Self {
        Self { custom_path: None }
    }

    #[cfg(test)]
    pub fn with_path(path: PathBuf) -> Self {
        Self {
            custom_path: Some(path),
        }
    }
}

#[async_trait]
impl SourceManager for UvManager {
    fn name(&self) -> &'static str {
        "uv"
    }

    fn requires_sudo(&self) -> bool {
        false
    }

    fn list_candidates(&self) -> Vec<Mirror> {
        config::get_candidates("uv")
    }

    fn config_path(&self) -> PathBuf {
        if let Some(ref path) = self.custom_path {
            return path.clone();
        }

        // 1. Check current directory (Project level)
        let local_path = PathBuf::from("uv.toml");
        if local_path.exists() {
            return local_path;
        }

        // 2. Global path
        // Linux: ~/.config/uv/uv.toml
        // macOS: ~/Library/Application Support/uv/uv.toml
        // Windows: %APPDATA%\uv\uv.toml
        BaseDirs::new()
            .map(|dirs| dirs.config_dir().join("uv").join("uv.toml"))
            .unwrap_or_else(|| PathBuf::from(".").join("uv.toml"))
    }

    async fn current_url(&self) -> Result<Option<String>> {
        let path = self.config_path();
        if !fs::try_exists(&path).await.unwrap_or(false) {
            return Ok(None);
        }

        let content = fs::read_to_string(&path).await?;
        let config: toml::Value =
            toml::from_str(&content).unwrap_or(toml::Value::Table(toml::map::Map::new()));

        // Look for [[index]] with default = true
        if let Some(indices) = config.get("index").and_then(|v| v.as_array()) {
            for index in indices {
                if index
                    .get("default")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    if let Some(url) = index.get("url").and_then(|v| v.as_str()) {
                        return Ok(Some(url.to_string()));
                    }
                }
            }
        }

        Ok(None)
    }

    async fn set_source(&self, mirror: &Mirror) -> Result<()> {
        let path = self.config_path();

        // 1. Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // 2. Read existing TOML or create empty
        let content = if fs::try_exists(&path).await.unwrap_or(false) {
            fs::read_to_string(&path).await?
        } else {
            String::new()
        };

        // 3. Backup
        if !content.is_empty() {
            utils::backup_file(&path).await?;
        }

        // 4. Update TOML
        let mut config: toml::Value =
            toml::from_str(&content).unwrap_or(toml::Value::Table(toml::map::Map::new()));

        let root = config
            .as_table_mut()
            .ok_or(MirrorError::Custom("Invalid uv.toml format".to_string()))?;

        // Handle [[index]]
        // If "index" key doesn't exist, create it as array
        if !root.contains_key("index") {
            root.insert("index".to_string(), toml::Value::Array(Vec::new()));
        }

        let indices =
            root.get_mut("index")
                .and_then(|v| v.as_array_mut())
                .ok_or(MirrorError::Custom(
                    "Invalid 'index' in uv.toml (expected array)".to_string(),
                ))?;

        // Check if there is already a default index
        let mut found_default = false;
        for index in indices.iter_mut() {
            if let Some(table) = index.as_table_mut() {
                if table
                    .get("default")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    // Update existing default
                    table.insert("url".to_string(), toml::Value::String(mirror.url.clone()));
                    // Ensure name matches if we want (optional, but keeping existing name is good)
                    // If no name, maybe set it to "pypi"? uv docs say name is optional.
                    found_default = true;
                    break;
                }
            }
        }

        if !found_default {
            // Append new default index
            let mut new_index = toml::map::Map::new();
            new_index.insert("name".to_string(), toml::Value::String("pypi".to_string()));
            new_index.insert("url".to_string(), toml::Value::String(mirror.url.clone()));
            new_index.insert("default".to_string(), toml::Value::Boolean(true));
            indices.push(toml::Value::Table(new_index));
        }

        // 5. Write back
        let new_content = toml::to_string_pretty(&config)?;
        fs::write(&path, new_content).await?;

        Ok(())
    }

    async fn restore(&self) -> Result<()> {
        utils::restore_latest_backup(&self.config_path()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_uv_flow() -> Result<()> {
        let dir = tempdir()?;
        let config_path = dir.path().join("uv.toml");
        let manager = UvManager::with_path(config_path.clone());

        // 1. Initial state
        assert!(manager.current_url().await?.is_none());

        // 2. Set source
        let mirror = Mirror {
            name: "TestUV".to_string(),
            url: "https://test.pypi.org/simple".to_string(),
        };
        manager.set_source(&mirror).await?;

        // 3. Check current
        assert_eq!(manager.current_url().await?, Some(mirror.url.clone()));

        // Check TOML structure
        let content = fs::read_to_string(&config_path).await?;
        assert!(content.contains("[[index]]"));
        assert!(content.contains("default = true"));
        assert!(content.contains(&format!("url = \"{}\"", mirror.url)));

        // 4. Set another
        let mirror2 = Mirror {
            name: "TestUV2".to_string(),
            url: "https://test2.pypi.org/simple".to_string(),
        };
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        manager.set_source(&mirror2).await?;
        assert_eq!(manager.current_url().await?, Some(mirror2.url.clone()));

        let content2 = fs::read_to_string(&config_path).await?;
        assert!(content2.contains(&format!("url = \"{}\"", mirror2.url)));

        // 5. Restore
        manager.restore().await?;
        assert_eq!(manager.current_url().await?, Some(mirror.url));

        Ok(())
    }
}
