use actions_toolkit::core::{self, Core};
use anyhow::Result;
use std::{path::Path, time::Instant};
use walkdir::{DirEntry, WalkDir};

fn filter_entries(entry: &DirEntry) -> bool {
    let skips = ["github/workspace/.git"];
    let name = entry.file_name().to_str().unwrap_or_default();
    if name.starts_with(".") || skips.contains(&name) {
        return false;
    }
    true
}

#[tokio::main]
async fn main() -> Result<()> {
    let start = Instant::now();
    let mut core = Core::new();

    let name = core::input("name")?;
    core.debug(&format!("Starting file scan for {}", name))?;

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
            core.debug(&format!("Processing: {}", path.path().display()))?;
        }
    }

    core.debug(&format!(
        "Scan complete! Found {} relevant files",
        file_count
    ))?;
    core.set_output("time", format!("{:?}", start.elapsed()))?;
    Ok(())
}
