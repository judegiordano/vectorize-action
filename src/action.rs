use actions_toolkit::core::{self};
use anyhow::Result;
use fastembed::{InitOptions, TextEmbedding};
use rayon::iter::{IntoParallelIterator, ParallelBridge, ParallelIterator};
use std::time::Instant;
use walkdir::{DirEntry, WalkDir};

use crate::metadata::Action;
use crate::models::{Model, file_embed::FileEmbedding};
use crate::{process_file, sql};

fn task(
    model: &TextEmbedding,
    action: &Action,
    entry: Result<DirEntry, walkdir::Error>,
) -> Option<FileEmbedding> {
    let data = match process_file::task(model, action, entry) {
        Ok(data) => data,
        Err(err) => {
            core::error(format!("[UNHANDLED ERROR]: [{err:#?}]"));
            return None;
        }
    };
    if let Some(embed) = data {
        return Some(FileEmbedding {
            file: embed.file,
            path: embed.path,
            vector: embed.vector,
            sha: action.commit_sha.to_string(),
            ..FileEmbedding::default()
        });
    }
    None
}

pub async fn run() -> Result<()> {
    let start = Instant::now();
    let action = Action::new()?;
    let model = TextEmbedding::try_new(InitOptions::default())?;
    let table_name = action.commit_sha.clone();
    // connect
    let pool = sql::connect(&action.db_url).await?;

    // migrate
    let result = FileEmbedding::create_table(&pool, &table_name).await?;
    core::debug(format!("[TABLE CREATED]: {result:?}"));
    let result = FileEmbedding::create_indexes(&pool, &table_name).await?;
    core::debug(format!("[INDEXES CREATED]: {result:?}"));

    // create parallel iterator
    core::debug(format!("[FILE EXCLUSIONS]: {:?}", action.inputs.excludes));
    let entries = WalkDir::new(&action.workspace_path)
        .follow_links(false)
        .into_iter()
        .par_bridge()
        .into_par_iter();

    // process / filter
    let data = entries
        .filter_map(|entry| task(&model, &action, entry))
        .collect();

    // save to sqlite
    core::debug("[WRITING TO DB]");
    FileEmbedding::bulk_insert(&pool, data, &table_name).await?;
    core::debug(format!("[OPERATION COMPLETE]: {:?}", start.elapsed()));
    Ok(())
}
