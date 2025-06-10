use actions_toolkit::core::{self, Core};
use anyhow::Result;
use std::{path::Path, time::Instant};
use walkdir::{DirEntry, WalkDir};

fn skip_entry(dir: &DirEntry) -> bool {
    dir.file_name()
        .to_str()
        .map(|a| a.starts_with(".git"))
        .unwrap_or(false)
}

#[tokio::main]
async fn main() -> Result<()> {
    let start = Instant::now();
    let mut core = Core::new();

    let name = core::input("name")?;
    core.debug(&format!("Hello, {}!", name))?;

    //
    let workspace = std::env::var("GITHUB_WORKSPACE")?;
    let file_path = Path::new(&workspace);
    let entries = WalkDir::new(file_path)
        .follow_links(true)
        .into_iter()
        .filter_entry(skip_entry);
    for entry in entries {
        match entry {
            Ok(entry) => {
                core.debug(&format!("[ENTRY]: {:?}", entry.path()))?;
            }
            Err(e) => {
                core.debug(&format!("[ERROR READING]: {}", e))?;
            }
        }
    }
    //
    core.set_output("time", format!("{:?}", start.elapsed()))?;
    Ok(())
}
