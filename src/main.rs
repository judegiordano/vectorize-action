use anyhow::Result;
use fastembed::TextEmbedding;
use sea_query::{Alias, ColumnDef, Expr, InsertStatement, SqliteQueryBuilder, Table};
use sqlx::SqlitePool;
use uuid::Uuid;
use walkdir::WalkDir;

use crate::{
    metadata::{Action, DATA_PATH},
    sql::FileEmbeddingIden,
};

mod metadata;
mod process_file;
mod sql;

#[tokio::main]
async fn main() -> Result<()> {
    let mut action = Action::new()?;
    let model = TextEmbedding::try_new(Default::default())?;
    // sql connect
    let table_name = action.commit_sha.clone();
    sql::generate_db_file(&action.db_url).await?;
    let pool = SqlitePool::connect(&action.db_url).await?;
    {
        let mut operation = Table::create();
        let statement = operation
            .table(Alias::new(&table_name))
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
        let stmt = statement.to_string(SqliteQueryBuilder);
        sqlx::query(&stmt).execute(&pool).await?;
    }
    // process
    let entries = WalkDir::new(&action.workspace_path)
        .follow_links(true)
        .into_iter();
    for entry in entries {
        let path = entry?;
        if let Some(embed) = process_file::task(&model, &action, &path)? {
            {
                let mut operation = InsertStatement::new();
                let statement = operation
                    .into_table(Alias::new(&table_name))
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
                        action.commit_sha.to_string().into(),
                        embed.file.into(),
                        embed.path.into(),
                        serde_json::to_string(&embed.vector)?.into(),
                    ])?;
                let stmt = statement.to_string(SqliteQueryBuilder);
                sqlx::query(&stmt).execute(&pool).await?;
            }
        }
    }
    // set outputs
    action.core.set_output("data_path", DATA_PATH)?;
    Ok(())
}
