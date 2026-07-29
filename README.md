# Ratatelier

**A terminal atelier for glyphs, Ratatui widgets, and motion.**

Terminal art usually starts life in the least artistic place imaginable: a Rust string full of escapes, spacing guesses, and crossed fingers.

Ratatelier gives that work a bench of its own. Sketch the glyphs, arrange the widgets, test the motion, then carry the finished piece back into your application as RON, plain text, or generated Ratatui code.

Think of it as mise en place for a TUI: shape, color, timing, and component states laid out before the real cooking begins.

## What is on the bench

- ASCII and terminal-safe Unicode canvases
- Pencil, eraser, line, rectangle, and fill tools
- A visible layer stack with navigation, visibility, duplication, naming, and ordering
- Frame animation with names, exact timing, widget state, duplication, and ordering
- Selections, system clipboard support, undo, and redo
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

## Layers: transparent sheets, fewer regrets

Every animation frame owns a stack of artwork layers. Think of them as transparent sheets laid over one another: the bottom layer might hold a background, the next a subject, and the top layer highlights or effects. Painting and erasing change only the active layer, while the canvas displays the visible stack composited from top to bottom.

The inspector title and summary always show the active layer index, name, and visibility. When space permits, the inspector also shows the surrounding layer stack:

```text
  [x]  3 Highlights
> [x]  2 Subject
  [x]  1 Background
```

`PgUp` and `PgDn` select adjacent layers; `{` and `}` provide the same controls for keyboard layouts where page keys are less convenient. `V` hides or reveals the active layer without deleting its cells. `D` duplicates it. The command line handles naming, direct selection, and ordering:

```text
:layer rename Character
:layer select 2
:layer hide
:layer move top
```

Layer order is bottom-to-top. Moving a layer **up** places it closer to the viewer; moving it **down** places it closer to the background. The active layer follows the move.

## Frames: content plus time

A frame is more than a canvas snapshot. Each frame carries:

- its own name;
- its own complete layer stack;
- a duration in milliseconds;
- a widget state: `normal`, `focused`, `active`, or `disabled`.

Use `,` and `.` to select frames. `Home` and `End` jump to the first and last frame. `<` and `>` move the active frame earlier or later in the animation while keeping it selected. `+` and `-` adjust the duration in 20 ms steps.

Exact properties are available through commands:

```text
:frame rename Blink hold
:frame duration 750
:frame state active
:frame move first
```

`duration`, `delay`, and `hold` are aliases for the same property: how long that frame remains visible during playback. Supported durations are 20 through 60,000 milliseconds.

## Clipboard and command line

Selections use a long-lived system clipboard provider, so copied text remains available to other applications while Ratatelier is running. `y` yanks and `x` cuts the selected rectangle as plain text while retaining the styled cells internally. Pasting that Ratatelier-owned content back restores its original colors and modifiers.

Text copied from another application is pasted with neutral cell styling. It does not inherit the active brush, because clipboard text and brush paint are separate ingredients. Terminal paste events and `Ctrl-v` accept multiline text; unsupported or double-width glyphs are skipped when the canvas mode cannot represent them.

In command mode, `Tab` and `Shift-Tab` cycle matching commands, composition actions, directories, and files. Path completion lists directories first and keeps walking as `/` is added.

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
| `PgUp` / `PgDn` | Previous / next artwork layer |
| `{` / `}` | Previous / next artwork layer |
| `a` / `A` / `D` / `V` | Add / delete / duplicate / toggle active layer |
| `n` / `N` / `X` | Add / duplicate / delete frame |
| `,` / `.` | Previous / next frame |
| `Home` / `End` | First / last frame |
| `<` / `>` | Move active frame earlier / later |
| `+` / `-` | Increase / decrease frame duration by 20 ms |
| `p` / `s` | Playback / cycle widget state outside select mode |
| `u` / `Ctrl-r` | Undo / redo |
| `F2` or `Ctrl-e` | Toggle the live Rust pane |
| `Ctrl-s` | Save |
| `Ctrl-x` | Quit, warning about unsaved work |
| `:` | Command mode |
| `?` or `F1` | Open help; scroll with arrows, `j/k`, `PgUp/PgDn`, `g/G`, or the wheel |

When help is open, `Home` or `g` jumps to the top, `End` or `G` jumps to the bottom, and `Esc`, `?`, or `F1` closes it.

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

:frame                                 show active frame status
:frame add [name]                      add a blank frame
:frame duplicate [name]                duplicate active frame and properties
:frame delete                          delete active frame
:frame previous|next|first|last        navigate frames
:frame select NUMBER                   select frame by one-based number
:frame rename NAME                     rename active frame
:frame duration MS                     set exact frame duration
:frame state normal|focused|active|disabled
:frame move left|right|first|last      reorder active frame

:layer                                 show active layer status
:layer add [name]                      add a blank top layer
:layer duplicate [name]                duplicate active layer
:layer delete                          delete active layer
:layer previous|next|first|last        navigate layers
:layer select NUMBER                   select layer by one-based number
:layer rename NAME                     rename active layer
:layer show|hide|toggle                control active-layer visibility
:layer move up|down|top|bottom         reorder active layer

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

Ratatelier projects are human-readable RON. The format stores the artwork frames, layer stack, visibility, styles, timing, component scene, and widget states together, so a project can move between machines without a separate asset directory.

The `0.2.0` application milestone does not change the project-file schema; existing version-1 RON projects remain the intended format. Unicode mode accepts printable single-cell glyphs. Wide glyph and grapheme-cluster editing remain outside the current format.

## License

MIT
