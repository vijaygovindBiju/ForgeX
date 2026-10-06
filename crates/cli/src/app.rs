use std::path::{Path, PathBuf};
use chrono::Utc;
use directories::ProjectDirs;
use forgex_core::{Commitment, ForgeConfig, ForgeError, Result};
use forgex_storage::ForgeDb;

pub struct AppContext {
    pub db: ForgeDb,
    pub config: ForgeConfig,
    pub db_path: PathBuf,
    pub config_path: PathBuf,
}

impl AppContext {
    pub fn init(custom_db: Option<PathBuf>, custom_config: Option<PathBuf>) -> Result<Self> {
        let (db_path, config_path) = Self::resolve_paths(custom_db, custom_config)?;

        // Ensure directories exist
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| ForgeError::Storage(format!("Failed to create db dir: {}", e)))?;
        }
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| ForgeError::Config(format!("Failed to create config dir: {}", e)))?;
        }

        let db = ForgeDb::open(&db_path)?;
        let config = Self::load_config(&config_path)?;

        Ok(Self {
            db,
            config,
            db_path,
            config_path,
        })
    }

    fn resolve_paths(
        custom_db: Option<PathBuf>,
        custom_config: Option<PathBuf>,
    ) -> Result<(PathBuf, PathBuf)> {
        let proj_dirs = ProjectDirs::from("com", "forgex", "forgex");

        let db_path = if let Some(p) = custom_db {
            p
        } else if let Ok(env_db) = std::env::var("FORGEX_DB") {
            PathBuf::from(env_db)
        } else if let Some(ref dirs) = proj_dirs {
            dirs.data_dir().join("forgex.db")
        } else {
            PathBuf::from("forgex.db")
        };

        let config_path = if let Some(p) = custom_config {
            p
        } else if let Ok(env_cfg) = std::env::var("FORGEX_CONFIG") {
            PathBuf::from(env_cfg)
        } else if let Some(ref dirs) = proj_dirs {
            dirs.config_dir().join("config.toml")
        } else {
            PathBuf::from("config.toml")
        };

        Ok((db_path, config_path))
    }

    pub fn load_config(path: &Path) -> Result<ForgeConfig> {
        if path.exists() {
            let content = std::fs::read_to_string(path)
                .map_err(|e| ForgeError::Config(format!("Failed to read config file: {}", e)))?;
            let config: ForgeConfig = toml::from_str(&content)
                .map_err(|e| ForgeError::Config(format!("Failed to parse config file: {}", e)))?;
            Ok(config)
        } else {
            Ok(ForgeConfig::default())
        }
    }

    pub fn save_config(&self) -> Result<()> {
        let content = toml::to_string_pretty(&self.config)
            .map_err(|e| ForgeError::Config(e.to_string()))?;
        std::fs::write(&self.config_path, content)
            .map_err(|e| ForgeError::Config(format!("Failed to write config: {}", e)))?;
        Ok(())
    }

    pub fn find_commitment_by_prefix(&self, prefix: &str) -> Result<Commitment> {
        let prefix = prefix.trim();
        let all = self.db.list_commitments()?;

        let matches: Vec<Commitment> = all
            .into_iter()
            .filter(|c| c.id.to_string().starts_with(prefix) || c.id.to_string().replace('-', "").starts_with(prefix))
            .collect();

        if matches.is_empty() {
            Err(ForgeError::NotFound(format!("No commitment found matching ID prefix '{}'", prefix)))
        } else if matches.len() > 1 {
            Err(ForgeError::Validation(format!(
                "Prefix '{}' is ambiguous (matches {} commitments). Please specify more characters.",
                prefix,
                matches.len()
            )))
        } else {
            Ok(matches.into_iter().next().unwrap())
        }
    }

    /// Checks for overdue planned or active commitments
    pub fn check_overdue_commitments(&self) -> Result<Vec<Commitment>> {
        let all = self.db.list_commitments()?;
        let now = Utc::now();
        let overdue: Vec<Commitment> = all
            .into_iter()
            .filter(|c| c.is_overdue(now, self.config.grace_period_mins))
            .collect();
        Ok(overdue)
    }
}
