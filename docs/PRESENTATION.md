# Viewing and Converting Ratatelier Projects

Ratatelier `0.3.0` uses the same version-1 RON project file for editing, presentation, and conversion. No secondary asset format or export cache is required.

## Minimal viewer

Open the artwork surface:

```bash
ratatelier view project.ron
```

Open the component surface:

```bash
ratatelier view project.ron --surface components
```

Projects with more than one frame start playing immediately from frame one. Playback stops on the final frame by default.

Viewer controls are intentionally small:

- `Space` pauses or resumes playback;
- after playback completes, `Space` starts again from frame one;
- `r` restarts immediately from frame one;
- `q` or `Esc` closes the viewer.

Continuous playback is opt-in:

```bash
ratatelier view project.ron --loop
```

The viewer does not expose editing, layers, palettes, inspectors, or commands. It renders artwork with its composited colors and modifiers. The component surface uses each animation frame's widget state, allowing normal, focused, active, and disabled presentations to animate with the same frame timing.

## Headless conversion

The export subcommand never enters raw terminal mode or opens the editor:

```bash
ratatelier export project.ron --format plain
```

Output goes to standard output by default, so ordinary shell composition works:

```bash
ratatelier export logo.ron --format plain > logo.txt
ratatelier export logo.ron --format rust-art | rustfmt > logo.rs
ratatelier export animation.ron --format rust-animation > animation.rs
```

Use `--output` when Ratatelier should write the file directly:

```bash
ratatelier export dashboard.ron \
  --format rust-component \
  --output dashboard.rs
```

## Formats

### `plain`

Glyph-only ASCII or Unicode text for the selected artwork frame. Colors and text modifiers are intentionally omitted.

```bash
ratatelier export project.ron --format plain
```

### `ansi`

Terminal-ready text containing ANSI SGR sequences for foreground, background, bold, dim, italic, underline, reverse, and crossed-out styling.

```bash
ratatelier export project.ron --format ansi
```

Redirecting ANSI output to a file preserves the escape sequences. Use `plain` when the consumer should receive portable glyph text instead.

### `rust-art`

A generated Ratatui `Widget` for the selected artwork frame. Styled spaces and background-only cells are retained.

```bash
ratatelier export project.ron --format rust-art --output artwork.rs
```

### `rust-animation`

A generated Ratatui animation widget containing every artwork frame, exact frame durations, colors, backgrounds, and text modifiers.

```bash
ratatelier export project.ron --format rust-animation --output animation.rs
```

Because this format always contains the complete animation, it rejects `--frame` rather than pretending to select only one frame.

### `rust-component`

A generated Ratatui component widget with normal, focused, active, and disabled style branches.

```bash
ratatelier export project.ron --format rust-component --output component.rs
```

The selected project frame determines the state displayed by the viewer. The generated Rust component exposes its own `ComponentState` value so the embedding application can select a state directly.

## Frame selection

Single-frame and component exports accept a one-based frame number, `first`, or `last`:

```bash
ratatelier export animation.ron --format plain --frame 3
ratatelier export animation.ron --format ansi --frame first
ratatelier export component.ron --format rust-component --frame last
```

Without `--frame`, conversion uses the active frame stored in the project.

## Editor compatibility

Existing launch forms remain editor mode:

```bash
ratatelier
ratatelier project.ron
ratatelier --width 64 --height 20
```

The new subcommands are explicit:

```text
ratatelier [PROJECT.ron]                 edit
ratatelier view PROJECT.ron              present
ratatelier export PROJECT.ron --format … convert
```
