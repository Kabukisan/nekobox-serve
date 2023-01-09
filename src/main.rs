use std::net::SocketAddr;
use std::path::PathBuf;
use axum::{routing::{get, post, delete}, http::StatusCode, response::IntoResponse, Json, Router};
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;
use redis::{Commands, Connection, RedisResult};
use serde::{Serialize, Deserialize};
use serde_json::json;
use crate::database::open_task_db_connection;
use crate::models::{TaskCreateRequest, TaskRequest, TaskResponse};
use crate::services::YoutubeDl;
use crate::traits::Delivery;

mod wrapper;
mod models;
mod services;
mod database;
mod traits;

#[tokio::main]
async fn main() {
    std::env::set_var("DEFAULT_SERVICE", "youtube-dl");
    std::env::set_var("REDIS_HOSTNAME", "127.0.0.1");

    let app = Router::new()
        .route("/task/create", post(create_task))
        .route("/task/status", post(status_task))
        .route("/task/delete", delete(delete_task))
        .route("/task/deliver", post(deliver_task));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn create_task(Json(payload): Json<TaskCreateRequest>) -> impl IntoResponse {
    let mut task_db = open_task_db_connection();
    let status_id = determine_unique_status_id(&mut task_db);

    let response = TaskResponse::pending(&status_id);
    let response_json = serde_json::to_string(&response)
        .expect("can't parse json (Invalid Json)");
    task_db.set::<_, String, String>(status_id, response_json)
        .expect("can't store task");

    // TODO: Initial download process, maybe push into a queue or something.
    // It could be a simple download process which start immediately
    // after call function und create the thread.
    // For better handling and limitation of simultaneous downloads a queue
    // implementation would be a better idea.

    (StatusCode::CREATED, Json(response))
}

fn determine_unique_status_id(task_db: &mut Connection) -> String {
    let mut status_id;
    loop {
        status_id = generate_random_status_id();
        if let _ = task_db.get::<&str, String>(&status_id).is_err() {
            break;
        }
    };
    status_id
}

fn generate_random_status_id() -> String {
    let rnd_status_id = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();

    rnd_status_id
}

async fn status_task(Json(payload): Json<TaskRequest>) -> impl IntoResponse {
    todo!()
}

async fn delete_task(Json(payload): Json<TaskRequest>) -> impl IntoResponse {
    todo!()
}

async fn deliver_task(Json(payload): Json<TaskRequest>) -> impl IntoResponse {
    todo!()
}