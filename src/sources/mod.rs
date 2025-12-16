pub mod apt;
pub mod brew;
pub mod cargo;
pub mod conda;
pub mod docker;
pub mod go;
pub mod npm;
pub mod pip;
pub mod uv;

use crate::error::{MirrorError, Result};
use crate::traits::SourceManager;

pub const SUPPORTED_TOOLS: &[&str] = &[
    "pip", "npm", "docker", "go", "cargo", "brew", "apt", "uv", "conda",
];

pub fn get_manager(name: &str) -> Result<Box<dyn SourceManager>> {
    match name.to_lowercase().as_str() {
        "pip" => Ok(Box::new(pip::PipManager::new())),
        "docker" => Ok(Box::new(docker::DockerManager::new())),
        "npm" => Ok(Box::new(npm::NpmManager::new())),
        "go" => Ok(Box::new(go::GoManager::new())),
        "cargo" => Ok(Box::new(cargo::CargoManager::new())),
        "brew" => Ok(Box::new(brew::BrewManager::new())),
        "apt" => Ok(Box::new(apt::AptManager::new())),
        "uv" => Ok(Box::new(uv::UvManager::new())),
        "conda" => Ok(Box::new(conda::CondaManager::new())),
        _ => Err(MirrorError::UnknownTool(format!(
            "Unsupported tool: '{}'. Available: {}",
            name,
            SUPPORTED_TOOLS.join(", ")
        ))),
    }
}
