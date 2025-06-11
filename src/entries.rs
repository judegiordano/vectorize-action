use std::path::Path;
use walkdir::{DirEntry, FilterEntry, IntoIter, WalkDir};

pub const DATA_PATH: &str = ".artifact_data";

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

pub fn task(path: &Path) -> FilterEntry<IntoIter, fn(&DirEntry) -> bool> {
    WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_entry(filter_entries)
}
