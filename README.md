# Ratatelier

**A terminal atelier for glyphs, Ratatui widgets, and motion.**

Terminal art usually starts life in the least artistic place imaginable: a Rust string full of escapes, spacing guesses, and crossed fingers.

Ratatelier gives that work a bench of its own. Sketch the glyphs, arrange the widgets, test the motion, then carry the finished piece back into your application as RON, plain text, or generated Ratatui code.

Think of it as mise en place for a TUI: shape, color, timing, and component states laid out before the real cooking begins.

## What is on the bench

- ASCII and terminal-safe Unicode canvases
- Pencil, eraser, line, rectangle, and fill tools
- Layers, selections, system clipboard support, undo, and redo
- Frame animation with per-frame timing and widget state
- A timeline that follows the active frame during navigation and playback
- Block, paragraph, gauge, list, and sparkline mockups
- Mouse painting, selection dragging, canvas panning, and component resizing
- Command, subcommand, directory, and file completion
- Versioned RON projects with atomic saves
- Live Rust export plus artwork, component, animation, and plain-text exports

## Start working

Clone the repository and install the local package:

```bash
git clone https://github.com/Gardlok/ratatelier.git
cd ratatelier
cargo install --path .
```

Start a new project:

```bash
ratatelier --width 64 --height 20
```

Open an existing project:

```bash
ratatelier artwork.ron
```

Inside the editor, `Tab` moves between the artwork and component benches. Press `?` whenever the key map escapes your memory.

## Clipboard and command line

Selections use the system clipboard without throwing away Ratatelier styling. `Ctrl-a` selects the full canvas, `y` yanks, and `x` cuts the selected rectangle as plain text for other applications while retaining the styled cells internally. Pasting the same content back restores those styles. Text copied elsewhere is painted onto the canvas with the current brush style.

Terminal paste events and `Ctrl-v` accept multiline external text. Unsupported or double-width glyphs are skipped when the canvas mode cannot represent them.

In command mode, `Tab` and `Shift-Tab` cycle matching commands, subcommands, directories, and files. Path completion lists directories first and keeps walking as `/` is added.

## Mouse controls

On the artwork canvas:

- Left click places the cursor and applies the current tool.
- Left drag paints, shapes, or extends a selection.
- Drag an existing selection to move its cells.
- Right drag grabs and pans a canvas larger than the window.
- Right click without dragging picks the glyph and style under the pointer.
- The wheel cycles the glyph palette.

On the component surface, left drag moves a widget and right drag resizes it.

The live Rust pane can be opened or tucked away from the right-side tool rail. Click inside it to focus, drag to select text, and use the wheel or navigation keys to scroll.

## Essential controls

| Key | Action |
| --- | --- |
| `Tab` | Switch Artwork / Components; complete commands in command mode |
| `Shift-Tab` | Previous command completion |
| `h j k l` | Move cursor, widget, or focused export pane |
| `d` / `e` / `i` / `v` | Draw / erase / insert / select mode |
| `Ctrl-a` | Select the entire artwork canvas |
| `y` / `x` / `p` | Yank / cut / paste selected cells through the system clipboard |
| `Ctrl-c` / `Ctrl-v` | Copy selection / paste external clipboard text |
| `1`..`5` | Pencil / eraser / line / rectangle / fill |
| `Space` | Apply tool or set/commit a shape anchor |
| `[` / `]` | Cycle brush glyph or selected widget |
| `n` / `N` / `X` | Add / duplicate / delete frame |
| `,` / `.` | Previous / next frame |
| `p` / `s` | Playback / cycle widget state outside select mode |
| `u` / `Ctrl-r` | Undo / redo |
| `F2` or `Ctrl-e` | Toggle the live Rust pane |
| `Ctrl-s` | Save |
| `Ctrl-x` | Quit, warning about unsaved work |
| `:` | Command mode |
| `?` or `F1` | Help |

When the live export pane is focused, use `j/k`, the arrow keys, the mouse wheel, `PgUp/PgDn`, or `g/G`. Horizontal scrolling uses `h/l` or the left/right arrows. `Esc` returns focus to the editor.

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
:panel                                 toggle the live Rust pane
:export [path]                         export current workspace as Rust
:export art|plain|animation|component [path]
:wq                                    save and quit
:q!                                    discard unsaved changes and quit
```

## Project files

Ratatelier projects are human-readable RON. The format stores the artwork frames, layers, styles, timing, component scene, and widget states together, so a project can move between machines without a separate asset directory.

Unicode mode accepts printable single-cell glyphs. Wide glyph and grapheme-cluster editing remain outside the current format.

## License

MIT
