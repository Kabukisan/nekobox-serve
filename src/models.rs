#![allow(dead_code)]
#![allow(unused_variables)]

use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    Video,
    Audio,
}

#[derive(Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Aac,
    Flv,
    M4a,
    Mp3,
    Mp4,
    Ogg,
    Wav,
    Webm,
}

#[derive(Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AudioFormat {
    Best,
    Aac,
    Flac,
    Mp3,
    M4a,
    Opus,
    Vorbis,
    Wav,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct TaskCreateRequest {
    #[validate(url)]
    pub url: String,
    pub media_type: MediaType,
    pub format: Format,
}

#[derive(Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Complete,
    Error,
}


#[derive(Serialize, Deserialize, Validate)]
pub struct TaskRequest {
    #[validate(length(equal = 16))]
    status_id: String,
}

impl TaskRequest {
    pub fn new<S: Into<String>>(status_id: S) -> Self {
        TaskRequest {
            status_id: status_id.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Validate)]
pub struct TaskResponse {
    #[validate(length(equal = 16))]
    pub status_id: String,
    pub user_id: usize,
    pub status: TaskStatus,
    pub message: Option<String>,
    pub percentage: f32,
}

impl TaskResponse {
    pub fn new<S: Into<String>>(status_id: S, user_id: usize, status: TaskStatus, message: Option<String>, percentage: f32) -> Self {
        TaskResponse {
            status_id: status_id.into(),
            user_id,
            status,
            message,
            percentage,
        }
    }

    pub fn pending<S: Into<String>>(status_id: S, user_id: usize) -> Self {
        Self::new(status_id,  user_id,TaskStatus::Pending, None, 0.0)
    }

    pub fn complete<S: Into<String>, M: Into<Option<String>>>(status_id: S, user_id: usize , message: M) -> Self {
        Self::new(status_id, user_id, TaskStatus::Complete, message.into(), 1.0)
    }

    pub fn error<S: Into<String>, M: Into<Option<String>>>(status_id: S, user_id: usize, message: M) -> Self {
        Self::new(status_id, user_id, TaskStatus::Error, message.into(), 0.0)
    }
}

#[derive(Serialize, Deserialize)]
pub struct AuthLoginRequest {
    pub username: String,
    pub password: String,
}

impl AuthLoginRequest {
    pub fn new<S: Into<String>>(username: S, password: S) -> Self {
        AuthLoginRequest {
            username: username.into(),
            password: password.into(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AuthLoginResponse {
    pub token: String,
    pub expires: usize,
}

impl AuthLoginResponse {
    pub fn new<S: Into<String>>(token: S, expires: usize) -> Self {
        AuthLoginResponse {
            token: token.into(),
            expires,
        }
    }
}

#[derive(Serialize, Deserialize, Validate, Debug)]
pub struct AuthRegisterRequest {
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(must_match = "password")]
    pub password: String,
    #[validate(must_match(other = "password"))]
    pub password_confirm: String,
}

impl AuthRegisterRequest {
    pub fn new<S: Into<String>>(username: S, email: S, password: S, password_confirm: S) -> Self {
        AuthRegisterRequest {
            username: username.into(),
            email: email.into(),
            password: password.into(),
            password_confirm: password_confirm.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorResponse {
    pub response: u16,
    pub error: String,
    pub message: Option<String>,
}
