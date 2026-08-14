use crate::model::friend::{Friend, FriendRequest, Status};
use sqlx::SqlitePool;
use uuid::Uuid;
use chrono::Utc;

pub struct FriendRepository {
    pool: SqlitePool,
}

impl FriendRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        friend: Friend,
    ) -> Result<Vec<Friend>, sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        let created_at = friend.created_at;

        let a = sqlx::query_as::<_, Friend>(
            r#"
            INSERT INTO friend(user_id, friend_id, created_at)
            VALUES (?, ?, ?)
            RETURNING id, user_id, friend_id, created_at
            "#,
        )
        .bind(&friend.user_id)
        .bind(&friend.friend_id)
        .bind(&created_at)
        .fetch_one(&mut *tx)
        .await?;

        let b = sqlx::query_as::<_, Friend>(
            r#"
            INSERT INTO friend(user_id, friend_id, created_at)
            VALUES (?, ?, ?)
            RETURNING id, user_id, friend_id, created_at
            "#,
        )
        .bind(&friend.friend_id)
        .bind(&friend.user_id)
        .bind(&created_at)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(vec![a, b])
    }

    pub async fn find(
        &self,
        user_id: Uuid,
        friend_id: Uuid,
    ) -> Result<Option<Friend>, sqlx::Error> {
        sqlx::query_as::<_, Friend>(
            r#"
            SELECT id, user_id, friend_id, created_at
            FROM friend
            WHERE user_id = ? AND friend_id = ?
            "#,
        )
        .bind(&user_id)
        .bind(&friend_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn delete(
        &self,
        user_id: Uuid,
        friend_id: Uuid,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM friend
            WHERE (user_id = ? AND friend_id = ?)
               OR (user_id = ? AND friend_id = ?)
            "#,
        )
        .bind(&user_id)
        .bind(&friend_id)
        .bind(&friend_id)
        .bind(&user_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn list(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<Friend>, sqlx::Error> {
        sqlx::query_as::<_, Friend>(
            r#"
            SELECT id, user_id, friend_id, created_at
            FROM friend
            WHERE user_id = ?
            "#,
        )
        .bind(&user_id)
        .fetch_all(&self.pool)
        .await
    }
}

pub struct FriendRequestRepository {
    pool: SqlitePool,
}

impl FriendRequestRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        sender_id: Uuid,
        receiver_id: Uuid,
    ) -> Result<FriendRequest, sqlx::Error> {
        let now = Utc::now();
        let status = Status::Pending;
        sqlx::query_as::<_, FriendRequest>(
            r#"
            INSERT INTO friend_request(sender_id, receiver_id, status, created_at, updated_at)
            VALUES (?,?,?,?,?)
            RETURNING id, sender_id, receiver_id, status, created_at, updated_at
            "#,
        )
        .bind(&sender_id)
        .bind(&receiver_id)
        .bind(&status)
        .bind(&now)
        .bind(&now)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_by_id(
        &self,
        id: i64,
    ) -> Result<Option<FriendRequest>, sqlx::Error> {
        sqlx::query_as::<_, FriendRequest>(
            r#"
            SELECT id, sender_id, receiver_id, status, created_at, updated_at
            FROM friend_request
            WHERE id = ?
            "#,
        )
        .bind(&id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_by_status(
        &self,
        sender_id: Uuid,
        receiver_id: Uuid,
        status: Status,
    ) -> Result<Option<FriendRequest>, sqlx::Error> {
        sqlx::query_as::<_, FriendRequest>(
            r#"
            SELECT id, sender_id, receiver_id, status, created_at, updated_at
            FROM friend_request
            WHERE sender_id = ? AND receiver_id = ? AND status = ?
            "#,
        )
        .bind(&sender_id)
        .bind(&receiver_id)
        .bind(&status)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list_received(
        &self,
        receiver_id: Uuid,
    ) -> Result<Vec<FriendRequest>, sqlx::Error> {
        sqlx::query_as::<_, FriendRequest>(
            r#"
            SELECT id, sender_id, receiver_id, status, created_at, updated_at
            FROM friend_request
            WHERE receiver_id = ?
            "#,
        )
        .bind(&receiver_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn list_sent(
        &self,
        sender_id: Uuid,
    ) -> Result<Vec<FriendRequest>, sqlx::Error> {
        sqlx::query_as::<_, FriendRequest>(
            r#"
            SELECT id, sender_id, receiver_id, status, created_at, updated_at
            FROM friend_request
            WHERE sender_id = ?
            "#,
        )
        .bind(&sender_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn update_status(
        &self,
        id: i64,
        status: Status,
    ) -> Result<Option<FriendRequest>, sqlx::Error> {
        sqlx::query_as::<_, FriendRequest>(
            r#"
            UPDATE friend_request
            SET status = ?, updated_at = ?
            WHERE id = ?
            RETURNING id, sender_id, receiver_id, status, created_at, updated_at
            "#,
        )
        .bind(&status)
        .bind(&Utc::now())
        .bind(&id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn delete(
        &self,
        id: i64,
    ) -> Result<Option<FriendRequest>, sqlx::Error> {
        sqlx::query_as::<_, FriendRequest>(
            r#"
            DELETE FROM friend_request
            WHERE id = ?
            RETURNING id, sender_id, receiver_id, status, created_at, updated_at
            "#,
        )
        .bind(&id)
        .fetch_optional(&self.pool)
        .await
    }
}