use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Registry {
    #[serde(rename = "tool")]
    pub tools: Vec<Tool>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Tool {
    pub name: String,
    pub path: String,
    pub parser: String,
    pub section: Option<String>,
    pub static_binds: Option<Vec<Vec<String>>>,
}

impl Tool {
    pub fn expanded_path(&self) -> PathBuf {
        PathBuf::from(shellexpand::tilde(&self.path).into_owned())
    }
}

pub fn load(path: &str) -> Result<Registry, Box<dyn std::error::Error>> {
    let raw = std::fs::read_to_string(path)?;
    Ok(toml::from_str(&raw)?)
}
