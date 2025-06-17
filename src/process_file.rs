use actions_toolkit::core;
use anyhow::Result;
use fastembed::TextEmbedding;
use serde::Serialize;
use std::fs;
use std::io::{BufReader, Read};
use walkdir::DirEntry;

use crate::metadata::Action;

#[derive(Debug, Serialize)]
pub struct Embed {
    pub file: String,
    pub path: String,
    pub vector: Vec<f32>,
}

pub fn task(
    model: &TextEmbedding,
    action: &Action,
    entry: Result<DirEntry, walkdir::Error>,
) -> Result<Option<Embed>> {
    let path = match entry {
        Ok(p) => p,
        Err(err) => {
            core::warning(format!("[PATH ERROR]: [{err:?}]"));
            return Ok(None);
        }
    };

    if !path.file_type().is_file() || path.file_type().is_symlink() {
        return Ok(None);
    }

    let file_name = path.file_name().to_string_lossy().to_string();

    let path = path.path();
    let path_str = path.to_string_lossy().to_string();

    if action.is_excluded(&path_str) || path_str.contains(&action.artifact_path) {
        return Ok(None);
    }

    let file_bytes = usize::try_from(path.metadata()?.len())?;

    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(err) => {
            core::warning(format!("[ERROR OPENING] [{file_name}]: [{err:?}]"));
            return Ok(None);
        }
    };

    let mut reader = BufReader::with_capacity(file_bytes, file);
    let mut content = String::with_capacity(file_bytes);

    if let Err(err) = reader.read_to_string(&mut content) {
        core::warning(format!("[ERROR READING] [{file_name}]: [{err:?}]"));
        return Ok(None);
    }

    if content.trim().is_empty() {
        core::warning(format!("[SKIPPING EMPTY CONTENT]: [{file_name}]"));
        return Ok(None);
    }

    // Generate embedding
    let embedding = model.embed(vec![content], None)?;
    let vector = if let Some(vec) = embedding.first() {
        vec.to_owned()
    } else {
        core::warning(format!("[ERROR EMBEDDING]: [{file_name}]"));
        return Ok(None);
    };

    Ok(Some(Embed {
        file: file_name,
        path: path_str,
        vector,
    }))
}
