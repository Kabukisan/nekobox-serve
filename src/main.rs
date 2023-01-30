use crate::auth::{generate_jwt, make_hash, validate_jwt, Claims};
use crate::database::models::User;
use crate::database::redis::open_task_db_connection;
use crate::database::sqlite::{open_sqlite_db_connection, SqliteDatabaseHandler};
use crate::database::{del_task_response, get_task_response, set_task_response};
use crate::environment::{delete_cache_subdir, provide_cache_subdir};
use crate::error::Error;
use crate::models::{
    AuthLoginRequest, AuthLoginResponse, AuthRegisterRequest, ErrorResponse, MediaType,
    TaskCreateRequest, TaskRequest, TaskResponse, TaskStatus,
};
use crate::queue::Queue;
use crate::service::ytdl::YoutubeDl;
use crate::service::{FetchServiceEvents, FetchServiceHandler};
use axum::body::StreamBody;
use axum::http::{header, Request};
use axum::response::Response;
use axum::{
    http::StatusCode,
    middleware::{self, Next},
    response::AppendHeaders,
    response::IntoResponse,
    routing::{delete, post},
    Json, Router,
};
use chrono::{Duration, Utc};
use lazy_static::lazy_static;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use redis::{Commands, Connection};
use std::fmt::Debug;
use std::fs::DirEntry;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio_util::io::ReaderStream;
use validator::Validate;

mod auth;
mod database;
mod environment;
mod error;
mod models;
mod queue;
mod service;
mod traits;

lazy_static! {
    static ref QUEUE: Mutex<Queue> = Mutex::new(Queue::new());
}

#[tokio::main]
async fn main() {
    std::env::set_var("DEFAULT_SERVICE", "youtube-dl");
    std::env::set_var("REDIS_HOSTNAME", "127.0.0.1");

    let _ = QUEUE.lock().unwrap().schedule();

    let app = Router::new()
        .route("/task/create", post(create_task))
        .route("/task/status", post(status_task))
        .route("/task/delete", delete(delete_task))
        .route("/task/deliver", post(deliver_task))
        .route_layer(middleware::from_fn(auth_middleware))
        .route("/auth/login", post(login))
        .route("/auth/register", post(register));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn create_task(
    user: Claims,
    Json(payload): Json<TaskCreateRequest>,
) -> Result<Response, Error> {
    let mut task_db = open_task_db_connection();
    let status_id = determine_unique_status_id(&mut task_db);
    let mut response = TaskResponse::pending(&status_id, user.id);
    response.media_type = Some(payload.media_type);
    response.audio_format = payload.audio_format;

    let mut service: FetchServiceHandler<Box<YoutubeDl>> = FetchServiceHandler::new(
        &payload.url,
        Box::new(YoutubeDl::service()),
        payload.media_type,
        payload.audio_format,
    );

    service.prepare(&response, &payload.url)?;
    let collection = service.collect()?;
    response.collection = Some(collection);

    let response_json = serde_json::to_string(&response)?;
    task_db.set::<_, String, String>(&status_id, response_json)?;

    service.add_observer(Arc::new(Mutex::new(TestObserver(TaskRequest::new(
        &status_id,
    )))));

    QUEUE.lock().unwrap().push(service);

    Ok((StatusCode::CREATED, Json(response)).into_response())
}

struct TestObserver(TaskRequest);

impl FetchServiceEvents for TestObserver {
    fn on_start(&mut self) {}

    fn on_end(&mut self) {}

    fn on_complete(&mut self) {
        let mut task = get_task_response(&self.0).unwrap();
        task.status = TaskStatus::Complete;
        task.percentage = 1.0;
        set_task_response(&task).unwrap();
    }

    fn on_error(&mut self) {}
}

fn determine_unique_status_id(task_db: &mut Connection) -> String {
    let mut status_id;
    loop {
        status_id = generate_random_status_id();
        if task_db.get::<&str, String>(&status_id).is_err() {
            break;
        }
    }
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

async fn status_task(user: Claims, Json(payload): Json<TaskRequest>) -> Result<Response, Error> {
    let response = get_task_response(&payload)?;
    if response.user_id != user.id {
        return Ok((StatusCode::UNAUTHORIZED).into_response());
    }

    Ok((StatusCode::OK, Json(response)).into_response())
}

async fn delete_task(user: Claims, Json(payload): Json<TaskRequest>) -> Result<Response, Error> {
    let response = get_task_response(&payload)?;
    if response.user_id != user.id {
        return Ok((StatusCode::UNAUTHORIZED).into_response());
    }

    del_task_response(&payload)?;
    delete_cache_subdir(&response.status_id)?;

    Ok((StatusCode::OK).into_response())
}

async fn deliver_task(user: Claims, Json(payload): Json<TaskRequest>) -> Result<Response, Error> {
    let response = get_task_response(&payload)?;
    if response.user_id != user.id {
        return Ok((StatusCode::UNAUTHORIZED).into_response());
    }

    let cache_dir = match provide_cache_subdir(&payload.status_id) {
        Some(directory) => directory,
        None => return Err(Error::InternalError),
    };

    send_file(response.media_type.unwrap_or(MediaType::Audio), &cache_dir).await
}

async fn send_file(media_type: MediaType, path: &Path) -> Result<Response, Error> {
    let files = match media_type {
        MediaType::Video => get_video_delivery(path),
        MediaType::Audio => get_audio_delivery(path),
    }?;

    let file_path = match files.first() {
        Some(file) => file.path(),
        None => return Err(Error::InternalError),
    };

    let file_name = match &file_path.file_name() {
        Some(file_name) => file_name.to_str().unwrap(),
        None => return Err(Error::InternalError),
    };

    let file = tokio::fs::File::open(&file_path).await?;
    let stream = ReaderStream::new(file);
    let body = StreamBody::new(stream);

    let content_type = get_content_type(&file_path).unwrap_or("application/octet-stream");
    let content_disposition = format!("attachment; filename=\"{}\"", file_name);

    let headers = AppendHeaders([
        (header::CONTENT_TYPE, content_type),
        (header::CONTENT_DISPOSITION, &content_disposition),
    ]);

    Ok((headers, body).into_response())
}

fn get_content_type(path: &Path) -> Option<&str> {
    let extension = path.extension()?.to_str();
    let content_type = match extension {
        Some("aac") => "audio/aac",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("mp3") => "audio/mpeg",
        Some("mp4") => "video/mp4",
        Some("oga") => "audio/ogg",
        Some("ogv") => "video/ogg",
        Some("opus") => "audio/opus",
        Some("png") => "image/png",
        Some("wav") => "audio/wave",
        Some("weba") => "audio/webm",
        Some("webm") => "video/webm",
        Some("webp") => "image/webp",
        _ => "application/octet-stream",
    };

    Some(content_type)
}

fn get_video_delivery(path: &Path) -> Result<Vec<DirEntry>, std::io::Error> {
    let video_extensions = vec!["3gp", "flv", "mp4", "webm"];
    get_delivery_entry_vec(path, &video_extensions)
}

fn get_audio_delivery(path: &Path) -> Result<Vec<DirEntry>, std::io::Error> {
    let audio_extensions = vec!["aac", "flac", "mp3", "m4a", "opus", "vorbis", "ogg", "wav"];
    get_delivery_entry_vec(path, &audio_extensions)
}

fn get_image_delivery(path: &Path) -> Result<Vec<DirEntry>, std::io::Error> {
    let image_extensions = vec!["jpg", "jpeg", "png", "webp"];
    get_delivery_entry_vec(path, &image_extensions)
}

fn get_delivery_entry_vec(
    path: &Path,
    extensions: &Vec<&str>,
) -> Result<Vec<DirEntry>, std::io::Error> {
    let entries: Vec<DirEntry> = std::fs::read_dir(path)?
        .filter(|file| {
            let file_path = match file {
                Ok(directory) => directory.path(),
                Err(_) => return false,
            };

            let file_extension = match file_path.extension() {
                Some(extension) => extension.to_str().unwrap(),
                None => return false,
            };

            if extensions.contains(&file_extension) {
                return true;
            }

            false
        })
        .map(|entry| entry.unwrap())
        .collect();

    Ok(entries)
}

async fn login(Json(payload): Json<AuthLoginRequest>) -> Response {
    let connection = open_sqlite_db_connection();
    let db_handler = SqliteDatabaseHandler::new(&connection);

    let user_result = match db_handler.find_user_by_name(&payload.username) {
        Some(user_result) => user_result,
        None => {
            let response = ErrorResponse {
                response: StatusCode::NOT_FOUND.as_u16(),
                error: "UserNotFound".to_string(),
                message: None,
            };
            return (StatusCode::NOT_FOUND, Json(response)).into_response();
        }
    };

    match user_result {
        Ok(user) => {
            if Some(make_hash(&payload.password)) == user.password {
                let token = generate_token_response_for_user(&user);
                (StatusCode::OK, Json(token)).into_response()
            } else {
                let response = ErrorResponse {
                    response: StatusCode::UNAUTHORIZED.as_u16(),
                    error: "BadCredentials".to_string(),
                    message: None,
                };
                (StatusCode::UNAUTHORIZED, Json(response)).into_response()
            }
        }
        Err(_) => {
            let response = ErrorResponse {
                response: StatusCode::UNAUTHORIZED.as_u16(),
                error: "BadCredentials".to_string(),
                message: None,
            };
            (StatusCode::UNAUTHORIZED, Json(response)).into_response()
        }
    }
}

async fn register(Json(payload): Json<AuthRegisterRequest>) -> Response {
    if let Err(e) = payload.validate() {
        let response = Json(ErrorResponse {
            response: StatusCode::BAD_REQUEST.as_u16(),
            error: "ValidationError".to_string(),
            message: Some(e.to_string()),
        });
        return (StatusCode::INTERNAL_SERVER_ERROR, response).into_response();
    }

    let connection = open_sqlite_db_connection();
    let sqlite_helper = SqliteDatabaseHandler::new(&connection);
    let user = User::from(payload);

    match sqlite_helper.create_user(&user) {
        Ok(_) => {
            let token = generate_token_response_for_user(&user);
            (StatusCode::OK, Json(token)).into_response()
        }
        Err(e) => {
            let response = Json(ErrorResponse {
                response: StatusCode::BAD_REQUEST.as_u16(),
                error: e.to_string(),
                message: None,
            });
            (StatusCode::BAD_REQUEST, response).into_response()
        }
    }
}

fn generate_token_response_for_user(user: &User) -> AuthLoginResponse {
    let expires: usize = (Utc::now() + Duration::days(12)).timestamp() as usize;
    let claim = Claims::new(user.id.unwrap(), &user.email, &user.username, expires);
    let token = generate_jwt(&claim).unwrap();

    AuthLoginResponse { token, expires }
}

async fn auth_middleware<B: Debug>(
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let token = match request.headers().get("authorization") {
        Some(token) => {
            let token = token.to_str().unwrap();
            token[7..token.len()].to_string()
        }
        None => {
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    match validate_jwt(&token) {
        Ok(_) => {
            let response = next.run(request).await;
            Ok(response)
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}
