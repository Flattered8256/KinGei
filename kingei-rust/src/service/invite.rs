use std::sync::Arc;

use uuid::Uuid;

use crate::error::AppError;
use crate::error::invite::InviteError;
use crate::repository::invite::InviteRepository;
use crate::serializer::invite::InviteResponse;

const ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
const CODE_LEN: usize = 8;

pub struct InviteService {
    repo: Arc<InviteRepository>,
}

impl InviteService {
    pub fn new(repo: Arc<InviteRepository>) -> Self {
        Self { repo }
    }

    pub async fn get(&self) -> Result<InviteResponse, AppError> {
        let invite = self
            .repo
            .get()
            .await?
            .ok_or(AppError::Invite(InviteError::InviteNotFound))?;
        Ok(InviteResponse::from(invite))
    }

    pub async fn regenerate(&self) -> Result<InviteResponse, AppError> {
        let bytes = Uuid::new_v4().into_bytes();
        let code: String = (0..CODE_LEN)
            .map(|i| ALPHABET[bytes[i] as usize % ALPHABET.len()] as char)
            .collect();
        let invite = self.repo.upsert(&code).await?;
        Ok(InviteResponse::from(invite))
    }
}

#[cfg(all(test, feature = "sqlite"))]
mod tests {
    use super::*;
    use crate::database::{DbPool, run_migrations};

    #[tokio::test]
    async fn generate_code_is_eight_alphabet_chars() {
        let service = test_service().await;
        let code = service.regenerate().await.unwrap().code;
        assert_eq!(code.len(), CODE_LEN);
        assert!(code.bytes().all(|b| ALPHABET.contains(&b)));
    }

    #[tokio::test]
    async fn get_before_generate_is_not_found() {
        let service = test_service().await;
        let err = service.get().await.unwrap_err();
        assert!(matches!(err, AppError::Invite(InviteError::InviteNotFound)));
    }

    #[tokio::test]
    async fn regenerate_replaces_code_and_keeps_row() {
        let service = test_service().await;

        let first = service.regenerate().await.unwrap();
        assert_eq!(first.code.len(), CODE_LEN);
        assert_eq!(service.get().await.unwrap().code, first.code);

        let second = service.regenerate().await.unwrap();
        assert_ne!(first.code, second.code);
        assert_eq!(service.get().await.unwrap().code, second.code);
    }

    async fn test_service() -> InviteService {
        let pool = test_pool().await;
        run_migrations(&pool).await;
        InviteService::new(Arc::new(InviteRepository::new(pool)))
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
