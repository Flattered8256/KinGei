use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct Invite {
    pub id: i64,
    pub code: String,
}
