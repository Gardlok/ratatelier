fn draw_unsaved_exit(frame: &mut Frame<'_>, app: &App) {
    let area = centered_rect(58, 30, frame.area());
    let destination = app.current_path.as_ref().map_or_else(
        || "A default project filename will be used.".to_owned(),
        |path| format!("Save destination: {}", path.display()),
    );
    let lines = vec![
        Line::styled(
            "Unsaved changes",
            Style::default()
                .fg(Color::LightYellow)
                .add_modifier(Modifier::BOLD),
        ),
        Line::from(""),
        Line::from("Save the current project before exiting Ratatelier?"),
        Line::styled(destination, Style::default().fg(Color::DarkGray)),
        Line::from(""),
        Line::from(vec![
            Span::styled(" [S] Save ", Style::default().fg(Color::LightGreen)),
            Span::raw("  "),
            Span::styled(" [D] Discard ", Style::default().fg(Color::LightRed)),
            Span::raw("  "),
            Span::styled(" [Esc] Cancel ", Style::default().fg(Color::LightCyan)),
        ]),
    ];
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(lines)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Exit Ratatelier ")
                    .border_style(
                        Style::default()
                            .fg(Color::LightYellow)
                            .add_modifier(Modifier::BOLD),
                    ),
            ),
        area,
    );
}
