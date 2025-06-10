use actions_toolkit::core::{self, Core};
use anyhow::Result;
use std::{
    path::Path,
    time::{Duration, Instant},
};

#[tokio::main]
async fn main() -> Result<()> {
    // std::env::var("GITHUB_WORKSPACE")
    let start = Instant::now();
    let mut core = Core::new();

    let name = core::input("name")?;
    core.debug(&format!("Hello, {}!", name))?;

    tokio::time::sleep(Duration::from_secs(1)).await;

    //
    let workspace = std::env::var("GITHUB_WORKSPACE")?;
    let file_path = Path::new(&workspace);
    let dirs = file_path.ancestors();
    core.debug(&format!("[DIRS]: [{}]", dirs.count()))?;
    //

    core.set_output("time", format!("{:?}", start.elapsed()))?;
    Ok(())
}
