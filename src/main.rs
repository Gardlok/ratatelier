use std::{
    io::{self, Write},
    path::PathBuf,
};

use clap::{Args, Parser, Subcommand, ValueEnum};
use ratatelier::{
    App, Project, app, export, storage,
    viewer::{self, ViewerOptions, ViewerSurface},
};

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Open an existing .ron project in the editor.
    project: Option<PathBuf>,

    /// Width for a newly created canvas.
    #[arg(long, default_value_t = 48)]
    width: u16,

    /// Height for a newly created canvas.
    #[arg(long, default_value_t = 18)]
    height: u16,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Open a project in the minimal playback viewer.
    View(ViewArgs),

    /// Convert a project without opening the editor.
    Export(ExportArgs),
}

#[derive(Debug, Args)]
struct ViewArgs {
    /// Project file to display.
    project: PathBuf,

    /// Project surface to display.
    #[arg(long, value_enum, default_value = "artwork")]
    surface: SurfaceArg,

    /// Restart automatically after the final animation frame.
    #[arg(long = "loop")]
    loop_animation: bool,
}

#[derive(Debug, Args)]
struct ExportArgs {
    /// Project file to convert.
    project: PathBuf,

    /// Output representation.
    #[arg(long, short = 'f', value_enum)]
    format: ExportFormat,

    /// Write to this file instead of standard output.
    #[arg(long, short = 'o')]
    output: Option<PathBuf>,

    /// Select a one-based frame number, `first`, or `last`.
    #[arg(long)]
    frame: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum SurfaceArg {
    Artwork,
    Components,
}

impl From<SurfaceArg> for ViewerSurface {
    fn from(surface: SurfaceArg) -> Self {
        match surface {
            SurfaceArg::Artwork => Self::Artwork,
            SurfaceArg::Components => Self::Components,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum ExportFormat {
    Plain,
    Ansi,
    #[value(name = "rust-art")]
    RustArt,
    #[value(name = "rust-animation")]
    RustAnimation,
    #[value(name = "rust-component")]
    RustComponent,
}

impl ExportFormat {
    const fn supports_frame_selection(self) -> bool {
        matches!(self, Self::Plain | Self::Ansi | Self::RustArt)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        Some(Command::View(args)) => run_view(args)?,
        Some(Command::Export(args)) => run_export(args)?,
        None => run_editor(cli.project, cli.width, cli.height)?,
    }
    Ok(())
}

fn run_editor(
    path: Option<PathBuf>,
    width: u16,
    height: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let (project, current_path) = match path {
        Some(path) => (storage::load_project(&path)?, Some(path)),
        None => (Project::new("Untitled", width, height), None),
    };
    let mut app = App::new(project, current_path);
    app::run(&mut app)?;
    Ok(())
}

fn run_view(args: ViewArgs) -> Result<(), Box<dyn std::error::Error>> {
    let project = storage::load_project(&args.project)?;
    viewer::run(
        project,
        ViewerOptions {
            surface: args.surface.into(),
            loop_animation: args.loop_animation,
        },
    )?;
    Ok(())
}

fn run_export(args: ExportArgs) -> Result<(), Box<dyn std::error::Error>> {
    if args.frame.is_some() && !args.format.supports_frame_selection() {
        return Err(invalid_input(format!(
            "--frame cannot be combined with --format {} because that format exports the complete {}",
            args.format.possible_value().expect("value enum has a name").get_name(),
            match args.format {
                ExportFormat::RustAnimation => "animation",
                ExportFormat::RustComponent => "component state set",
                _ => unreachable!(),
            }
        ))
        .into());
    }

    let mut project = storage::load_project(&args.project)?;
    if let Some(selector) = args.frame.as_deref() {
        select_frame(&mut project, selector)?;
    }

    let output = match args.format {
        ExportFormat::Plain => export::export_plain(&project),
        ExportFormat::Ansi => export::export_ansi(&project),
        ExportFormat::RustArt => export::export_art(&project),
        ExportFormat::RustAnimation => export::export_animation(&project),
        ExportFormat::RustComponent => export::export_component(&project),
    };

    if let Some(path) = args.output {
        storage::save_text(&path, &output)?;
    } else {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        handle.write_all(output.as_bytes())?;
        handle.flush()?;
    }
    Ok(())
}

fn select_frame(project: &mut Project, selector: &str) -> io::Result<()> {
    let index = match selector.trim().to_ascii_lowercase().as_str() {
        "first" => 0,
        "last" => project.frames.len().saturating_sub(1),
        value => {
            let number = value.parse::<usize>().map_err(|_| {
                invalid_input("frame must be a one-based number, `first`, or `last`")
            })?;
            number
                .checked_sub(1)
                .ok_or_else(|| invalid_input("frame numbers begin at 1"))?
        }
    };
    if index >= project.frames.len() {
        return Err(invalid_input(format!(
            "frame must be between 1 and {}",
            project.frames.len()
        )));
    }
    project.active_frame = index;
    Ok(())
}

fn invalid_input(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_project_argument_still_opens_editor_mode() {
        let cli = Cli::try_parse_from(["ratatelier", "demo.ron"]).unwrap();
        assert!(cli.command.is_none());
        assert_eq!(cli.project, Some(PathBuf::from("demo.ron")));
    }

    #[test]
    fn viewer_subcommand_parses_surface_and_loop() {
        let cli = Cli::try_parse_from([
            "ratatelier",
            "view",
            "demo.ron",
            "--surface",
            "components",
            "--loop",
        ])
        .unwrap();
        let Some(Command::View(args)) = cli.command else {
            panic!("view command was not parsed");
        };
        assert_eq!(args.project, PathBuf::from("demo.ron"));
        assert_eq!(args.surface, SurfaceArg::Components);
        assert!(args.loop_animation);
    }

    #[test]
    fn export_subcommand_parses_headless_format() {
        let cli = Cli::try_parse_from([
            "ratatelier",
            "export",
            "demo.ron",
            "--format",
            "rust-animation",
        ])
        .unwrap();
        let Some(Command::Export(args)) = cli.command else {
            panic!("export command was not parsed");
        };
        assert_eq!(args.format, ExportFormat::RustAnimation);
    }

    #[test]
    fn only_artwork_formats_support_frame_selection() {
        assert!(ExportFormat::Plain.supports_frame_selection());
        assert!(ExportFormat::Ansi.supports_frame_selection());
        assert!(ExportFormat::RustArt.supports_frame_selection());
        assert!(!ExportFormat::RustAnimation.supports_frame_selection());
        assert!(!ExportFormat::RustComponent.supports_frame_selection());
    }

    #[test]
    fn frame_selector_is_one_based() {
        let mut project = Project::new("frames", 2, 1);
        project.add_frame();
        select_frame(&mut project, "1").unwrap();
        assert_eq!(project.active_frame, 0);
        select_frame(&mut project, "last").unwrap();
        assert_eq!(project.active_frame, 1);
        assert!(select_frame(&mut project, "0").is_err());
        assert!(select_frame(&mut project, "3").is_err());
    }
}
