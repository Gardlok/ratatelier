impl App {
    fn handle_mouse(&mut self, mouse: MouseEvent) {
        if self.show_help || self.mode == Mode::Command {
            return;
        }

        if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left))
            && region_contains(self.regions.toolbar, mouse.column, mouse.row)
        {
            self.toggle_export_panel();
            return;
        }

        if self.show_export && region_contains(self.regions.code, mouse.column, mouse.row) {
            self.handle_export_mouse(mouse);
            return;
        }

        if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) {
            self.export_focused = false;
        }

        match self.workspace {
            Workspace::Artwork => self.handle_art_mouse(mouse),
            Workspace::Components => self.handle_component_mouse(mouse),
        }
    }

    fn handle_art_mouse(&mut self, mouse: MouseEvent) {
        let point = self.regions.canvas_point(mouse.column, mouse.row);
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                let Some(point) = point else {
                    return;
                };
                self.cursor = point;
                if self.mode == Mode::Select {
                    self.begin_selection_drag(point);
                    return;
                }
                match self.tool {
                    Tool::Line | Tool::Rectangle => {
                        self.shape_drag = Some(ShapeDrag {
                            start: point,
                            current: point,
                        });
                    }
                    _ => self.apply_tool_at_cursor(),
                }
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                let Some(point) = point else {
                    return;
                };
                self.cursor = point;
                if self.mode == Mode::Select {
                    self.update_selection_drag(point);
                    return;
                }
                if let Some(drag) = &mut self.shape_drag {
                    drag.current = point;
                } else if matches!(self.tool, Tool::Pencil | Tool::Eraser) {
                    self.apply_tool_at_cursor();
                }
            }
            MouseEventKind::Up(MouseButton::Left) => {
                if self.mode == Mode::Select {
                    if let Some(point) = point {
                        self.cursor = point;
                        self.update_selection_drag(point);
                    }
                    self.finish_selection_drag();
                    return;
                }
                if let Some(point) = point {
                    self.cursor = point;
                    if let Some(mut drag) = self.shape_drag.take() {
                        drag.current = point;
                        self.commit_shape(drag);
                    }
                }
            }
            MouseEventKind::Down(MouseButton::Right) => {
                if point.is_some() {
                    self.pan_drag = Some(PanDrag {
                        start_column: mouse.column,
                        start_row: mouse.row,
                        origin: self.regions.viewport_origin,
                        moved: false,
                    });
                }
            }
            MouseEventKind::Drag(MouseButton::Right) => {
                let Some(mut drag) = self.pan_drag else {
                    return;
                };
                let dx = i32::from(mouse.column) - i32::from(drag.start_column);
                let dy = i32::from(mouse.row) - i32::from(drag.start_row);
                drag.moved |= dx != 0 || dy != 0;
                self.pan_viewport(drag.origin, -dx, -dy);
                self.pan_drag = Some(drag);
            }
            MouseEventKind::Up(MouseButton::Right) => {
                let Some(drag) = self.pan_drag.take() else {
                    return;
                };
                if !drag.moved {
                    if let Some(point) = point {
                        if let Some(cell) = self.project.canvas().composite_cell(point).cloned() {
                            if !cell.transparent {
                                self.brush = cell;
                                self.status = "Picked glyph and style".to_owned();
                            }
                        }
                    }
                }
            }
            MouseEventKind::ScrollUp => self.cycle_glyph(true),
            MouseEventKind::ScrollDown => self.cycle_glyph(false),
            _ => {}
        }
    }

    fn handle_export_mouse(&mut self, mouse: MouseEvent) {
        match mouse.kind {
            MouseEventKind::ScrollUp => self.scroll_export(0, -3),
            MouseEventKind::ScrollDown => self.scroll_export(0, 3),
            MouseEventKind::Down(MouseButton::Left) => {
                self.export_focused = true;
                if let Some(point) = self.export_text_point(mouse.column, mouse.row) {
                    self.export_drag_anchor = Some(point);
                    self.export_selection = Some(TextSelection {
                        anchor: point,
                        head: point,
                    });
                    self.status = "Live export focused; drag to select".to_owned();
                }
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                if let (Some(anchor), Some(point)) = (
                    self.export_drag_anchor,
                    self.export_text_point(mouse.column, mouse.row),
                ) {
                    self.export_selection = Some(TextSelection {
                        anchor,
                        head: point,
                    });
                }
            }
            MouseEventKind::Up(MouseButton::Left) => {
                if let (Some(anchor), Some(point)) = (
                    self.export_drag_anchor,
                    self.export_text_point(mouse.column, mouse.row),
                ) {
                    self.export_selection = Some(TextSelection {
                        anchor,
                        head: point,
                    });
                }
                self.export_drag_anchor = None;
            }
            MouseEventKind::Down(MouseButton::Right) => {
                self.export_selection = None;
                self.export_drag_anchor = None;
                self.status = "Live export selection cleared".to_owned();
            }
            _ => {}
        }
    }

    fn handle_component_mouse(&mut self, mouse: MouseEvent) {
        let point = self.regions.component_point(mouse.column, mouse.row);
        match mouse.kind {
            MouseEventKind::Down(button @ (MouseButton::Left | MouseButton::Right)) => {
                let Some(point) = point else {
                    return;
                };
                if let Some((index, widget)) = self
                    .project
                    .components
                    .widgets
                    .iter()
                    .enumerate()
                    .rev()
                    .find(|(_, widget)| widget.rect.contains(point))
                {
                    self.component_selected = index;
                    self.component_drag = Some(ComponentDrag {
                        widget: index,
                        start: point,
                        original: widget.rect,
                        resizing: button == MouseButton::Right,
                    });
                    self.snapshot();
                }
            }
            MouseEventKind::Drag(MouseButton::Left | MouseButton::Right) => {
                let Some(point) = point else {
                    return;
                };
                let Some(drag) = self.component_drag else {
                    return;
                };
                let dx = i32::from(point.x) - i32::from(drag.start.x);
                let dy = i32::from(point.y) - i32::from(drag.start.y);
                if let Some(widget) = self.project.components.widgets.get_mut(drag.widget) {
                    if drag.resizing {
                        widget.rect.width = add_signed(drag.original.width, dx).max(1);
                        widget.rect.height = add_signed(drag.original.height, dy).max(1);
                    } else {
                        widget.rect.x = add_signed(drag.original.x, dx);
                        widget.rect.y = add_signed(drag.original.y, dy);
                    }
                    self.project.components.clamp_widget(drag.widget);
                    self.dirty = true;
                    self.status = if drag.resizing {
                        "Resizing component widget".to_owned()
                    } else {
                        "Moving component widget".to_owned()
                    };
                }
            }
            MouseEventKind::Up(MouseButton::Left | MouseButton::Right) => {
                self.component_drag = None;
            }
            MouseEventKind::ScrollUp => self.select_previous_widget(),
            MouseEventKind::ScrollDown => self.select_next_widget(),
            _ => {}
        }
    }
}