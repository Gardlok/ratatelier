use std::{
    io::{self, stdout},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
        KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
    },
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use ratatui::{backend::CrosstermBackend, layout::Rect, Terminal};

use crate::{
    export,
    geometry::{flood_region, line_points, rectangle_points},
    model::{
        CanvasMode, Cell, CellStyle, ColorSpec, Point, Project, RectSpec, WidgetKind,
    },
    storage,
    ui::{self, UiRegions},
};

const HISTORY_LIMIT: usize = 128;
const TICK_RATE: Duration = Duration::from_millis(16);
const ASCII_GLYPHS: &[char] = &['#', '@', '*', '+', '-', '|', '=', '.', ':', '/', '\\'];
const UNICODE_GLYPHS: &[char] = &[
    '█', '▓', '▒', '░', '─', '│', '┌', '┐', '└', '┘', '◆', '●', '▲', '▼', '◀', '▶',
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Workspace {
    #[default]
    Artwork,
    Components,
}

impl Workspace {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Artwork => "Artwork",
            Self::Components => "Components",
        }
    }

    #[must_use]
    pub const fn toggled(self) -> Self {
        match self {
            Self::Artwork => Self::Components,
            Self::Components => Self::Artwork,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    #[default]
    Normal,
    Draw,
    Insert,
    Select,
    Command,
}

impl Mode {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Normal => "NORMAL",
            Self::Draw => "DRAW",
            Self::Insert => "INSERT",
            Self::Select => "SELECT",
            Self::Command => "COMMAND",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tool {
    #[default]
    Pencil,
    Eraser,
    Line,
    Rectangle,
    Fill,
}

impl Tool {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Pencil => "Pencil",
            Self::Eraser => "Eraser",
            Self::Line => "Line",
            Self::Rectangle => "Rectangle",
            Self::Fill => "Fill",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    pub anchor: Point,
    pub head: Point,
}

impl Selection {
    #[must_use]
    pub const fn bounds(self) -> RectSpec {
        let left = if self.anchor.x < self.head.x {
            self.anchor.x
        } else {
            self.head.x
        };
        let top = if self.anchor.y < self.head.y {
            self.anchor.y
        } else {
            self.head.y
        };
        let right = if self.anchor.x > self.head.x {
            self.anchor.x
        } else {
            self.head.x
        };
        let bottom = if self.anchor.y > self.head.y {
            self.anchor.y
        } else {
            self.head.y
        };
        RectSpec::new(left, top, right - left + 1, bottom - top + 1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct TextPoint {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextSelection {
    pub anchor: TextPoint,
    pub head: TextPoint,
}

impl TextSelection {
    #[must_use]
    pub fn ordered(self) -> (TextPoint, TextPoint) {
        if self.anchor <= self.head {
            (self.anchor, self.head)
        } else {
            (self.head, self.anchor)
        }
    }
}

#[derive(Debug, Clone)]
struct Clipboard {
    width: u16,
    height: u16,
    cells: Vec<Cell>,
}

#[derive(Debug, Clone, Copy)]
struct ShapeDrag {
    start: Point,
    current: Point,
}

#[derive(Debug, Clone, Copy)]
struct ComponentDrag {
    widget: usize,
    start: Point,
    original: RectSpec,
    resizing: bool,
}

#[derive(Debug, Clone, Copy)]
struct PanDrag {
    start_column: u16,
    start_row: u16,
    origin: Point,
    moved: bool,
}

#[derive(Debug, Clone)]
enum SelectionDragKind {
    Create,
    Move {
        original: Selection,
        cells: Vec<Cell>,
    },
}

#[derive(Debug, Clone)]
struct SelectionDrag {
    start: Point,
    current: Point,
    kind: SelectionDragKind,
}

#[derive(Debug)]
pub struct App {
    pub project: Project,
    pub current_path: Option<PathBuf>,
    pub workspace: Workspace,
    pub mode: Mode,
    pub tool: Tool,
    pub cursor: Point,
    pub viewport_origin: Point,
    pub brush: Cell,
    pub selection: Option<Selection>,
    pub command: String,
    pub status: String,
    pub dirty: bool,
    pub running: bool,
    pub show_help: bool,
    pub show_export: bool,
    pub export_focused: bool,
    pub export_scroll: Point,
    pub export_selection: Option<TextSelection>,
    pub component_selected: usize,
    pub regions: UiRegions,
    history: Vec<Project>,
    future: Vec<Project>,
    clipboard: Option<Clipboard>,
    shape_drag: Option<ShapeDrag>,
    component_drag: Option<ComponentDrag>,
    pan_drag: Option<PanDrag>,
    selection_drag: Option<SelectionDrag>,
    export_drag_anchor: Option<TextPoint>,
    playing: bool,
    last_tick: Instant,
    elapsed_in_frame: Duration,
}