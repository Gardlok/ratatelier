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
        self.ensure_active_layer_visible();
        true
    }

    fn select_previous_layer(&mut self) {
        let index = {
            let canvas = self.project.canvas();
            (canvas.active_layer + 1) % canvas.layers.len()
        };
        self.select_layer_index(index);
    }

    fn select_next_layer(&mut self) {
        let index = {
            let canvas = self.project.canvas();
            (canvas.active_layer + canvas.layers.len() - 1) % canvas.layers.len()
        };
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
        let fallback = format!("Layer {}", self.project.canvas().layers.len() + 1);
        let name = name
            .filter(|name| !name.trim().is_empty())
            .map_or(fallback, |name| name.trim().to_owned());
        self.snapshot();
        self.project.canvas_mut().add_layer(name.clone());
        self.mark_dirty(format!("Layer added: {name}"));
        self.ensure_active_layer_visible();
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
        self.ensure_active_layer_visible();
    }

    fn delete_active_layer(&mut self) {
        let name = self.project.canvas().layers[self.project.canvas().active_layer]
            .name
            .clone();
        self.set_next_history_action(
            format!("Restored layer \"{name}\""),
            format!("Deleted layer \"{name}\""),
        );
        self.snapshot();
        if self.project.canvas_mut().delete_active_layer() {
            self.selection = None;
            self.mark_dirty(format!("Deleted layer \"{name}\" · u undo"));
            self.ensure_active_layer_visible();
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
        let active = self.project.canvas().active_layer;
        self.snapshot();
        self.project.canvas_mut().layers[active].name = name.to_owned();
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
        let status = self.layer_status();
        self.mark_dirty(status);
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
            self.ensure_active_layer_visible();
        } else {
            self.discard_snapshot();
            self.status = "Layer is already at that edge, or move target is invalid".to_owned();
        }
    }
}

#[cfg(test)]
mod layer_management_tests {
    use super::*;

    #[test]
    fn navigation_wraps_without_dirtying_project() {
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
    fn duplicate_preserves_cells_and_can_be_reordered() {
        let mut app = App::new(Project::new("test", 4, 4), None);
        app.project.canvas_mut().set_cell(
            Point::new(1, 1),
            Cell::painted('x', CellStyle::default()),
        );
        app.duplicate_active_layer(Some("Copy"));
        assert_eq!(app.project.canvas().layers.len(), 2);
        assert_eq!(app.project.canvas().layers[1].name, "Copy");
        assert_eq!(
            app.project
                .canvas()
                .active_cell(Point::new(1, 1))
                .unwrap()
                .glyph,
            "x"
        );
        app.move_active_layer("bottom");
        assert_eq!(app.project.canvas().active_layer, 0);
        assert_eq!(app.project.canvas().layers[0].name, "Copy");
    }

    #[test]
    fn visibility_changes_composite_without_removing_cells() {
        let mut app = App::new(Project::new("test", 4, 4), None);
        app.project.canvas_mut().set_cell(
            Point::new(0, 0),
            Cell::painted('b', CellStyle::default()),
        );
        app.add_artwork_layer(Some("Top"));
        app.project.canvas_mut().set_cell(
            Point::new(0, 0),
            Cell::painted('t', CellStyle::default()),
        );
        app.set_active_layer_visibility(false);
        assert_eq!(
            app.project
                .canvas()
                .composite_cell(Point::new(0, 0))
                .unwrap()
                .glyph,
            "b"
        );
        assert_eq!(
            app.project
                .canvas()
                .active_cell(Point::new(0, 0))
                .unwrap()
                .glyph,
            "t"
        );
    }

    #[test]
    fn deleting_and_undoing_names_the_layer() {
        let mut app = App::new(Project::new("test", 4, 4), None);
        app.add_artwork_layer(Some("Highlights"));
        app.delete_active_layer();
        assert!(app.status.contains("Highlights"));
        app.undo();
        assert!(app.status.contains("Restored layer \"Highlights\""));
    }
}
