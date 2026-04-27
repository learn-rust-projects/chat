mod user;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
pub use user::*;
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq)]

pub struct User {
    pub id: i64,
    pub fullname: String,
    pub email: String,
    #[sqlx(default)]
    // Option<T> ≠ 允许字段缺失 NULL → None
    // 如果 SQL 查询里“没有这个字段”，也不要报错，直接填 None
    #[serde(skip)]
    pub password_hash: Option<String>,
    pub created_at: DateTime<Utc>,
}
