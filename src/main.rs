use std::path::PathBuf;

use clap::Parser;
use ratatelier::{App, Project, app, storage};

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Cli {
    /// Open an existing .ron project.
    project: Option<PathBuf>,

    /// Width for a newly created canvas.
    #[arg(long, default_value_t = 48)]
    width: u16,

    /// Height for a newly created canvas.
    #[arg(long, default_value_t = 18)]
    height: u16,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let (project, path) = match cli.project {
        Some(path) => (storage::load_project(&path)?, Some(path)),
        None => (Project::new("Untitled", cli.width, cli.height), None),
    };

    let mut app = App::new(project, path);
    app::run(&mut app)?;
    Ok(())
}
