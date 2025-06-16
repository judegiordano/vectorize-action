use actions_toolkit::core;
use anyhow::Result;
use fastembed::TextEmbedding;
use serde::Serialize;
use std::fs;
use walkdir::DirEntry;

use crate::metadata::Action;

#[derive(Debug, Serialize)]
pub struct Embed {
    pub file: String,
    pub path: String,
    pub vector: Vec<f32>,
}

pub fn task(model: &TextEmbedding, action: &Action, path: &DirEntry) -> Result<Option<Embed>> {
    if path.path_is_symlink() || path.file_type().is_dir() {
        return Ok(None);
    }
    let file_name = path.file_name().to_string_lossy().to_string();
    let path = path.path();
    if let Err(err) = path.try_exists() {
        core::warning(&format!("[ERROR ACCESSING] [{file_name}]: [{err:?}]"));
        return Ok(None);
    }
    let path_str = path.to_string_lossy().to_string();
    // excludes
    if action
        .inputs
        .excludes
        .iter()
        .any(|skip| path_str.contains(skip))
        || path_str.contains(&action.artifact_path)
    {
        return Ok(None);
    }
    core::debug(&format!("[PROCESSING]: {path_str}"));
    let file_content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(err) => {
            core::warning(&format!("[ERROR READING] [{file_name}]: [{err:?}]"));
            return Ok(None);
        }
    };
    let embedding = model.embed(vec![file_content], None)?;
    Ok(Some(Embed {
        file: file_name,
        path: path_str,
        vector: embedding.first().unwrap().to_vec(),
    }))
}
