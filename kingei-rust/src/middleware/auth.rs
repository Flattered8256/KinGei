use std::env;

use crate::error::AppError;
use crate::error::user::UserError;
use crate::utils::auth::{ACCESS_TOKEN_DURATION_MINUTES, verify_access_token};
use axum::http::header::{HeaderValue, SET_COOKIE};
use axum::http::{HeaderMap, header};
use axum::response::{AppendHeaders, IntoResponse};
use axum::{extract::Request, middleware::Next, response::Response};

pub const ACCESS_COOKIE: &str = "kingei.access_token";
const ACCESS_COOKIE_MAX_AGE: i64 = ACCESS_TOKEN_DURATION_MINUTES * 60;

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

pub fn with_access_cookie<T: IntoResponse>(token: &str, body: T) -> impl IntoResponse + use<T> {
    (AppendHeaders([(SET_COOKIE, access_cookie(token))]), body)
}

pub fn cleared_access_cookie() -> impl IntoResponse + use<> {
    (
        AppendHeaders([(SET_COOKIE, clear_access_cookie())]),
        axum::http::StatusCode::NO_CONTENT,
    )
}

fn access_cookie(token: &str) -> HeaderValue {
    cookie_header(token, ACCESS_COOKIE_MAX_AGE)
}

fn clear_access_cookie() -> HeaderValue {
    cookie_header("", 0)
}

fn cookie_header(value: &str, max_age: i64) -> HeaderValue {
    let same_site = cookie_same_site();
    let mut buf = format!(
        "{ACCESS_COOKIE}={value}; Path=/; Max-Age={max_age}; HttpOnly; SameSite={same_site}"
    );
    if same_site == "None" {
        buf.push_str("; Secure");
    }
    HeaderValue::from_str(&buf).expect("access cookie is ascii")
}

fn cookie_same_site() -> &'static str {
    match env::var("COOKIE_SAMESITE") {
        Ok(value) if value.eq_ignore_ascii_case("none") => "None",
        _ => "Lax",
    }
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
