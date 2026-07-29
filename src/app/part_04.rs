impl App {
    fn execute_command(&mut self, command: &str) {
        let (name, rest) = command
            .split_once(char::is_whitespace)
            .map_or((command, ""), |(name, rest)| (name, rest.trim()));
        match name {
            "w" | "write" => self.save((!rest.is_empty()).then(|| Path::new(rest))),
            "wq" => {
                self.save((!rest.is_empty()).then(|| Path::new(rest)));
                if !self.dirty {
                    self.running = false;
                }
            }
            "q" | "quit" => self.request_quit(false),
            "q!" | "quit!" => self.request_quit(true),
            "o" | "open" => {
                if rest.is_empty() {
                    self.status = "Usage: :open path/to/project.ron".to_owned();
                } else {
                    self.open(Path::new(rest));
                }
            }
            "e" | "export" => self.export_command(rest),
            "new" => self.new_command(rest),
            "resize" => self.resize_command(rest),
            "name" => {
                if rest.is_empty() {
                    self.status = "Usage: :name Project Name".to_owned();
                } else {
                    self.snapshot();
                    self.project.name = rest.to_owned();
                    self.mark_dirty("Project renamed");
                }
            }
            "glyph" => {
                if let Some(glyph) = rest.chars().next() {
                    self.set_brush_glyph(glyph);
                } else {
                    self.status = "Usage: :glyph X".to_owned();
                }
            }
            "title" => self.set_widget_title(rest),
            "text" => self.set_widget_text(rest),
            "value" => self.set_widget_value(rest),
            "widget" => self.widget_command(rest),
            "frame" => self.frame_command(rest),
            "layer" => self.layer_command(rest),
            "scene" => self.scene_command(rest),
            "state" => self.cycle_widget_state(),
            "help" => self.show_help = true,
            "" => {}
            unknown => self.status = format!("Unknown command: {unknown}"),
        }
    }

    fn export_command(&mut self, rest: &str) {
        let mut arguments = rest.split_whitespace();
        let first = arguments.next();
        let (kind, path) = match first {
            Some("art" | "plain" | "animation" | "component") => {
                (first.unwrap(), arguments.next().map(PathBuf::from))
            }
            Some(path) => (
                match self.workspace {
                    Workspace::Artwork => "art",
                    Workspace::Components => "component",
                },
                Some(PathBuf::from(path)),
            ),
            None => (
                match self.workspace {
                    Workspace::Artwork => "art",
                    Workspace::Components => "component",
                },
                None,
            ),
        };

        let (source, extension) = match kind {
            "plain" => (export::export_plain(&self.project), "txt"),
            "animation" => (export::export_animation(&self.project), "rs"),
            "component" => (export::export_component(&self.project), "rs"),
            _ => (export::export_art(&self.project), "rs"),
        };
        let path = path.unwrap_or_else(|| {
            PathBuf::from(format!(
                "{}-{kind}.{extension}",
                safe_file_stem(&self.project.name)
            ))
        });
        match storage::save_text(&path, &source) {
            Ok(()) => self.status = format!("Exported {}", path.display()),
            Err(error) => self.status = error.to_string(),
        }
    }

    fn new_command(&mut self, rest: &str) {
        let (width, height) = parse_dimensions(rest).unwrap_or((48, 18));
        self.snapshot();
        self.project = Project::new("Untitled", width, height);
        self.current_path = None;
        self.cursor = Point::default();
        self.component_selected = 0;
        self.mark_dirty("New project created");
    }

    fn resize_command(&mut self, rest: &str) {
        let Some((width, height)) = parse_dimensions(rest) else {
            self.status = "Usage: :resize WIDTHxHEIGHT".to_owned();
            return;
        };
        self.snapshot();
        self.project.resize_all_frames(width, height);
        self.clamp_cursor();
        self.mark_dirty("All animation frames resized");
    }

    fn scene_command(&mut self, rest: &str) {
        let Some((width, height)) = parse_dimensions(rest) else {
            self.status = "Usage: :scene WIDTHxHEIGHT".to_owned();
            return;
        };
        self.snapshot();
        self.project.components.width = width.max(1);
        self.project.components.height = height.max(1);
        for index in 0..self.project.components.widgets.len() {
            self.project.components.clamp_widget(index);
        }
        self.mark_dirty("Component design surface resized");
    }

    fn widget_command(&mut self, rest: &str) {
        let mut parts = rest.split_whitespace();
        match (parts.next(), parts.next()) {
            (Some("add"), kind) => {
                let kind = parse_widget_kind(kind.unwrap_or("paragraph"));
                self.snapshot();
                self.component_selected = self.project.components.add_widget(kind);
                self.mark_dirty("Component widget added");
            }
            (Some("delete" | "del"), _) => self.delete_selected_widget(),
            _ => self.status = "Usage: :widget add paragraph|block|gauge|list|sparkline".to_owned(),
        }
    }

    fn frame_command(&mut self, rest: &str) {
        match rest {
            "add" | "new" => {
                self.snapshot();
                self.project.add_frame();
                self.mark_dirty("Frame added");
            }
            "duplicate" | "dup" => {
                self.snapshot();
                self.project.duplicate_frame();
                self.mark_dirty("Frame duplicated");
            }
            "delete" | "del" => {
                self.snapshot();
                if self.project.delete_active_frame() {
                    self.mark_dirty("Frame deleted");
                } else {
                    self.discard_snapshot();
                    self.status = "The final frame cannot be deleted".to_owned();
                }
            }
            _ => self.status = "Usage: :frame add|duplicate|delete".to_owned(),
        }
    }

    fn layer_command(&mut self, rest: &str) {
        let (action, name) = rest
            .split_once(char::is_whitespace)
            .map_or((rest, ""), |(action, name)| (action, name.trim()));
        match action {
            "add" | "new" => {
                self.snapshot();
                let name = if name.is_empty() {
                    format!("Layer {}", self.project.canvas().layers.len() + 1)
                } else {
                    name.to_owned()
                };
                self.project.canvas_mut().add_layer(name);
                self.mark_dirty("Layer added");
            }
            "delete" | "del" => {
                self.snapshot();
                if self.project.canvas_mut().delete_active_layer() {
                    self.mark_dirty("Layer deleted");
                } else {
                    self.discard_snapshot();
                    self.status = "The final layer cannot be deleted".to_owned();
                }
            }
            _ => self.status = "Usage: :layer add [name]|delete".to_owned(),
        }
    }

    fn set_widget_title(&mut self, title: &str) {
        if self.workspace != Workspace::Components {
            self.status = ":title is available in the Components workspace".to_owned();
            return;
        }
        self.snapshot();
        if let Some(widget) = self
            .project
            .components
            .widgets
            .get_mut(self.component_selected)
        {
            widget.title = title.to_owned();
            self.mark_dirty("Widget title changed");
        } else {
            self.discard_snapshot();
        }
    }

    fn set_widget_text(&mut self, text: &str) {
        if self.workspace != Workspace::Components {
            self.status = ":text is available in the Components workspace".to_owned();
            return;
        }
        self.snapshot();
        if let Some(widget) = self
            .project
            .components
            .widgets
            .get_mut(self.component_selected)
        {
            widget.text = text.replace("\\n", "\n");
            self.mark_dirty("Widget text changed");
        } else {
            self.discard_snapshot();
        }
    }

    fn set_widget_value(&mut self, value: &str) {
        let Ok(value) = value.parse::<u16>() else {
            self.status = "Usage: :value 0..100".to_owned();
            return;
        };
        self.snapshot();
        if let Some(widget) = self
            .project
            .components
            .widgets
            .get_mut(self.component_selected)
        {
            widget.value = value.min(100);
            self.mark_dirty("Widget value changed");
        } else {
            self.discard_snapshot();
        }
    }

}
