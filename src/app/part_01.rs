impl App {
    #[must_use]
    pub fn new(project: Project, current_path: Option<PathBuf>) -> Self {
        Self {
            project,
            current_path,
            workspace: Workspace::Artwork,
            mode: Mode::Normal,
            tool: Tool::Pencil,
            cursor: Point::default(),
            brush: Cell::painted(
                '#',
                CellStyle {
                    fg: ColorSpec::White,
                    ..CellStyle::default()
                },
            ),
            selection: None,
            command: String::new(),
            status: "Ready. Tab switches workspaces; ? opens help.".to_owned(),
            dirty: false,
            running: true,
            show_help: false,
            component_selected: 0,
            regions: UiRegions::default(),
            history: Vec::new(),
            future: Vec::new(),
            clipboard: None,
            shape_drag: None,
            component_drag: None,
            playing: false,
            last_tick: Instant::now(),
            elapsed_in_frame: Duration::ZERO,
        }
    }

    #[must_use]
    pub const fn is_playing(&self) -> bool {
        self.playing
    }

    #[must_use]
    pub fn shape_preview(&self) -> Vec<Point> {
        let Some(drag) = self.shape_drag else {
            return Vec::new();
        };
        match self.tool {
            Tool::Line => line_points(drag.start, drag.current),
            Tool::Rectangle => rectangle_points(drag.start, drag.current, false),
            _ => Vec::new(),
        }
    }

    pub fn tick(&mut self) {
        let now = Instant::now();
        let delta = now.saturating_duration_since(self.last_tick);
        self.last_tick = now;
        if !self.playing || self.project.frames.len() <= 1 {
            return;
        }

        self.elapsed_in_frame += delta;
        let mut duration = Duration::from_millis(self.project.frame().duration_ms.max(1));
        while self.elapsed_in_frame >= duration {
            self.elapsed_in_frame -= duration;
            self.project.next_frame();
            self.clamp_cursor();
            duration = Duration::from_millis(self.project.frame().duration_ms.max(1));
        }
    }

    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key) if matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) => {
                self.handle_key(key);
            }
            Event::Mouse(mouse) => self.handle_mouse(mouse),
            Event::Resize(_, _) | Event::FocusGained | Event::FocusLost | Event::Paste(_) => {}
            _ => {}
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('q') => {
                    self.request_quit(false);
                    return;
                }
                KeyCode::Char('s') => {
                    self.save(None);
                    return;
                }
                KeyCode::Char('r') => {
                    self.redo();
                    return;
                }
                _ => {}
            }
        }

        if self.show_help {
            if matches!(key.code, KeyCode::Esc | KeyCode::Char('?') | KeyCode::F(1)) {
                self.show_help = false;
            }
            return;
        }

        if self.mode == Mode::Command {
            self.handle_command_key(key);
            return;
        }

        match key.code {
            KeyCode::F(1) | KeyCode::Char('?') => {
                self.show_help = true;
                return;
            }
            KeyCode::Tab => {
                self.workspace = self.workspace.toggled();
                self.mode = Mode::Normal;
                self.selection = None;
                self.shape_drag = None;
                self.status = format!("{} workspace", self.workspace.label());
                return;
            }
            KeyCode::Char(':') => {
                self.mode = Mode::Command;
                self.command.clear();
                return;
            }
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.selection = None;
                self.shape_drag = None;
                self.status = "Normal mode".to_owned();
                return;
            }
            _ => {}
        }

        match self.workspace {
            Workspace::Artwork => self.handle_art_key(key),
            Workspace::Components => self.handle_component_key(key),
        }
    }

}
