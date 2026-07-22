use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::env;
use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: i64,
    pub iat: i64,
}

pub fn create_token(user_id: Uuid) ->Result<String,  AppError> {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let now = chrono::Utc::now();
    let claims = Claims {
        sub: user_id,
        iat:now.timestamp(),
        exp: (now + chrono::Duration::hours(24)).timestamp(),
    };
    Ok(encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))?)
}