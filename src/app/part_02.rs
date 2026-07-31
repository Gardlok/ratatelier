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
                self.status = "Selection started; y yanks, x cuts, p pastes".to_owned();
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
            KeyCode::Char('c') => self.activate_color_palette(ColorTarget::Foreground),
            KeyCode::Char('C') => self.activate_color_palette(ColorTarget::Background),
            KeyCode::Char('b') => {
                self.brush.style.bold = !self.brush.style.bold;
                self.status = format!("Bold: {}", self.brush.style.bold);
            }
            KeyCode::Char('m') => self.toggle_canvas_mode(),
            KeyCode::Char('a') => self.add_artwork_layer(None),
            KeyCode::Char('A') => self.delete_active_layer(),
            KeyCode::Char('D') => self.duplicate_active_layer(None),
            KeyCode::Char('V') => self.toggle_active_layer_visibility(),
            KeyCode::PageUp | KeyCode::Char('{') => self.select_previous_layer(),
            KeyCode::PageDown | KeyCode::Char('}') => self.select_next_layer(),
            KeyCode::Char(',') => self.previous_frame(),
            KeyCode::Char('.') => self.next_frame(),
            KeyCode::Home => self.select_first_frame(),
            KeyCode::End => self.select_last_frame(),
            KeyCode::Char('<') => self.move_active_frame("left"),
            KeyCode::Char('>') => self.move_active_frame("right"),
            KeyCode::Char('n') => self.add_animation_frame(None),
            KeyCode::Char('N') => self.duplicate_animation_frame(None),
            KeyCode::Char('X') => self.delete_animation_frame(),
            KeyCode::Char('p') => self.toggle_playback(),
            KeyCode::Char('s') => self.cycle_widget_state(),
            KeyCode::Char('+') | KeyCode::Char('=') => self.adjust_duration(20),
            KeyCode::Char('-') => self.adjust_duration(-20),
            KeyCode::Char('P') => self.paste_from_system_clipboard(),
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
                self.paste_from_system_clipboard();
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
            KeyCode::Char('c') => self.activate_color_palette(ColorTarget::Foreground),
            KeyCode::Char('C') => self.activate_color_palette(ColorTarget::Background),
            KeyCode::Char('u') => self.undo(),
            KeyCode::Char(',') => self.previous_frame(),
            KeyCode::Char('.') => self.next_frame(),
            KeyCode::Home => self.select_first_frame(),
            KeyCode::End => self.select_last_frame(),
            KeyCode::Char('<') => self.move_active_frame("left"),
            KeyCode::Char('>') => self.move_active_frame("right"),
            KeyCode::Char('n') => self.add_animation_frame(None),
            KeyCode::Char('N') => self.duplicate_animation_frame(None),
            KeyCode::Char('X') => self.delete_animation_frame(),
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
                self.reset_command_completion();
            }
            KeyCode::Enter => {
                let command = std::mem::take(&mut self.command);
                self.mode = Mode::Normal;
                self.reset_command_completion();
                self.execute_command(command.trim());
            }
            KeyCode::Backspace => {
                self.command.pop();
                self.reset_command_completion();
            }
            KeyCode::Tab => {
                if !self.complete_palette_command(false) {
                    self.complete_command(false);
                }
            }
            KeyCode::BackTab => {
                if !self.complete_palette_command(true) {
                    self.complete_command(true);
                }
            }
            KeyCode::Char(character) => {
                self.command.push(character);
                self.reset_command_completion();
            }
            _ => {}
        }
    }
}
