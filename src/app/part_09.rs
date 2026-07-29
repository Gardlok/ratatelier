const COMMAND_NAMES: &[&str] = &[
    "e", "export", "frame", "glyph", "help", "layer", "name", "new", "o", "open",
    "panel", "q", "q!", "quit", "quit!", "resize", "scene", "state", "text", "title",
    "tools", "value", "w", "widget", "wq", "write",
];
const FRAME_ACTIONS: &[&str] = &[
    "add", "duplicate", "delete", "previous", "next", "first", "last", "select",
    "rename", "duration", "hold", "state", "move", "status",
];
const LAYER_ACTIONS: &[&str] = &[
    "add", "duplicate", "delete", "previous", "next", "first", "last", "select",
    "rename", "show", "hide", "toggle", "move", "status",
];
const FRAME_MOVE_TARGETS: &[&str] = &["left", "right", "first", "last"];
const LAYER_MOVE_TARGETS: &[&str] = &["up", "down", "top", "bottom"];
const FRAME_STATES: &[&str] = &["normal", "focused", "active", "disabled"];
const WIDGET_ACTIONS: &[&str] = &["add", "delete"];
const WIDGET_KINDS: &[&str] = &["block", "paragraph", "gauge", "list", "sparkline"];
const EXPORT_KINDS: &[&str] = &["art", "plain", "animation", "component"];
const PASTE_TAB_WIDTH: usize = 4;

impl App {
    fn system_clipboard(&mut self) -> Result<&mut SystemClipboard, String> {
        if self.system_clipboard.is_none() {
            self.system_clipboard = Some(SystemClipboard::connect()?);
        }
        self.system_clipboard
            .as_mut()
            .ok_or_else(|| "System clipboard initialization failed".to_owned())
    }

    fn write_system_clipboard(&mut self, text: String) -> Result<(), String> {
        self.system_clipboard()?.write(text)
    }

    fn read_system_clipboard(&mut self) -> Result<String, String> {
        self.system_clipboard()?.read()
    }

    fn reset_command_completion(&mut self) {
        self.command_completion = None;
        self.command_hint.clear();
    }

    fn complete_command(&mut self, reverse: bool) {
        let can_cycle = self.command_completion.as_ref().is_some_and(|completion| {
            completion
                .candidates
                .get(completion.index)
                .is_some_and(|candidate| candidate == &self.command)
        });

        if !can_cycle {
            let candidates = command_completions(&self.command);
            if candidates.is_empty() {
                self.command_completion = None;
                self.command_hint = "No completion found".to_owned();
                return;
            }
            let index = if reverse { candidates.len() - 1 } else { 0 };
            self.command_completion = Some(CommandCompletion { candidates, index });
        } else if let Some(completion) = &mut self.command_completion {
            completion.index = if reverse {
                (completion.index + completion.candidates.len() - 1) % completion.candidates.len()
            } else {
                (completion.index + 1) % completion.candidates.len()
            };
        }

        if let Some(completion) = &self.command_completion {
            self.command = completion.candidates[completion.index].clone();
            self.command_hint = format!(
                "Completion {}/{} · Tab next · Shift-Tab previous",
                completion.index + 1,
                completion.candidates.len()
            );
        }
    }

    fn append_command_text(&mut self, text: &str) {
        let sanitized: String = text
            .chars()
            .map(|character| match character {
                '\r' | '\n' | '\t' => ' ',
                other if other.is_control() => ' ',
                other => other,
            })
            .collect();
        self.command.push_str(&sanitized);
        self.reset_command_completion();
    }

    fn handle_paste_event(&mut self, text: &str) {
        if self.mode == Mode::Command {
            self.append_command_text(text);
            return;
        }
        if self.workspace != Workspace::Artwork {
            self.status = "Text paste is available on the artwork canvas".to_owned();
            return;
        }
        self.paste_external_text(text);
        if self.mode == Mode::Select {
            self.mode = Mode::Normal;
        }
    }

    fn paste_from_system_clipboard(&mut self) {
        match self.read_system_clipboard() {
            Ok(text) if !text.is_empty() => {
                if self.mode == Mode::Command {
                    self.append_command_text(&text);
                    return;
                }
                if self.workspace != Workspace::Artwork {
                    self.status = "System paste is available on the artwork canvas".to_owned();
                    return;
                }
                let styled_match = self
                    .clipboard
                    .as_ref()
                    .is_some_and(|clipboard| clipboard.text == text);
                if styled_match {
                    self.paste_clipboard();
                } else {
                    self.paste_external_text(&text);
                }
            }
            Ok(_) => {
                if !self.paste_clipboard() {
                    self.status = "System clipboard is empty".to_owned();
                }
            }
            Err(error) => {
                if self.paste_clipboard() {
                    self.status = format!(
                        "Styled clipboard pasted; system clipboard unavailable: {error}"
                    );
                } else {
                    self.status = format!("System clipboard unavailable: {error}");
                }
            }
        }
    }

    fn paste_external_text(&mut self, text: &str) {
        let (clipboard, rejected) =
            clipboard_from_text(text, &CellStyle::default(), self.project.canvas().mode);
        let Some(clipboard) = clipboard else {
            self.status = "Clipboard text contains no pasteable cells".to_owned();
            return;
        };
        let width = clipboard.width;
        let height = clipboard.height;
        self.clipboard = Some(clipboard);
        self.paste_clipboard();
        self.status = if rejected == 0 {
            format!("Pasted {width}×{height} neutral text block from system clipboard")
        } else {
            format!(
                "Pasted {width}×{height} neutral text block; skipped {rejected} unsupported glyphs"
            )
        };
    }
}

fn cells_to_text(width: u16, height: u16, cells: &[Cell]) -> String {
    let mut text = String::new();
    for y in 0..height {
        if y > 0 {
            text.push('\n');
        }
        for x in 0..width {
            let index = usize::from(y) * usize::from(width) + usize::from(x);
            let Some(cell) = cells.get(index) else {
                text.push(' ');
                continue;
            };
            if cell.transparent {
                text.push(' ');
            } else {
                text.push_str(&cell.glyph);
            }
        }
    }
    text
}

fn clipboard_from_text(
    text: &str,
    style: &CellStyle,
    mode: CanvasMode,
) -> (Option<Clipboard>, usize) {
    let mut normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    if normalized.ends_with('\n') {
        normalized.pop();
    }
    if normalized.is_empty() {
        return (None, 0);
    }

    let rows: Vec<Vec<char>> = normalized.lines().map(expand_paste_tabs).collect();
    let width = rows.iter().map(Vec::len).max().unwrap_or(0);
    if width == 0 || rows.is_empty() {
        return (None, 0);
    }
    let width_u16 = u16::try_from(width).unwrap_or(u16::MAX);
    let height_u16 = u16::try_from(rows.len()).unwrap_or(u16::MAX);
    let width = usize::from(width_u16);
    let height = usize::from(height_u16);
    let mut cells = vec![Cell::default(); width.saturating_mul(height)];
    let mut rejected = 0;

    for (y, row) in rows.iter().take(height).enumerate() {
        for (x, character) in row.iter().take(width).copied().enumerate() {
            if !mode.accepts(character) {
                rejected += 1;
                continue;
            }
            let mut cell = Cell::painted(character, style.clone());
            cell.transparent = false;
            cells[y * width + x] = cell;
        }
    }

    let text = cells_to_text(width_u16, height_u16, &cells);
    (
        Some(Clipboard {
            width: width_u16,
            height: height_u16,
            cells,
            text,
        }),
        rejected,
    )
}

fn expand_paste_tabs(line: &str) -> Vec<char> {
    let mut expanded = Vec::new();
    for character in line.chars() {
        if character == '\t' {
            let spaces = PASTE_TAB_WIDTH - expanded.len() % PASTE_TAB_WIDTH;
            expanded.extend(std::iter::repeat_n(' ', spaces));
        } else {
            expanded.push(character);
        }
    }
    expanded
}

fn command_completions(input: &str) -> Vec<String> {
    let token_start = if input.chars().last().is_some_and(char::is_whitespace) {
        input.len()
    } else {
        input
            .char_indices()
            .rev()
            .find(|(_, character)| character.is_whitespace())
            .map_or(0, |(index, character)| index + character.len_utf8())
    };
    let head = &input[..token_start];
    let prefix = &input[token_start..];
    let preceding: Vec<&str> = head.split_whitespace().collect();
    let position = preceding.len();

    let replacements = if position == 0 {
        complete_values(prefix, COMMAND_NAMES, true)
    } else {
        argument_completions(&preceding, position, prefix)
    };

    let mut completions: Vec<String> = replacements
        .into_iter()
        .map(|replacement| format!("{head}{replacement}"))
        .collect();
    completions.sort();
    completions.dedup();
    completions
}

fn argument_completions(preceding: &[&str], position: usize, prefix: &str) -> Vec<String> {
    let command = preceding[0];
    match (command, position) {
        ("frame", 1) => complete_values(prefix, FRAME_ACTIONS, false),
        ("frame", 2) if preceding.get(1) == Some(&"move") => {
            complete_values(prefix, FRAME_MOVE_TARGETS, false)
        }
        ("frame", 2) if preceding.get(1) == Some(&"state") => {
            complete_values(prefix, FRAME_STATES, false)
        }
        ("layer", 1) => complete_values(prefix, LAYER_ACTIONS, false),
        ("layer", 2) if preceding.get(1) == Some(&"move") => {
            complete_values(prefix, LAYER_MOVE_TARGETS, false)
        }
        ("widget", 1) => complete_values(prefix, WIDGET_ACTIONS, true),
        ("widget", 2) if preceding.get(1) == Some(&"add") => {
            complete_values(prefix, WIDGET_KINDS, false)
        }
        ("export" | "e", 1) => {
            let mut matches = complete_values(prefix, EXPORT_KINDS, true);
            matches.extend(path_completions(prefix));
            matches
        }
        ("export" | "e", 2)
            if preceding
                .get(1)
                .is_some_and(|kind| EXPORT_KINDS.contains(kind)) =>
        {
            path_completions(prefix)
        }
        ("open" | "o" | "w" | "write" | "wq", 1) => path_completions(prefix),
        _ if looks_like_path(prefix) => path_completions(prefix),
        _ => Vec::new(),
    }
}

fn complete_values(prefix: &str, values: &[&str], trailing_space: bool) -> Vec<String> {
    values
        .iter()
        .filter(|value| value.starts_with(prefix))
        .map(|value| {
            if trailing_space {
                format!("{value} ")
            } else {
                (*value).to_owned()
            }
        })
        .collect()
}

fn looks_like_path(prefix: &str) -> bool {
    prefix.starts_with('.')
        || prefix.starts_with('~')
        || prefix.starts_with('/')
        || prefix.contains('/')
        || prefix.contains('\\')
}

fn path_completions(prefix: &str) -> Vec<String> {
    let (display_directory, name_prefix) = split_path_prefix(prefix);
    let directory = expand_completion_directory(display_directory);
    let Ok(entries) = std::fs::read_dir(&directory) else {
        return Vec::new();
    };
    let separator = if display_directory.contains('\\') && !display_directory.contains('/') {
        '\\'
    } else {
        '/'
    };
    let mut matches = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with(name_prefix) {
            continue;
        }
        let is_directory = entry.file_type().is_ok_and(|kind| kind.is_dir());
        let mut candidate = format!("{display_directory}{name}");
        if is_directory {
            candidate.push(separator);
        }
        matches.push((!is_directory, candidate.to_ascii_lowercase(), candidate));
    }
    matches.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
    matches
        .into_iter()
        .map(|(_, _, candidate)| candidate)
        .collect()
}

fn split_path_prefix(prefix: &str) -> (&str, &str) {
    prefix
        .char_indices()
        .rev()
        .find(|(_, character)| *character == '/' || *character == '\\')
        .map_or(("", prefix), |(index, character)| {
            let split = index + character.len_utf8();
            (&prefix[..split], &prefix[split..])
        })
}

fn expand_completion_directory(directory: &str) -> PathBuf {
    if directory.is_empty() {
        return PathBuf::from(".");
    }
    if directory == "~" || directory.starts_with("~/") || directory.starts_with("~\\") {
        if let Some(home) = std::env::var_os("HOME") {
            let suffix = directory
                .trim_start_matches('~')
                .trim_start_matches(|character| character == '/' || character == '\\');
            return PathBuf::from(home).join(suffix);
        }
    }
    PathBuf::from(directory)
}

#[cfg(test)]
mod clipboard_completion_tests {
    use super::*;

    #[test]
    fn text_clipboard_expands_tabs_and_preserves_rows() {
        let (clipboard, rejected) = clipboard_from_text(
            "ab\tc\nde",
            &CellStyle::default(),
            CanvasMode::Ascii,
        );
        let clipboard = clipboard.unwrap();
        assert_eq!(rejected, 0);
        assert_eq!(clipboard.width, 5);
        assert_eq!(clipboard.height, 2);
        assert_eq!(clipboard.text, "ab  c\nde   ");
    }

    #[test]
    fn external_text_pastes_with_neutral_style() {
        let mut app = App::new(Project::new("test", 4, 4), None);
        app.brush.style.fg = ColorSpec::LightRed;
        app.brush.style.bold = true;
        app.paste_external_text("x");
        let cell = app.project.canvas().active_cell(Point::default()).unwrap();
        assert_eq!(cell.style, CellStyle::default());
    }

    #[test]
    fn internal_clipboard_paste_preserves_cell_style() {
        let mut app = App::new(Project::new("test", 4, 4), None);
        let style = CellStyle {
            fg: ColorSpec::LightMagenta,
            bold: true,
            ..CellStyle::default()
        };
        app.clipboard = Some(Clipboard {
            width: 1,
            height: 1,
            cells: vec![Cell::painted('x', style.clone())],
            text: "x".to_owned(),
        });
        app.cursor = Point::new(2, 1);
        assert!(app.paste_clipboard());
        let cell = app
            .project
            .canvas()
            .active_cell(Point::new(2, 1))
            .unwrap();
        assert_eq!(cell.style, style);
    }

    #[test]
    fn command_completion_matches_composition_actions() {
        assert!(command_completions("op").contains(&"open ".to_owned()));
        assert!(command_completions("frame d").contains(&"frame delete".to_owned()));
        assert!(command_completions("frame move r").contains(&"frame move right".to_owned()));
        assert!(command_completions("layer move t").contains(&"layer move top".to_owned()));
        assert!(command_completions("widget add sp")
            .contains(&"widget add sparkline".to_owned()));
    }

    #[test]
    fn filesystem_completion_lists_matching_files_and_directories() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("ratatelier-completion-{unique}"));
        std::fs::create_dir_all(root.join("frames")).unwrap();
        std::fs::write(root.join("frame.ron"), "test").unwrap();
        let prefix = format!("{}/fra", root.display());
        let matches = path_completions(&prefix);
        assert!(matches.iter().any(|candidate| candidate.ends_with("frames/")));
        assert!(matches.iter().any(|candidate| candidate.ends_with("frame.ron")));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn active_completion_cycles_forward_and_backward() {
        let mut app = App::new(Project::new("test", 4, 4), None);
        app.command = "q".to_owned();
        app.complete_command(false);
        let first = app.command.clone();
        app.complete_command(false);
        assert_ne!(app.command, first);
        app.complete_command(true);
        assert_eq!(app.command, first);
    }
}
