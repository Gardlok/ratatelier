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
    let source = app.live_export_source();
    let border_style = if app.export_focused {
        Style::default()
            .fg(Color::LightCyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let title = if app.export_focused {
        " Live Rust export · focused "
    } else {
        " Live Rust export "
    };
    frame.render_widget(
        Paragraph::new(export_lines(&source, app.export_selection))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(border_style),
            )
            .style(Style::default().fg(Color::Gray))
            .scroll((app.export_scroll.y, app.export_scroll.x)),
        app.regions.code,
    );
}

fn export_lines(source: &str, selection: Option<TextSelection>) -> Vec<Line<'static>> {
    source
        .lines()
        .enumerate()
        .map(|(line_index, text)| export_line(line_index, text, selection))
        .collect()
}

fn export_line(
    line_index: usize,
    text: &str,
    selection: Option<TextSelection>,
) -> Line<'static> {
    let Some(selection) = selection else {
        return Line::from(text.to_owned());
    };
    let (start, end) = selection.ordered();
    if line_index < start.line || line_index > end.line {
        return Line::from(text.to_owned());
    }

    let characters: Vec<char> = text.chars().collect();
    let start_column = if line_index == start.line {
        start.column.min(characters.len())
    } else {
        0
    };
    let end_column = if line_index == end.line {
        end.column.saturating_add(1).min(characters.len())
    } else {
        characters.len()
    };
    if start_column >= end_column {
        return Line::from(text.to_owned());
    }

    let before: String = characters[..start_column].iter().collect();
    let selected: String = characters[start_column..end_column].iter().collect();
    let after: String = characters[end_column..].iter().collect();
    Line::from(vec![
        Span::raw(before),
        Span::styled(
            selected,
            Style::default().fg(Color::Black).bg(Color::LightCyan),
        ),
        Span::raw(after),
    ])
}

fn draw_toolbar(frame: &mut Frame<'_>, app: &App) {
    let state = if app.show_export { "OPEN" } else { "closed" };
    let style = if app.show_export {
        Style::default().fg(Color::LightCyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let lines = vec![
        Line::from(""),
        Line::styled(" [F2] ", style.add_modifier(Modifier::BOLD)),
        Line::styled(" Rust ", style),
        Line::styled(format!(" {state} "), style),
        Line::from(""),
        Line::styled(" click ", Style::default().fg(Color::DarkGray)),
    ];
    frame.render_widget(
        Paragraph::new(lines)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" Tools ")),
        app.regions.toolbar,
    );
}

fn draw_timeline(frame: &mut Frame<'_>, app: &App) {
    let labels: Vec<String> = app
        .project
        .frames
        .iter()
        .enumerate()
        .map(|(index, animation_frame)| {
            format!(
                " {} · {}ms · {} ",
                index + 1,
                animation_frame.duration_ms,
                animation_frame.widget_state.label()
            )
        })
        .collect();
    let available_width = usize::from(app.regions.timeline.width.saturating_sub(2));
    let (start, end) = timeline_visible_range(
        &labels,
        app.project.active_frame,
        available_width,
    );
    let mut spans = Vec::new();
    for (index, label) in labels.iter().enumerate().take(end).skip(start) {
        let style = if index == app.project.active_frame {
            Style::default()
                .fg(Color::Black)
                .bg(Color::LightCyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        spans.push(Span::styled(label.clone(), style));
        spans.push(Span::raw(" "));
    }
    let playback = if app.is_playing() { "▶" } else { "■" };
    let active = app.project.active_frame + 1;
    let total = app.project.frames.len();
    frame.render_widget(
        Paragraph::new(Line::from(spans)).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Timeline {playback} · {active}/{total} ")),
        ),
        app.regions.timeline,
    );
}

fn timeline_visible_range(
    labels: &[String],
    active_frame: usize,
    available_width: usize,
) -> (usize, usize) {
    if labels.is_empty() {
        return (0, 0);
    }
    let active = active_frame.min(labels.len() - 1);
    let label_width = |index: usize| labels[index].chars().count().saturating_add(1);
    let mut start = active;
    let mut end = active + 1;
    let mut used = label_width(active);

    while start > 0 {
        let previous_width = label_width(start - 1);
        if used.saturating_add(previous_width) > available_width {
            break;
        }
        start -= 1;
        used = used.saturating_add(previous_width);
    }

    while end < labels.len() {
        let next_width = label_width(end);
        if used.saturating_add(next_width) > available_width {
            break;
        }
        used = used.saturating_add(next_width);
        end += 1;
    }

    (start, end)
}

fn draw_footer(frame: &mut Frame<'_>, app: &App) {
    let content = if app.mode == Mode::Command {
        vec![
            Line::from(vec![
                Span::styled(":", Style::default().fg(Color::LightCyan)),
                Span::raw(app.command.as_str()),
            ]),
            Line::styled(
                app.command_hint.as_str(),
                Style::default().fg(Color::DarkGray),
            ),
        ]
    } else {
        vec![Line::from(vec![
            Span::styled(
                format!(" {} ", app.mode.label()),
                Style::default()
                    .fg(Color::Black)
                    .bg(mode_color(app.mode))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(app.status.as_str(), Style::default().fg(Color::Gray)),
            Span::raw("  │  Tab workspace  F2 export  ? help  Ctrl-S save  Ctrl-X quit"),
        ])]
    };
    frame.render_widget(Paragraph::new(content), app.regions.footer);
}

fn draw_help(frame: &mut Frame<'_>, app: &App) {
    let area = centered_rect(78, 88, frame.area());
    frame.render_widget(Clear, area);
    let help = vec![
        Line::styled(
            "Ratatelier command map",
            Style::default()
                .fg(Color::LightCyan)
                .add_modifier(Modifier::BOLD),
        ),
        Line::from(""),
        Line::from(
            "Global: Tab workspace · F2/Ctrl-E export · Ctrl-S save · Ctrl-X quit · : command",
        ),
        Line::from(""),
        Line::styled("Artwork", Style::default().fg(Color::LightGreen)),
        Line::from("h/j/k/l move · d draw · e erase · i insert · v select · u undo · Ctrl-R redo"),
        Line::from("1 pencil · 2 eraser · 3 line · 4 rectangle · 5 fill · Space apply/anchor"),
        Line::from(
            "Selection: Ctrl-A all · y/Ctrl-C yank · x cut · p/P/Ctrl-V paste through system clipboard",
        ),
        Line::from("Ratatelier yanks retain styles · external text pastes with neutral styling"),
        Line::from("Terminal paste accepts multiline external text · unsupported glyphs are skipped"),
        Line::from("Mouse: left places/paints · drag selected cells to move · right drag pans"),
        Line::from("Right click without dragging picks a glyph · wheel cycles glyphs"),
        Line::from("n add frame · N duplicate · X delete · ,/. switch · p playback · s widget state"),
        Line::from("a add layer · A delete layer · m ASCII/Unicode · c/C colors · b bold"),
        Line::from(""),
        Line::styled("Components", Style::default().fg(Color::LightGreen)),
        Line::from("h/j/k/l move · H/J/K/L resize · [/] select · a add · x delete · t type"),
        Line::from("Left drag moves · right drag resizes · c changes current-state color"),
        Line::from(":title TITLE · :text TEXT · :value 0..100 · :widget add KIND"),
        Line::from(""),
        Line::styled("Command line", Style::default().fg(Color::LightGreen)),
        Line::from("Tab/Shift-Tab completes commands, subcommands, directories, and files"),
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
