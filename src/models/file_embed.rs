use anyhow::Result;
use chrono::{DateTime, Utc};
use sea_query::{Alias, ColumnDef, Expr, InsertStatement, SqliteQueryBuilder, Table, enum_def};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, prelude::FromRow, sqlite::SqliteQueryResult};
use uuid::Uuid;

use crate::models::Model;

#[enum_def]
#[derive(Debug, Serialize, FromRow, Deserialize, Default)]
pub struct FileEmbedding {
    pub id: Uuid,
    pub sha: String,
    pub file: String,
    pub path: String,
    pub vector: Vec<f32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Model for FileEmbedding {
    async fn create_table(pool: &Pool<Sqlite>, table: &str) -> Result<SqliteQueryResult> {
        let mut operation = Table::create();
        let statement = operation
            .table(Alias::new(table))
            .if_not_exists()
            .col(
                ColumnDef::new(FileEmbeddingIden::Id)
                    .primary_key()
                    .uuid()
                    .not_null(),
            )
            .col(ColumnDef::new(FileEmbeddingIden::Sha).string().not_null())
            .col(ColumnDef::new(FileEmbeddingIden::File).string().not_null())
            .col(ColumnDef::new(FileEmbeddingIden::Path).string().not_null())
            // use jsonb column type because sqlite does not have vector support
            .col(
                ColumnDef::new(FileEmbeddingIden::Vector)
                    .json_binary()
                    .not_null(),
            )
            .col(
                ColumnDef::new(FileEmbeddingIden::UpdatedAt)
                    .timestamp_with_time_zone()
                    .default(Expr::current_timestamp())
                    .not_null(),
            )
            .col(
                ColumnDef::new(FileEmbeddingIden::CreatedAt)
                    .timestamp_with_time_zone()
                    .default(Expr::current_timestamp())
                    .not_null(),
            );
        let sql = statement.to_string(SqliteQueryBuilder);
        Ok(sqlx::query(&sql).execute(pool).await?)
    }

    async fn insert_one(&self, pool: &Pool<Sqlite>, table: &str) -> Result<SqliteQueryResult> {
        let mut operation = InsertStatement::new();
        let statement = operation
            .into_table(Alias::new(table))
            .returning_all()
            .columns([
                FileEmbeddingIden::Id,
                FileEmbeddingIden::Sha,
                FileEmbeddingIden::File,
                FileEmbeddingIden::Path,
                FileEmbeddingIden::Vector,
            ])
            .values([
                Uuid::new_v4().into(),
                self.sha.to_string().into(),
                self.file.to_string().into(),
                self.path.to_string().into(),
                serde_json::to_string(&self.vector)?.into(),
            ])?;
        let sql = statement.to_string(SqliteQueryBuilder);
        Ok(sqlx::query(&sql).execute(pool).await?)
    }
}
