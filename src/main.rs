mod cli;
mod config;
mod core;
mod models;
mod display;
mod utils;

use anyhow::Result;
use cli::run;

#[tokio::main]
async fn main() -> Result<()> {
    run().await
}
