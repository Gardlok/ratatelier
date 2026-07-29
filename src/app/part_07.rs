pub fn run(app: &mut App) -> io::Result<()> {
    enable_raw_mode()?;
    let mut output = stdout();
    execute!(output, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(output);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let result = run_loop(&mut terminal, app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    while app.running {
        let size = terminal.size()?;
        let area = ratatui::layout::Rect::new(0, 0, size.width, size.height);
        app.regions = ui::calculate_regions(area, app);
        terminal.draw(|frame| ui::draw(frame, app))?;
        if event::poll(TICK_RATE)? {
            app.handle_event(event::read()?);
        }
        app.tick();
    }
    Ok(())
}

fn add_signed(value: u16, delta: i32) -> u16 {
    if delta.is_negative() {
        value.saturating_sub(delta.unsigned_abs() as u16)
    } else {
        value.saturating_add(delta as u16)
    }
}

fn parse_dimensions(value: &str) -> Option<(u16, u16)> {
    let (width, height) = value
        .split_once('x')
        .or_else(|| value.split_once('X'))?;
    Some((width.trim().parse().ok()?, height.trim().parse().ok()?))
}

fn parse_widget_kind(value: &str) -> WidgetKind {
    match value.to_ascii_lowercase().as_str() {
        "block" => WidgetKind::Block,
        "gauge" => WidgetKind::Gauge,
        "list" => WidgetKind::List,
        "spark" | "sparkline" => WidgetKind::Sparkline,
        _ => WidgetKind::Paragraph,
    }
}

fn safe_file_stem(name: &str) -> String {
    let stem: String = name
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let stem = stem.trim_matches('-');
    if stem.is_empty() {
        "ratatelier-project".to_owned()
    } else {
        stem.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mouse(kind: MouseEventKind, column: u16, row: u16) -> Event {
        Event::Mouse(MouseEvent {
            kind,
            column,
            row,
            modifiers: KeyModifiers::NONE,
        })
    }

    #[test]
    fn parses_common_dimensions() {
        assert_eq!(parse_dimensions("80x24"), Some((80, 24)));
        assert_eq!(parse_dimensions("12 X 7"), Some((12, 7)));
        assert_eq!(parse_dimensions("nonsense"), None);
    }

    #[test]
    fn file_stem_is_shell_friendly() {
        assert_eq!(safe_file_stem("My Great Logo!"), "my-great-logo");
    }

    #[test]
    fn ctrl_x_requests_quit() {
        let mut app = App::new(Project::new("test", 4, 4), None);
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Char('x'),
            KeyModifiers::CONTROL,
        )));
        assert!(!app.running);
    }

    #[test]
    fn ctrl_q_does_not_request_quit() {
        let mut app = App::new(Project::new("test", 4, 4), None);
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Char('q'),
            KeyModifiers::CONTROL,
        )));
        assert!(app.running);
    }

    #[test]
    fn canvas_click_places_cursor_without_recentering_viewport() {
        let mut app = App::new(Project::new("test", 100, 100), None);
        app.viewport_origin = Point::new(10, 4);
        app.regions = ui::calculate_regions(Rect::new(0, 0, 160, 50), &app);
        let column = app.regions.canvas_inner.x + 5;
        let row = app.regions.canvas_inner.y + 3;

        app.handle_event(mouse(
            MouseEventKind::Down(MouseButton::Left),
            column,
            row,
        ));

        assert_eq!(app.cursor, Point::new(15, 7));
        assert_eq!(app.viewport_origin, Point::new(10, 4));
    }

    #[test]
    fn right_drag_pans_canvas_without_moving_cursor() {
        let mut app = App::new(Project::new("test", 100, 100), None);
        app.cursor = Point::new(20, 10);
        app.viewport_origin = Point::new(10, 4);
        app.regions = ui::calculate_regions(Rect::new(0, 0, 160, 50), &app);
        let column = app.regions.canvas_inner.x + 10;
        let row = app.regions.canvas_inner.y + 5;

        app.handle_event(mouse(
            MouseEventKind::Down(MouseButton::Right),
            column,
            row,
        ));
        app.handle_event(mouse(
            MouseEventKind::Drag(MouseButton::Right),
            column + 3,
            row + 2,
        ));

        assert_eq!(app.viewport_origin, Point::new(7, 2));
        assert_eq!(app.cursor, Point::new(20, 10));
    }
}
