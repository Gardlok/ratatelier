const MIN_FRAME_DURATION_MS: u64 = 20;
const MAX_FRAME_DURATION_MS: u64 = 60_000;

impl App {
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
        let frame_name = self.project.frame().name.clone();
        self.mark_dirty(format!("Frame added: {frame_name}"));
    }

    fn duplicate_animation_frame(&mut self, name: Option<&str>) {
        self.snapshot();
        self.project.duplicate_frame();
        if let Some(name) = name.filter(|name| !name.trim().is_empty()) {
            self.project.frame_mut().name = name.trim().to_owned();
        }
        self.selection = None;
        let frame_name = self.project.frame().name.clone();
        self.mark_dirty(format!("Frame duplicated: {frame_name}"));
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
mod frame_management_tests {
    use super::*;

    #[test]
    fn properties_and_order_follow_the_active_frame() {
        let mut app = App::new(Project::new("test", 4, 4), None);
        app.add_animation_frame(Some("Second"));
        app.set_active_frame_duration("750");
        app.set_active_frame_state("active");
        app.move_active_frame("first");
        assert_eq!(app.project.active_frame, 0);
        assert_eq!(app.project.frame().name, "Second");
        assert_eq!(app.project.frame().duration_ms, 750);
        assert_eq!(
            app.project.frame().widget_state,
            crate::model::WidgetState::Active
        );
    }

    #[test]
    fn navigation_does_not_dirty_project() {
        let mut app = App::new(Project::new("test", 4, 4), None);
        app.add_animation_frame(Some("Second"));
        app.dirty = false;
        app.select_first_frame();
        assert_eq!(app.project.active_frame, 0);
        assert!(!app.dirty);
    }

    #[test]
    fn exact_duration_rejects_values_outside_supported_range() {
        let mut app = App::new(Project::new("test", 4, 4), None);
        app.set_active_frame_duration("1");
        assert_eq!(app.project.frame().duration_ms, 120);
        app.set_active_frame_duration("60000");
        assert_eq!(app.project.frame().duration_ms, 60_000);
    }

    #[test]
    fn duplicate_retains_frame_properties() {
        let mut app = App::new(Project::new("test", 4, 4), None);
        app.rename_active_frame("Opening");
        app.set_active_frame_duration("480");
        app.set_active_frame_state("focused");
        app.duplicate_animation_frame(Some("Opening hold"));
        assert_eq!(app.project.frame().name, "Opening hold");
        assert_eq!(app.project.frame().duration_ms, 480);
        assert_eq!(
            app.project.frame().widget_state,
            crate::model::WidgetState::Focused
        );
    }
}
