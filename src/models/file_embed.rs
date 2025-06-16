use anyhow::Result;
use chrono::{DateTime, Utc};
use sea_query::{
    Alias, ColumnDef, Expr, IndexCreateStatement, InsertStatement, SqliteQueryBuilder, Table,
    enum_def,
};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, prelude::FromRow, sqlite::SqliteQueryResult};
use uuid::Uuid;

use crate::models::Model;

#[enum_def]
#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Serialize, FromRow, Deserialize)]
pub struct FileEmbedding {
    pub id: Uuid,
    pub sha: String,
    pub file: String,
    pub path: String,
    pub vector: Vec<f32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for FileEmbedding {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            sha: String::default(),
            file: String::default(),
            path: String::default(),
            vector: Vec::default(),
            created_at: DateTime::default(),
            updated_at: DateTime::default(),
        }
    }
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
                self.id.into(),
                self.sha.to_string().into(),
                self.file.to_string().into(),
                self.path.to_string().into(),
                serde_json::to_string(&self.vector)?.into(),
            ])?;
        let sql = statement.to_string(SqliteQueryBuilder);
        Ok(sqlx::query(&sql).execute(pool).await?)
    }
}

impl FileEmbedding {
    pub async fn create_indexes(
        pool: &Pool<Sqlite>,
        table: &str,
    ) -> Result<(SqliteQueryResult, SqliteQueryResult)> {
        let mut operation = IndexCreateStatement::new();
        let first_statement = operation
            .if_not_exists()
            .table(Alias::new(table))
            .name("file_name_idx")
            .col(FileEmbeddingIden::File);
        let first_sql = first_statement.to_string(SqliteQueryBuilder);
        let second_statement = operation
            .if_not_exists()
            .table(Alias::new(table))
            .name("path_name_idx")
            .col(FileEmbeddingIden::Path);
        let second_sql = second_statement.to_string(SqliteQueryBuilder);
        // migrate
        let created = futures::try_join!(
            sqlx::query(&first_sql).execute(pool),
            sqlx::query(&second_sql).execute(pool)
        )?;
        Ok(created)
    }

    pub async fn bulk_insert(
        pool: &Pool<Sqlite>,
        data: Vec<Self>,
        table: &str,
    ) -> Result<SqliteQueryResult> {
        let mut transaction = pool.begin().await?;

        let mut operation = InsertStatement::new();
        let statement = operation.into_table(Alias::new(table)).columns([
            FileEmbeddingIden::Id,
            FileEmbeddingIden::Sha,
            FileEmbeddingIden::File,
            FileEmbeddingIden::Path,
            FileEmbeddingIden::Vector,
        ]);

        const CHUNK_SIZE: usize = 1_000;
        let mut last_result = None;

        for chunk in data.chunks(CHUNK_SIZE) {
            let mut batch_statement = statement.clone();

            for entry in chunk {
                batch_statement.values([
                    entry.id.into(),
                    entry.sha.to_string().into(),
                    entry.file.to_string().into(),
                    entry.path.to_string().into(),
                    serde_json::to_string(&entry.vector)?.into(),
                ])?;
            }

            let sql = batch_statement.to_string(SqliteQueryBuilder);
            last_result = Some(
                sqlx::query(&sql)
                    .persistent(true)
                    .execute(&mut *transaction)
                    .await?,
            );
        }

        transaction.commit().await?;

        Ok(last_result.unwrap_or_else(|| SqliteQueryResult::default()))
    }
}
