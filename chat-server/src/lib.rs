use std::{fmt, ops::Deref, sync::Arc};

mod config;
mod error;
mod handlers;
mod middlewares;
mod models;
mod utils;
use anyhow::Context;
use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{get, post},
};
pub use config::AppConfig;
pub use error::AppError;
use handlers::*;
use middlewares::*;
use models::*;

use crate::utils::{DecodingKey, EncodingKey};
#[derive(Debug, Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

#[allow(dead_code)]
pub struct AppStateInner {
    pub(crate) config: AppConfig,
    pub(crate) dk: DecodingKey,
    pub(crate) ek: EncodingKey,
    pub(crate) db: sqlx::PgPool,
}

impl fmt::Debug for AppStateInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppStateInner")
            .field("config", &self.config)
            .finish()
    }
}

// 当我调用 state.config => state.inner.config
impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AppState {
    pub async fn try_new(config: AppConfig) -> Result<Self, AppError> {
        let dk = DecodingKey::load(&config.auth.pk).expect("Failed to load public key");
        let ek = EncodingKey::load(&config.auth.sk).expect("Failed to load private key");
        let db = sqlx::PgPool::connect(&config.server.db_url)
            .await
            .context("Failed to connect to database")?;
        Ok(Self {
            inner: Arc::new(AppStateInner { config, dk, ek, db }),
        })
    }
}
pub async fn get_router(config: AppConfig) -> Result<Router, AppError> {
    let state = AppState::try_new(config).await?;
    let api = Router::new()
        .route("/users", get(list_chat_users_handler))
        .route("/chats", get(list_chat_handler).post(create_chat_handler))
        .route(
            "/chats/{id}",
            get(get_chat_handler)
                .patch(update_chat_handler)
                .delete(delete_chat_handler)
                .post(send_message_handler),
        )
        .route("/chat/{id}/messages", get(list_message_handler))
        .layer(from_fn_with_state(state.clone(), verify_token))
        .route("/signin", post(signin_handler))
        .route("/signup", post(signup_handler));

    let app = Router::new()
        .route("/", axum::routing::get(index_handler))
        .nest("/api", api)
        .with_state(state);

    Ok(set_layer(app))
}

#[cfg(test)]
mod test_util {
    use sqlx::{Executor, PgPool};
    use sqlx_db_tester::TestPg;

    use super::*;

    impl AppState {
        pub async fn new_for_test_with_config(
            config: AppConfig,
        ) -> Result<(sqlx_db_tester::TestPg, Self), AppError> {
            let dk = DecodingKey::load(&config.auth.pk).context("load pk failed")?;
            let ek = EncodingKey::load(&config.auth.sk).context("load sk failed")?;
            let (tdb, pool) = get_test_pool_with_config(&config).await?;
            let state = Self {
                inner: Arc::new(AppStateInner {
                    config,
                    ek,
                    dk,
                    db: pool,
                }),
            };
            Ok((tdb, state))
        }
        pub async fn new_for_test() -> Result<(sqlx_db_tester::TestPg, Self), AppError> {
            let config = AppConfig::load()?;
            let (tdb, state) = Self::new_for_test_with_config(config).await?;
            Ok((tdb, state))
        }
    }

    pub async fn get_test_pool() -> Result<(TestPg, PgPool), AppError> {
        let config = AppConfig::load()?;
        get_test_pool_with_config(&config).await
    }

    pub async fn get_test_pool_with_config(
        config: &AppConfig,
    ) -> Result<(TestPg, PgPool), AppError> {
        let (server_url, _) = config.server.db_url.rsplit_once('/').unwrap();
        get_test_pool_with_url(Some(server_url)).await
    }

    pub async fn get_test_pool_with_url(url: Option<&str>) -> Result<(TestPg, PgPool), AppError> {
        let url = match url {
            Some(url) => url.to_string(),
            None => "postgres://postgres:123456@localhost:5432".to_string(),
        };
        let tdb = TestPg::new(url, std::path::Path::new("../migrations"));
        let pool = tdb.get_pool().await;

        // run prepared sql to insert test dat
        let sql = include_str!("../fixtures/test.sql").split(';');
        let mut ts = pool.begin().await.expect("begin transaction failed");
        for s in sql {
            if s.trim().is_empty() {
                continue;
            }
            ts.execute(s).await.expect("execute sql failed");
        }
        ts.commit().await.expect("commit transaction failed");

        Ok((tdb, pool))
    }
}
