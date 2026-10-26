use chrono::Utc;
use sqlx::{QueryBuilder, SqlitePool};
use uuid::Uuid;

use crate::model::message::Message;

pub struct MessageRepository {
    pool: SqlitePool,
}

impl MessageRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        chat_id: i64,
        sender_id: Uuid,
        content: &str,
        file_id: Option<i64>,
    ) -> Result<Message, sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        let now = Utc::now();

        let message = sqlx::query_as::<_, Message>(
            r#"
            INSERT INTO message(chat_id, sender_id, content, file_id, created_at)
            VALUES (?, ?, ?, ?, ?)
            RETURNING id, chat_id, sender_id, content, file_id, created_at
            "#,
        )
        .bind(chat_id)
        .bind(sender_id)
        .bind(content)
        .bind(file_id)
        .bind(now)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            UPDATE chat_member
            SET last_read_message_id = ?
            WHERE chat_id = ? AND user_id = ?
              AND (last_read_message_id IS NULL OR last_read_message_id < ?)
            "#,
        )
        .bind(message.id)
        .bind(chat_id)
        .bind(sender_id)
        .bind(message.id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(message)
    }

    pub async fn find_in_chat(
        &self,
        chat_id: i64,
        id: i64,
    ) -> Result<Option<Message>, sqlx::Error> {
        sqlx::query_as::<_, Message>(
            r#"
            SELECT id, chat_id, sender_id, content, file_id, created_at
            FROM message
            WHERE chat_id = ? AND id = ?
            "#,
        )
        .bind(chat_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list(
        &self,
        chat_id: i64,
        before: Option<i64>,
        limit: i64,
    ) -> Result<Vec<Message>, sqlx::Error> {
        let mut builder = QueryBuilder::new(
            r#"
            SELECT id, chat_id, sender_id, content, file_id, created_at
            FROM message
            WHERE chat_id = 
            "#,
        );
        builder.push_bind(chat_id);
        if let Some(before) = before {
            builder.push(" AND id < ");
            builder.push_bind(before);
        }
        builder.push(" ORDER BY id DESC LIMIT ");
        builder.push_bind(limit);

        builder
            .build_query_as::<Message>()
            .fetch_all(&self.pool)
            .await
    }

    pub async fn update_read_cursor(
        &self,
        chat_id: i64,
        user_id: Uuid,
        message_id: i64,
    ) -> Result<Option<i64>, sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE chat_member
            SET last_read_message_id = ?
            WHERE chat_id = ? AND user_id = ?
              AND (last_read_message_id IS NULL OR last_read_message_id < ?)
            "#,
        )
        .bind(message_id)
        .bind(chat_id)
        .bind(user_id)
        .bind(message_id)
        .execute(&self.pool)
        .await?;

        let row: Option<(Option<i64>,)> = sqlx::query_as(
            r#"
            SELECT last_read_message_id
            FROM chat_member
            WHERE chat_id = ? AND user_id = ?
            "#,
        )
        .bind(chat_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some((cursor,)) => Ok(Some(cursor.unwrap_or(message_id))),
            None => Ok(None),
        }
    }

    pub async fn unread_count(
        &self,
        chat_id: i64,
        user_id: Uuid,
        last_read_message_id: Option<i64>,
    ) -> Result<i64, sqlx::Error> {
        let cursor = last_read_message_id.unwrap_or(0);
        sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM message
            WHERE chat_id = ? AND id > ? AND sender_id != ?
            "#,
        )
        .bind(chat_id)
        .bind(cursor)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn unread_counts_for_user(
        &self,
        user_id: Uuid,
        chat_ids: &[i64],
    ) -> Result<Vec<(i64, i64)>, sqlx::Error> {
        if chat_ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut builder = QueryBuilder::new(
            r#"
            SELECT cm.chat_id, COUNT(m.id)
            FROM chat_member cm
            LEFT JOIN message m
              ON m.chat_id = cm.chat_id
             AND m.sender_id != cm.user_id
             AND m.id > COALESCE(cm.last_read_message_id, 0)
            WHERE cm.user_id = 
            "#,
        );
        builder.push_bind(user_id);
        builder.push(" AND cm.chat_id IN (");
        let mut separated = builder.separated(", ");
        for id in chat_ids {
            separated.push_bind(id);
        }
        builder.push(") GROUP BY cm.chat_id");

        builder
            .build_query_as::<(i64, i64)>()
            .fetch_all(&self.pool)
            .await
    }

    pub async fn unread_counts_for_chat(
        &self,
        chat_id: i64,
    ) -> Result<Vec<(Uuid, i64)>, sqlx::Error> {
        sqlx::query_as::<_, (Uuid, i64)>(
            r#"
            SELECT cm.user_id, COUNT(m.id)
            FROM chat_member cm
            LEFT JOIN message m
              ON m.chat_id = cm.chat_id
             AND m.sender_id != cm.user_id
             AND m.id > COALESCE(cm.last_read_message_id, 0)
            WHERE cm.chat_id = ?
            GROUP BY cm.user_id
            "#,
        )
        .bind(chat_id)
        .fetch_all(&self.pool)
        .await
    }
}
