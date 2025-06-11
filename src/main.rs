use actions_toolkit::core::{self, Core};
use anyhow::Result;
use fastembed::TextEmbedding;
use serde::Serialize;
use serde_json::json;
use std::{fs, path::Path, time::Instant};
use walkdir::{DirEntry, WalkDir};

const DATA_PATH: &str = ".artifact_data";

#[derive(Debug, Serialize)]
pub struct Embed {
    pub file: String,
    pub path: String,
    pub vector: Vec<f32>,
}

fn filter_entries(entry: &DirEntry) -> bool {
    let skips = [
        "github/workspace/.git/",
        "github/workspace/.fastembed_cache/",
        &format!("github/workspace/{DATA_PATH}"),
    ];
    // todo: extend skips from actions yml
    let name = entry.path().to_str().unwrap_or_default();
    if skips.iter().any(|skip| name.contains(skip)) {
        return false;
    }
    true
}

#[tokio::main]
async fn main() -> Result<()> {
    let model = TextEmbedding::try_new(Default::default())?;
    let start = Instant::now();
    let mut core = Core::new();

    let name = core::input("name")?;
    core.debug(&format!("hello, {}", name))?;

    let workspace = std::env::var("GITHUB_WORKSPACE")?;
    let commit_sha = std::env::var("GITHUB_SHA")?;
    let workspace_path = Path::new(&workspace);

    // process
    let mut embeds = vec![];
    let entries = WalkDir::new(workspace_path)
        .follow_links(true)
        .into_iter()
        .filter_entry(filter_entries);
    for entry in entries {
        let path = entry?;
        if path.clone().file_type().is_file() {
            core.debug(&format!("[PROCESSING]: {}", path.path().display()))?;
            let file_name = path.file_name().to_string_lossy().to_string();
            let file_content = match fs::read_to_string(path.path()) {
                Ok(content) => content,
                Err(err) => {
                    core.warning(&format!("[ERROR READING] [{file_name}]: [{err:?}]"))?;
                    continue;
                }
            };
            let embedding = model.embed(vec![file_content], None)?;
            let embedding = Embed {
                file: file_name,
                path: path.path().to_string_lossy().to_string(),
                vector: embedding.first().unwrap().to_vec(),
            };
            embeds.push(embedding);
        }
    }
    let report = json!({
        "sha": commit_sha,
        "total": embeds.len(),
        "time_taken": format!("{:?}", start.elapsed()),
        "embeddings": embeds,
    });
    let output_file_name = format!("{commit_sha}.json");
    let artifact_dir = workspace_path.join(DATA_PATH);
    fs::create_dir_all(&artifact_dir)?;

    // flush
    let joined_path = artifact_dir.join(output_file_name);
    fs::write(&joined_path, serde_json::to_string_pretty(&report)?)?;

    // set outputs
    core.set_output("data_path", DATA_PATH)?;
    Ok(())
}
