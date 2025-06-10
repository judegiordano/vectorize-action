use actions_toolkit::core::{self, Core};
use anyhow::Result;
use std::{path::Path, time::Instant};
use walkdir::{DirEntry, WalkDir};

fn filter_entries(entry: &DirEntry) -> bool {
    // by default, only skip .git folder
    let skips = ["github/workspace/.git"];
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
    let file_path = Path::new(&workspace);

    let mut file_count = 0;
    let entries = WalkDir::new(file_path)
        .follow_links(true)
        .into_iter()
        .filter_entry(filter_entries);
    for entry in entries {
        let path = entry?;
        if path.file_type().is_file() {
            file_count += 1;
            core.debug(&format!("[PROCESSING]: {}", path.path().display()))?;
        }
    }

    core.debug(&format!("[PROCESSING COMPLETE]: [{} FILES]", file_count))?;
    core.set_output("time", format!("{:?}", start.elapsed()))?;
    Ok(())
}
