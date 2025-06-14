use actions_toolkit::core;
use anyhow::Result;
use fastembed::TextEmbedding;
use sqlx::SqlitePool;
use walkdir::WalkDir;

use crate::{
    metadata::{Action, DATA_PATH},
    models::{Model, file_embed::FileEmbedding},
};

mod metadata;
mod models;
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

    let result = FileEmbedding::create_table(&pool, &table_name).await?;
    core::debug(&format!("[TABLE CREATED]: {result:?}"));

    // process files
    let entries = WalkDir::new(&action.workspace_path)
        .follow_links(true)
        .into_iter();
    core::debug(&format!("[EXCLUSIONS]: {:?}", action.inputs.excludes));
    for entry in entries {
        let path = entry?;
        if let Some(embed) = process_file::task(&model, &action, &path)? {
            let data = FileEmbedding {
                file: embed.file,
                path: embed.path,
                vector: embed.vector,
                sha: action.commit_sha.to_string(),
                ..Default::default()
            };
            let inserted = data.insert_one(&pool, &table_name).await?;
            core::debug(&format!("[SAVING]: {inserted:?}"));
        }
    }
    // set outputs
    action.core.set_output("data_path", DATA_PATH)?;
    Ok(())
}
