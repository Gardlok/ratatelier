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
    let layer = &canvas.layers[canvas.active_layer];
    vec![
        labeled("Mode", app.mode.label()),
        labeled("Tool", app.tool.label()),
        labeled("Canvas", format!("{}×{}", canvas.width, canvas.height)),
        labeled("Glyph mode", canvas.mode.label()),
        labeled("Cursor", format!("{}, {}", app.cursor.x, app.cursor.y)),
        labeled("Brush", format!("[{}]", app.brush.glyph)),
        labeled("Foreground", app.brush.style.fg.label()),
        labeled("Background", app.brush.style.bg.label()),
        labeled("Layer", format!("{} ({}/{})", layer.name, canvas.active_layer + 1, canvas.layers.len())),
        Line::from(""),
        Line::styled("Helix-like controls", Style::default().fg(Color::DarkGray)),
        Line::from("h j k l   move"),
        Line::from("d/e/i/v   mode"),
        Line::from("1..5      tools"),
        Line::from("Space     apply"),
        Line::from("[ ]       glyph"),
        Line::from("c/C/b     style"),
        Line::from("n/N/X     frames"),
        Line::from("p/s       play/state"),
        Line::from(":         command"),
    ]
}

fn component_inspector(app: &App) -> Vec<Line<'static>> {
    let scene = &app.project.components;
    let mut lines = vec![
        labeled("Mode", app.mode.label()),
        labeled("Surface", format!("{}×{}", scene.width, scene.height)),
        labeled("State", app.project.frame().widget_state.label()),
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
