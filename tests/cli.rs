use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use ratatelier::{
    Project,
    model::{Cell, CellStyle, ColorSpec, Point},
    storage,
};

fn project_path(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after the Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "ratatelier-cli-{label}-{}-{nonce}.ron",
        std::process::id()
    ))
}

fn save_fixture(label: &str) -> PathBuf {
    let path = project_path(label);
    let mut project = Project::new(label, 3, 1);
    project.canvas_mut().set_cell(
        Point::new(1, 0),
        Cell::painted(
            'X',
            CellStyle {
                fg: ColorSpec::LightGreen,
                bold: true,
                ..CellStyle::default()
            },
        ),
    );
    storage::save_project(&path, &project).expect("fixture should save");
    path
}

#[test]
fn plain_export_writes_project_to_stdout() {
    let path = save_fixture("plain");
    let output = Command::new(env!("CARGO_BIN_EXE_ratatelier"))
        .arg("export")
        .arg(&path)
        .args(["--format", "plain"])
        .output()
        .expect("ratatelier export should run");
    let _ = fs::remove_file(path);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), " X \n");
}

#[test]
fn ansi_export_preserves_terminal_style() {
    let path = save_fixture("ansi");
    let output = Command::new(env!("CARGO_BIN_EXE_ratatelier"))
        .arg("export")
        .arg(&path)
        .args(["--format", "ansi"])
        .output()
        .expect("ratatelier export should run");
    let _ = fs::remove_file(path);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\x1b[0;92;49;1mX\x1b[0m"));
}

#[test]
fn animation_export_rejects_single_frame_selection() {
    let path = save_fixture("animation");
    let output = Command::new(env!("CARGO_BIN_EXE_ratatelier"))
        .arg("export")
        .arg(&path)
        .args(["--format", "rust-animation", "--frame", "1"])
        .output()
        .expect("ratatelier export should run");
    let _ = fs::remove_file(path);

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot be combined"));
}

#[test]
fn viewer_and_export_subcommand_help_are_available() {
    for command in ["view", "export"] {
        let output = Command::new(env!("CARGO_BIN_EXE_ratatelier"))
            .args([command, "--help"])
            .output()
            .expect("subcommand help should run");
        assert!(
            output.status.success(),
            "{command} help failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
