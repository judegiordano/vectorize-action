use anyhow::Result;
use serde::{Serialize, de::DeserializeOwned};
use sqlx::{Pool, Sqlite, sqlite::SqliteQueryResult};
use std::fmt::Debug;

pub mod file_embed;

pub trait Model: Debug + Serialize + DeserializeOwned + Default {
    async fn create_table(pool: &Pool<Sqlite>, table: &str) -> Result<SqliteQueryResult>;
    #[allow(dead_code)]
    async fn insert_one(&self, pool: &Pool<Sqlite>, table: &str) -> Result<SqliteQueryResult>;
}
