mod action;
mod metadata;
mod models;
mod process_file;
mod sql;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    action::run().await
}
