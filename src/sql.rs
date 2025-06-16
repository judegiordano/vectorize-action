use anyhow::Result;
use sqlx::{Sqlite, migrate::MigrateDatabase};
use std::fs;

use crate::metadata::DATA_PATH;

pub async fn generate_db_file(db_url: &str) -> Result<()> {
    if !Sqlite::database_exists(db_url).await? {
        fs::create_dir_all(DATA_PATH)?;
        Sqlite::create_database(db_url).await?;
    }
    Ok(())
}
