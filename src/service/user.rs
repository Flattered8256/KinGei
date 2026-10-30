use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::error::AppError;
use crate::error::user::UserError;
use crate::model::user::{User, UserRole};
use crate::repository::file::FileRepository;
use crate::repository::user::{UpdateUser, UserRepository};
use crate::serializer::user::{
    ChangePasswordRequest, CreateUserRequest, LoginRequest, LoginResponse, RefreshTokenRequest,
    RefreshTokenResponse, UpdateUserRequest, UserResponse,
};
use crate::service::file::require_image;
use crate::utils::auth::{create_access_token, create_refresh_token, verify_refresh_token};

pub struct UserService {
    repo: Arc<UserRepository>,
    file_repo: Arc<FileRepository>,
}

impl UserService {
    pub fn new(repo: Arc<UserRepository>, file_repo: Arc<FileRepository>) -> Self {
        Self { repo, file_repo }
    }

    pub async fn register(&self, request: CreateUserRequest) -> Result<UserResponse, AppError> {
        if self.repo.find_by_email(&request.email).await?.is_some() {
            return Err(AppError::User(UserError::UserAlreadyExists));
        }
        let password_hash = bcrypt::hash(&request.password, bcrypt::DEFAULT_COST)?;

        let user = User {
            id: Uuid::new_v4(),
            username: request.username,
            email: request.email,
            created_at: Utc::now(),
            password_hash,
            avatar: None,
            role: UserRole::Member,
        };

        let created = self.repo.create(user).await?;
        Ok(UserResponse::from(created))
    }

    pub async fn login(&self, request: LoginRequest) -> Result<LoginResponse, AppError> {
        let user = self
            .repo
            .find_by_email(&request.email)
            .await?
            .ok_or(AppError::User(UserError::InvalidCredentials))?;

        if !bcrypt::verify(&request.password, &user.password_hash)? {
            return Err(AppError::User(UserError::InvalidCredentials));
        }
        let access_token = create_access_token(user.id)?;
        let refresh_token = create_refresh_token(user.id)?;
        Ok(LoginResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            user: UserResponse::from(user),
        })
    }

    pub async fn change_password(
        &self,
        user_id: Uuid,
        request: ChangePasswordRequest,
    ) -> Result<UserResponse, AppError> {
        let user = self
            .repo
            .find_by_id(user_id)
            .await?
            .ok_or(AppError::User(UserError::NotFound))?;

        if !bcrypt::verify(&request.old_password, &user.password_hash)? {
            return Err(AppError::User(UserError::PasswordMismatch));
        }

        let password_hash = bcrypt::hash(&request.new_password, bcrypt::DEFAULT_COST)?;

        let update_data = UpdateUser {
            username: None,
            email: None,
            password_hash: Some(password_hash),
            avatar: None,
        };
        let updated = self
            .repo
            .update(user_id, update_data)
            .await?
            .ok_or(AppError::User(UserError::NotFound))?;
        return Ok(UserResponse::from(updated));
    }

    pub async fn update(
        &self,
        user_id: Uuid,
        request: UpdateUserRequest,
    ) -> Result<UserResponse, AppError> {
        if let Some(ref email) = request.email {
            if let Some(existing) = self.repo.find_by_email(email).await? {
                if existing.id != user_id {
                    return Err(AppError::User(UserError::UserAlreadyExists));
                }
            }
        }

        if let Some(file_id) = request.avatar {
            require_image(&self.file_repo, user_id, file_id).await?;
        }

        let update_data = UpdateUser {
            username: request.username,
            email: request.email,
            password_hash: None,
            avatar: request.avatar,
        };
        let updated = self
            .repo
            .update(user_id, update_data)
            .await?
            .ok_or(AppError::User(UserError::NotFound))?;
        return Ok(UserResponse::from(updated));
    }

    pub async fn find(&self, email: &str) -> Result<UserResponse, AppError> {
        let user = self
            .repo
            .find_by_email(email)
            .await?
            .ok_or(AppError::User(UserError::NotFound))?;
        return Ok(UserResponse::from(user));
    }

    pub async fn refresh(
        &self,
        request: RefreshTokenRequest,
    ) -> Result<RefreshTokenResponse, AppError> {
        let claims = verify_refresh_token(&request.refresh_token)?;

        // 验证用户是否存在
        self.repo
            .find_by_id(claims.sub)
            .await?
            .ok_or(AppError::User(UserError::NotFound))?;

        let new_access_token = create_access_token(claims.sub)?;
        let new_refresh_token = create_refresh_token(claims.sub)?;

        Ok(RefreshTokenResponse {
            access_token: new_access_token,
            refresh_token: new_refresh_token,
            token_type: "Bearer".to_string(),
        })
    }

    pub async fn list(&self) -> Result<Vec<UserResponse>, AppError> {
        let users = self.repo.list().await?;
        Ok(users.into_iter().map(UserResponse::from).collect())
    }

    pub async fn require_admin(&self, user_id: Uuid) -> Result<(), AppError> {
        let user = self
            .repo
            .find_by_id(user_id)
            .await?
            .ok_or(AppError::User(UserError::NotFound))?;
        if user.role != UserRole::Admin {
            return Err(AppError::User(UserError::NotAdmin));
        }
        Ok(())
    }
}
