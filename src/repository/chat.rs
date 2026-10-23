use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::model::chat::{Chat, ChatMember, ChatType};

pub struct ChatRepository {
    pool: SqlitePool,
}

impl ChatRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<Chat>, sqlx::Error> {
        sqlx::query_as::<_, Chat>(
            r#"
            SELECT id, chat_type, name, owner_id, created_at, avatar
            FROM chat
            WHERE id = ?
            "#,
        )
        .bind(&id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_direct(
        &self,
        user_id: Uuid,
        friend_id: Uuid,
    ) -> Result<Option<Chat>, sqlx::Error> {
        sqlx::query_as::<_, Chat>(
            r#"
            SELECT c.id, c.chat_type, c.name, c.owner_id, c.created_at, c.avatar
            FROM chat c
            INNER JOIN chat_member m1 ON m1.chat_id = c.id AND m1.user_id = ?
            INNER JOIN chat_member m2 ON m2.chat_id = c.id AND m2.user_id = ?
            WHERE c.chat_type = ?
            "#,
        )
        .bind(&user_id)
        .bind(&friend_id)
        .bind(&ChatType::Direct)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list_by_user(&self, user_id: Uuid) -> Result<Vec<Chat>, sqlx::Error> {
        sqlx::query_as::<_, Chat>(
            r#"
            SELECT c.id, c.chat_type, c.name, c.owner_id, c.created_at, c.avatar
            FROM chat c
            INNER JOIN chat_member m ON m.chat_id = c.id
            WHERE m.user_id = ?
            ORDER BY c.created_at DESC
            "#,
        )
        .bind(&user_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn create_direct(&self, user_id: Uuid, friend_id: Uuid) -> Result<Chat, sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        let now = Utc::now();
        let chat_type = ChatType::Direct;

        let chat = sqlx::query_as::<_, Chat>(
            r#"
            INSERT INTO chat(chat_type, name, owner_id, created_at)
            VALUES (?, ?, ?, ?)
            RETURNING id, chat_type, name, owner_id, created_at, avatar
            "#,
        )
        .bind(&chat_type)
        .bind(Option::<String>::None)
        .bind(Option::<Uuid>::None)
        .bind(&now)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO chat_member(chat_id, user_id, created_at)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(&chat.id)
        .bind(&user_id)
        .bind(&now)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO chat_member(chat_id, user_id, created_at)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(&chat.id)
        .bind(&friend_id)
        .bind(&now)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(chat)
    }

    pub async fn create_group(&self, owner_id: Uuid, name: &str) -> Result<Chat, sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        let now = Utc::now();
        let chat_type = ChatType::Group;

        let chat = sqlx::query_as::<_, Chat>(
            r#"
            INSERT INTO chat(chat_type, name, owner_id, created_at)
            VALUES (?, ?, ?, ?)
            RETURNING id, chat_type, name, owner_id, created_at, avatar
            "#,
        )
        .bind(&chat_type)
        .bind(&name)
        .bind(&owner_id)
        .bind(&now)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query(
            r#"
        INSERT INTO chat_member(chat_id, user_id, created_at)
        VALUES (?, ?, ?)
        "#,
        )
        .bind(&chat.id)
        .bind(&owner_id)
        .bind(&now)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(chat)
    }

    pub async fn update_name(&self, id: i64, name: &str) -> Result<Option<Chat>, sqlx::Error> {
        sqlx::query_as::<_, Chat>(
            r#"
            UPDATE chat
            SET name = ?
            WHERE id = ?
            RETURNING id, chat_type, name, owner_id, created_at, avatar
            "#,
        )
        .bind(&name)
        .bind(&id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn update_avatar(&self, id: i64, avatar: i64) -> Result<Option<Chat>, sqlx::Error> {
        sqlx::query_as::<_, Chat>(
            r#"
            UPDATE chat
            SET avatar = ?
            WHERE id = ?
            RETURNING id, chat_type, name, owner_id, created_at, avatar
            "#,
        )
        .bind(avatar)
        .bind(&id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn update_owner(&self, id: i64, owner_id: Uuid) -> Result<Option<Chat>, sqlx::Error> {
        sqlx::query_as::<_, Chat>(
            r#"
            UPDATE chat
            SET owner_id = ?
            WHERE id = ?
            RETURNING id, chat_type, name, owner_id, created_at, avatar
            "#,
        )
        .bind(&owner_id)
        .bind(&id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list_members(&self, chat_id: i64) -> Result<Vec<ChatMember>, sqlx::Error> {
        sqlx::query_as::<_, ChatMember>(
            r#"
            SELECT id, chat_id, user_id, created_at, last_read_message_id
            FROM chat_member
            WHERE chat_id = ?
            ORDER BY created_at ASC, id ASC
            "#,
        )
        .bind(&chat_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn find_member(
        &self,
        chat_id: i64,
        user_id: Uuid,
    ) -> Result<Option<ChatMember>, sqlx::Error> {
        sqlx::query_as::<_, ChatMember>(
            r#"
            SELECT id, chat_id, user_id, created_at, last_read_message_id
            FROM chat_member
            WHERE chat_id = ? AND user_id = ?
            "#,
        )
        .bind(&chat_id)
        .bind(&user_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn add_member(&self, chat_id: i64, user_id: Uuid) -> Result<ChatMember, sqlx::Error> {
        let now = Utc::now();
        sqlx::query_as::<_, ChatMember>(
            r#"
            INSERT INTO chat_member(chat_id, user_id, created_at)
            VALUES (?, ?, ?)
            RETURNING id, chat_id, user_id, created_at, last_read_message_id
            "#,
        )
        .bind(&chat_id)
        .bind(&user_id)
        .bind(&now)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn delete_member(
        &self,
        chat_id: i64,
        user_id: Uuid,
    ) -> Result<Option<ChatMember>, sqlx::Error> {
        sqlx::query_as::<_, ChatMember>(
            r#"
            DELETE FROM chat_member
            WHERE chat_id = ? AND user_id = ?
            RETURNING id, chat_id, user_id, created_at, last_read_message_id
            "#,
        )
        .bind(&chat_id)
        .bind(&user_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn delete_chat(&self, id: i64) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            r#"
            DELETE FROM message
            WHERE chat_id = ?
            "#,
        )
        .bind(&id)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            DELETE FROM chat_member
            WHERE chat_id = ?
            "#,
        )
        .bind(&id)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            DELETE FROM chat
            WHERE id = ?
            "#,
        )
        .bind(&id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }
}
