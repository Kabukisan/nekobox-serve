#![allow(dead_code)]
#![allow(unused_variables)]

const DEFAULT_SALT_COST: u32 = 10;

use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use base64::{Engine, engine::general_purpose::STANDARD as Base64Std};
use crypto::bcrypt::bcrypt;
use serde::{Serialize, Deserialize};
use jsonwebtoken::{
    encode,
    decode,
    Header,
    Validation,
    EncodingKey,
    DecodingKey,
    TokenData,
    errors::Result as JwtResult,
};
use crate::environment::CONFIG;
use crate::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub id: usize,
    pub email: String,
    pub username: String,
    pub exp: usize,
}

impl Claims {
    pub fn new<S: Into<String>>(id: usize, email: S, username: S, exp: usize) -> Claims {
        Claims {
            id,
            email: email.into(),
            username: username.into(),
            exp,
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let token_header = parts.headers.get("Authorization");
        let token_string = match token_header {
            Some(value) => {
                let token = value.to_str().unwrap();
                token[7..token.len()].to_string()
            }
            None => return Err(Error::InvalidToken)
        };
        let token = validate_jwt(&token_string)?;

        Ok(token.claims)
    }
}

pub fn generate_jwt(claims: &Claims) -> JwtResult<String> {
    let secret = &CONFIG.lock().unwrap().auth.token_secret;
    encode(&Header::default(), claims, &EncodingKey::from_secret(secret.as_ref()))
}

pub fn renew_jwt(token: &str) -> JwtResult<String> {
    let token = validate_jwt(token)?;
    let claim = token.claims.clone();
    generate_jwt(&claim)
}

pub fn validate_jwt(token: &str) -> JwtResult<TokenData<Claims>> {
    let secret = &CONFIG.lock().unwrap().auth.token_secret;
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default()
    )
}

pub fn make_hash(value: &str) -> String {
    let key = CONFIG.lock().unwrap().app.key.clone();
    let salt = key.as_bytes();

    let mut output = [0u8; 24];
    bcrypt(DEFAULT_SALT_COST, salt, value.as_bytes(), &mut output);

    Base64Std.encode(output)
}
