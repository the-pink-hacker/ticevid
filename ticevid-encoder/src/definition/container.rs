use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::Deserialize;

use crate::definition::title::TitleDefinition;

#[derive(Debug, Deserialize)]
pub struct ContainerDefinition {
    /// A list of paths relative from the folder containing the container to each title.
    pub titles: Vec<TitleDefinition>,
    /// The font pack to be used for the container.
    #[serde(default)]
    pub font_pack: Option<PathBuf>,
}

impl ContainerDefinition {
    pub async fn load(path: &Path) -> anyhow::Result<Self> {
        let raw = tokio::fs::read_to_string(path)
            .await
            .with_context(|| format!("Failed to load container at {}", path.display()))?;

        toml::from_str(&raw)
            .with_context(|| format!("Failed to parse the container at {}", path.display()))
    }
}
