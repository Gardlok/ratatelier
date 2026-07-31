impl App {
    fn copy_selection(&mut self, cut: bool) {
        let Some(selection) = self.selection.take() else {
            return;
        };
        let bounds = selection.bounds();
        let mut cells = Vec::with_capacity(usize::from(bounds.width) * usize::from(bounds.height));
        for y in 0..bounds.height {
            for x in 0..bounds.width {
                let point = Point::new(bounds.x + x, bounds.y + y);
                cells.push(
                    self.project
                        .canvas()
                        .active_cell(point)
                        .cloned()
                        .unwrap_or_default(),
                );
            }
        }
        let text = cells_to_text(bounds.width, bounds.height, &cells);
        let system_error = self
            .write_system_clipboard(text.clone())
            .err()
            .map(|error| error.to_string());
        self.clipboard = Some(Clipboard {
            width: bounds.width,
            height: bounds.height,
            cells,
            text,
        });
        if cut {
            self.snapshot();
            for y in 0..bounds.height {
                for x in 0..bounds.width {
                    self.project
                        .canvas_mut()
                        .erase_cell(Point::new(bounds.x + x, bounds.y + y));
                }
            }
            self.mark_dirty("Selection cut to clipboard");
        } else {
            self.status = "Selection yanked to clipboard".to_owned();
        }
        if let Some(error) = system_error {
            self.status = format!("{}; system clipboard unavailable: {error}", self.status);
        }
    }

    fn paste_clipboard(&mut self) -> bool {
        let Some(clipboard) = self.clipboard.clone() else {
            return false;
        };
        self.snapshot();
        for y in 0..clipboard.height {
            for x in 0..clipboard.width {
                let destination = Point::new(self.cursor.x + x, self.cursor.y + y);
                if self.project.canvas().contains(destination) {
                    let index = usize::from(y) * usize::from(clipboard.width) + usize::from(x);
                    self.project
                        .canvas_mut()
                        .set_cell(destination, clipboard.cells[index].clone());
                }
            }
        }
        self.mark_dirty("Clipboard pasted");
        true
    }

    fn move_widget(&mut self, dx: i32, dy: i32) {
        self.snapshot();
        if let Some(widget) = self
            .project
            .components
            .widgets
            .get_mut(self.component_selected)
        {
            widget.rect.x = add_signed(widget.rect.x, dx);
            widget.rect.y = add_signed(widget.rect.y, dy);
            self.project.components.clamp_widget(self.component_selected);
            self.mark_dirty("Widget moved");
        } else {
            self.discard_snapshot();
        }
    }

    fn resize_widget(&mut self, dw: i32, dh: i32) {
        self.snapshot();
        if let Some(widget) = self
            .project
            .components
            .widgets
            .get_mut(self.component_selected)
        {
            widget.rect.width = add_signed(widget.rect.width, dw).max(1);
            widget.rect.height = add_signed(widget.rect.height, dh).max(1);
            self.project.components.clamp_widget(self.component_selected);
            self.mark_dirty("Widget resized");
        } else {
            self.discard_snapshot();
        }
    }

    fn delete_selected_widget(&mut self) {
        let Some(widget) = self.project.components.widgets.get(self.component_selected) else {
            self.status = "No widget selected".to_owned();
            return;
        };
        let name = if widget.title.trim().is_empty() {
            widget.kind.label().to_owned()
        } else {
            widget.title.clone()
        };
        self.set_next_history_action(
            format!("Restored widget \"{name}\""),
            format!("Deleted widget \"{name}\""),
        );
        self.snapshot();
        self.project
            .components
            .widgets
            .remove(self.component_selected);
        self.clamp_component_selection();
        self.mark_dirty(format!("Deleted widget \"{name}\" · u undo"));
    }

    fn select_next_widget(&mut self) {
        if !self.project.components.widgets.is_empty() {
            self.component_selected =
                (self.component_selected + 1) % self.project.components.widgets.len();
        }
    }

    fn select_previous_widget(&mut self) {
        if !self.project.components.widgets.is_empty() {
            self.component_selected = (self.component_selected
                + self.project.components.widgets.len()
                - 1)
                % self.project.components.widgets.len();
        }
    }

    fn clamp_component_selection(&mut self) {
        self.component_selected = self
            .component_selected
            .min(self.project.components.widgets.len().saturating_sub(1));
    }

    fn next_frame(&mut self) {
        let index = (self.project.active_frame + 1) % self.project.frames.len();
        self.select_frame_index(index);
    }

    fn previous_frame(&mut self) {
        let index = (self.project.active_frame + self.project.frames.len() - 1)
            % self.project.frames.len();
        self.select_frame_index(index);
    }

    fn clamp_cursor(&mut self) {
        self.cursor.x = self.cursor.x.min(self.project.canvas().width.saturating_sub(1));
        self.cursor.y = self.cursor.y.min(self.project.canvas().height.saturating_sub(1));
        self.ensure_cursor_visible();
    }

    fn toggle_playback(&mut self) {
        self.playing = !self.playing;
        self.elapsed_in_frame = Duration::ZERO;
        self.last_tick = Instant::now();
        self.status = if self.playing {
            "Animation playing".to_owned()
        } else {
            "Animation paused".to_owned()
        };
    }

    fn cycle_widget_state(&mut self) {
        self.snapshot();
        let state = self.project.frame().widget_state.next();
        self.project.frame_mut().widget_state = state;
        self.mark_dirty(format!("Frame widget state: {}", state.label()));
    }

    fn adjust_duration(&mut self, delta: i64) {
        self.snapshot();
        let current = self.project.frame().duration_ms as i64;
        self.project.frame_mut().duration_ms =
            (current + delta).clamp(MIN_FRAME_DURATION_MS as i64, MAX_FRAME_DURATION_MS as i64)
                as u64;
        self.elapsed_in_frame = Duration::ZERO;
        self.mark_dirty(format!(
            "Frame duration: {} ms",
            self.project.frame().duration_ms
        ));
    }
}
