use std::fs;

use actions_toolkit::core;
use anyhow::Result;
use fastembed::TextEmbedding;
use serde::Serialize;
use walkdir::DirEntry;

#[derive(Debug, Serialize)]
pub struct Embed {
    pub file: String,
    pub path: String,
    pub vector: Vec<f32>,
}

pub fn task(model: &TextEmbedding, path: &DirEntry) -> Result<Option<Embed>> {
    if !path.file_type().is_file() {
        return Ok(None);
    }
    let file_name = path.file_name().to_string_lossy().to_string();
    let path = path.path();
    core::debug(&format!("[PROCESSING]: {}", file_name));
    let file_content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(err) => {
            core::warning(&format!("[ERROR READING] [{file_name}]: [{err:?}]"));
            return Ok(None);
        }
    };
    let embedding = model.embed(vec![file_content], None)?;
    let embed = Embed {
        file: file_name,
        path: path.to_string_lossy().to_string(),
        vector: embedding.first().unwrap().to_vec(),
    };
    Ok(Some(embed))
}
