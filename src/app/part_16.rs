#[cfg(test)]
mod color_space_commit_tests {
    use super::*;

    #[test]
    fn selected_color_commits_with_space_before_insert_mode_consumes_it() {
        let mut app = App::new(Project::new("test", 4, 4), None);
        app.mode = Mode::Insert;
        app.activate_color_palette(ColorTarget::Foreground);
        app.apply_target_color(ColorSpec::LightBlue);

        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Char(' '),
            KeyModifiers::NONE,
        )));

        let cell = app
            .project
            .canvas()
            .active_cell(Point::default())
            .expect("Space should paint the active cell");
        assert_eq!(cell.glyph, "#");
        assert_eq!(cell.style.fg, ColorSpec::LightBlue);
        assert!(!cell.transparent);
        assert_eq!(app.mode, Mode::Insert);
        assert!(!app.color_commit_pending());
    }
}
