use anyhow::Result;
use sea_query::enum_def;
use serde::Serialize;
use sqlx::{
    Sqlite,
    migrate::MigrateDatabase,
    prelude::FromRow,
    types::chrono::{DateTime, Utc},
};
use std::fs;
use uuid::Uuid;

use crate::metadata::DATA_PATH;

#[enum_def]
#[derive(Debug, Serialize, FromRow)]
pub struct FileEmbedding {
    pub id: Uuid,
    pub sha: String,
    pub file: String,
    pub path: String,
    pub vector: Vec<f32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn generate_db_file(db_url: &str) -> Result<()> {
    if !Sqlite::database_exists(&db_url).await? {
        fs::create_dir_all(&DATA_PATH)?;
        Sqlite::create_database(&db_url).await?
    }
    Ok(())
}
