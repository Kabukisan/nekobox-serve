#![allow(dead_code)]
#![allow(unused_variables)]

const DEFAULT_SALT_COST: u32 = 10;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    username: String,
    exp: usize,
}

impl Claims {
    pub fn new<S: Into<String>>(username: S, exp: usize) -> Claims {
        Claims {
            username: username.into(),
            exp,
        }
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
