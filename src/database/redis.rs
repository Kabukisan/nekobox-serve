#![allow(dead_code)]
#![allow(unused_variables)]

use redis::{Client, Connection};
use std::env;

pub fn open_task_db_connection() -> Connection {
    open_redis_db_connection("0")
}

pub fn open_redis_db_connection(db: &str) -> Connection {
    let redis_host_name =
        env::var("REDIS_HOSTNAME").expect("missing environment variable REDIS_HOSTNAME");

    let redis_port = env::var("REDIS_PORT").unwrap_or("6379".to_string());

    let redis_username = env::var("REDIS_USERNAME").unwrap_or_default();

    let redis_password = env::var("REDIS_PASSWORD").unwrap_or_default();

    let uri_scheme = match env::var("REDIS_USE_TLS") {
        Ok(_) => "rediss",
        Err(_) => "redis",
    };

    let redis_url_connection = format!(
        "{}://{}:{}@{}:{}/{}",
        uri_scheme, redis_username, redis_password, redis_host_name, redis_port, db
    );

    let connection = Client::open(redis_url_connection)
        .expect("Invalid connection url")
        .get_connection()
        .expect("Failed to connect to Redis");

    connection
}
