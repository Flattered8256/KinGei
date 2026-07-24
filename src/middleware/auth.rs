use axum::{extract::Request, middleware::Next, response::Response};
use crate::utils::auth:: verify_access_token;
use crate::error::AppError;
use crate::error::user::UserError;
pub async fn auth_middleware(mut req: Request, next: Next) -> Result<Response, AppError>{
    let auth_header = req.headers().get("Authorization").and_then(|v| v.to_str().ok());

    let token = match auth_header {
        Some(value) if value.starts_with("Bearer ") => {
            value.trim_start_matches("Bearer ")
        },
        _ => {
            return Err(AppError::User(UserError::InvalidCredentials));
        }
    };

    let claims = verify_access_token(token)?;

    req.extensions_mut().insert(claims.sub);

    Ok(next.run(req).await)
}
