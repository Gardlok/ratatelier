const PALETTE_ENTRY_COUNT: usize = ColorSpec::PALETTE.len() + 1;

#[must_use]
pub fn palette_scroll_limit(_app: &App, area: Rect) -> u16 {
    let visible = usize::from(area.height.saturating_sub(2));
    u16::try_from(PALETTE_ENTRY_COUNT.saturating_sub(visible)).unwrap_or(u16::MAX)
}

#[must_use]
pub fn palette_entry_at(app: &App, area: Rect, screen_row: u16, scroll: u16) -> Option<usize> {
    if !app.palette_visible()
        || area.height <= 2
        || screen_row <= area.y
        || screen_row >= area.bottom().saturating_sub(1)
    {
        return None;
    }
    let maximum = palette_scroll_limit(app, area);
    let index = usize::from(
        scroll
            .min(maximum)
            .saturating_add(screen_row.saturating_sub(area.y).saturating_sub(1)),
    );
    (index < PALETTE_ENTRY_COUNT).then_some(index)
}

fn draw_palette(frame: &mut Frame<'_>, app: &App) {
    let area = app.regions.palette;
    if area.width == 0 || area.height == 0 {
        return;
    }
    let maximum = palette_scroll_limit(app, area);
    let scroll = usize::from(app.palette_scroll().min(maximum));
    let visible = usize::from(area.height.saturating_sub(2));
    let current = app.current_target_color();
    let lines = (scroll..PALETTE_ENTRY_COUNT.min(scroll.saturating_add(visible)))
        .map(|entry| {
            let (symbol, mut style) = match entry {
                0 => ("×", Style::default().fg(Color::Gray)),
                1 => (
                    if app.color_pick_pending() { "◎" } else { "◉" },
                    Style::default().fg(Color::LightCyan),
                ),
                _ => {
                    let color_spec = ColorSpec::PALETTE[entry - 1];
                    let selected = color_spec == current;
                    (
                        if selected { "▓" } else { "█" },
                        Style::default().fg(color(color_spec)),
                    )
                }
            };
            if entry == 0 && current == ColorSpec::Reset {
                style = style.add_modifier(Modifier::BOLD | Modifier::REVERSED);
            }
            Line::from(Span::styled(symbol, style))
        })
        .collect::<Vec<_>>();
    let (title, border_color) = match app.color_target() {
        ColorTarget::Foreground => ("●", Color::LightCyan),
        ColorTarget::Background => ("○", Color::LightMagenta),
    };
    frame.render_widget(
        Paragraph::new(lines)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(Style::default().fg(border_color)),
            ),
        area,
    );
}

fn timeline_labels(app: &App) -> Vec<String> {
    app.project
        .frames
        .iter()
        .enumerate()
        .map(|(index, animation_frame)| {
            format!(
                " {}:{} · {}ms · {} ",
                index + 1,
                compact_frame_name(&animation_frame.name, 12),
                animation_frame.duration_ms,
                animation_frame.widget_state.label()
            )
        })
        .collect()
}

#[must_use]
pub fn timeline_frame_at(app: &App, area: Rect, column: u16) -> Option<usize> {
    if area.width <= 2 || column <= area.x || column >= area.right().saturating_sub(1) {
        return None;
    }
    let labels = timeline_labels(app);
    let available_width = usize::from(area.width.saturating_sub(2));
    let (start, end) = timeline_visible_range(&labels, app.project.active_frame, available_width);
    let target = usize::from(column.saturating_sub(area.x).saturating_sub(1));
    let mut cursor = 0usize;
    for (index, label) in labels.iter().enumerate().take(end).skip(start) {
        let width = label.chars().count();
        if target >= cursor && target < cursor.saturating_add(width) {
            return Some(index);
        }
        cursor = cursor.saturating_add(width).saturating_add(1);
    }
    None
}

fn context_hint(app: &App) -> String {
    if app.color_pick_pending() {
        return format!(
            "click canvas to sample {} · Esc cancel",
            app.color_target().label()
        );
    }
    if app.inspector_focused() {
        return "inspector: wheel/j/k scroll · click layer · PgUp/PgDn · g/G · Esc release"
            .to_owned();
    }
    if app.export_focused {
        return "export: wheel/j/k scroll · h/l horizontal · drag select · Esc release".to_owned();
    }
    if app.is_playing() {
        return "p pause · click timeline frame · wheel timeline · +/- duration".to_owned();
    }
    match (app.workspace, app.mode) {
        (Workspace::Artwork, Mode::Draw) => {
            "draw: hjkl paint · c/C color target · [/] glyph · Space apply · Esc finish".to_owned()
        }
        (Workspace::Artwork, Mode::Insert) => {
            "insert: type glyphs · arrows move · Backspace erase · Esc finish".to_owned()
        }
        (Workspace::Artwork, Mode::Select) => {
            "select: hjkl extend · y copy · x cut · p paste · Esc cancel".to_owned()
        }
        (Workspace::Artwork, _) => {
            "1-5 tool · c/C fg/bg rail · F3 palette · F2 export · ? help".to_owned()
        }
        (Workspace::Components, _) => {
            "components: hjkl move · HJKL resize · c/C fg/bg rail · [/] select · ? help"
                .to_owned()
        }
    }
}

#[cfg(test)]
mod interaction_polish_ui_tests {
    use super::*;
    use crate::model::Project;

    #[test]
    fn short_palette_scrolls_to_reach_every_entry() {
        let app = App::new(Project::new("test", 4, 4), None);
        assert!(palette_scroll_limit(&app, Rect::new(0, 0, 3, 8)) > 0);
    }

    #[test]
    fn timeline_hit_testing_returns_a_visible_frame() {
        let mut project = Project::new("test", 4, 4);
        project.add_frame();
        let app = App::new(project, None);
        let area = Rect::new(0, 0, 100, 4);
        let labels = timeline_labels(&app);
        let first_label_middle = area.x + 1 + u16::try_from(labels[0].chars().count() / 2).unwrap();
        assert_eq!(timeline_frame_at(&app, area, first_label_middle), Some(0));
    }
}
