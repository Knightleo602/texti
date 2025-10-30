use crate::app::App;
use crate::cli::Cli;
use clap::Parser;
use color_eyre::eyre::Result;

mod action;
mod app;
mod cli;
mod component;
mod config;
mod errors;
mod event;
mod tui;

#[tokio::main]
async fn main() -> Result<()> {
    errors::init()?;
    let cli = Cli::parse();
    match App::new_in_editor(cli.file_dir) {
        Ok(mut app) => app.run().await?,
        Err(e) => {
            let msg = format!("Error creating application: {:?}", e);
            color_eyre::eyre::bail!(msg)
        }
    }
    Ok(())
}
