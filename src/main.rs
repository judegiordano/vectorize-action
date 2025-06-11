use anyhow::Result;
use fastembed::TextEmbedding;
use serde_json::json;
use std::{fs, time::Instant};

use crate::{entries::DATA_PATH, metadata::Action};

mod entries;
mod metadata;
mod process_file;

#[tokio::main]
async fn main() -> Result<()> {
    let model = TextEmbedding::try_new(Default::default())?;
    let start = Instant::now();

    let mut action = Action::new()?;
    action
        .core
        .debug(&format!("hello, {}", action.inputs.name))?;

    // process
    let mut embeds = vec![];
    let entries = entries::task(&action.workspace_path);
    for entry in entries {
        let path = entry?;
        if let Some(embed) = process_file::task(&model, &path)? {
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
    let artifact_dir = action.workspace_path.join(DATA_PATH);
    fs::create_dir_all(&artifact_dir)?;

    // flush
    let joined_path = artifact_dir.join(output_file_name);
    fs::write(&joined_path, serde_json::to_string_pretty(&report)?)?;

    // set outputs
    action.core.set_output("data_path", DATA_PATH)?;
    Ok(())
}
