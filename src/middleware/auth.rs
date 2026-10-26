use crate::error::AppError;
use crate::error::user::UserError;
use crate::utils::auth::verify_access_token;
use axum::http::{HeaderMap, header};
use axum::{extract::Request, middleware::Next, response::Response};

pub const ACCESS_COOKIE: &str = "kingei.access_token";

pub async fn auth_middleware(req: Request, next: Next) -> Result<Response, AppError> {
    let token = bearer_token(req.headers())
        .ok_or(AppError::User(UserError::InvalidCredentials))?
        .to_owned();
    insert_user(req, next, &token).await
}

pub async fn auth_bearer_or_cookie(req: Request, next: Next) -> Result<Response, AppError> {
    let token = bearer_token(req.headers())
        .or_else(|| cookie_token(req.headers(), ACCESS_COOKIE))
        .ok_or(AppError::User(UserError::InvalidCredentials))?
        .to_owned();
    insert_user(req, next, &token).await
}

async fn insert_user(mut req: Request, next: Next, token: &str) -> Result<Response, AppError> {
    let claims = verify_access_token(token)?;
    req.extensions_mut().insert(claims.sub);
    Ok(next.run(req).await)
}

fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    let value = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    value.strip_prefix("Bearer ")
}

fn cookie_token<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    let cookie = headers.get(header::COOKIE)?.to_str().ok()?;
    cookie.split(';').find_map(|part| {
        let part = part.trim();
        let (key, value) = part.split_once('=')?;
        (key == name).then_some(value)
    })
}
