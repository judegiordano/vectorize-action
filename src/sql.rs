use anyhow::Result;
use sqlx::{Pool, Sqlite, migrate::MigrateDatabase, pool::PoolOptions};
use std::{fs, sync::Arc};

use crate::metadata::DATA_PATH;

static DB_POOL_CONNECTIONS: u32 = 10;

async fn generate_db_file(db_url: &str) -> Result<()> {
    if !Sqlite::database_exists(db_url).await? {
        fs::create_dir_all(DATA_PATH)?;
        Sqlite::create_database(db_url).await?;
    }
    Ok(())
}

pub async fn connect(url: &str) -> Result<Arc<Pool<Sqlite>>> {
    generate_db_file(url).await?;
    let pool = PoolOptions::<Sqlite>::new()
        .max_connections(DB_POOL_CONNECTIONS)
        .connect(url)
        .await?;
    // sqlite settings
    {
        sqlx::query("PRAGMA journal_mode = WAL")
            .execute(&pool)
            .await?;
        // Reduce fsync calls for better performance at cost of some durability
        sqlx::query("PRAGMA synchronous = NORMAL")
            .execute(&pool)
            .await?;
        // Use 2GB memory for cache
        sqlx::query("PRAGMA cache_size = -2000000")
            .execute(&pool)
            .await?;
        // Enable memory mapping for better performance
        sqlx::query("PRAGMA mmap_size = 30000000000")
            .execute(&pool)
            .await?;
        // Enable multi-threaded processing
        sqlx::query("PRAGMA threads = 4").execute(&pool).await?;
        // Larger page size for better performance with large data
        sqlx::query("PRAGMA page_size = 32768")
            .execute(&pool)
            .await?;
    }
    Ok(Arc::new(pool))
}
