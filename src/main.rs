mod cli;
mod config;
mod analyzer;
mod models;
mod output;
mod utils;
mod batch;

use anyhow::Result;
use cli::run;

#[tokio::main]
async fn main() -> Result<()> {
    run().await
}
