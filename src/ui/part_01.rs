fn draw_header(frame: &mut Frame<'_>, app: &App) {
    let tabs = Tabs::new(["Artwork", "Components"])
        .select(match app.workspace {
            Workspace::Artwork => 0,
            Workspace::Components => 1,
        })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Ratatelier — {}{} ", app.project.name, dirty_mark(app))),
        )
        .highlight_style(
            Style::default()
                .fg(Color::LightCyan)
                .add_modifier(Modifier::BOLD),
        )
        .divider(" │ ");
    frame.render_widget(tabs, app.regions.header);
}

fn draw_inspector(frame: &mut Frame<'_>, app: &App) {
    let lines = match app.workspace {
        Workspace::Artwork => artwork_inspector(app),
        Workspace::Components => component_inspector(app),
    };
    let inspector = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Inspector "))
        .wrap(Wrap { trim: false });
    frame.render_widget(inspector, app.regions.inspector);
}

fn artwork_inspector(app: &App) -> Vec<Line<'static>> {
    let canvas = app.project.canvas();
    let frame = app.project.frame();
    let mut lines = vec![
        labeled("Mode", app.mode.label()),
        labeled("Tool", app.tool.label()),
        labeled("Canvas", format!("{}×{}", canvas.width, canvas.height)),
        labeled("Cursor", format!("{}, {}", app.cursor.x, app.cursor.y)),
        labeled(
            "Frame",
            format!(
                "{} ({}/{})",
                frame.name,
                app.project.active_frame + 1,
                app.project.frames.len()
            ),
        ),
        labeled("Duration", format!("{} ms", frame.duration_ms)),
        labeled("State", frame.widget_state.label()),
        Line::from(""),
        labeled("Brush", format!("[{}]", app.brush.glyph)),
        labeled("Foreground", app.brush.style.fg.label()),
        labeled("Background", app.brush.style.bg.label()),
        Line::from(""),
        Line::styled("Layers · top to bottom", Style::default().fg(Color::DarkGray)),
    ];
    lines.extend(layer_stack_lines(app, 7));
    lines.extend([
        Line::from(""),
        Line::styled("Composition controls", Style::default().fg(Color::DarkGray)),
        Line::from("PgUp/PgDn  layer"),
        Line::from("V/D         vis/duplicate"),
        Line::from("a/A         add/delete"),
        Line::from(",/.         frame select"),
        Line::from("</>         frame reorder"),
        Line::from("+/-         duration"),
        Line::from(":frame …    properties"),
        Line::from(":layer …    management"),
    ]);
    lines
}

fn layer_stack_lines(app: &App, capacity: usize) -> Vec<Line<'static>> {
    let canvas = app.project.canvas();
    let total = canvas.layers.len();
    let capacity = capacity.max(1).min(total);
    let half = capacity / 2;
    let mut start = canvas.active_layer.saturating_sub(half);
    let end = (start + capacity).min(total);
    start = end.saturating_sub(capacity);

    let mut lines = Vec::new();
    if end < total {
        lines.push(Line::styled(
            "  … higher layers",
            Style::default().fg(Color::DarkGray),
        ));
    }
    for index in (start..end).rev() {
        let layer = &canvas.layers[index];
        let active = if index == canvas.active_layer { ">" } else { " " };
        let visible = if layer.visible { "[x]" } else { "[ ]" };
        let style = if index == canvas.active_layer {
            Style::default()
                .fg(Color::LightCyan)
                .add_modifier(Modifier::BOLD)
        } else if layer.visible {
            Style::default().fg(Color::Gray)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        lines.push(Line::styled(
            format!("{active} {visible} {:>2} {}", index + 1, layer.name),
            style,
        ));
    }
    if start > 0 {
        lines.push(Line::styled(
            "  … lower layers",
            Style::default().fg(Color::DarkGray),
        ));
    }
    lines
}

fn component_inspector(app: &App) -> Vec<Line<'static>> {
    let scene = &app.project.components;
    let frame = app.project.frame();
    let mut lines = vec![
        labeled("Mode", app.mode.label()),
        labeled("Surface", format!("{}×{}", scene.width, scene.height)),
        labeled(
            "Frame",
            format!(
                "{} ({}/{})",
                frame.name,
                app.project.active_frame + 1,
                app.project.frames.len()
            ),
        ),
        labeled("Duration", format!("{} ms", frame.duration_ms)),
        labeled("State", frame.widget_state.label()),
        labeled("Widgets", scene.widgets.len().to_string()),
    ];
    if let Some(widget) = scene.widgets.get(app.component_selected) {
        lines.extend([
            labeled("Selected", format!("#{} {}", widget.id, widget.kind.label())),
            labeled(
                "Rect",
                format!(
                    "{},{} {}×{}",
                    widget.rect.x, widget.rect.y, widget.rect.width, widget.rect.height
                ),
            ),
            labeled("Title", widget.title.clone()),
            labeled("Value", widget.value.to_string()),
        ]);
    }
    lines.extend([
        Line::from(""),
        Line::styled("Component controls", Style::default().fg(Color::DarkGray)),
        Line::from("h j k l   move"),
        Line::from("H J K L   resize"),
        Line::from("[ ]       select"),
        Line::from("a/x/t     add/del/type"),
        Line::from("c/s       color/state"),
        Line::from(",/.       frame select"),
        Line::from("</>       frame reorder"),
        Line::from("+/-       duration"),
        Line::from("LMB drag  move"),
        Line::from("RMB drag  resize"),
        Line::from(":title …  title"),
        Line::from(":text …   content"),
        Line::from(":value N  gauge"),
    ]);
    lines
}

fn draw_artwork(frame: &mut Frame<'_>, app: &App) {
    let title = format!(
        " Canvas {}×{} · {} ",
        app.project.canvas().width,
        app.project.canvas().height,
        app.project.canvas().mode.label()
    );
    frame.render_widget(
        Block::default().borders(Borders::ALL).title(title),
        app.regions.workspace,
    );
    frame.render_widget(
        CanvasView {
            app,
            origin: app.regions.viewport_origin,
        },
        app.regions.canvas_inner,
    );
}

struct CanvasView<'a> {
    app: &'a App,
    origin: Point,
}

impl Widget for CanvasView<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let preview = self.app.shape_preview();
        let selection = self.app.selection.map(|selection| selection.bounds());
        for screen_y in 0..area.height {
            for screen_x in 0..area.width {
                let point = Point::new(self.origin.x + screen_x, self.origin.y + screen_y);
                let Some(cell) = self.app.project.canvas().composite_cell(point) else {
                    continue;
                };
                let x = area.x + screen_x;
                let y = area.y + screen_y;
                let mut style = cell_style(&cell.style);
                if selection.is_some_and(|selection| selection.contains(point)) {
                    style = style.bg(Color::DarkGray);
                }
                if preview.contains(&point) {
                    style = style
                        .fg(Color::LightMagenta)
                        .add_modifier(Modifier::BOLD);
                }
                if self.app.cursor == point && !self.app.is_playing() {
                    style = style.add_modifier(Modifier::REVERSED);
                }
                buffer[(x, y)]
                    .set_symbol(if cell.transparent { " " } else { &cell.glyph })
                    .set_style(style);
            }
        }
    }
}
