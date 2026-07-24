use jsonwebtoken::{EncodingKey, Header, TokenData, encode, decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::env;
use crate::error::AppError;
use crate::error::user::UserError;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: i64,
    pub iat: i64,
    pub token_type: TokenType,
}

const ACCESS_TOKEN_DURATION_MINUTES: i64 = 15;          // Access Token 15 分钟
const REFRESH_TOKEN_DURATION_DAYS: i64 = 7;        // Refresh Token 7 天

fn get_secret() -> String {
    env::var("JWT_SECRET").expect("JWT_SECRET must be set")
}

fn create_token_internal(user_id: Uuid, token_type: TokenType) -> Result<String, AppError> {
    let secret = get_secret();
    let now = chrono::Utc::now();
    let exp = match token_type {
        TokenType::Access => now + chrono::Duration::minutes(ACCESS_TOKEN_DURATION_MINUTES),
        TokenType::Refresh => now + chrono::Duration::days(REFRESH_TOKEN_DURATION_DAYS),
    };
    let claims = Claims {
        sub: user_id,
        iat: now.timestamp(),
        exp: exp.timestamp(),
        token_type,
    };
    Ok(encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))?)
}

pub fn create_access_token(user_id: Uuid) -> Result<String, AppError> {
    create_token_internal(user_id, TokenType::Access)
}

pub fn create_refresh_token(user_id: Uuid) -> Result<String, AppError> {
    create_token_internal(user_id, TokenType::Refresh)
}

fn verify_token_generic(token: &str) -> Result<Claims, AppError> {
    let secret = get_secret();
    let data: TokenData<Claims> = decode(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    ).map_err(|_| AppError::User(UserError::InvalidToken))?;
    Ok(data.claims)
}

pub fn verify_access_token(token: &str) -> Result<Claims, AppError> {
    let claims = verify_token_generic(token)?;
    if claims.token_type != TokenType::Access {
        return Err(AppError::User(UserError::InvalidToken));
    }
    Ok(claims)
}

pub fn verify_refresh_token(token: &str) -> Result<Claims, AppError> {
    let claims = verify_token_generic(token)?;
    if claims.token_type != TokenType::Refresh {
        return Err(AppError::User(UserError::InvalidToken));
    }
    Ok(claims)
}
