use std::mem;

use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use sqlx::PgPool;

use crate::{AppError, User};
impl User {
    /// Find a user by email
    pub async fn find_by_email(email: &str, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let user = sqlx::query_as("SELECT id, fullname, email, created_at FROM users WHERE email = $1")
                .bind(email)
                // 返回 0 或 1 条记录（Option<T>）
                .fetch_optional(pool)
                .await?;
        Ok(user)
    }
    /// Create a new user
    pub async fn create(
        fullname: &str,
        email: &str,
        password: &str,
        pool: &PgPool,
    ) -> Result<Self, AppError> {
        let password_hash = hash_password(password)?;
        let user    = sqlx::query_as("INSERT INTO users (fullname, email, password_hash) VALUES ($1, $2, $3) RETURNING id, fullname, email, created_at")
                .bind(fullname)
                .bind(email)
                .bind(password_hash)
                .fetch_one(pool)
                .await?;
        Ok(user)
    }
    /// Verify a user's password
    pub async fn verify_password(
        &self,
        email: &str,
        password: &str,
        pool: &PgPool,
    ) -> Result<Option<Self>, AppError> {
        let user: Option<User> = sqlx::query_as(
            "SELECT id, fullname, email, password_hash, created_at FROM users WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(pool)
        .await?;
        match user {
            Some(mut user) => {
                let password_hash = mem::take(&mut user.password_hash);
                let is_valid = verify_password(password, &password_hash.unwrap_or_default())?;
                if is_valid { Ok(Some(user)) } else { Ok(None) }
            }
            None => Ok(None),
        }
    }
}

fn hash_password(password: &str) -> Result<String, AppError> {
    // 从操作系统生成随机盐
    let salt = SaltString::generate(&mut OsRng);

    // Argon2 with default params (Argon2id v19)
    let argon2 = Argon2::default();

    // Hash password to PHC string ($argon2id$v=19$...)
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

    Ok(password_hash)
}

fn verify_password(password: &str, password_hash: &str) -> Result<bool, AppError> {
    let argon2 = Argon2::default();
    let password_hash = PasswordHash::new(password_hash)?;

    // Verify password
    let is_valid = argon2
        .verify_password(password.as_bytes(), &password_hash)
        .is_ok();

    Ok(is_valid)
}

#[cfg(test)]
impl User {
    pub fn new(id: i64, fullname: &str, email: &str) -> Self {
        Self {
            id,
            fullname: fullname.to_string(),
            email: email.to_string(),
            password_hash: None,
            created_at: chrono::Utc::now(),
        }
    }
}
#[cfg(test)]
mod tests {
    use std::{env, path::Path};

    use anyhow::Result;
    use sqlx_db_tester::TestPg;

    use super::*;

    #[test]
    fn hash_password_and_verify_should_work() -> Result<()> {
        let password = "hunter42";
        let password_hash = hash_password(password)?;
        assert_eq!(password_hash.len(), 97);
        assert!(verify_password(password, &password_hash)?);
        Ok(())
    }

    #[tokio::test]
    async fn create_and_verify_user_should_work() -> Result<()> {
        let db_url = env::var("DB_URL").unwrap();
        let tdb = TestPg::new(db_url, Path::new("../migrations"));
        let pool = tdb.get_pool().await;
        let email = "tchen@acme.org";
        let name = "Tyr Chen";
        let password = "hunter42";
        let user = User::create(name, email, password, &pool).await?;
        assert_eq!(user.email, email);
        assert_eq!(user.fullname, name);
        assert!(user.id > 0);

        let user = User::find_by_email(email, &pool).await?;
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.email, email);
        assert_eq!(user.fullname, name);

        let user = user.verify_password(email, password, &pool).await?;
        assert!(user.is_some());

        Ok(())
    }
}
