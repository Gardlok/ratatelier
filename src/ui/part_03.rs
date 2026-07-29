fn labeled(label: &'static str, value: impl Into<String>) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label}: "), Style::default().fg(Color::DarkGray)),
        Span::raw(value.into()),
    ])
}

fn dirty_mark(app: &App) -> &'static str {
    if app.dirty {
        " [+]"
    } else {
        ""
    }
}

fn scale_rect(rect: RectSpec, area: Rect, source_width: u16, source_height: u16) -> Rect {
    if source_width == 0 || source_height == 0 {
        return Rect::default();
    }
    let x = area.x + rect.x.saturating_mul(area.width) / source_width;
    let y = area.y + rect.y.saturating_mul(area.height) / source_height;
    let width = rect
        .width
        .saturating_mul(area.width)
        .div_ceil(source_width)
        .max(1)
        .min(area.right().saturating_sub(x));
    let height = rect
        .height
        .saturating_mul(area.height)
        .div_ceil(source_height)
        .max(1)
        .min(area.bottom().saturating_sub(y));
    Rect::new(x, y, width, height)
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

fn contains(area: Rect, x: u16, y: u16) -> bool {
    x >= area.x && y >= area.y && x < area.right() && y < area.bottom()
}

fn mode_color(mode: Mode) -> Color {
    match mode {
        Mode::Normal => Color::LightBlue,
        Mode::Draw => Color::LightGreen,
        Mode::Insert => Color::LightYellow,
        Mode::Select => Color::LightMagenta,
        Mode::Command => Color::LightCyan,
    }
}

#[must_use]
pub fn cell_style(style: &CellStyle) -> Style {
    let mut result = Style::default().fg(color(style.fg)).bg(color(style.bg));
    for (enabled, modifier) in [
        (style.bold, Modifier::BOLD),
        (style.dim, Modifier::DIM),
        (style.italic, Modifier::ITALIC),
        (style.underlined, Modifier::UNDERLINED),
        (style.reversed, Modifier::REVERSED),
        (style.crossed_out, Modifier::CROSSED_OUT),
    ] {
        if enabled {
            result = result.add_modifier(modifier);
        }
    }
    result
}

const fn color(color: ColorSpec) -> Color {
    match color {
        ColorSpec::Reset => Color::Reset,
        ColorSpec::Black => Color::Black,
        ColorSpec::Red => Color::Red,
        ColorSpec::Green => Color::Green,
        ColorSpec::Yellow => Color::Yellow,
        ColorSpec::Blue => Color::Blue,
        ColorSpec::Magenta => Color::Magenta,
        ColorSpec::Cyan => Color::Cyan,
        ColorSpec::Gray => Color::Gray,
        ColorSpec::DarkGray => Color::DarkGray,
        ColorSpec::LightRed => Color::LightRed,
        ColorSpec::LightGreen => Color::LightGreen,
        ColorSpec::LightYellow => Color::LightYellow,
        ColorSpec::LightBlue => Color::LightBlue,
        ColorSpec::LightMagenta => Color::LightMagenta,
        ColorSpec::LightCyan => Color::LightCyan,
        ColorSpec::White => Color::White,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Project;

    #[test]
    fn canvas_mapping_accounts_for_viewport_origin() {
        let mut app = App::new(Project::new("test", 100, 40), None);
        app.viewport_origin = Point::new(12, 7);
        let regions = calculate_regions(Rect::new(0, 0, 140, 40), &app);
        let point = regions
            .canvas_point(regions.canvas_inner.x, regions.canvas_inner.y)
            .unwrap();
        assert_eq!(point, Point::new(12, 7));
    }

    #[test]
    fn viewport_does_not_recenter_when_cursor_moves() {
        let mut app = App::new(Project::new("test", 100, 40), None);
        app.viewport_origin = Point::new(8, 5);
        app.cursor = Point::new(80, 30);
        let regions = calculate_regions(Rect::new(0, 0, 140, 40), &app);
        assert_eq!(regions.viewport_origin, Point::new(8, 5));
    }

    #[test]
    fn hiding_export_gives_workspace_more_room() {
        let mut app = App::new(Project::new("test", 100, 40), None);
        let shown = calculate_regions(Rect::new(0, 0, 140, 40), &app);
        app.show_export = false;
        let hidden = calculate_regions(Rect::new(0, 0, 140, 40), &app);
        assert!(hidden.workspace.width > shown.workspace.width);
        assert_eq!(hidden.code.width, 0);
        assert!(hidden.toolbar.width > 0);
    }

    #[test]
    fn component_mapping_stays_in_scene_coordinates() {
        let app = App::new(Project::new("test", 10, 10), None);
        let regions = calculate_regions(Rect::new(0, 0, 140, 50), &app);
        let point = regions
            .component_point(
                regions.component_inner.right() - 1,
                regions.component_inner.bottom() - 1,
            )
            .unwrap();
        assert!(point.x < app.project.components.width);
        assert!(point.y < app.project.components.height);
    }

    #[test]
    fn export_selection_highlights_requested_characters() {
        let line = export_line(
            0,
            "abcdef",
            Some(TextSelection {
                anchor: crate::app::TextPoint { line: 0, column: 1 },
                head: crate::app::TextPoint { line: 0, column: 3 },
            }),
        );
        assert_eq!(line.spans.len(), 3);
        assert_eq!(line.spans[1].content.as_ref(), "bcd");
    }

    #[test]
    fn timeline_window_keeps_the_active_frame_visible() {
        let labels: Vec<String> = (1..=24)
            .map(|index| format!(" {index} · 120ms · normal "))
            .collect();
        let (start, end) = timeline_visible_range(&labels, 19, 48);
        assert!(start <= 19);
        assert!(19 < end);
        assert!(start > 0);
    }
}
