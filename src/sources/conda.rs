use crate::config;
use crate::error::Result;
use crate::traits::SourceManager;
use crate::types::Mirror;
use crate::utils;
use async_trait::async_trait;
use directories::BaseDirs;
use regex::Regex;
use std::path::PathBuf;
use tokio::fs;

pub struct CondaManager {
    custom_path: Option<PathBuf>,
}

impl CondaManager {
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
impl SourceManager for CondaManager {
    fn name(&self) -> &'static str {
        "conda"
    }

    fn requires_sudo(&self) -> bool {
        false
    }

    fn list_candidates(&self) -> Vec<Mirror> {
        config::get_candidates("conda")
    }

    fn config_path(&self) -> PathBuf {
        if let Some(ref path) = self.custom_path {
            return path.clone();
        }

        // Standard: ~/.condarc
        BaseDirs::new()
            .map(|dirs| dirs.home_dir().join(".condarc"))
            .unwrap_or_else(|| PathBuf::from(".").join(".condarc"))
    }

    async fn current_url(&self) -> Result<Option<String>> {
        let path = self.config_path();
        if !fs::try_exists(&path).await.unwrap_or(false) {
            return Ok(None);
        }

        let content = fs::read_to_string(&path).await?;

        // Look for the first url in default_channels
        // Pattern: default_channels:
        //  - (url)
        let re = Regex::new(r"(?m)^default_channels:\s*\n\s*-\s*(.+)$")?;

        if let Some(caps) = re.captures(&content) {
            return Ok(Some(caps[1].trim().to_string()));
        }

        // Fallback: check channels (if user uses simple config)
        let re_channels = Regex::new(r"(?m)^channels:\s*\n\s*-\s*(.+)$")?;
        if let Some(caps) = re_channels.captures(&content) {
            let val = caps[1].trim();
            if val != "defaults" {
                return Ok(Some(val.to_string()));
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

        // 2. Read existing
        let content = if fs::try_exists(&path).await.unwrap_or(false) {
            fs::read_to_string(&path).await?
        } else {
            String::new()
        };

        // 3. Backup
        if !content.is_empty() {
            utils::backup_file(&path).await?;
        }

        // 4. Construct new content
        // Strategy:
        // We assume the mirror.url is the base URL (e.g., https://mirrors.tuna.tsinghua.edu.cn/anaconda)
        // We will configure default_channels and custom_channels.

        let base_url = mirror.url.trim_end_matches('/');

        let new_config_block = format!(
            "show_channel_urls: true\ndefault_channels:\n  - {}/pkgs/main\n  - {}/pkgs/r\n  - {}/pkgs/msys2\ncustom_channels:\n  conda-forge: {}/cloud\n  msys2: {}/cloud\n  bioconda: {}/cloud\n  menpo: {}/cloud\n  pytorch: {}/cloud\n  pytorch-lts: {}/cloud\n  simpleitk: {}/cloud",
            base_url, base_url, base_url, base_url, base_url, base_url, base_url, base_url, base_url, base_url
        );

        // Remove existing default_channels and custom_channels blocks
        // This is a naive regex removal. It assumes blocks end at the next top-level key (start of line)

        let mut new_content = content.clone();

        // Remove default_channels block
        let re_default = Regex::new(r"(?m)^default_channels:(\s*\n\s+-.*)*\n?")?;
        new_content = re_default.replace(&new_content, "").to_string();

        // Remove custom_channels block
        let re_custom = Regex::new(r"(?m)^custom_channels:(\s*\n\s+.*)*\n?")?;
        new_content = re_custom.replace(&new_content, "").to_string();

        // Remove show_channel_urls line
        let re_show = Regex::new(r"(?m)^show_channel_urls:.*\n?")?;
        new_content = re_show.replace(&new_content, "").to_string();

        // Append new config
        // Ensure we have a newline separator if file not empty
        let prefix = if new_content.trim().is_empty() {
            ""
        } else {
            "\n"
        };
        new_content = format!("{}{}{}\n", new_content.trim(), prefix, new_config_block);

        // 5. Write
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
    async fn test_conda_flow() -> Result<()> {
        let dir = tempdir()?;
        let config_path = dir.path().join(".condarc");
        let manager = CondaManager::with_path(config_path.clone());

        // 1. Initial state
        assert!(manager.current_url().await?.is_none());

        // 2. Set source
        let mirror = Mirror {
            name: "TestConda".to_string(),
            url: "https://mirrors.tuna.tsinghua.edu.cn/anaconda".to_string(),
        };
        manager.set_source(&mirror).await?;

        // 3. Check current (should match first default channel)
        let current = manager.current_url().await?;
        assert_eq!(
            current,
            Some("https://mirrors.tuna.tsinghua.edu.cn/anaconda/pkgs/main".to_string())
        );

        // Check content
        let content = fs::read_to_string(&config_path).await?;
        assert!(content.contains("default_channels:"));
        assert!(content.contains("  - https://mirrors.tuna.tsinghua.edu.cn/anaconda/pkgs/main"));
        assert!(content.contains("custom_channels:"));
        assert!(
            content.contains("conda-forge: https://mirrors.tuna.tsinghua.edu.cn/anaconda/cloud")
        );

        // 4. Set another
        let mirror2 = Mirror {
            name: "TestConda2".to_string(),
            url: "https://mirrors.ustc.edu.cn/anaconda".to_string(),
        };
        // Sleep for backup timestamp
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        manager.set_source(&mirror2).await?;
        assert_eq!(
            manager.current_url().await?,
            Some("https://mirrors.ustc.edu.cn/anaconda/pkgs/main".to_string())
        );

        // 5. Restore
        manager.restore().await?;
        // Should be back to mirror 1
        assert_eq!(
            manager.current_url().await?,
            Some("https://mirrors.tuna.tsinghua.edu.cn/anaconda/pkgs/main".to_string())
        );

        Ok(())
    }
}
