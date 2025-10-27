use clap::Parser;
use color_eyre::eyre::Result;
use crate::app::App;
use crate::cli::Cli;

mod errors;
mod tui;
mod event;
mod app;
mod component;
mod action;
mod config;
mod cli;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    better_panic::install();
    let cli = Cli::parse();
    match App::new_in_editor(cli.file_dir) {
        Ok(mut app) => app.run().await?,
        Err(e) => {
            let msg = format!("Error creating application: {:?}", e);
            color_eyre::eyre::bail!(msg)
        },
    }
    Ok(())
}
