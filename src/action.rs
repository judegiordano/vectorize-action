use std::time::Instant;

use actions_toolkit::core;
use anyhow::Result;
use fastembed::{InitOptions, TextEmbedding};
use sqlx::SqlitePool;
use walkdir::WalkDir;

use crate::metadata::{Action, DATA_PATH};
use crate::models::{Model, file_embed::FileEmbedding};
use crate::{process_file, sql};

pub async fn run() -> Result<()> {
    let start = Instant::now();
    let mut action = Action::new()?;
    let model = TextEmbedding::try_new(InitOptions::default())?;
    // sql connect
    let table_name = action.commit_sha.clone();
    sql::generate_db_file(&action.db_url).await?;
    let pool = SqlitePool::connect(&action.db_url).await?;

    // migrate
    let result = FileEmbedding::create_table(&pool, &table_name).await?;
    core::debug(format!("[TABLE CREATED]: {result:?}"));
    let result = FileEmbedding::create_indexes(&pool, &table_name).await?;
    core::debug(format!("[INDEXES CREATED]: {result:?}"));

    // process files
    let entries = WalkDir::new(&action.workspace_path)
        .follow_links(false)
        .into_iter();
    core::debug(format!("[EXCLUSIONS]: {:?}", action.inputs.excludes));
    'file_iter: for entry in entries {
        let path = entry?;
        let data = match process_file::task(&model, &action, &path) {
            Ok(data) => data,
            Err(err) => {
                core::error(format!("[UNHANDLED ERROR]: [{err:#?}]"));
                continue 'file_iter;
            }
        };
        if let Some(embed) = data {
            let data = FileEmbedding {
                file: embed.file,
                path: embed.path,
                vector: embed.vector,
                sha: action.commit_sha.to_string(),
                ..Default::default()
            };
            let inserted = data.insert_one(&pool, &table_name).await?;
            core::debug(format!("[SAVING]: {inserted:?}"));
        }
    }
    // set outputs
    action.core.set_output("data_path", DATA_PATH)?;
    core::debug(format!("[OPERATION COMPLETE]: {:?}", start.elapsed()));
    Ok(())
}
