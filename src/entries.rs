use actions_toolkit::core;
use walkdir::{DirEntry, FilterEntry, IntoIter, WalkDir};

use crate::metadata::Action;

pub const DATA_PATH: &str = ".artifact_data";

fn filter_entries(entry: &DirEntry, skips: Vec<String>) -> bool {
    let name = entry.path().to_str().unwrap_or_default();
    if skips.iter().any(|skip| name.contains(skip)) {
        return false;
    }
    true
}

pub fn task(action: &Action) -> FilterEntry<IntoIter, impl FnMut(&DirEntry) -> bool> {
    let mut skips = vec![format!("github/workspace/{DATA_PATH}")];
    skips.extend(action.inputs.excludes.iter().cloned());
    core::debug(&format!("[SKIPPING]: {:#?}", skips));
    WalkDir::new(action.workspace_path.clone())
        .follow_links(true)
        .into_iter()
        .filter_entry(move |dir| filter_entries(dir, skips.clone()))
}
