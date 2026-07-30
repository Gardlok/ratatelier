fn inspector_line_height(line: &Line<'_>, width: usize) -> usize {
    let line_width = line
        .spans
        .iter()
        .map(|span| span.content.chars().count())
        .sum::<usize>();
    line_width.max(1).div_ceil(width.max(1))
}

#[must_use]
pub fn inspector_scroll_limit(app: &App, area: Rect) -> u16 {
    let width = usize::from(area.width.saturating_sub(2).max(1));
    let visible_height = usize::from(area.height.saturating_sub(2));
    let visual_height = inspector_lines(app)
        .iter()
        .map(|line| inspector_line_height(line, width))
        .sum::<usize>();
    u16::try_from(visual_height.saturating_sub(visible_height)).unwrap_or(u16::MAX)
}

fn artwork_layer_visual_range(app: &App, area: Rect, layer_index: usize) -> Option<(u16, u16)> {
    if app.workspace != Workspace::Artwork || layer_index >= app.project.canvas().layers.len() {
        return None;
    }
    let width = usize::from(area.width.saturating_sub(2).max(1));
    let total = app.project.canvas().layers.len();
    let line_index = ARTWORK_LAYER_LINE_START + total - 1 - layer_index;
    let lines = inspector_lines(app);
    let start = lines
        .iter()
        .take(line_index)
        .map(|line| inspector_line_height(line, width))
        .sum::<usize>();
    let height = inspector_line_height(lines.get(line_index)?, width);
    let end = start.saturating_add(height);
    Some((
        u16::try_from(start).unwrap_or(u16::MAX),
        u16::try_from(end).unwrap_or(u16::MAX),
    ))
}

#[must_use]
pub fn inspector_layer_at(app: &App, area: Rect, screen_row: u16, scroll: u16) -> Option<usize> {
    if app.workspace != Workspace::Artwork
        || area.height <= 2
        || screen_row <= area.y
        || screen_row >= area.bottom().saturating_sub(1)
    {
        return None;
    }
    let maximum = inspector_scroll_limit(app, area);
    let target = scroll
        .min(maximum)
        .saturating_add(screen_row.saturating_sub(area.y).saturating_sub(1));
    (0..app.project.canvas().layers.len()).find(|&index| {
        artwork_layer_visual_range(app, area, index)
            .is_some_and(|(start, end)| target >= start && target < end)
    })
}

#[must_use]
pub fn inspector_scroll_to_reveal_layer(
    app: &App,
    area: Rect,
    scroll: u16,
    layer_index: usize,
) -> u16 {
    let maximum = inspector_scroll_limit(app, area);
    let current = scroll.min(maximum);
    let visible = area.height.saturating_sub(2);
    let Some((start, end)) = artwork_layer_visual_range(app, area, layer_index) else {
        return current;
    };
    if visible == 0 {
        return current;
    }
    if start < current {
        start.min(maximum)
    } else if end > current.saturating_add(visible) {
        end.saturating_sub(visible).min(maximum)
    } else {
        current
    }
}

#[cfg(test)]
mod inspector_scroll_tests {
    use super::*;
    use crate::model::Project;

    #[test]
    fn layer_rows_map_back_to_their_model_index() {
        let mut app = App::new(Project::new("test", 4, 4), None);
        app.project.canvas_mut().add_layer("Middle");
        app.project.canvas_mut().add_layer("Top");
        let area = Rect::new(0, 0, 26, 30);
        let (start, _) = artwork_layer_visual_range(&app, area, 2).unwrap();
        let row = area.y + 1 + start;
        assert_eq!(inspector_layer_at(&app, area, row, 0), Some(2));
    }

    #[test]
    fn reveal_scroll_keeps_selected_layer_inside_viewport() {
        let mut app = App::new(Project::new("test", 4, 4), None);
        for index in 0..12 {
            app.project.canvas_mut().add_layer(format!("Layer {index}"));
        }
        let area = Rect::new(0, 0, 26, 10);
        let scroll = inspector_scroll_to_reveal_layer(&app, area, 0, 0);
        assert!(scroll > 0);
        assert!(scroll <= inspector_scroll_limit(&app, area));
    }
}
