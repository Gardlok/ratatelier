impl App {
    #[must_use]
    pub fn live_export_source(&self) -> String {
        match self.workspace {
            Workspace::Artwork => export::export_art(&self.project),
            Workspace::Components => export::export_component(&self.project),
        }
    }

    fn toggle_export_panel(&mut self) {
        self.show_export = !self.show_export;
        self.export_focused = self.show_export;
        self.export_drag_anchor = None;
        if self.show_export {
            self.clamp_export_scroll();
            self.status = "Live Rust export opened".to_owned();
        } else {
            self.status = "Live Rust export tucked into the tool rail".to_owned();
        }
    }

    fn handle_export_key(&mut self, key: KeyEvent) -> bool {
        let page = i32::from(self.regions.code_inner.height.saturating_sub(1).max(1));
        match key.code {
            KeyCode::Esc => {
                self.export_focused = false;
                self.export_drag_anchor = None;
                self.status = "Live export focus released".to_owned();
            }
            KeyCode::Up | KeyCode::Char('k') => self.scroll_export(0, -1),
            KeyCode::Down | KeyCode::Char('j') => self.scroll_export(0, 1),
            KeyCode::Left | KeyCode::Char('h') => self.scroll_export(-1, 0),
            KeyCode::Right | KeyCode::Char('l') => self.scroll_export(1, 0),
            KeyCode::PageUp => self.scroll_export(0, -page),
            KeyCode::PageDown => self.scroll_export(0, page),
            KeyCode::Home | KeyCode::Char('g') => {
                self.export_scroll = Point::default();
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.export_scroll.y = u16::MAX;
                self.clamp_export_scroll();
            }
            _ => return false,
        }
        true
    }

    fn scroll_export(&mut self, dx: i32, dy: i32) {
        self.export_scroll.x = add_signed(self.export_scroll.x, dx);
        self.export_scroll.y = add_signed(self.export_scroll.y, dy);
        self.clamp_export_scroll();
    }

    fn clamp_export_scroll(&mut self) {
        let source = self.live_export_source();
        let line_count = source.lines().count().max(1);
        let line_width = source
            .lines()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0);
        let visible_height = usize::from(self.regions.code_inner.height.max(1));
        let visible_width = usize::from(self.regions.code_inner.width.max(1));
        let max_y = line_count.saturating_sub(visible_height);
        let max_x = line_width.saturating_sub(visible_width);
        self.export_scroll.y = self.export_scroll.y.min(usize_to_u16(max_y));
        self.export_scroll.x = self.export_scroll.x.min(usize_to_u16(max_x));
    }

    fn export_text_point(&self, column: u16, row: u16) -> Option<TextPoint> {
        if !region_contains(self.regions.code_inner, column, row) {
            return None;
        }
        let source = self.live_export_source();
        let line = usize::from(self.export_scroll.y)
            + usize::from(row.saturating_sub(self.regions.code_inner.y));
        let text = source.lines().nth(line)?;
        let column = usize::from(self.export_scroll.x)
            + usize::from(column.saturating_sub(self.regions.code_inner.x));
        Some(TextPoint {
            line,
            column: column.min(text.chars().count()),
        })
    }

    fn pan_viewport(&mut self, origin: Point, dx: i32, dy: i32) {
        self.viewport_origin.x = add_signed(origin.x, dx);
        self.viewport_origin.y = add_signed(origin.y, dy);
        self.clamp_viewport();
        self.status = format!(
            "Canvas viewport: {}, {}",
            self.viewport_origin.x, self.viewport_origin.y
        );
    }

    fn clamp_viewport(&mut self) {
        let canvas = self.project.canvas();
        let visible_width = self.regions.canvas_inner.width.min(canvas.width).max(1);
        let visible_height = self.regions.canvas_inner.height.min(canvas.height).max(1);
        let max_x = canvas.width.saturating_sub(visible_width);
        let max_y = canvas.height.saturating_sub(visible_height);
        self.viewport_origin.x = self.viewport_origin.x.min(max_x);
        self.viewport_origin.y = self.viewport_origin.y.min(max_y);
    }

    fn ensure_cursor_visible(&mut self) {
        let canvas = self.project.canvas();
        let visible_width = self.regions.canvas_inner.width.min(canvas.width).max(1);
        let visible_height = self.regions.canvas_inner.height.min(canvas.height).max(1);

        if self.cursor.x < self.viewport_origin.x {
            self.viewport_origin.x = self.cursor.x;
        } else if self.cursor.x >= self.viewport_origin.x.saturating_add(visible_width) {
            self.viewport_origin.x = self.cursor.x.saturating_add(1).saturating_sub(visible_width);
        }

        if self.cursor.y < self.viewport_origin.y {
            self.viewport_origin.y = self.cursor.y;
        } else if self.cursor.y >= self.viewport_origin.y.saturating_add(visible_height) {
            self.viewport_origin.y = self.cursor.y.saturating_add(1).saturating_sub(visible_height);
        }

        self.clamp_viewport();
    }

    fn begin_selection_drag(&mut self, point: Point) {
        if let Some(selection) = self.selection {
            let bounds = selection.bounds();
            if bounds.contains(point) {
                self.selection_drag = Some(SelectionDrag {
                    start: point,
                    current: point,
                    kind: SelectionDragKind::Move {
                        original: selection,
                        cells: self.capture_active_cells(bounds),
                    },
                });
                self.status = "Dragging selected cells".to_owned();
                return;
            }
        }

        let selection = Selection {
            anchor: point,
            head: point,
        };
        self.selection = Some(selection);
        self.selection_drag = Some(SelectionDrag {
            start: point,
            current: point,
            kind: SelectionDragKind::Create,
        });
        self.status = "Drag to select cells".to_owned();
    }

    fn update_selection_drag(&mut self, point: Point) {
        let Some(mut drag) = self.selection_drag.take() else {
            return;
        };
        drag.current = point;
        match &drag.kind {
            SelectionDragKind::Create => {
                self.selection = Some(Selection {
                    anchor: drag.start,
                    head: point,
                });
            }
            SelectionDragKind::Move { original, .. } => {
                let bounds = original.bounds();
                let raw_dx = i32::from(point.x) - i32::from(drag.start.x);
                let raw_dy = i32::from(point.y) - i32::from(drag.start.y);
                let min_dx = -i32::from(bounds.x);
                let min_dy = -i32::from(bounds.y);
                let max_dx = i32::from(self.project.canvas().width)
                    - i32::from(bounds.x.saturating_add(bounds.width));
                let max_dy = i32::from(self.project.canvas().height)
                    - i32::from(bounds.y.saturating_add(bounds.height));
                let dx = raw_dx.clamp(min_dx, max_dx);
                let dy = raw_dy.clamp(min_dy, max_dy);
                self.selection = Some(Selection {
                    anchor: shifted_point(original.anchor, dx, dy),
                    head: shifted_point(original.head, dx, dy),
                });
            }
        }
        self.selection_drag = Some(drag);
    }

    fn finish_selection_drag(&mut self) {
        let Some(drag) = self.selection_drag.take() else {
            return;
        };
        match drag.kind {
            SelectionDragKind::Create => {
                if let Some(selection) = self.selection {
                    let bounds = selection.bounds();
                    self.status = format!("Selected {}×{} cells", bounds.width, bounds.height);
                }
            }
            SelectionDragKind::Move { original, cells } => {
                let Some(destination) = self.selection else {
                    return;
                };
                if destination == original {
                    self.status = "Selection unchanged".to_owned();
                    return;
                }
                let source_bounds = original.bounds();
                let destination_bounds = destination.bounds();
                self.snapshot();
                for y in 0..source_bounds.height {
                    for x in 0..source_bounds.width {
                        self.project.canvas_mut().erase_cell(Point::new(
                            source_bounds.x + x,
                            source_bounds.y + y,
                        ));
                    }
                }
                for y in 0..destination_bounds.height {
                    for x in 0..destination_bounds.width {
                        let index = usize::from(y) * usize::from(destination_bounds.width)
                            + usize::from(x);
                        self.project.canvas_mut().set_cell(
                            Point::new(destination_bounds.x + x, destination_bounds.y + y),
                            cells[index].clone(),
                        );
                    }
                }
                self.mark_dirty("Selection moved");
            }
        }
    }

    fn capture_active_cells(&self, bounds: RectSpec) -> Vec<Cell> {
        let mut cells = Vec::with_capacity(usize::from(bounds.width) * usize::from(bounds.height));
        for y in 0..bounds.height {
            for x in 0..bounds.width {
                cells.push(
                    self.project
                        .canvas()
                        .active_cell(Point::new(bounds.x + x, bounds.y + y))
                        .cloned()
                        .unwrap_or_default(),
                );
            }
        }
        cells
    }
}

fn shifted_point(point: Point, dx: i32, dy: i32) -> Point {
    Point::new(add_signed(point.x, dx), add_signed(point.y, dy))
}

fn region_contains(rect: Rect, column: u16, row: u16) -> bool {
    column >= rect.x && row >= rect.y && column < rect.right() && row < rect.bottom()
}

fn usize_to_u16(value: usize) -> u16 {
    u16::try_from(value).unwrap_or(u16::MAX)
}

#[cfg(test)]
mod interaction_tests {
    use super::*;

    #[test]
    fn moving_selection_moves_active_layer_cells() {
        let mut app = App::new(Project::new("test", 8, 4), None);
        app.project.canvas_mut().set_cell(
            Point::new(1, 1),
            Cell::painted('x', CellStyle::default()),
        );
        app.mode = Mode::Select;
        app.selection = Some(Selection {
            anchor: Point::new(1, 1),
            head: Point::new(1, 1),
        });

        app.begin_selection_drag(Point::new(1, 1));
        app.update_selection_drag(Point::new(4, 2));
        app.finish_selection_drag();

        assert!(app
            .project
            .canvas()
            .active_cell(Point::new(1, 1))
            .unwrap()
            .transparent);
        assert_eq!(
            app.project
                .canvas()
                .active_cell(Point::new(4, 2))
                .unwrap()
                .glyph,
            "x"
        );
    }

    #[test]
    fn export_panel_toggle_preserves_tool_rail_behavior() {
        let mut app = App::new(Project::new("test", 8, 4), None);
        assert!(app.show_export);
        app.toggle_export_panel();
        assert!(!app.show_export);
        assert!(!app.export_focused);
        app.toggle_export_panel();
        assert!(app.show_export);
        assert!(app.export_focused);
    }
}