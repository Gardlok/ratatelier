# Ratatelier

**A terminal atelier for glyphs, Ratatui components, and motion.**

Ratatelier is a Rust + Ratatui workbench for designing terminal artwork and interface components away from the codebase that will eventually consume them. It combines a cell editor, frame animation, component-state prototyping, RON project files, and live Rust code generation in one mouse-aware TUI.

## Current capabilities

### Artwork studio

- Cell-accurate ASCII or terminal-safe Unicode canvases
- Pencil, eraser, line, rectangle, and flood-fill tools
- Foreground/background color and text modifiers
- Mouse painting, shape dragging, glyph picking, and wheel palettes
- Helix-like modal controls for normal, draw, insert, select, and command modes
- Layer creation and deletion
- Internal selection clipboard with copy, cut, and paste
- Bounded undo and redo history

### Animation studio

- Add, duplicate, delete, and navigate frames
- Per-frame duration
- Playback in the editor
- Per-frame widget state: normal, focused, active, or disabled
- Shared frame timeline for artwork and component-state animation

Widget-state animation means that an animation frame can change not only the glyph artwork, but also the state used to render a component mockup. A sequence can therefore demonstrate focus changes, active controls, disabled states, progress transitions, and other interface behavior.

### Component workshop

- Fixed logical design surface that scales into the current terminal
- Block, paragraph, gauge, list, and sparkline widgets
- Keyboard and mouse movement/resizing
- Per-state styling
- Editable title, text, values, type, and geometry
- Live Ratatui `Widget` implementation preview

### Files and export

- Human-readable, versioned RON project files
- Atomic save through a temporary file and rename
- Rust artwork widget export
- Rust component widget export
- Plain-text art export
- Frame-array animation export
- Live generated-code pane

## Installation

```bash
cargo install --path .
```

Run a new project:

```bash
ratatelier --width 64 --height 20
```

Open an existing project:

```bash
ratatelier artwork.ron
```

## Essential controls

| Key | Action |
| --- | --- |
| `Tab` | Switch Artwork / Components |
| `h j k l` | Move cursor or selected component |
| `d` / `e` / `i` / `v` | Draw / erase / insert / select mode |
| `1`..`5` | Pencil / eraser / line / rectangle / fill |
| `Space` | Apply tool or set/commit shape anchor |
| `[` / `]` | Cycle brush glyph or selected widget |
| `n` / `N` / `X` | Add / duplicate / delete frame |
| `,` / `.` | Previous / next frame |
| `p` / `s` | Playback / cycle widget state |
| `u` / `Ctrl-r` | Undo / redo |
| `Ctrl-s` | Save |
| `Ctrl-x` | Quit, prompting when unsaved changes exist |
| `:` | Command mode |
| `?` or `F1` | In-app help |

Mouse support is enabled at startup. In the artwork workspace, left-drag paints or defines shapes, right-click picks a cell, and the wheel cycles glyphs. In the component workspace, left-drag moves and right-drag resizes.

## Commands

```text
:w [path]                              save project
:open path                             open project
:new 48x18                             new project
:resize 80x24                          resize every artwork frame
:scene 100x30                          resize component design surface
:name Project Name                     rename project
:glyph X                               set brush glyph
:frame add|duplicate|delete            edit timeline
:layer add [name]|delete               edit active frame layers
:widget add block|paragraph|gauge|list|sparkline
:title Widget title                    edit selected widget
:text Text with optional \n escapes    edit selected widget content
:value 0..100                          edit selected gauge value
:export [path]                         export current workspace as Rust
:export art|plain|animation|component [path]
:wq                                    save and quit
:q!                                    discard unsaved changes and quit
```

## Project structure

```text
src/
├── app.rs       event loop, modes, commands, history, mouse/keyboard input
├── app/         implementation sections included by app.rs
├── export.rs    Rust, plain text, component, and animation generators
├── geometry.rs  line, rectangle, and flood-fill algorithms
├── model.rs     versioned project, frames, layers, cells, and widgets
├── storage.rs   RON loading and atomic saving
├── ui.rs        Ratatui rendering and coordinate mapping
├── ui/          rendering sections included by ui.rs
├── lib.rs
└── main.rs
```

See [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) for the design boundaries and extension path.

## Unicode scope

Unicode mode accepts printable glyphs that occupy at most one terminal cell according to `unicode-width`. This includes box drawing, blocks, geometric symbols, and a broad range of terminal-safe characters while avoiding ambiguous two-cell overlap. Full grapheme-cluster and wide-glyph editing is a future project-format extension.

## Status

Ratatelier is at `0.1.0`: a functional first implementation intended for local use and iteration. The project format is versioned, but compatibility guarantees begin after the editor model has received real-world use.

## License

MIT
