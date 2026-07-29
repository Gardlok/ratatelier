use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, Gauge, List, ListItem, Paragraph, Sparkline, Tabs, Widget, Wrap,
    },
    Frame,
};

use crate::{
    app::{App, Mode, Workspace},
    export,
    model::{CellStyle, ColorSpec, Point, RectSpec, WidgetKind},
};

#[derive(Debug, Clone, Copy, Default)]
pub struct UiRegions {
    pub header: Rect,
    pub inspector: Rect,
    pub workspace: Rect,
    pub code: Rect,
    pub timeline: Rect,
    pub footer: Rect,
    pub canvas_inner: Rect,
    pub component_inner: Rect,
    pub viewport_origin: Point,
    pub component_size: Point,
}

impl UiRegions {
    #[must_use]
    pub fn canvas_point(self, column: u16, row: u16) -> Option<Point> {
        contains(self.canvas_inner, column, row).then(|| {
            Point::new(
                self.viewport_origin.x + column - self.canvas_inner.x,
                self.viewport_origin.y + row - self.canvas_inner.y,
            )
        })
    }

    #[must_use]
    pub fn component_point(self, column: u16, row: u16) -> Option<Point> {
        if !contains(self.component_inner, column, row)
            || self.component_inner.width == 0
            || self.component_inner.height == 0
        {
            return None;
        }
        let local_x = column - self.component_inner.x;
        let local_y = row - self.component_inner.y;
        Some(Point::new(
            local_x.saturating_mul(self.component_size.x) / self.component_inner.width,
            local_y.saturating_mul(self.component_size.y) / self.component_inner.height,
        ))
    }
}

#[must_use]
pub fn calculate_regions(area: Rect, app: &App) -> UiRegions {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(4),
            Constraint::Length(2),
        ])
        .split(area);
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(26),
            Constraint::Min(30),
            Constraint::Length(42),
        ])
        .split(vertical[1]);

    let workspace_inner = body[1].inner(ratatui::layout::Margin {
        horizontal: 1,
        vertical: 1,
    });
    let canvas = app.project.canvas();
    let visible_width = workspace_inner.width.min(canvas.width);
    let visible_height = workspace_inner.height.min(canvas.height);
    let max_x = canvas.width.saturating_sub(visible_width);
    let max_y = canvas.height.saturating_sub(visible_height);
    let origin_x = app
        .cursor
        .x
        .saturating_sub(visible_width / 2)
        .min(max_x);
    let origin_y = app
        .cursor
        .y
        .saturating_sub(visible_height / 2)
        .min(max_y);
    let centered_x = workspace_inner.x + workspace_inner.width.saturating_sub(visible_width) / 2;
    let centered_y = workspace_inner.y + workspace_inner.height.saturating_sub(visible_height) / 2;

    UiRegions {
        header: vertical[0],
        inspector: body[0],
        workspace: body[1],
        code: body[2],
        timeline: vertical[2],
        footer: vertical[3],
        canvas_inner: Rect::new(centered_x, centered_y, visible_width, visible_height),
        component_inner: workspace_inner,
        viewport_origin: Point::new(origin_x, origin_y),
        component_size: Point::new(app.project.components.width, app.project.components.height),
    }
}

pub fn draw(frame: &mut Frame<'_>, app: &App) {
    draw_header(frame, app);
    draw_inspector(frame, app);
    match app.workspace {
        Workspace::Artwork => draw_artwork(frame, app),
        Workspace::Components => draw_components(frame, app),
    }
    draw_code(frame, app);
    draw_timeline(frame, app);
    draw_footer(frame, app);
    if app.show_help {
        draw_help(frame, app);
    }
}
