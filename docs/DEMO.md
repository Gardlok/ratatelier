# Ratatelier Demo Capture

Use a terminal large enough to show the inspector, artwork, color rail, timeline, and footer without wrapping. A window around 140×42 cells works well.

## Suggested project

Start a fresh 48×18 canvas and create a small three-layer scene:

1. Rename the bottom layer `Background` and add a simple frame or horizon.
2. Add `Subject` and draw the primary glyph artwork.
3. Add `Highlights`, choose a bright color from the rail, and add accents.
4. Duplicate the frame, change its name and duration, then alter the highlight layer for a simple blink or pulse.
5. Add a paragraph or gauge in Components and demonstrate normal and active state styling.

## Screenshot set

Capture these views for README, crates.io, and GitHub releases:

1. **Artwork bench** — inspector, layer stack, color rail, canvas, and timeline visible.
2. **Component bench** — several widget types and a styled selected widget.
3. **Live export** — F2 pane open beside the corresponding artwork or component.
4. **Help** — scrollable command map at a useful section.
5. **Safety dialog** — modified project with the Save / Discard / Cancel modal visible.

Avoid screenshots containing personal paths, unrelated terminal tabs, private project names, tokens, or clipboard contents.

## Short recording sequence

A 20–30 second recording can demonstrate the interaction model without narration:

1. Click a layer to select it.
2. Choose a foreground color from the rail and press Space to apply it.
3. Arm the eyedropper and sample an existing canvas color.
4. Click a timeline frame and wheel to the adjacent frame.
5. Enter `:` and use Up/Down to recall a command.
6. Delete a named layer, press `u`, and show the named restore message.
7. Press Ctrl-X to display the unsaved-exit dialog, then cancel with Esc.
8. Toggle the live Rust export pane with F2.

## Terminal recording tools

Any local recorder is acceptable. Keep generated recordings outside the crate package unless intentionally added to repository documentation. Before committing media, check file size, legibility, and whether a static screenshot would communicate the same feature more efficiently.
