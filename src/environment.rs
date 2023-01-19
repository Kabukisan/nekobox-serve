use serde::{Deserialize, Serialize};
use crate::environment::AuthDriver::Sqlite;
use crate::environment::QueueDriver::Redis;

#[derive(Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Config {
    app: AppSection,
    auth: AuthSection,
    database: DatabaseSection,
    redis: Option<RedisSection>,
    sqlite: Option<SqliteSection>,
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct AppSection {
    name: String,
    key: String,
}

impl Default for AppSection {
    fn default() -> Self {
        AppSection {
            name: "Nekobox".to_string(),
            key: "".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct AuthSection {
    allow_registration: bool,
    token_secret: String,
    token_timeout: i32,
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct DatabaseSection {
    queue_driver: QueueDriver,
    auth_driver: AuthDriver,
}

impl Default for DatabaseSection {
    fn default() -> Self {
        DatabaseSection {
            queue_driver: Redis,
            auth_driver: Sqlite,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueueDriver {
    Redis,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum AuthDriver {
    Sqlite,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct RedisSection {
    host: String,
    port: u32,
    username: Option<String>,
    password: Option<String>,
    use_tls: bool,
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

#[derive(Serialize, Deserialize, Debug)]
pub struct SqliteSection {
    file: String,
}

impl Default for SqliteSection {
    fn default() -> Self {
        SqliteSection {
            file: "nekobox.db".to_string(),
        }
    }
}