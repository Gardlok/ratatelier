# Ratatelier architecture

Ratatelier separates the editable project model from terminal rendering and from generated output. The RON file remains the canonical editable artifact; generated Rust is disposable output.

## Boundaries

### `model`

The model owns all serializable state:

- project version and name;
- animation frames and durations;
- frame-specific widget state;
- canvases, layers, cells, colors, and modifiers;
- the component design surface and widget specifications.

The model does not know about terminal events or application history. This keeps RON compatibility and model tests independent from the interactive front end.

### `app`

The application layer owns ephemeral editor state:

- active workspace and mode;
- cursor, selection, brush, tool, and clipboard;
- undo/redo snapshots;
- mouse drags;
- playback timing;
- command parsing;
- current file path and dirty status.

Actions mutate the model through this layer. Undo and redo currently use bounded whole-project snapshots. The model is intentionally small enough for this to be predictable; a command-delta history can replace it later without changing the file format.

### `ui`

The UI computes deterministic regions and renders the project through Ratatui. Screen-to-model coordinate mapping lives next to layout computation, keeping mouse hit testing consistent with what is visible.

The artwork renderer composites visible layers into Ratatui buffer cells. The component renderer maps a fixed logical scene into the available preview area, which allows the same project to be tested at different terminal sizes.

### `export`

Exporters read the model and produce standalone text:

- direct-buffer Ratatui artwork widget;
- Ratatui component widget;
- plain terminal art;
- animation frame arrays.

Exporters never modify the project and are covered by focused tests.

### `storage`

Storage uses `serde` and RON. Save writes a sibling temporary file before renaming it over the destination so an interrupted serialization does not truncate the existing project.

## Widget-state animation

Every animation frame carries a `WidgetState` value. Artwork playback changes the canvas frame as usual; component playback applies the same frame's state to every component's matching style slot. This lets one timeline demonstrate both spatial animation and UI behavior.

A later extension can add per-widget state overrides, transitions, and model values without replacing the current frame concept.

## Extension path

The intended next architectural additions are:

1. grapheme clusters and explicit continuation cells for wide Unicode;
2. layer visibility/reordering controls and shared layers;
3. onion-skin rendering;
4. component z-order controls and alignment guides;
5. richer widget schemas and stateful widget data;
6. configurable keymaps stored separately from projects;
7. exporter templates and generated module tests;
8. autosave and crash-recovery journals.
