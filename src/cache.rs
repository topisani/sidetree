use crate::file_tree::ExpandedPaths;
use std::fs::File;
use std::path::{Path, PathBuf};

#[derive(Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct Cache {
  #[serde(default)]
  pub selected_path: PathBuf,
  
  #[serde(default)]
  pub expanded_paths: ExpandedPaths,
}

impl Cache {
  pub fn from_file(path: &Path) -> Result<Cache, String> {
    if !path.exists() {
      File::create(path).expect("Cannot create config file");
    }
    match std::fs::read_to_string(path) {
      Ok(contents) => toml::from_str(&contents).map_err(|e| e.to_string()),
      Err(err) => Err(err.to_string()),
    }
  }

  pub fn write_file(&self, path: &Path) {
    std::fs::write(
      path,
      toml::to_string(self).expect("Couldn't serialize cache"),
    )
    .expect("Couldn't write cache to file");
  }

  pub fn default_file_path() -> PathBuf {
    let xdg = xdg::BaseDirectories::with_prefix("sidetree").unwrap();
    xdg
      .place_cache_file("sidetreecache.toml")
      .expect("Cannot create cache directory")
  }
}
