impl App {
    fn save(&mut self, explicit_path: Option<&Path>) {
        let path = explicit_path
            .map(Path::to_path_buf)
            .or_else(|| self.current_path.clone())
            .unwrap_or_else(|| PathBuf::from(format!("{}.ron", safe_file_stem(&self.project.name))));
        match storage::save_project(&path, &self.project) {
            Ok(()) => {
                self.current_path = Some(path.clone());
                self.dirty = false;
                self.status = format!("Saved {}", path.display());
            }
            Err(error) => self.status = error.to_string(),
        }
    }

    fn open(&mut self, path: &Path) {
        match storage::load_project(path) {
            Ok(project) => {
                self.snapshot();
                self.project = project;
                self.current_path = Some(path.to_path_buf());
                self.cursor = Point::default();
                self.component_selected = 0;
                self.dirty = false;
                self.status = format!("Opened {}", path.display());
            }
            Err(error) => self.status = error.to_string(),
        }
    }

    fn request_quit(&mut self, force: bool) {
        if self.dirty && !force {
            self.status = "Unsaved changes. Use :w, :wq, or :q!".to_owned();
        } else {
            self.running = false;
        }
    }

    fn snapshot(&mut self) {
        self.history.push(self.project.clone());
        if self.history.len() > HISTORY_LIMIT {
            self.history.remove(0);
        }
        self.future.clear();
    }

    fn discard_snapshot(&mut self) {
        self.history.pop();
    }

    fn undo(&mut self) {
        if let Some(previous) = self.history.pop() {
            self.future.push(std::mem::replace(&mut self.project, previous));
            self.dirty = true;
            self.clamp_cursor();
            self.clamp_component_selection();
            self.status = "Undo".to_owned();
        } else {
            self.status = "Nothing to undo".to_owned();
        }
    }

    fn redo(&mut self) {
        if let Some(next) = self.future.pop() {
            self.history.push(std::mem::replace(&mut self.project, next));
            self.dirty = true;
            self.clamp_cursor();
            self.clamp_component_selection();
            self.status = "Redo".to_owned();
        } else {
            self.status = "Nothing to redo".to_owned();
        }
    }

    fn mark_dirty(&mut self, status: impl Into<String>) {
        self.dirty = true;
        self.status = status.into();
    }

    fn select_tool(&mut self, tool: Tool) {
        self.tool = tool;
        self.mode = Mode::Normal;
        self.shape_drag = None;
        self.status = format!("{} tool", tool.label());
    }

    fn move_cursor(&mut self, dx: i32, dy: i32) -> bool {
        let old = self.cursor;
        self.cursor.x = add_signed(self.cursor.x, dx).min(self.project.canvas().width - 1);
        self.cursor.y = add_signed(self.cursor.y, dy).min(self.project.canvas().height - 1);
        self.cursor != old
    }

    fn apply_tool_at_cursor(&mut self) {
        match self.tool {
            Tool::Pencil => {
                self.snapshot();
                self.project
                    .canvas_mut()
                    .set_cell(self.cursor, self.brush.clone());
                self.mark_dirty("Painted cell");
            }
            Tool::Eraser => {
                self.snapshot();
                self.project.canvas_mut().erase_cell(self.cursor);
                self.mark_dirty("Erased cell");
            }
            Tool::Fill => {
                let points = flood_region(self.project.canvas(), self.cursor);
                if points.is_empty() {
                    return;
                }
                self.snapshot();
                for point in points {
                    self.project
                        .canvas_mut()
                        .set_cell(point, self.brush.clone());
                }
                self.mark_dirty("Filled region");
            }
            Tool::Line | Tool::Rectangle => {
                if let Some(drag) = self.shape_drag.take() {
                    self.commit_shape(ShapeDrag {
                        current: self.cursor,
                        ..drag
                    });
                } else {
                    self.shape_drag = Some(ShapeDrag {
                        start: self.cursor,
                        current: self.cursor,
                    });
                    self.status = "Shape anchor set; move and press Space again".to_owned();
                }
            }
        }
    }

    fn commit_shape(&mut self, drag: ShapeDrag) {
        let points = match self.tool {
            Tool::Line => line_points(drag.start, drag.current),
            Tool::Rectangle => rectangle_points(drag.start, drag.current, false),
            _ => return,
        };
        self.snapshot();
        for point in points {
            self.project
                .canvas_mut()
                .set_cell(point, self.brush.clone());
        }
        self.mark_dirty(format!("{} committed", self.tool.label()));
    }

    fn toggle_canvas_mode(&mut self) {
        self.snapshot();
        let canvas = self.project.canvas_mut();
        canvas.mode = match canvas.mode {
            CanvasMode::Ascii => CanvasMode::Unicode,
            CanvasMode::Unicode => CanvasMode::Ascii,
        };
        let mode = canvas.mode;
        if let Some(glyph) = self.brush.glyph.chars().next() {
            if !mode.accepts(glyph) {
                self.brush.glyph = if mode == CanvasMode::Ascii { "#" } else { "█" }.to_owned();
            }
        }
        self.mark_dirty(format!("Canvas mode: {}", mode.label()));
    }

    fn cycle_glyph(&mut self, forward: bool) {
        let palette = match self.project.canvas().mode {
            CanvasMode::Ascii => ASCII_GLYPHS,
            CanvasMode::Unicode => UNICODE_GLYPHS,
        };
        let current = self.brush.glyph.chars().next().unwrap_or(palette[0]);
        let index = palette.iter().position(|glyph| *glyph == current).unwrap_or(0);
        let next = if forward {
            (index + 1) % palette.len()
        } else {
            (index + palette.len() - 1) % palette.len()
        };
        self.brush.glyph = palette[next].to_string();
        self.brush.transparent = false;
        self.status = format!("Brush glyph: {}", self.brush.glyph);
    }

    fn set_brush_glyph(&mut self, glyph: char) {
        if self.project.canvas().mode.accepts(glyph) {
            self.brush.glyph = glyph.to_string();
            self.brush.transparent = false;
            self.status = format!("Brush glyph: {glyph}");
        } else {
            self.status = format!(
                "{} mode rejects that glyph or its terminal width",
                self.project.canvas().mode.label()
            );
        }
    }

}
