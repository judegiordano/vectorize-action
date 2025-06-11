use actions_toolkit::core::{self, Core};
use anyhow::Result;
use std::{
    io::Stdout,
    path::{Path, PathBuf},
};

pub struct Inputs {
    pub name: String,
}

impl Inputs {
    pub fn new() -> Result<Self> {
        Ok(Self {
            name: core::input("name")?,
        })
    }
}

pub struct Action {
    pub core: Core<Stdout>,
    pub commit_sha: String,
    pub workspace_path: PathBuf,
    pub inputs: Inputs,
}

impl Action {
    pub fn new() -> Result<Self> {
        let core = Core::new();
        let workspace = std::env::var("GITHUB_WORKSPACE")?;
        let commit_sha = std::env::var("GITHUB_SHA")?;
        let workspace_path = Path::new(&workspace).to_owned();
        Ok(Self {
            core,
            commit_sha,
            workspace_path,
            inputs: Inputs::new()?,
        })
    }
}
