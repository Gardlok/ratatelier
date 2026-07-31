# Ratatelier Demo Capture

Use a terminal large enough to show the inspector, artwork, color rail, timeline, and footer without wrapping. A window around 140×42 cells works well for the editor. The viewer should also be captured at a smaller size to demonstrate that it remains useful without the editing bench around it.

## Suggested project

Start a fresh 48×18 canvas and create a small three-layer scene:

1. Rename the bottom layer `Background` and add a simple frame or horizon.
2. Add `Subject` and draw the primary glyph artwork.
3. Add `Highlights`, choose a bright color from the rail, and add accents.
4. Duplicate the frame, change its name and duration, then alter the highlight layer for a simple blink or pulse.
5. Add a paragraph or gauge in Components and demonstrate normal and active state styling.
6. Save the project so the same RON file can be shown in editor, viewer, and headless conversion modes.

## Screenshot set

Capture these views for README, crates.io, and GitHub releases:

1. **Artwork bench** — inspector, layer stack, color rail, canvas, and timeline visible.
2. **Component bench** — several widget types and a styled selected widget.
3. **Minimal artwork viewer** — the saved project centred with only the playback footer.
4. **Component viewer** — the same project opened with `--surface components` on a visibly different widget-state frame.
5. **Live export** — F2 pane open beside the corresponding artwork or component.
6. **Headless conversion** — shell output from `--format plain` and a visibly styled `--format ansi` rendering.
7. **Help** — scrollable command map at a useful section.
8. **Safety dialog** — modified project with the Save / Discard / Cancel modal visible.

Avoid screenshots containing personal paths, unrelated terminal tabs, private project names, tokens, or clipboard contents.

## Editor recording sequence

A 20–30 second recording can demonstrate the editing model without narration:

1. Click a layer to select it.
2. Choose a foreground color from the rail and press Space to apply it.
3. Arm the eyedropper and sample an existing canvas color.
4. Click a timeline frame and wheel to the adjacent frame.
5. Enter `:` and use Up/Down to recall a command.
6. Delete a named layer, press `u`, and show the named restore message.
7. Press Ctrl-X to display the unsaved-exit dialog, then cancel with Esc.
8. Toggle the live Rust export pane with F2.

## Viewer and conversion recording sequence

A separate short recording should show that a finished project no longer needs the editor around it:

1. Launch `ratatelier view demo.ron` and allow the animation to begin automatically.
2. Pause and resume with Space.
3. Let the one-pass animation finish, then replay it with `r`.
4. Close with `q`.
5. Launch `ratatelier view demo.ron --surface components` and show the frame-driven widget-state changes.
6. Run `ratatelier export demo.ron --format plain` to standard output.
7. Run `ratatelier export demo.ron --format ansi` and show the terminal styling.
8. Redirect a Rust animation export into a file and display its first few lines:

```bash
ratatelier export demo.ron --format rust-animation > demo-animation.rs
head -n 20 demo-animation.rs
```

For a looping capture, use `ratatelier view demo.ron --loop`; otherwise prefer the default one-pass behavior because it communicates the viewer contract more clearly.

## Terminal recording tools

Any local recorder is acceptable. Keep generated recordings outside the crate package unless intentionally added to repository documentation. Before committing media, check file size, legibility, and whether a static screenshot would communicate the same feature more efficiently.
