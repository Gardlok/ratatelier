use std::cell::Cell as InspectorCell;

thread_local! {
    static INSPECTOR_SCROLL: InspectorCell<u16> = const { InspectorCell::new(0) };
    static INSPECTOR_FOCUSED: InspectorCell<bool> = const { InspectorCell::new(false) };
}

impl App {
    pub(crate) fn inspector_scroll(&self) -> u16 {
        INSPECTOR_SCROLL.with(InspectorCell::get)
    }

    pub(crate) fn inspector_focused(&self) -> bool {
        INSPECTOR_FOCUSED.with(InspectorCell::get)
    }

    fn set_inspector_scroll(&self, value: u16) {
        INSPECTOR_SCROLL.with(|scroll| scroll.set(value));
    }

    fn set_inspector_focused(&self, focused: bool) {
        INSPECTOR_FOCUSED.with(|state| state.set(focused));
    }

    fn reset_inspector_scroll(&self) {
        self.set_inspector_scroll(0);
    }

    fn unfocus_inspector(&self) {
        self.set_inspector_focused(false);
    }

    fn scroll_inspector(&self, delta: i32) {
        let maximum = ui::inspector_scroll_limit(self, self.regions.inspector);
        let current = i32::from(self.inspector_scroll().min(maximum));
        let next = (current + delta).clamp(0, i32::from(maximum));
        self.set_inspector_scroll(u16::try_from(next).unwrap_or(maximum));
    }

    fn handle_inspector_key(&mut self, key: KeyEvent) -> bool {
        if !self.inspector_focused() {
            return false;
        }
        match key.code {
            KeyCode::Esc => {
                self.unfocus_inspector();
                self.status = "Inspector focus released".to_owned();
            }
            KeyCode::Up | KeyCode::Char('k') => self.scroll_inspector(-1),
            KeyCode::Down | KeyCode::Char('j') => self.scroll_inspector(1),
            KeyCode::PageUp => self.scroll_inspector(-8),
            KeyCode::PageDown => self.scroll_inspector(8),
            KeyCode::Home | KeyCode::Char('g') => self.reset_inspector_scroll(),
            KeyCode::End | KeyCode::Char('G') => {
                self.set_inspector_scroll(ui::inspector_scroll_limit(self, self.regions.inspector));
            }
            _ => return false,
        }
        true
    }

    fn handle_inspector_mouse(&mut self, mouse: MouseEvent) -> bool {
        if !region_contains(self.regions.inspector, mouse.column, mouse.row) {
            return false;
        }
        match mouse.kind {
            MouseEventKind::ScrollUp => self.scroll_inspector(-3),
            MouseEventKind::ScrollDown => self.scroll_inspector(3),
            MouseEventKind::Down(MouseButton::Left) => {
                self.set_inspector_focused(true);
                self.export_focused = false;
                if let Some(index) = ui::inspector_layer_at(
                    self,
                    self.regions.inspector,
                    mouse.row,
                    self.inspector_scroll(),
                ) {
                    self.select_layer_index(index);
                } else {
                    self.status =
                        "Inspector focused; j/k, arrows, PgUp/PgDn, g/G scroll; Esc returns"
                            .to_owned();
                }
            }
            _ => {}
        }
        true
    }

    fn ensure_active_layer_visible(&self) {
        if self.workspace != Workspace::Artwork {
            return;
        }
        let active = self.project.canvas().active_layer;
        let next = ui::inspector_scroll_to_reveal_layer(
            self,
            self.regions.inspector,
            self.inspector_scroll(),
            active,
        );
        self.set_inspector_scroll(next);
    }
}

#[cfg(test)]
mod inspector_interaction_tests {
    use super::*;

    #[test]
    fn focused_inspector_scrolls_without_moving_canvas_cursor() {
        let mut app = App::new(Project::new("test", 4, 4), None);
        app.regions.inspector = Rect::new(0, 0, 26, 8);
        app.set_inspector_focused(true);
        let cursor = app.cursor;
        app.handle_inspector_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(app.cursor, cursor);
        assert!(app.inspector_scroll() > 0);
        app.unfocus_inspector();
        app.reset_inspector_scroll();
    }

    #[test]
    fn clicking_a_rendered_layer_row_selects_that_layer() {
        let mut app = App::new(Project::new("test", 4, 4), None);
        app.project.canvas_mut().add_layer("Middle");
        app.project.canvas_mut().add_layer("Top");
        app.regions.inspector = Rect::new(0, 0, 26, 30);
        app.reset_inspector_scroll();
        let row = (1..app.regions.inspector.height - 1)
            .find(|row| {
                ui::inspector_layer_at(&app, app.regions.inspector, *row, 0) == Some(0)
            })
            .expect("bottom layer should have a rendered row");
        app.handle_inspector_mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 2,
            row,
            modifiers: KeyModifiers::NONE,
        });
        assert_eq!(app.project.canvas().active_layer, 0);
        assert!(app.inspector_focused());
        app.unfocus_inspector();
        app.reset_inspector_scroll();
    }
}
