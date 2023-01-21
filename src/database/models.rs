use chrono::Utc;
use serde::{Deserialize, Serialize};
use crate::auth::make_hash;
use crate::models::AuthRegisterRequest;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    pub id: Option<u32>,
    pub email: String,
    pub username: String,
    pub password: Option<String>,
    pub created_at: String,
}

impl User {
    pub fn new<S, P>(email: S, username: S, password: P) -> Self
        where S: Into<String>, P: Into<Option<String>>
    {
        User {
            id: None,
            email: email.into(),
            username: username.into(),
            password: password.into(),
            created_at: Utc::now().to_rfc3339(),
        }
    }
}

impl From<AuthRegisterRequest> for User {
    fn from(value: AuthRegisterRequest) -> Self {
        User {
            id: None,
            email: value.email,
            username: value.username,
            password: Some(make_hash(&value.password)),
            created_at: Utc::now().to_rfc3339(),
        }
    }
}