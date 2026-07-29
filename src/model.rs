use serde::{Deserialize, Serialize};
use unicode_width::UnicodeWidthChar;

pub const PROJECT_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

impl Point {
    #[must_use]
    pub const fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CanvasMode {
    #[default]
    Ascii,
    Unicode,
}

impl CanvasMode {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Ascii => "ASCII",
            Self::Unicode => "Unicode",
        }
    }

    #[must_use]
    pub fn accepts(self, glyph: char) -> bool {
        match self {
            Self::Ascii => glyph.is_ascii() && !glyph.is_ascii_control(),
            Self::Unicode => !glyph.is_control() && glyph.width().is_some_and(|width| width <= 1),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ColorSpec {
    #[default]
    Reset,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Gray,
    DarkGray,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    White,
}

impl ColorSpec {
    pub const PALETTE: [Self; 17] = [
        Self::Reset,
        Self::White,
        Self::Gray,
        Self::DarkGray,
        Self::Black,
        Self::Red,
        Self::LightRed,
        Self::Yellow,
        Self::LightYellow,
        Self::Green,
        Self::LightGreen,
        Self::Cyan,
        Self::LightCyan,
        Self::Blue,
        Self::LightBlue,
        Self::Magenta,
        Self::LightMagenta,
    ];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Reset => "reset",
            Self::Black => "black",
            Self::Red => "red",
            Self::Green => "green",
            Self::Yellow => "yellow",
            Self::Blue => "blue",
            Self::Magenta => "magenta",
            Self::Cyan => "cyan",
            Self::Gray => "gray",
            Self::DarkGray => "dark gray",
            Self::LightRed => "light red",
            Self::LightGreen => "light green",
            Self::LightYellow => "light yellow",
            Self::LightBlue => "light blue",
            Self::LightMagenta => "light magenta",
            Self::LightCyan => "light cyan",
            Self::White => "white",
        }
    }

    #[must_use]
    pub fn next(self) -> Self {
        let index = Self::PALETTE.iter().position(|color| *color == self).unwrap_or(0);
        Self::PALETTE[(index + 1) % Self::PALETTE.len()]
    }

    #[must_use]
    pub fn previous(self) -> Self {
        let index = Self::PALETTE.iter().position(|color| *color == self).unwrap_or(0);
        Self::PALETTE[(index + Self::PALETTE.len() - 1) % Self::PALETTE.len()]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CellStyle {
    pub fg: ColorSpec,
    pub bg: ColorSpec,
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underlined: bool,
    pub reversed: bool,
    pub crossed_out: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cell {
    pub glyph: String,
    pub style: CellStyle,
    pub transparent: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            glyph: " ".to_owned(),
            style: CellStyle::default(),
            transparent: true,
        }
    }
}

impl Cell {
    #[must_use]
    pub fn painted(glyph: char, style: CellStyle) -> Self {
        Self {
            glyph: glyph.to_string(),
            style,
            transparent: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layer {
    pub name: String,
    pub visible: bool,
    pub cells: Vec<Cell>,
}

impl Layer {
    #[must_use]
    pub fn new(name: impl Into<String>, width: u16, height: u16) -> Self {
        Self {
            name: name.into(),
            visible: true,
            cells: vec![Cell::default(); usize::from(width) * usize::from(height)],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Canvas {
    pub width: u16,
    pub height: u16,
    pub mode: CanvasMode,
    pub active_layer: usize,
    pub layers: Vec<Layer>,
}

impl Canvas {
    #[must_use]
    pub fn new(width: u16, height: u16) -> Self {
        let width = width.max(1);
        let height = height.max(1);
        Self {
            width,
            height,
            mode: CanvasMode::Ascii,
            active_layer: 0,
            layers: vec![Layer::new("Base", width, height)],
        }
    }

    #[must_use]
    pub const fn contains(&self, point: Point) -> bool {
        point.x < self.width && point.y < self.height
    }

    #[must_use]
    pub fn index(&self, point: Point) -> Option<usize> {
        self.contains(point)
            .then(|| usize::from(point.y) * usize::from(self.width) + usize::from(point.x))
    }

    #[must_use]
    pub fn active_cell(&self, point: Point) -> Option<&Cell> {
        let index = self.index(point)?;
        self.layers.get(self.active_layer)?.cells.get(index)
    }

    pub fn active_cell_mut(&mut self, point: Point) -> Option<&mut Cell> {
        let index = self.index(point)?;
        self.layers.get_mut(self.active_layer)?.cells.get_mut(index)
    }

    #[must_use]
    pub fn composite_cell(&self, point: Point) -> Option<&Cell> {
        let index = self.index(point)?;
        self.layers
            .iter()
            .rev()
            .filter(|layer| layer.visible)
            .filter_map(|layer| layer.cells.get(index))
            .find(|cell| !cell.transparent)
            .or_else(|| self.layers.first()?.cells.get(index))
    }

    pub fn set_cell(&mut self, point: Point, cell: Cell) {
        if let Some(target) = self.active_cell_mut(point) {
            *target = cell;
        }
    }

    pub fn erase_cell(&mut self, point: Point) {
        if let Some(target) = self.active_cell_mut(point) {
            *target = Cell::default();
        }
    }

    pub fn add_layer(&mut self, name: impl Into<String>) {
        self.layers.push(Layer::new(name, self.width, self.height));
        self.active_layer = self.layers.len() - 1;
    }

    pub fn delete_active_layer(&mut self) -> bool {
        if self.layers.len() <= 1 {
            return false;
        }
        self.layers.remove(self.active_layer);
        self.active_layer = self.active_layer.min(self.layers.len() - 1);
        true
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        let width = width.max(1);
        let height = height.max(1);
        for layer in &mut self.layers {
            let mut resized = vec![Cell::default(); usize::from(width) * usize::from(height)];
            for y in 0..self.height.min(height) {
                for x in 0..self.width.min(width) {
                    let old_index = usize::from(y) * usize::from(self.width) + usize::from(x);
                    let new_index = usize::from(y) * usize::from(width) + usize::from(x);
                    resized[new_index] = layer.cells[old_index].clone();
                }
            }
            layer.cells = resized;
        }
        self.width = width;
        self.height = height;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum WidgetState {
    #[default]
    Normal,
    Focused,
    Active,
    Disabled,
}

impl WidgetState {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Focused => "focused",
            Self::Active => "active",
            Self::Disabled => "disabled",
        }
    }

    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Normal => Self::Focused,
            Self::Focused => Self::Active,
            Self::Active => Self::Disabled,
            Self::Disabled => Self::Normal,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnimationFrame {
    pub name: String,
    pub duration_ms: u64,
    pub widget_state: WidgetState,
    pub canvas: Canvas,
}

impl AnimationFrame {
    #[must_use]
    pub fn new(index: usize, width: u16, height: u16) -> Self {
        Self {
            name: format!("Frame {}", index + 1),
            duration_ms: 120,
            widget_state: WidgetState::Normal,
            canvas: Canvas::new(width, height),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RectSpec {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl RectSpec {
    #[must_use]
    pub const fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    #[must_use]
    pub const fn contains(self, point: Point) -> bool {
        point.x >= self.x
            && point.y >= self.y
            && point.x < self.x.saturating_add(self.width)
            && point.y < self.y.saturating_add(self.height)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum WidgetKind {
    #[default]
    Block,
    Paragraph,
    Gauge,
    List,
    Sparkline,
}

impl WidgetKind {
    pub const ALL: [Self; 5] = [
        Self::Block,
        Self::Paragraph,
        Self::Gauge,
        Self::List,
        Self::Sparkline,
    ];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Block => "Block",
            Self::Paragraph => "Paragraph",
            Self::Gauge => "Gauge",
            Self::List => "List",
            Self::Sparkline => "Sparkline",
        }
    }

    #[must_use]
    pub fn next(self) -> Self {
        let index = Self::ALL.iter().position(|kind| *kind == self).unwrap_or(0);
        Self::ALL[(index + 1) % Self::ALL.len()]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WidgetStyleSet {
    pub normal: CellStyle,
    pub focused: CellStyle,
    pub active: CellStyle,
    pub disabled: CellStyle,
}

impl Default for WidgetStyleSet {
    fn default() -> Self {
        Self {
            normal: CellStyle {
                fg: ColorSpec::Gray,
                ..CellStyle::default()
            },
            focused: CellStyle {
                fg: ColorSpec::LightCyan,
                bold: true,
                ..CellStyle::default()
            },
            active: CellStyle {
                fg: ColorSpec::LightGreen,
                bold: true,
                ..CellStyle::default()
            },
            disabled: CellStyle {
                fg: ColorSpec::DarkGray,
                dim: true,
                ..CellStyle::default()
            },
        }
    }
}

impl WidgetStyleSet {
    #[must_use]
    pub const fn for_state(&self, state: WidgetState) -> &CellStyle {
        match state {
            WidgetState::Normal => &self.normal,
            WidgetState::Focused => &self.focused,
            WidgetState::Active => &self.active,
            WidgetState::Disabled => &self.disabled,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WidgetSpec {
    pub id: u32,
    pub kind: WidgetKind,
    pub rect: RectSpec,
    pub title: String,
    pub text: String,
    pub value: u16,
    pub styles: WidgetStyleSet,
}

impl WidgetSpec {
    #[must_use]
    pub fn starter(id: u32, kind: WidgetKind, rect: RectSpec) -> Self {
        Self {
            id,
            kind,
            rect,
            title: kind.label().to_owned(),
            text: match kind {
                WidgetKind::List => "First item\nSecond item\nThird item".to_owned(),
                WidgetKind::Sparkline => "2,4,3,7,5,9,8,12".to_owned(),
                _ => "Edit with :text …".to_owned(),
            },
            value: 62,
            styles: WidgetStyleSet::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentScene {
    pub width: u16,
    pub height: u16,
    pub widgets: Vec<WidgetSpec>,
    pub next_id: u32,
}

impl Default for ComponentScene {
    fn default() -> Self {
        Self {
            width: 80,
            height: 24,
            widgets: vec![
                WidgetSpec::starter(1, WidgetKind::Block, RectSpec::new(0, 0, 80, 24)),
                WidgetSpec::starter(2, WidgetKind::Paragraph, RectSpec::new(2, 2, 48, 8)),
                WidgetSpec::starter(3, WidgetKind::Gauge, RectSpec::new(52, 2, 26, 3)),
            ],
            next_id: 4,
        }
    }
}

impl ComponentScene {
    pub fn add_widget(&mut self, kind: WidgetKind) -> usize {
        let offset = (self.widgets.len() as u16 * 2) % 12;
        let rect = RectSpec::new(3 + offset, 3 + offset / 2, 24, 7);
        let index = self.widgets.len();
        self.widgets.push(WidgetSpec::starter(self.next_id, kind, rect));
        self.next_id += 1;
        index
    }

    pub fn clamp_widget(&mut self, index: usize) {
        let Some(widget) = self.widgets.get_mut(index) else {
            return;
        };
        widget.rect.width = widget.rect.width.clamp(1, self.width);
        widget.rect.height = widget.rect.height.clamp(1, self.height);
        widget.rect.x = widget.rect.x.min(self.width.saturating_sub(widget.rect.width));
        widget.rect.y = widget.rect.y.min(self.height.saturating_sub(widget.rect.height));
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    pub version: u32,
    pub name: String,
    pub active_frame: usize,
    pub frames: Vec<AnimationFrame>,
    pub components: ComponentScene,
}

impl Project {
    #[must_use]
    pub fn new(name: impl Into<String>, width: u16, height: u16) -> Self {
        Self {
            version: PROJECT_VERSION,
            name: name.into(),
            active_frame: 0,
            frames: vec![AnimationFrame::new(0, width, height)],
            components: ComponentScene::default(),
        }
    }

    #[must_use]
    pub fn frame(&self) -> &AnimationFrame {
        &self.frames[self.active_frame]
    }

    pub fn frame_mut(&mut self) -> &mut AnimationFrame {
        &mut self.frames[self.active_frame]
    }

    #[must_use]
    pub fn canvas(&self) -> &Canvas {
        &self.frame().canvas
    }

    pub fn canvas_mut(&mut self) -> &mut Canvas {
        &mut self.frame_mut().canvas
    }

    pub fn add_frame(&mut self) {
        let index = self.frames.len();
        let (width, height) = (self.canvas().width, self.canvas().height);
        self.frames.push(AnimationFrame::new(index, width, height));
        self.active_frame = index;
    }

    pub fn duplicate_frame(&mut self) {
        let mut duplicate = self.frame().clone();
        duplicate.name = format!("Frame {}", self.frames.len() + 1);
        let insert_at = self.active_frame + 1;
        self.frames.insert(insert_at, duplicate);
        self.active_frame = insert_at;
    }

    pub fn delete_active_frame(&mut self) -> bool {
        if self.frames.len() <= 1 {
            return false;
        }
        self.frames.remove(self.active_frame);
        self.active_frame = self.active_frame.min(self.frames.len() - 1);
        true
    }

    pub fn next_frame(&mut self) {
        self.active_frame = (self.active_frame + 1) % self.frames.len();
    }

    pub fn previous_frame(&mut self) {
        self.active_frame = (self.active_frame + self.frames.len() - 1) % self.frames.len();
    }

    pub fn resize_all_frames(&mut self, width: u16, height: u16) {
        for frame in &mut self.frames {
            frame.canvas.resize(width, height);
        }
    }

    pub fn validate(&mut self) {
        self.version = PROJECT_VERSION;
        if self.frames.is_empty() {
            self.frames.push(AnimationFrame::new(0, 48, 18));
        }
        self.active_frame = self.active_frame.min(self.frames.len() - 1);
        for frame in &mut self.frames {
            if frame.canvas.layers.is_empty() {
                frame.canvas.layers.push(Layer::new(
                    "Base",
                    frame.canvas.width.max(1),
                    frame.canvas.height.max(1),
                ));
            }
            frame.canvas.active_layer = frame
                .canvas
                .active_layer
                .min(frame.canvas.layers.len().saturating_sub(1));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upper_visible_layer_wins_compositing() {
        let mut canvas = Canvas::new(4, 4);
        canvas.set_cell(
            Point::new(1, 1),
            Cell::painted('a', CellStyle::default()),
        );
        canvas.add_layer("Top");
        canvas.set_cell(
            Point::new(1, 1),
            Cell::painted('b', CellStyle::default()),
        );
        assert_eq!(canvas.composite_cell(Point::new(1, 1)).unwrap().glyph, "b");
    }

    #[test]
    fn duplicate_frame_is_independent() {
        let mut project = Project::new("test", 4, 4);
        project.duplicate_frame();
        project.canvas_mut().set_cell(
            Point::new(0, 0),
            Cell::painted('x', CellStyle::default()),
        );
        assert_ne!(project.frames[0].canvas, project.frames[1].canvas);
    }

    #[test]
    fn ascii_mode_rejects_unicode() {
        assert!(!CanvasMode::Ascii.accepts('─'));
        assert!(CanvasMode::Unicode.accepts('─'));
    }
}
