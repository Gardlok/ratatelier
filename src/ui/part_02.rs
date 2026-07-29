fn draw_components(frame: &mut Frame<'_>, app: &App) {
    frame.render_widget(
        Block::default().borders(Borders::ALL).title(format!(
            " Component surface {}×{} · {} state ",
            app.project.components.width,
            app.project.components.height,
            app.project.frame().widget_state.label()
        )),
        app.regions.workspace,
    );

    let scene = &app.project.components;
    for (index, widget) in scene.widgets.iter().enumerate() {
        let area = scale_rect(widget.rect, app.regions.component_inner, scene.width, scene.height);
        if area.width == 0 || area.height == 0 {
            continue;
        }
        let selected = index == app.component_selected;
        let style = cell_style(widget.styles.for_state(app.project.frame().widget_state));
        let border_style = if selected {
            Style::default()
                .fg(Color::LightMagenta)
                .add_modifier(Modifier::BOLD)
        } else {
            style
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .title(widget.title.as_str())
            .border_style(border_style);

        match widget.kind {
            WidgetKind::Block => frame.render_widget(block.style(style), area),
            WidgetKind::Paragraph => frame.render_widget(
                Paragraph::new(widget.text.as_str())
                    .block(block)
                    .style(style)
                    .wrap(Wrap { trim: false }),
                area,
            ),
            WidgetKind::Gauge => frame.render_widget(
                Gauge::default()
                    .block(block)
                    .ratio(f64::from(widget.value.min(100)) / 100.0)
                    .gauge_style(style),
                area,
            ),
            WidgetKind::List => {
                let items: Vec<ListItem<'_>> = widget.text.lines().map(ListItem::new).collect();
                frame.render_widget(List::new(items).block(block).style(style), area);
            }
            WidgetKind::Sparkline => {
                let data = export::parse_sparkline(&widget.text);
                frame.render_widget(
                    Sparkline::default().block(block).data(&data).style(style),
                    area,
                );
            }
        }
    }
}

fn draw_code(frame: &mut Frame<'_>, app: &App) {
    let source = match app.workspace {
        Workspace::Artwork => export::export_art(&app.project),
        Workspace::Components => export::export_component(&app.project),
    };
    frame.render_widget(
        Paragraph::new(source)
            .block(Block::default().borders(Borders::ALL).title(" Live Rust export "))
            .style(Style::default().fg(Color::Gray))
            .wrap(Wrap { trim: false }),
        app.regions.code,
    );
}

fn draw_timeline(frame: &mut Frame<'_>, app: &App) {
    let mut spans = Vec::new();
    for (index, animation_frame) in app.project.frames.iter().enumerate() {
        let label = format!(
            " {} · {}ms · {} ",
            index + 1,
            animation_frame.duration_ms,
            animation_frame.widget_state.label()
        );
        let style = if index == app.project.active_frame {
            Style::default()
                .fg(Color::Black)
                .bg(Color::LightCyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        spans.push(Span::styled(label, style));
        spans.push(Span::raw(" "));
    }
    let playback = if app.is_playing() { "▶" } else { "■" };
    frame.render_widget(
        Paragraph::new(Line::from(spans)).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Timeline {playback} ")),
        ),
        app.regions.timeline,
    );
}

fn draw_footer(frame: &mut Frame<'_>, app: &App) {
    let content = if app.mode == Mode::Command {
        Line::from(vec![
            Span::styled(":", Style::default().fg(Color::LightCyan)),
            Span::raw(app.command.as_str()),
        ])
    } else {
        Line::from(vec![
            Span::styled(
                format!(" {} ", app.mode.label()),
                Style::default()
                    .fg(Color::Black)
                    .bg(mode_color(app.mode))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(app.status.as_str(), Style::default().fg(Color::Gray)),
            Span::raw("  │  Tab workspace  ? help  Ctrl-S save  Ctrl-Q quit"),
        ])
    };
    frame.render_widget(Paragraph::new(content), app.regions.footer);
}

fn draw_help(frame: &mut Frame<'_>, app: &App) {
    let area = centered_rect(76, 80, frame.area());
    frame.render_widget(Clear, area);
    let help = vec![
        Line::styled(
            "Ratatelier command map",
            Style::default()
                .fg(Color::LightCyan)
                .add_modifier(Modifier::BOLD),
        ),
        Line::from(""),
        Line::from("Global: Tab workspace · Ctrl-S save · Ctrl-Q quit · : command · ? close help"),
        Line::from(""),
        Line::styled("Artwork", Style::default().fg(Color::LightGreen)),
        Line::from("h/j/k/l move · d draw · e erase · i insert · v select · u undo · Ctrl-R redo"),
        Line::from("1 pencil · 2 eraser · 3 line · 4 rectangle · 5 fill · Space apply/anchor"),
        Line::from("Mouse: left paints/drags shapes · right picks · wheel cycles glyphs"),
        Line::from("n add frame · N duplicate · X delete · ,/. switch · p playback · s widget state"),
        Line::from("a add layer · A delete layer · m ASCII/Unicode · c/C colors · b bold"),
        Line::from(""),
        Line::styled("Components", Style::default().fg(Color::LightGreen)),
        Line::from("h/j/k/l move · H/J/K/L resize · [/] select · a add · x delete · t type"),
        Line::from("Left drag moves · right drag resizes · c changes current-state color"),
        Line::from(":title TITLE · :text TEXT · :value 0..100 · :widget add KIND"),
        Line::from(""),
        Line::styled("Files and export", Style::default().fg(Color::LightGreen)),
        Line::from(":w [path] · :open path · :new 48x18 · :resize 80x24 · :scene 80x24"),
        Line::from(":export [path] · :export art|plain|animation|component [path] · :wq · :q!"),
        Line::from(""),
        Line::styled(
            format!("Current project: {}", app.project.name),
            Style::default().fg(Color::DarkGray),
        ),
    ];
    frame.render_widget(
        Paragraph::new(help)
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Help — Esc to close ")
                    .border_style(Style::default().fg(Color::LightCyan)),
            ),
        area,
    );
}
