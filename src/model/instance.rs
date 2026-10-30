use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct Instance {
    pub id: i64,
    pub allow_add_friend: bool,
    pub allow_create_group: bool,
    pub initializing: bool,
}
