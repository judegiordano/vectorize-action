use anyhow::Result;
use fastembed::TextEmbedding;
use serde_json::json;
use std::{fs, path::Path, time::Instant};
use walkdir::WalkDir;

use crate::metadata::{Action, DATA_PATH};

mod metadata;
mod process_file;

#[tokio::main]
async fn main() -> Result<()> {
    let model = TextEmbedding::try_new(Default::default())?;
    let start = Instant::now();

    let mut action = Action::new()?;

    // process
    let mut embeds = vec![];
    // let entries = entries::task(&action);
    let entries = WalkDir::new(&action.workspace_path)
        .follow_links(true)
        .into_iter();
    for entry in entries {
        let path = entry?;
        if let Some(embed) = process_file::task(&model, &action, &path)? {
            embeds.push(embed);
        }
    }
    let report = json!({
        "sha": action.commit_sha,
        "total": embeds.len(),
        "time_taken": format!("{:?}", start.elapsed()),
        "embeddings": embeds,
    });
    let output_file_name = format!("{}.json", action.commit_sha);
    let artifact_path = Path::new(&action.artifact_path);
    fs::create_dir_all(&artifact_path)?;

    // flush
    let joined_path = artifact_path.join(output_file_name);
    fs::write(&joined_path, serde_json::to_string_pretty(&report)?)?;

    // set outputs
    action.core.set_output("data_path", DATA_PATH)?;
    Ok(())
}
