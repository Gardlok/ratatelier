const MIN_FRAME_DURATION_MS: u64 = 20;
const MAX_FRAME_DURATION_MS: u64 = 60_000;

impl App {
    fn layer_status(&self) -> String {
        let canvas = self.project.canvas();
        let layer = &canvas.layers[canvas.active_layer];
        let visibility = if layer.visible { "visible" } else { "hidden" };
        format!(
            "Layer {} of {}: {} ({visibility})",
            canvas.active_layer + 1,
            canvas.layers.len(),
            layer.name
        )
    }

    fn select_layer_index(&mut self, index: usize) -> bool {
        if index >= self.project.canvas().layers.len() {
            return false;
        }
        self.project.canvas_mut().active_layer = index;
        self.selection = None;
        self.selection_drag = None;
        self.status = self.layer_status();
        true
    }

    fn select_previous_layer(&mut self) {
        let canvas = self.project.canvas();
        let index = (canvas.active_layer + canvas.layers.len() - 1) % canvas.layers.len();
        self.select_layer_index(index);
    }

    fn select_next_layer(&mut self) {
        let canvas = self.project.canvas();
        let index = (canvas.active_layer + 1) % canvas.layers.len();
        self.select_layer_index(index);
    }

    fn select_first_layer(&mut self) {
        self.select_layer_index(0);
    }

    fn select_last_layer(&mut self) {
        let index = self.project.canvas().layers.len().saturating_sub(1);
        self.select_layer_index(index);
    }

    fn add_artwork_layer(&mut self, name: Option<&str>) {
        self.snapshot();
        let fallback = format!("Layer {}", self.project.canvas().layers.len() + 1);
        let name = name.filter(|name| !name.trim().is_empty()).unwrap_or(&fallback);
        self.project.canvas_mut().add_layer(name.trim());
        self.mark_dirty(format!("Layer added: {}", name.trim()));
    }

    fn duplicate_active_layer(&mut self, name: Option<&str>) {
        self.snapshot();
        let duplicate_name = {
            let canvas = self.project.canvas_mut();
            let mut duplicate = canvas.layers[canvas.active_layer].clone();
            duplicate.name = name
                .filter(|name| !name.trim().is_empty())
                .map_or_else(|| format!("{} copy", duplicate.name), |name| name.trim().to_owned());
            let duplicate_name = duplicate.name.clone();
            let insert_at = canvas.active_layer + 1;
            canvas.layers.insert(insert_at, duplicate);
            canvas.active_layer = insert_at;
            duplicate_name
        };
        self.selection = None;
        self.mark_dirty(format!("Layer duplicated: {duplicate_name}"));
    }

    fn delete_active_layer(&mut self) {
        self.snapshot();
        if self.project.canvas_mut().delete_active_layer() {
            self.selection = None;
            self.mark_dirty("Layer deleted");
        } else {
            self.discard_snapshot();
            self.status = "The final layer cannot be deleted".to_owned();
        }
    }

    fn rename_active_layer(&mut self, name: &str) {
        let name = name.trim();
        if name.is_empty() {
            self.status = "Usage: :layer rename NAME".to_owned();
            return;
        }
        self.snapshot();
        self.project.canvas_mut().layers[self.project.canvas().active_layer].name = name.to_owned();
        self.mark_dirty(format!("Layer renamed: {name}"));
    }

    fn set_active_layer_visibility(&mut self, visible: bool) {
        let active = self.project.canvas().active_layer;
        if self.project.canvas().layers[active].visible == visible {
            self.status = self.layer_status();
            return;
        }
        self.snapshot();
        self.project.canvas_mut().layers[active].visible = visible;
        self.mark_dirty(self.layer_status());
    }

    fn toggle_active_layer_visibility(&mut self) {
        let active = self.project.canvas().active_layer;
        let visible = !self.project.canvas().layers[active].visible;
        self.set_active_layer_visibility(visible);
    }

    fn move_active_layer(&mut self, target: &str) {
        self.snapshot();
        let moved = {
            let canvas = self.project.canvas_mut();
            let active = canvas.active_layer;
            match target {
                "up" | "next" if active + 1 < canvas.layers.len() => {
                    canvas.layers.swap(active, active + 1);
                    canvas.active_layer += 1;
                    true
                }
                "down" | "previous" | "prev" if active > 0 => {
                    canvas.layers.swap(active, active - 1);
                    canvas.active_layer -= 1;
                    true
                }
                "top" | "last" if active + 1 < canvas.layers.len() => {
                    let layer = canvas.layers.remove(active);
                    canvas.layers.push(layer);
                    canvas.active_layer = canvas.layers.len() - 1;
                    true
                }
                "bottom" | "first" if active > 0 => {
                    let layer = canvas.layers.remove(active);
                    canvas.layers.insert(0, layer);
                    canvas.active_layer = 0;
                    true
                }
                _ => false,
            }
        };
        if moved {
            self.mark_dirty(format!("Layer moved {target}"));
        } else {
            self.discard_snapshot();
            self.status = "Layer is already at that edge, or move target is invalid".to_owned();
        }
    }

    fn frame_status(&self) -> String {
        let frame = self.project.frame();
        format!(
            "Frame {} of {}: {} · {} ms · {}",
            self.project.active_frame + 1,
            self.project.frames.len(),
            frame.name,
            frame.duration_ms,
            frame.widget_state.label()
        )
    }

    fn select_frame_index(&mut self, index: usize) -> bool {
        if index >= self.project.frames.len() {
            return false;
        }
        self.project.active_frame = index;
        self.selection = None;
        self.shape_drag = None;
        self.elapsed_in_frame = Duration::ZERO;
        self.clamp_cursor();
        self.status = self.frame_status();
        true
    }

    fn select_first_frame(&mut self) {
        self.select_frame_index(0);
    }

    fn select_last_frame(&mut self) {
        let index = self.project.frames.len().saturating_sub(1);
        self.select_frame_index(index);
    }

    fn add_animation_frame(&mut self, name: Option<&str>) {
        self.snapshot();
        self.project.add_frame();
        if let Some(name) = name.filter(|name| !name.trim().is_empty()) {
            self.project.frame_mut().name = name.trim().to_owned();
        }
        self.selection = None;
        self.mark_dirty(format!("Frame added: {}", self.project.frame().name));
    }

    fn duplicate_animation_frame(&mut self, name: Option<&str>) {
        self.snapshot();
        self.project.duplicate_frame();
        if let Some(name) = name.filter(|name| !name.trim().is_empty()) {
            self.project.frame_mut().name = name.trim().to_owned();
        }
        self.selection = None;
        self.mark_dirty(format!("Frame duplicated: {}", self.project.frame().name));
    }

    fn delete_animation_frame(&mut self) {
        self.snapshot();
        if self.project.delete_active_frame() {
            self.selection = None;
            self.clamp_cursor();
            self.mark_dirty("Frame deleted");
        } else {
            self.discard_snapshot();
            self.status = "The final frame cannot be deleted".to_owned();
        }
    }

    fn rename_active_frame(&mut self, name: &str) {
        let name = name.trim();
        if name.is_empty() {
            self.status = "Usage: :frame rename NAME".to_owned();
            return;
        }
        self.snapshot();
        self.project.frame_mut().name = name.to_owned();
        self.mark_dirty(format!("Frame renamed: {name}"));
    }

    fn set_active_frame_duration(&mut self, value: &str) {
        let Ok(duration_ms) = value.trim().parse::<u64>() else {
            self.status = format!(
                "Usage: :frame duration {MIN_FRAME_DURATION_MS}..{MAX_FRAME_DURATION_MS}"
            );
            return;
        };
        if !(MIN_FRAME_DURATION_MS..=MAX_FRAME_DURATION_MS).contains(&duration_ms) {
            self.status = format!(
                "Frame duration must be {MIN_FRAME_DURATION_MS}..{MAX_FRAME_DURATION_MS} ms"
            );
            return;
        }
        self.snapshot();
        self.project.frame_mut().duration_ms = duration_ms;
        self.elapsed_in_frame = Duration::ZERO;
        self.mark_dirty(format!("Frame duration: {duration_ms} ms"));
    }

    fn set_active_frame_state(&mut self, value: &str) {
        let state = match value.trim() {
            "normal" => crate::model::WidgetState::Normal,
            "focused" | "focus" => crate::model::WidgetState::Focused,
            "active" => crate::model::WidgetState::Active,
            "disabled" | "disable" => crate::model::WidgetState::Disabled,
            _ => {
                self.status = "Usage: :frame state normal|focused|active|disabled".to_owned();
                return;
            }
        };
        self.snapshot();
        self.project.frame_mut().widget_state = state;
        self.mark_dirty(format!("Frame widget state: {}", state.label()));
    }

    fn move_active_frame(&mut self, target: &str) {
        self.snapshot();
        let moved = {
            let active = self.project.active_frame;
            match target {
                "left" | "previous" | "prev" if active > 0 => {
                    self.project.frames.swap(active, active - 1);
                    self.project.active_frame -= 1;
                    true
                }
                "right" | "next" if active + 1 < self.project.frames.len() => {
                    self.project.frames.swap(active, active + 1);
                    self.project.active_frame += 1;
                    true
                }
                "first" | "start" if active > 0 => {
                    let frame = self.project.frames.remove(active);
                    self.project.frames.insert(0, frame);
                    self.project.active_frame = 0;
                    true
                }
                "last" | "end" if active + 1 < self.project.frames.len() => {
                    let frame = self.project.frames.remove(active);
                    self.project.frames.push(frame);
                    self.project.active_frame = self.project.frames.len() - 1;
                    true
                }
                _ => false,
            }
        };
        if moved {
            self.elapsed_in_frame = Duration::ZERO;
            self.mark_dirty(format!("Frame moved {target}"));
        } else {
            self.discard_snapshot();
            self.status = "Frame is already at that edge, or move target is invalid".to_owned();
        }
    }
}

#[cfg(test)]
mod composition_management_tests {
    use super::*;

    #[test]
    fn layer_navigation_wraps_without_dirtying_project() {
        let mut app = App::new(Project::new("test", 4, 4), None);
        app.add_artwork_layer(Some("Subject"));
        app.dirty = false;
        app.select_next_layer();
        assert_eq!(app.project.canvas().active_layer, 0);
        assert!(!app.dirty);
        app.select_previous_layer();
        assert_eq!(app.project.canvas().active_layer, 1);
    }

    #[test]
    fn duplicated_layer_preserves_cells_and_can_be_reordered() {
        let mut app = App::new(Project::new("test", 4, 4), None);
        app.project.canvas_mut().set_cell(
            Point::new(1, 1),
            Cell::painted('x', CellStyle::default()),
        );
        app.duplicate_active_layer(Some("Copy"));
        assert_eq!(app.project.canvas().layers.len(), 2);
        assert_eq!(app.project.canvas().layers[1].name, "Copy");
        assert_eq!(app.project.canvas().active_cell(Point::new(1, 1)).unwrap().glyph, "x");
        app.move_active_layer("bottom");
        assert_eq!(app.project.canvas().active_layer, 0);
        assert_eq!(app.project.canvas().layers[0].name, "Copy");
    }

    #[test]
    fn frame_properties_and_order_follow_the_active_frame() {
        let mut app = App::new(Project::new("test", 4, 4), None);
        app.add_animation_frame(Some("Second"));
        app.set_active_frame_duration("750");
        app.set_active_frame_state("active");
        app.move_active_frame("first");
        assert_eq!(app.project.active_frame, 0);
        assert_eq!(app.project.frame().name, "Second");
        assert_eq!(app.project.frame().duration_ms, 750);
        assert_eq!(app.project.frame().widget_state, crate::model::WidgetState::Active);
    }

    #[test]
    fn frame_navigation_does_not_dirty_project() {
        let mut app = App::new(Project::new("test", 4, 4), None);
        app.add_animation_frame(Some("Second"));
        app.dirty = false;
        app.select_first_frame();
        assert_eq!(app.project.active_frame, 0);
        assert!(!app.dirty);
    }
}
