use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;

use crate::utils::auth::{create_access_token, create_refresh_token, verify_refresh_token};
use crate::model::user::User;
use crate::repository::user::{UserRepository, UpdateUser};
use crate::serializer::user::{
    UserResponse, CreateUserRequest, LoginRequest,
    UpdateUserRequest, ChangePasswordRequest, LoginResponse,
    RefreshTokenRequest, RefreshTokenResponse,
};
use crate::error::user::UserError;
use crate::error::AppError;                     
pub struct UserService {
    repo: Arc<UserRepository>
}

impl UserService {
    pub fn new(repo: Arc<UserRepository>) -> Self {
        Self { repo }
    }

    pub async fn register(&self, request: CreateUserRequest) -> Result<UserResponse, AppError> {
        if self.repo.find_by_email(&request.email).await?.is_some() {
            return Err(AppError::User(UserError::UserAlreadyExists));
        }
        let password_hash = bcrypt::hash(&request.password,bcrypt::DEFAULT_COST)?;

        let user = User {
            id: Uuid::new_v4(),
            username: request.username,
            email: request.email,
            created_at: Utc::now(),
            password_hash,
        };

        let created = self.repo.create(user).await?;
        Ok(UserResponse::from(created))
    }

    pub async fn login(&self, request: LoginRequest) -> Result<LoginResponse, AppError> {
        let user = self.repo.find_by_email(&request.email).await?.ok_or(AppError::User(UserError::InvalidCredentials))?;

        if !bcrypt::verify(&request.password, &user.password_hash)? {
            return Err(AppError::User(UserError::InvalidCredentials));
        }
        let access_token = create_access_token(user.id)?;
        let refresh_token = create_refresh_token(user.id)?;
        Ok(LoginResponse{
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            user: UserResponse::from(user),
        })
    }

    pub async fn change_password(&self, user_id: Uuid, request: ChangePasswordRequest) -> Result<UserResponse, AppError>{
        let user = self.repo.find_by_id(user_id).await?.ok_or(AppError::User(UserError::NotFound))?;

        if !bcrypt::verify(&request.old_password, &user.password_hash)? {
            return Err(AppError::User(UserError::PasswordMismatch));
        }

        let password_hash = bcrypt::hash(&request.new_password,bcrypt::DEFAULT_COST)?;

        let update_data = UpdateUser {
            username: None,
            email: None,
            password_hash: Some(password_hash),
        };
        let updated = self.repo.update(user_id, update_data).await?.ok_or(AppError::User(UserError::NotFound))?;
        return Ok(UserResponse::from(updated))
    }

    pub async fn update(&self, user_id: Uuid, request: UpdateUserRequest) -> Result<UserResponse, AppError> {

        if let Some(ref email) = request.email{
            if let Some(existing) = self.repo.find_by_email(email).await?{
                if existing.id != user_id {
                    return Err(AppError::User(UserError::UserAlreadyExists));
                }
            }
        }

        let update_data = UpdateUser {
            username: request.username,
            email: request.email,
            password_hash: None,
        };
        let updated = self.repo.update(user_id, update_data).await?.ok_or(AppError::User(UserError::NotFound))?;
        return Ok(UserResponse::from(updated))
    }

    pub async fn find(&self, email: &str) -> Result<UserResponse, AppError> {
        let user = self.repo.find_by_email(email).await?.ok_or(AppError::User(UserError::NotFound))?;
        return Ok(UserResponse::from(user))
    }

    pub async fn refresh(&self, request: RefreshTokenRequest) -> Result<RefreshTokenResponse, AppError> {
        let claims = verify_refresh_token(&request.refresh_token)?;
        
        // 验证用户是否存在
        self.repo.find_by_id(claims.sub).await?.ok_or(AppError::User(UserError::NotFound))?;

        let new_access_token = create_access_token(claims.sub)?;
        let new_refresh_token = create_refresh_token(claims.sub)?;

        Ok(RefreshTokenResponse {
            access_token: new_access_token,
            refresh_token: new_refresh_token,
            token_type: "Bearer".to_string(),
        })
    }
}
