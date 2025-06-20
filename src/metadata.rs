use actions_toolkit::core::{self};
use anyhow::Result;
use std::path::{Path, PathBuf};

pub const DATA_PATH: &str = ".artifact_data";

pub struct Inputs {
    pub excludes: Vec<String>,
}

impl Inputs {
    pub fn new() -> Result<Self> {
        Ok(Self {
            excludes: core::input("exclude")?
                .split(',')
                .map(|str: &str| str.to_string())
                .collect(),
        })
    }
}

pub struct Action {
    pub commit_sha: String,
    pub artifact_path: String,
    pub db_url: String,
    pub workspace_path: PathBuf,
    pub inputs: Inputs,
}

impl Action {
    pub fn new() -> Result<Self> {
        let workspace = std::env::var("GITHUB_WORKSPACE")?;
        let commit_sha = std::env::var("GITHUB_SHA")?;
        let workspace_path = Path::new(&workspace).to_owned();
        let artifact_dir = Path::new(&DATA_PATH);
        let mut db_path = artifact_dir.join(&commit_sha);
        db_path.set_extension("db");
        Ok(Self {
            commit_sha,
            artifact_path: workspace_path.join(DATA_PATH).to_string_lossy().to_string(),
            db_url: format!("sqlite:{}", db_path.to_string_lossy()),
            workspace_path,
            inputs: Inputs::new()?,
        })
    }

    #[inline]
    pub fn is_excluded(&self, path: &str) -> bool {
        self.inputs.excludes.iter().any(|skip| path.contains(skip))
    }
}
