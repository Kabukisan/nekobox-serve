#![allow(dead_code)]
#![allow(unused_variables)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    Video,
    Audio,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaFormat {
    Aac,
    Flv,
    M4a,
    Mp3,
    Mp4,
    Ogg,
    Wav,
    Webm,
}

#[derive(Serialize, Deserialize)]
pub struct TaskCreateRequest {
    pub url: String,
    pub media_type: MediaType,
    pub format: MediaFormat,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Complete,
    Error,
}


#[derive(Serialize, Deserialize)]
pub struct TaskRequest {
    status_id: String,
}

impl TaskRequest {
    pub fn new<S: Into<String>>(status_id: S) -> Self {
        TaskRequest {
            status_id: status_id.into(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct TaskResponse {
    pub status_id: String,
    pub status: TaskStatus,
    pub message: Option<String>,
    pub percentage: f32,
}

impl TaskResponse {
    pub fn new<S: Into<String>>(status_id: S, status: TaskStatus, message: Option<String>, percentage: f32) -> Self {
        TaskResponse {
            status_id: status_id.into(),
            status,
            message,
            percentage,
        }
    }

    pub fn pending<S: Into<String>>(status_id: S) -> Self {
        Self::new(status_id, TaskStatus::Pending, None, 0.0)
    }

    pub fn complete<S: Into<String>, M: Into<Option<String>>>(status_id: S, message: M) -> Self {
        Self::new(status_id, TaskStatus::Complete, message.into(), 1.0)
    }

    pub fn error<S: Into<String>, M: Into<Option<String>>>(status_id: S, message: M) -> Self {
        Self::new(status_id, TaskStatus::Error, message.into(), 0.0)
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
    pub expires: DateTime<Utc>,
}

impl AuthLoginResponse {
    pub fn new<S: Into<String>>(token: S, expires: DateTime<Utc>) -> Self {
        AuthLoginResponse {
            token: token.into(),
            expires,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AuthRegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
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