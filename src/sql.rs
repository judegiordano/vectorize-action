use anyhow::Result;
use sqlx::{Pool, Sqlite, migrate::MigrateDatabase, pool::PoolOptions};
use std::{fs, sync::Arc};

use crate::metadata::DATA_PATH;

static DB_POOL_CONNECTIONS: u32 = 10;

const CACHE_SIZE_MB: i32 = 512; // 512MB cache instead of 2GB
const MMAP_SIZE: i64 = 2_147_483_648; // 2GB mmap size instead of 30GB

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
    optimize_connection(&pool).await?;
    Ok(Arc::new(pool))
}

async fn optimize_connection(pool: &Pool<Sqlite>) -> Result<()> {
    // Enable WAL mode for better concurrent performance
    sqlx::query("PRAGMA journal_mode = WAL")
        .execute(pool)
        .await?;

    // Use normal sync for better performance in container environments
    sqlx::query("PRAGMA synchronous = NORMAL")
        .execute(pool)
        .await?;

    // Set cache size based on available memory
    sqlx::query(&format!("PRAGMA cache_size = -{}", CACHE_SIZE_MB * 1024))
        .execute(pool)
        .await?;

    // Set memory map size
    sqlx::query(&format!("PRAGMA mmap_size = {MMAP_SIZE}"))
        .execute(pool)
        .await?;

    // Enable multi-threaded processing but limit threads
    sqlx::query("PRAGMA threads = 4").execute(pool).await?;

    // Use larger page size for better performance with large data
    sqlx::query("PRAGMA page_size = 32768")
        .execute(pool)
        .await?;

    // Disable file system directory sync for better performance
    sqlx::query("PRAGMA fullfsync = OFF").execute(pool).await?;

    // Temporary tables in memory
    sqlx::query("PRAGMA temp_store = MEMORY")
        .execute(pool)
        .await?;
    Ok(())
}
