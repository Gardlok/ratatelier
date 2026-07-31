# Changelog

All notable changes to Ratatelier are documented here.

## [Unreleased]

## [0.3.0] - 2026-07-31

### Added

- Minimal `ratatelier view PROJECT.ron` presentation mode.
- Automatic one-pass playback for multi-frame projects.
- Viewer pause, resume, replay, close, optional looping, and artwork/component surface selection.
- Headless `ratatelier export PROJECT.ron --format FORMAT` conversion.
- Standard-output conversion with optional direct file output.
- One-based, first, and last frame selection for single-frame and component exports.
- ANSI terminal export preserving colors and text modifiers.
- Black-box CLI tests for conversion output and installed subcommand surfaces.
- A viewing and conversion guide.

### Changed

- Generated Rust animation output is now a styled Ratatui widget instead of glyph-only string constants.
- Generated animation output preserves exact frame durations, foregrounds, backgrounds, and modifiers.
- Generated artwork retains styled spaces and background-only cells.
- Generated component widgets honor normal, focused, active, and disabled styles.
- Installed-package smoke tests now cover `view --help` and `export --help`.

### Compatibility

- Existing `ratatelier [PROJECT.ron]` usage remains editor mode.
- The RON project schema remains unchanged.
- Rust 1.88 or newer remains required.

## [0.2.2] - 2026-07-30

### Added

- Unsaved-exit dialog with explicit Save, Discard, and Cancel actions.
- Session-local command history through Up and Down in command mode.
- Draft restoration after browsing command history.
- Named delete and undo feedback for layers, frames, and component widgets.
- Documentation, package-construction, and isolated installation checks in CI.
- An exact Rust 1.88 MSRV check and test job.
- A repeatable publishing checklist and demo-capture guide.

### Changed

- `Ctrl-X` and `:q` now open the unsaved-exit dialog instead of relying on a footer warning.
- Delete notifications identify the removed object and advertise `u` for immediate recovery.
- Undoing or redoing a deletion identifies the restored or removed object.
- Package metadata now advertises the project homepage and docs.rs documentation URL.
- The declared minimum supported Rust version is now 1.88, matching Ratatui 0.30.2 and the dependency graph actually verified by CI.

### Compatibility

- The RON project schema remains unchanged.
- Command history and dialog state are editor-session state only.
- Rust 1.88 or newer is required to build Ratatelier 0.2.2.

## [0.2.1] - 2026-07-30

### Added

- Compact scrollable color rail with foreground/background targeting.
- Reset and one-shot canvas color-sampling entries.
- Context-sensitive footer guidance.
- Clickable and wheel-selectable timeline frames.
- Styled brush and component previews.

### Changed

- The live Rust export pane starts hidden.
- Inspector panels show state instead of permanent key maps.

## [0.2.0] - 2026-07-30

### Added

- Full artwork layer management.
- Full animation frame management with timing and widget state.
- Scrollable inspector with clickable layer selection.
- Scrollable in-app help.

[Unreleased]: https://github.com/Gardlok/ratatelier/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/Gardlok/ratatelier/compare/v0.2.2...v0.3.0
[0.2.2]: https://github.com/Gardlok/ratatelier/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/Gardlok/ratatelier/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/Gardlok/ratatelier/releases/tag/v0.2.0
