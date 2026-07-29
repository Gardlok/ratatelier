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
}
