use crate::environment::AuthDriver::Sqlite;
use crate::environment::QueueDriver::Redis;
use directories::UserDirs;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use std::{fs, io};

static BUILD_VARIANT: Option<&str> = option_env!("BUILD_VARIANT");

lazy_static! {
    #[derive(Debug)]
    pub static ref CONFIG: Mutex<Config> = Mutex::new(load_environment_config());
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Config {
    pub app: AppSection,
    pub auth: AuthSection,
    pub database: DatabaseSection,
    pub redis: Option<RedisSection>,
    pub sqlite: Option<SqliteSection>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            app: AppSection::default(),
            auth: AuthSection::default(),
            database: DatabaseSection::default(),
            redis: Some(RedisSection::default()),
            sqlite: Some(SqliteSection::default()),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct AppSection {
    pub name: String,
    pub key: String,
}

impl Default for AppSection {
    fn default() -> Self {
        AppSection {
            name: "Nekobox".to_string(),
            key: "".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct AuthSection {
    pub allow_registration: bool,
    pub token_secret: String,
    pub token_timeout: i32,
}

impl Default for AuthSection {
    fn default() -> Self {
        AuthSection {
            allow_registration: false,
            token_secret: "".to_string(),
            token_timeout: 120,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct DatabaseSection {
    pub queue_driver: QueueDriver,
    pub auth_driver: AuthDriver,
}

impl Default for DatabaseSection {
    fn default() -> Self {
        DatabaseSection {
            queue_driver: Redis,
            auth_driver: Sqlite,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueueDriver {
    Redis,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum AuthDriver {
    Sqlite,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct RedisSection {
    pub host: String,
    pub port: u32,
    pub username: Option<String>,
    pub password: Option<String>,
    pub use_tls: bool,
}

impl Default for RedisSection {
    fn default() -> Self {
        RedisSection {
            host: "localhost".to_string(),
            port: 6379,
            username: None,
            password: None,
            use_tls: bool::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SqliteSection {
    pub file: String,
}

impl Default for SqliteSection {
    fn default() -> Self {
        SqliteSection {
            file: "nekobox.db".to_string(),
        }
    }
}

pub fn provide_cache_subdir(ident: &str) -> Option<PathBuf> {
    let cache_dir = provide_cache_dir()?.join(ident);

    if cache_dir.exists() == false {
        fs::create_dir(&cache_dir).expect("Can't create cache subdir");
    }

    Some(cache_dir)
}

pub fn delete_cache_subdir(ident: &str) -> Result<bool, io::Error> {
    let cache_dir = match provide_cache_dir() {
        Some(directory) => directory.join(ident),
        None => return Ok(false),
    };
    fs::remove_dir_all(&cache_dir)?;
    Ok(true)
}

pub fn provide_cache_dir() -> Option<PathBuf> {
    let (_, cache_path) = provide_directories().expect("Cannot provide cache path");

    if cache_path.exists() == false {
        fs::create_dir(&cache_path).expect("Can't create cache directory");
    }

    Some(cache_path)
}

pub fn load_environment_config() -> Config {
    let (cfg_path, _) = provide_directories().expect("Cannot provide configuration file");

    let config_file = cfg_path.join("config.toml");

    let config_string = fs::read_to_string(&config_file).expect("Can't read configuration file");

    let config: Config = toml::from_str(&config_string).expect("Can't parse toml file correctly");

    config
}

pub fn provide_directories() -> Result<(PathBuf, PathBuf), io::Error> {
    let (cfg_path, cache_path) = match BUILD_VARIANT.unwrap_or("standalone") {
        "system" => system_directories(),
        "user" => user_directories(),
        _ => standalone_directories(),
    }?;

    Ok((cfg_path, cache_path))
}

fn standalone_directories() -> Result<(PathBuf, PathBuf), io::Error> {
    let cfg_path = std::env::current_dir()?;
    let cache_path = cfg_path.clone().join("cache");

    Ok((cfg_path, cache_path))
}

fn system_directories() -> Result<(PathBuf, PathBuf), io::Error> {
    let (cfg_path_string, cache_path_string) = if cfg!(target_os = "macos") {
        let cfg = option_env!("BUILD_MACOS_SYSTEM_CFG").unwrap_or("/etc/nekobox/");
        let cache = option_env!("BUILD_MACOS_SYSTEM_CACHE").unwrap_or("/tmp/nekobox/");

        (cfg, cache)
    } else if cfg!(target_od = "windows") {
        let cfg = option_env!("BUILD_WINDOWS_SYSTEM_CFG").unwrap_or("{BINARY_DIR}/config/");
        let cache = option_env!("BUILD_WINDOWS_SYSTEM_CACHE").unwrap_or("{BINARY_DIR}/cache/");

        (cfg, cache)
    } else if cfg!(target_os = "linux") {
        let cfg = option_env!("BUILD_LINUX_SYSTEM_CFG").unwrap_or("/etc/nekobox/");
        let cache = option_env!("BUILD_LINUX_SYSTEM_CACHE").unwrap_or("/tmp/nekobox/");

        (cfg, cache)
    } else {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Unsupported environment",
        ));
    };

    let cfg_path: PathBuf = interpret_value(cfg_path_string)?.into();
    let cache_path: PathBuf = interpret_value(cache_path_string)?.into();

    Ok((cfg_path, cache_path))
}

fn user_directories() -> Result<(PathBuf, PathBuf), io::Error> {
    let (cfg_path_string, cache_path_string) = if cfg!(target_os = "macos") {
        let cfg = option_env!("BUILD_MACOS_USER_CFG")
            .unwrap_or("{USER_DIR}/Library/Preferences/com.github.nekobox/");
        let cache = option_env!("BUILD_MACOS_USER_CACHE")
            .unwrap_or("{USER_DIR}/Library/Caches/com.github.nekobox/");

        (cfg, cache)
    } else if cfg!(target_os = "windows") {
        let cfg =
            option_env!("BUILD_WINDOWS_USER_CFG").unwrap_or("{USER_DIR}/AppData/Local/nekobox/");
        let cache = option_env!("BUILD_WINDOWS_USER_CACHE")
            .unwrap_or("{USER_DIR}/AppData/Local/nekobox/cache/");

        (cfg, cache)
    } else if cfg!(target_os = "linux") {
        let cfg = option_env!("BUILD_LINUX_USER_CFG").unwrap_or("{USER_DIR}/.nekobox/");
        let cache = option_env!("BUILD_LINUX_USER_CACHE").unwrap_or("{USER_DIR}/.nekobox/cache/");

        (cfg, cache)
    } else {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Unsupported environment",
        ));
    };

    let cfg_path: PathBuf = interpret_value(cfg_path_string)?.into();
    let cache_path: PathBuf = interpret_value(cache_path_string)?.into();

    Ok((cfg_path, cache_path))
}

fn interpret_value(key: &str) -> Result<String, io::Error> {
    let binary_dir = std::env::current_dir()?
        .to_str()
        .ok_or(io::Error::new(
            io::ErrorKind::Other,
            "Cannot convert path to str",
        ))?
        .to_string();
    let user_dir = {
        let user_dirs = UserDirs::new().ok_or(io::Error::new(
            io::ErrorKind::NotFound,
            "Cannot find USER_DIR in this environment",
        ))?;
        user_dirs
            .home_dir()
            .to_path_buf()
            .to_str()
            .ok_or(io::Error::new(
                io::ErrorKind::Other,
                "Cannot convert path to str",
            ))?
            .to_string()
    };

    let result = key
        .replace("{BINARY_DIR}", &binary_dir)
        .replace("{USER_DIR}", &user_dir)
        .to_string();

    Ok(result)
}
