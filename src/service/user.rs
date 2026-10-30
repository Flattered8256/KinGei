use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::error::AppError;
use crate::error::invite::InviteError;
use crate::error::user::UserError;
use crate::model::user::{User, UserRole};
use crate::repository::file::FileRepository;
use crate::repository::instance::InstanceRepository;
use crate::repository::invite::InviteRepository;
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
    instance_repo: Arc<InstanceRepository>,
    invite_repo: Arc<InviteRepository>,
}

impl UserService {
    pub fn new(
        repo: Arc<UserRepository>,
        file_repo: Arc<FileRepository>,
        instance_repo: Arc<InstanceRepository>,
        invite_repo: Arc<InviteRepository>,
    ) -> Self {
        Self {
            repo,
            file_repo,
            instance_repo,
            invite_repo,
        }
    }

    pub async fn register(&self, request: CreateUserRequest) -> Result<UserResponse, AppError> {
        let instance = self.instance_repo.get().await?;
        if !instance.initializing && instance.invite_only {
            let code = request
                .invite_code
                .as_deref()
                .unwrap_or("")
                .trim()
                .to_ascii_uppercase();
            if code.is_empty() {
                return Err(AppError::Invite(InviteError::InviteNotFound));
            }
            self.invite_repo
                .find_by_code(&code)
                .await?
                .ok_or(AppError::Invite(InviteError::InviteNotFound))?;
        }
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

#[cfg(all(test, feature = "sqlite"))]
mod tests {
    use super::*;
    use crate::database::{DbPool, run_migrations};
    use crate::repository::instance::UpdateInstance;
    use crate::service::invite::InviteService;

    #[tokio::test]
    async fn register_open_without_code() {
        let (users, _, _) = test_service().await;
        let created = users.register(sample("alice", None)).await.unwrap();
        assert_eq!(created.role, UserRole::Admin);
    }

    #[tokio::test]
    async fn initializing_skips_invite_even_when_invite_only() {
        let (users, instance, _) = test_service().await;
        instance
            .update(UpdateInstance {
                allow_add_friend: None,
                allow_create_group: None,
                invite_only: Some(true),
            })
            .await
            .unwrap();

        let created = users.register(sample("alice", None)).await.unwrap();
        assert_eq!(created.role, UserRole::Admin);
    }

    #[tokio::test]
    async fn invite_only_rejects_missing_or_wrong_code() {
        let (users, instance, invites) = test_service().await;
        users.register(sample("alice", None)).await.unwrap();
        instance
            .update(UpdateInstance {
                allow_add_friend: None,
                allow_create_group: None,
                invite_only: Some(true),
            })
            .await
            .unwrap();
        let code = invites.regenerate().await.unwrap().code;

        let missing = users.register(sample("bob", None)).await.unwrap_err();
        assert!(matches!(
            missing,
            AppError::Invite(InviteError::InviteNotFound)
        ));

        let wrong = users
            .register(sample("bob", Some("NOTACODE")))
            .await
            .unwrap_err();
        assert!(matches!(
            wrong,
            AppError::Invite(InviteError::InviteNotFound)
        ));

        let created = users
            .register(sample("bob", Some(&code.to_lowercase())))
            .await
            .unwrap();
        assert_eq!(created.role, UserRole::Member);
        assert_eq!(invites.get().await.unwrap().code, code);
    }

    fn sample(username: &str, invite_code: Option<&str>) -> CreateUserRequest {
        CreateUserRequest {
            username: username.to_string(),
            email: format!("{username}@example.com"),
            password: "password".to_string(),
            invite_code: invite_code.map(str::to_string),
        }
    }

    async fn test_service() -> (UserService, Arc<InstanceRepository>, InviteService) {
        let pool = test_pool().await;
        run_migrations(&pool).await;
        let invite_repo = Arc::new(InviteRepository::new(pool.clone()));
        let invites = InviteService::new(invite_repo.clone());
        let instance = Arc::new(InstanceRepository::new(pool.clone()));
        let users = UserService::new(
            Arc::new(UserRepository::new(pool.clone())),
            Arc::new(FileRepository::new(pool)),
            instance.clone(),
            invite_repo,
        );
        (users, instance, invites)
    }

    async fn test_pool() -> DbPool {
        use std::str::FromStr;

        use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

        let options = SqliteConnectOptions::from_str("sqlite::memory:")
            .unwrap()
            .create_if_missing(true)
            .foreign_keys(true);
        SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .unwrap()
    }
}
