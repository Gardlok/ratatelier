fn help_lines(app: &App) -> Vec<Line<'static>> {
    vec![
        Line::styled(
            "Ratatelier command map",
            Style::default()
                .fg(Color::LightCyan)
                .add_modifier(Modifier::BOLD),
        ),
        Line::from(""),
        Line::from(
            "Global: Tab workspace · F2/Ctrl-E export · F3 palette · Ctrl-S save · Ctrl-X quit · : command",
        ),
        Line::from("Unsaved exit: S saves · D discards · Esc cancels"),
        Line::from(""),
        Line::styled("Artwork", Style::default().fg(Color::LightGreen)),
        Line::from("h/j/k/l move · d draw · e erase · i insert · v select · u undo · Ctrl-R redo"),
        Line::from("1 pencil · 2 eraser · 3 line · 4 rectangle · 5 fill · Space apply/anchor"),
        Line::from(
            "Selection: Ctrl-A all · y/Ctrl-C yank · x cut · p/P/Ctrl-V system paste",
        ),
        Line::from("Ratatelier yanks retain styles · external text uses neutral styling"),
        Line::from("Mouse: left paints/selects · selected cells drag · right drag pans/picks"),
        Line::from(""),
        Line::styled("Color rail", Style::default().fg(Color::LightGreen)),
        Line::from("c targets foreground · C targets background · F3 or :palette hides/shows"),
        Line::from("click a swatch to apply · wheel scrolls · × resets · ◉ samples the canvas"),
        Line::from("the rail starts visible; the live Rust export starts hidden"),
        Line::from(""),
        Line::styled("Layers", Style::default().fg(Color::LightGreen)),
        Line::from("PgUp/{ previous · PgDn/} next · a add · A delete · D duplicate · V visibility"),
        Line::from(":layer select N|rename NAME|show|hide|toggle|move up|down|top|bottom"),
        Line::from("Inspector: wheel scrolls · click focuses/selects layers · j/k PgUp/PgDn g/G · Esc"),
        Line::from(""),
        Line::styled("Frames", Style::default().fg(Color::LightGreen)),
        Line::from(",/. previous/next · Home/End first/last · </> reorder · n/N/X add/dup/delete"),
        Line::from("+/- duration · p playback · s state · click or wheel the timeline to select"),
        Line::from(":frame select N|rename NAME|duration MS|state STATE|move left|right|first|last"),
        Line::from(""),
        Line::styled("Components", Style::default().fg(Color::LightGreen)),
        Line::from("h/j/k/l move · H/J/K/L resize · [/] select · a add · x delete · t type"),
        Line::from("Left drag moves · right drag resizes · c/C targets current-state fg/bg"),
        Line::from(":title TITLE · :text TEXT · :value 0..100 · :widget add KIND"),
        Line::from(""),
        Line::styled("Command line", Style::default().fg(Color::LightGreen)),
        Line::from("Tab/Shift-Tab completes commands, actions, directories, and files"),
        Line::from("Up/Down browses session command history and restores the unfinished draft"),
        Line::from(""),
        Line::styled("Live export", Style::default().fg(Color::LightGreen)),
        Line::from("F2 or tool rail toggles · click to focus · drag to select · right click clears"),
        Line::from("Mouse wheel or j/k scroll · h/l horizontal · PgUp/PgDn · g/G top/bottom"),
        Line::from(""),
        Line::styled("Files and export", Style::default().fg(Color::LightGreen)),
        Line::from(":w [path] · :open path · :new 48x18 · :resize 80x24 · :scene 80x24"),
        Line::from(":export [path] · :export art|plain|animation|component [path] · :wq · :q!"),
        Line::from(""),
        Line::styled(
            format!("Current project: {}", app.project.name),
            Style::default().fg(Color::DarkGray),
        ),
    ]
}

#[must_use]
pub fn help_scroll_limit(app: &App, area: Rect) -> u16 {
    let width = usize::from(area.width.saturating_sub(2).max(1));
    let visible_height = usize::from(area.height.saturating_sub(2));
    let visual_height = help_lines(app)
        .iter()
        .map(|line| {
            let line_width = line
                .spans
                .iter()
                .map(|span| span.content.chars().count())
                .sum::<usize>();
            line_width.max(1).div_ceil(width)
        })
        .sum::<usize>();
    u16::try_from(visual_height.saturating_sub(visible_height)).unwrap_or(u16::MAX)
}

fn draw_scrollable_help(frame: &mut Frame<'_>, app: &App) {
    let area = app.regions.help;
    let maximum = help_scroll_limit(app, area);
    if maximum == 0 {
        draw_help(frame, app);
        return;
    }

    let scroll = app.help_scroll().min(maximum);
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(help_lines(app))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false })
            .scroll((scroll, 0))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(
                        " Help · {}/{} · j/k PgUp/PgDn g/G · Esc close ",
                        scroll.saturating_add(1),
                        maximum.saturating_add(1)
                    ))
                    .border_style(Style::default().fg(Color::LightCyan)),
            ),
        area,
    );
}

#[cfg(test)]
mod help_scroll_tests {
    use super::*;
    use crate::model::Project;

    #[test]
    fn short_help_dialog_has_a_positive_scroll_limit() {
        let app = App::new(Project::new("test", 4, 4), None);
        assert!(help_scroll_limit(&app, Rect::new(0, 0, 60, 12)) > 0);
    }

    #[test]
    fn tall_help_dialog_does_not_scroll_past_content() {
        let app = App::new(Project::new("test", 4, 4), None);
        assert_eq!(help_scroll_limit(&app, Rect::new(0, 0, 120, 80)), 0);
    }
}
