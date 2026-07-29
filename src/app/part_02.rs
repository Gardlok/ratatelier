impl App {
    fn handle_art_key(&mut self, key: KeyEvent) {
        if self.mode == Mode::Insert {
            self.handle_insert_key(key);
            return;
        }

        if self.mode == Mode::Select {
            self.handle_select_key(key);
            return;
        }

        let moved = match key.code {
            KeyCode::Left | KeyCode::Char('h') => self.move_cursor(-1, 0),
            KeyCode::Down | KeyCode::Char('j') => self.move_cursor(0, 1),
            KeyCode::Up | KeyCode::Char('k') => self.move_cursor(0, -1),
            KeyCode::Right | KeyCode::Char('l') => self.move_cursor(1, 0),
            _ => false,
        };
        if moved {
            if self.mode == Mode::Draw && matches!(self.tool, Tool::Pencil | Tool::Eraser) {
                self.apply_tool_at_cursor();
            }
            return;
        }

        match key.code {
            KeyCode::Char('d') => {
                self.mode = Mode::Draw;
                self.tool = Tool::Pencil;
                self.status = "Draw mode: movement paints".to_owned();
            }
            KeyCode::Char('e') => {
                self.mode = Mode::Draw;
                self.tool = Tool::Eraser;
                self.status = "Erase mode: movement clears the active layer".to_owned();
            }
            KeyCode::Char('i') => {
                self.mode = Mode::Insert;
                self.status = "Insert mode: type directly onto the canvas".to_owned();
            }
            KeyCode::Char('v') => {
                self.mode = Mode::Select;
                self.selection = Some(Selection {
                    anchor: self.cursor,
                    head: self.cursor,
                });
                self.status = "Selection started; y copies, x cuts, p pastes".to_owned();
            }
            KeyCode::Char('u') => self.undo(),
            KeyCode::Char('1') => self.select_tool(Tool::Pencil),
            KeyCode::Char('2') => self.select_tool(Tool::Eraser),
            KeyCode::Char('3') => self.select_tool(Tool::Line),
            KeyCode::Char('4') => self.select_tool(Tool::Rectangle),
            KeyCode::Char('5') => self.select_tool(Tool::Fill),
            KeyCode::Char(' ') => self.apply_tool_at_cursor(),
            KeyCode::Char('[') => self.cycle_glyph(false),
            KeyCode::Char(']') => self.cycle_glyph(true),
            KeyCode::Char('c') => {
                self.brush.style.fg = self.brush.style.fg.next();
                self.status = format!("Foreground: {}", self.brush.style.fg.label());
            }
            KeyCode::Char('C') => {
                self.brush.style.bg = self.brush.style.bg.next();
                self.status = format!("Background: {}", self.brush.style.bg.label());
            }
            KeyCode::Char('b') => {
                self.brush.style.bold = !self.brush.style.bold;
                self.status = format!("Bold: {}", self.brush.style.bold);
            }
            KeyCode::Char('m') => self.toggle_canvas_mode(),
            KeyCode::Char('a') => {
                self.snapshot();
                let name = format!("Layer {}", self.project.canvas().layers.len() + 1);
                self.project.canvas_mut().add_layer(name);
                self.mark_dirty("Layer added");
            }
            KeyCode::Char('A') => {
                self.snapshot();
                if self.project.canvas_mut().delete_active_layer() {
                    self.mark_dirty("Layer deleted");
                } else {
                    self.discard_snapshot();
                    self.status = "The final layer cannot be deleted".to_owned();
                }
            }
            KeyCode::Char(',') => self.previous_frame(),
            KeyCode::Char('.') => self.next_frame(),
            KeyCode::Char('n') => {
                self.snapshot();
                self.project.add_frame();
                self.mark_dirty("Frame added");
            }
            KeyCode::Char('N') => {
                self.snapshot();
                self.project.duplicate_frame();
                self.mark_dirty("Frame duplicated");
            }
            KeyCode::Char('X') => {
                self.snapshot();
                if self.project.delete_active_frame() {
                    self.mark_dirty("Frame deleted");
                    self.clamp_cursor();
                } else {
                    self.discard_snapshot();
                    self.status = "The final frame cannot be deleted".to_owned();
                }
            }
            KeyCode::Char('p') => self.toggle_playback(),
            KeyCode::Char('s') => self.cycle_widget_state(),
            KeyCode::Char('+') | KeyCode::Char('=') => self.adjust_duration(20),
            KeyCode::Char('-') => self.adjust_duration(-20),
            KeyCode::Char('P') => self.paste_clipboard(),
            _ => {}
        }
    }

    fn handle_insert_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(character) => {
                if self.project.canvas().mode.accepts(character) {
                    self.snapshot();
                    let mut cell = self.brush.clone();
                    cell.glyph = character.to_string();
                    cell.transparent = false;
                    self.project.canvas_mut().set_cell(self.cursor, cell);
                    self.mark_dirty("Inserted glyph");
                    self.move_cursor(1, 0);
                } else {
                    self.status = format!(
                        "{} mode rejects that glyph or its terminal width",
                        self.project.canvas().mode.label()
                    );
                }
            }
            KeyCode::Backspace => {
                self.move_cursor(-1, 0);
                self.snapshot();
                self.project.canvas_mut().erase_cell(self.cursor);
                self.mark_dirty("Cleared cell");
            }
            KeyCode::Enter => {
                self.cursor.x = 0;
                self.move_cursor(0, 1);
            }
            KeyCode::Left => {
                self.move_cursor(-1, 0);
            }
            KeyCode::Down => {
                self.move_cursor(0, 1);
            }
            KeyCode::Up => {
                self.move_cursor(0, -1);
            }
            KeyCode::Right => {
                self.move_cursor(1, 0);
            }
            _ => {}
        }
    }

    fn handle_select_key(&mut self, key: KeyEvent) {
        let moved = match key.code {
            KeyCode::Left | KeyCode::Char('h') => self.move_cursor(-1, 0),
            KeyCode::Down | KeyCode::Char('j') => self.move_cursor(0, 1),
            KeyCode::Up | KeyCode::Char('k') => self.move_cursor(0, -1),
            KeyCode::Right | KeyCode::Char('l') => self.move_cursor(1, 0),
            _ => false,
        };
        if moved {
            if let Some(selection) = &mut self.selection {
                selection.head = self.cursor;
            }
            return;
        }

        match key.code {
            KeyCode::Char('y') => {
                self.copy_selection(false);
                self.mode = Mode::Normal;
            }
            KeyCode::Char('x') => {
                self.copy_selection(true);
                self.mode = Mode::Normal;
            }
            KeyCode::Char('p') => {
                self.paste_clipboard();
                self.mode = Mode::Normal;
            }
            _ => {}
        }
    }

    fn handle_component_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Left | KeyCode::Char('h') => self.move_widget(-1, 0),
            KeyCode::Down | KeyCode::Char('j') => self.move_widget(0, 1),
            KeyCode::Up | KeyCode::Char('k') => self.move_widget(0, -1),
            KeyCode::Right | KeyCode::Char('l') => self.move_widget(1, 0),
            KeyCode::Char('H') => self.resize_widget(-1, 0),
            KeyCode::Char('J') => self.resize_widget(0, 1),
            KeyCode::Char('K') => self.resize_widget(0, -1),
            KeyCode::Char('L') => self.resize_widget(1, 0),
            KeyCode::Char('[') => self.select_previous_widget(),
            KeyCode::Char(']') => self.select_next_widget(),
            KeyCode::Char('a') => {
                self.snapshot();
                self.component_selected = self.project.components.add_widget(WidgetKind::Paragraph);
                self.mark_dirty("Component widget added");
            }
            KeyCode::Char('x') => self.delete_selected_widget(),
            KeyCode::Char('t') => {
                self.snapshot();
                if let Some(widget) = self
                    .project
                    .components
                    .widgets
                    .get_mut(self.component_selected)
                {
                    widget.kind = widget.kind.next();
                    widget.title = widget.kind.label().to_owned();
                    self.mark_dirty("Widget type changed");
                } else {
                    self.discard_snapshot();
                }
            }
            KeyCode::Char('c') => {
                self.snapshot();
                let state = self.project.frame().widget_state;
                if let Some(widget) = self
                    .project
                    .components
                    .widgets
                    .get_mut(self.component_selected)
                {
                    let style = match state {
                        crate::model::WidgetState::Normal => &mut widget.styles.normal,
                        crate::model::WidgetState::Focused => &mut widget.styles.focused,
                        crate::model::WidgetState::Active => &mut widget.styles.active,
                        crate::model::WidgetState::Disabled => &mut widget.styles.disabled,
                    };
                    style.fg = style.fg.next();
                    self.mark_dirty("Widget state color changed");
                } else {
                    self.discard_snapshot();
                }
            }
            KeyCode::Char('u') => self.undo(),
            KeyCode::Char(',') => self.previous_frame(),
            KeyCode::Char('.') => self.next_frame(),
            KeyCode::Char('n') => {
                self.snapshot();
                self.project.add_frame();
                self.mark_dirty("State frame added");
            }
            KeyCode::Char('N') => {
                self.snapshot();
                self.project.duplicate_frame();
                self.mark_dirty("State frame duplicated");
            }
            KeyCode::Char('X') => {
                self.snapshot();
                if self.project.delete_active_frame() {
                    self.mark_dirty("State frame deleted");
                } else {
                    self.discard_snapshot();
                    self.status = "The final frame cannot be deleted".to_owned();
                }
            }
            KeyCode::Char('p') => self.toggle_playback(),
            KeyCode::Char('s') => self.cycle_widget_state(),
            KeyCode::Char('+') | KeyCode::Char('=') => self.adjust_duration(20),
            KeyCode::Char('-') => self.adjust_duration(-20),
            _ => {}
        }
    }

    fn handle_command_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.command.clear();
            }
            KeyCode::Enter => {
                let command = std::mem::take(&mut self.command);
                self.mode = Mode::Normal;
                self.execute_command(command.trim());
            }
            KeyCode::Backspace => {
                self.command.pop();
            }
            KeyCode::Char(character) => self.command.push(character),
            _ => {}
        }
    }

}
