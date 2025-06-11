use actions_toolkit::core::{self, Core};
use anyhow::Result;
use serde_json::json;
use std::{fs, path::Path, time::Instant};
use walkdir::{DirEntry, WalkDir};

fn filter_entries(entry: &DirEntry) -> bool {
    // by default, only skip .git folder
    let skips = ["github/workspace/.git/", "github/workspace/.artifact_data/"];
    // todo: extend skips from actions yml
    let name = entry.path().to_str().unwrap_or_default();
    if skips.iter().any(|skip| name.contains(skip)) {
        return false;
    }
    true
}

#[tokio::main]
async fn main() -> Result<()> {
    let start = Instant::now();
    let mut core = Core::new();

    let name = core::input("name")?;
    core.debug(&format!("hello, {}", name))?;

    let workspace = std::env::var("GITHUB_WORKSPACE")?;
    let commit_sha = std::env::var("GITHUB_SHA")?;
    let workspace_path = Path::new(&workspace);

    // process
    let mut files = vec![];
    let entries = WalkDir::new(workspace_path)
        .follow_links(true)
        .into_iter()
        .filter_entry(filter_entries);
    for entry in entries {
        let path = entry?;
        if path.clone().file_type().is_file() {
            let file_name = path.file_name().to_string_lossy().to_string();
            files.push(file_name);
            core.debug(&format!("[PROCESSING]: {}", path.path().display()))?;
        }
    }
    let report = json!({
        "scan_info": {
            "sha": commit_sha,
            "total_files": files.len(),
            "time_taken": format!("{:?}", start.elapsed()),
            "scanned_paths": files
        }
    });
    let output_file_name = format!("{commit_sha}.json");
    let artifact_dir = workspace_path.join(".artifact_data");
    fs::create_dir_all(&artifact_dir)?;

    // flush
    let data_path = artifact_dir.join(output_file_name);
    fs::write(&data_path, serde_json::to_string_pretty(&report)?)?;

    // set outputs
    core.set_output("data_path", data_path.to_string_lossy())?;
    Ok(())
}
