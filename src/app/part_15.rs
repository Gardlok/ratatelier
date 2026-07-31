use std::cell::Cell as PaletteStateCell;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorTarget {
    Foreground,
    Background,
}

impl ColorTarget {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Foreground => "foreground",
            Self::Background => "background",
        }
    }
}

thread_local! {
    static PALETTE_VISIBLE: PaletteStateCell<bool> = const { PaletteStateCell::new(true) };
    static PALETTE_SCROLL: PaletteStateCell<u16> = const { PaletteStateCell::new(0) };
    static COLOR_TARGET: PaletteStateCell<ColorTarget> = const { PaletteStateCell::new(ColorTarget::Foreground) };
    static COLOR_PICK_PENDING: PaletteStateCell<bool> = const { PaletteStateCell::new(false) };
    static COLOR_COMMIT_PENDING: PaletteStateCell<bool> = const { PaletteStateCell::new(false) };
}

impl App {
    pub(crate) fn palette_visible(&self) -> bool {
        PALETTE_VISIBLE.with(PaletteStateCell::get)
    }

    pub(crate) fn palette_scroll(&self) -> u16 {
        PALETTE_SCROLL.with(PaletteStateCell::get)
    }

    pub(crate) fn color_target(&self) -> ColorTarget {
        COLOR_TARGET.with(PaletteStateCell::get)
    }

    pub(crate) fn color_pick_pending(&self) -> bool {
        COLOR_PICK_PENDING.with(PaletteStateCell::get)
    }

    #[cfg(test)]
    fn color_commit_pending(&self) -> bool {
        COLOR_COMMIT_PENDING.with(PaletteStateCell::get)
    }

    pub(crate) fn current_target_color(&self) -> ColorSpec {
        match self.workspace {
            Workspace::Artwork => match self.color_target() {
                ColorTarget::Foreground => self.brush.style.fg,
                ColorTarget::Background => self.brush.style.bg,
            },
            Workspace::Components => self
                .project
                .components
                .widgets
                .get(self.component_selected)
                .map(|widget| widget.styles.for_state(self.project.frame().widget_state))
                .map_or(ColorSpec::Reset, |style| match self.color_target() {
                    ColorTarget::Foreground => style.fg,
                    ColorTarget::Background => style.bg,
                }),
        }
    }

    fn set_palette_visible(&self, visible: bool) {
        PALETTE_VISIBLE.with(|state| state.set(visible));
    }

    fn set_palette_scroll(&self, value: u16) {
        PALETTE_SCROLL.with(|scroll| scroll.set(value));
    }

    fn set_color_target(&self, target: ColorTarget) {
        COLOR_TARGET.with(|state| state.set(target));
    }

    fn set_color_pick_pending(&self, pending: bool) {
        COLOR_PICK_PENDING.with(|state| state.set(pending));
    }

    fn set_color_commit_pending(&self, pending: bool) {
        COLOR_COMMIT_PENDING.with(|state| state.set(pending));
    }

    fn take_color_commit_pending(&self) -> bool {
        COLOR_COMMIT_PENDING.with(|state| {
            let pending = state.get();
            state.set(false);
            pending
        })
    }

    fn toggle_palette(&mut self) {
        let visible = !self.palette_visible();
        self.set_palette_visible(visible);
        self.status = if visible {
            format!("Color rail shown for {}", self.color_target().label())
        } else {
            "Color rail hidden; F3 restores it".to_owned()
        };
    }

    fn activate_color_palette(&mut self, target: ColorTarget) {
        self.set_color_target(target);
        self.set_color_pick_pending(false);
        self.set_color_commit_pending(false);
        self.set_palette_visible(true);
        self.status = format!(
            "Color rail targets {}; click a swatch, × resets, ◉ samples the canvas",
            target.label()
        );
    }

    fn cancel_color_pick(&mut self) {
        self.set_color_pick_pending(false);
        self.set_color_commit_pending(false);
        self.status = "Color picker cancelled".to_owned();
    }

    fn complete_palette_command(&mut self, reverse: bool) -> bool {
        if self.command.chars().any(char::is_whitespace)
            || !(self.command.starts_with("pal") || self.command.starts_with("colo"))
        {
            return false;
        }
        let candidates: Vec<String> = ["palette ", "colors ", "colours "]
            .into_iter()
            .filter(|candidate| candidate.starts_with(&self.command))
            .map(str::to_owned)
            .collect();
        if candidates.is_empty() {
            return false;
        }
        let can_cycle = self.command_completion.as_ref().is_some_and(|completion| {
            completion.candidates == candidates
                && completion
                    .candidates
                    .get(completion.index)
                    .is_some_and(|candidate| candidate == &self.command)
        });
        let index = if can_cycle {
            let current = self
                .command_completion
                .as_ref()
                .map_or(0, |completion| completion.index);
            if reverse {
                (current + candidates.len() - 1) % candidates.len()
            } else {
                (current + 1) % candidates.len()
            }
        } else if reverse {
            candidates.len() - 1
        } else {
            0
        };
        self.command = candidates[index].clone();
        self.command_hint = format!(
            "Completion {}/{} · Tab next · Shift-Tab previous",
            index + 1,
            candidates.len()
        );
        self.command_completion = Some(CommandCompletion { candidates, index });
        true
    }

    fn scroll_palette(&self, delta: i32) {
        let maximum = ui::palette_scroll_limit(self, self.regions.palette);
        let current = i32::from(self.palette_scroll().min(maximum));
        let next = (current + delta).clamp(0, i32::from(maximum));
        self.set_palette_scroll(u16::try_from(next).unwrap_or(maximum));
    }

    fn apply_target_color(&mut self, color: ColorSpec) {
        let target = self.color_target();
        match self.workspace {
            Workspace::Artwork => {
                match target {
                    ColorTarget::Foreground => self.brush.style.fg = color,
                    ColorTarget::Background => self.brush.style.bg = color,
                }
                self.set_color_commit_pending(true);
                self.status = format!(
                    "Brush {}: {} · Space applies once",
                    target.label(),
                    color.label()
                );
            }
            Workspace::Components => {
                let state = self.project.frame().widget_state;
                self.snapshot();
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
                    match target {
                        ColorTarget::Foreground => style.fg = color,
                        ColorTarget::Background => style.bg = color,
                    }
                    self.mark_dirty(format!(
                        "Widget {} {}: {}",
                        state.label(),
                        target.label(),
                        color.label()
                    ));
                } else {
                    self.discard_snapshot();
                    self.status = "No widget selected".to_owned();
                }
            }
        }
    }

    fn apply_palette_entry(&mut self, entry: usize) {
        match entry {
            0 => self.apply_target_color(ColorSpec::Reset),
            1 if self.workspace == Workspace::Artwork => {
                self.set_color_commit_pending(false);
                self.set_color_pick_pending(true);
                self.status = format!(
                    "Click the canvas to sample its {} color; Esc cancels",
                    self.color_target().label()
                );
            }
            1 => {
                self.status = "Canvas color sampling is available in Artwork".to_owned();
            }
            _ => {
                let Some(color) = ColorSpec::PALETTE.get(entry.saturating_sub(1)).copied() else {
                    return;
                };
                self.apply_target_color(color);
            }
        }
    }

    fn sample_canvas_color(&mut self, point: Point) -> bool {
        if !self.color_pick_pending() {
            return false;
        }
        self.set_color_pick_pending(false);
        let Some(cell) = self.project.canvas().composite_cell(point).cloned() else {
            self.status = "Nothing to sample at that canvas cell".to_owned();
            return true;
        };
        if cell.transparent {
            self.status = "Transparent canvas cell has no color to sample".to_owned();
            return true;
        }
        let color = match self.color_target() {
            ColorTarget::Foreground => cell.style.fg,
            ColorTarget::Background => cell.style.bg,
        };
        self.apply_target_color(color);
        true
    }

    fn handle_palette_mouse(&mut self, mouse: MouseEvent) -> bool {
        if !self.palette_visible()
            || !region_contains(self.regions.palette, mouse.column, mouse.row)
        {
            return false;
        }
        match mouse.kind {
            MouseEventKind::ScrollUp => self.scroll_palette(-3),
            MouseEventKind::ScrollDown => self.scroll_palette(3),
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(entry) = ui::palette_entry_at(
                    self,
                    self.regions.palette,
                    mouse.row,
                    self.palette_scroll(),
                ) {
                    self.apply_palette_entry(entry);
                }
            }
            _ => {}
        }
        true
    }

    fn handle_timeline_mouse(&mut self, mouse: MouseEvent) -> bool {
        if !region_contains(self.regions.timeline, mouse.column, mouse.row) {
            return false;
        }
        match mouse.kind {
            MouseEventKind::ScrollUp => self.previous_frame(),
            MouseEventKind::ScrollDown => self.next_frame(),
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(index) = ui::timeline_frame_at(self, self.regions.timeline, mouse.column) {
                    self.select_frame_index(index);
                }
            }
            _ => {}
        }
        true
    }
}

#[cfg(test)]
mod color_rail_tests {
    use super::*;

    #[test]
    fn palette_color_changes_the_brush_without_dirtying_the_project() {
        let mut app = App::new(Project::new("test", 4, 4), None);
        app.dirty = false;
        app.apply_palette_entry(2);
        assert_eq!(app.brush.style.fg, ColorSpec::White);
        assert!(app.color_commit_pending());
        assert!(!app.dirty);
    }

    #[test]
    fn eyedropper_samples_the_requested_canvas_channel() {
        let mut app = App::new(Project::new("test", 4, 4), None);
        app.project.canvas_mut().set_cell(
            Point::new(1, 1),
            Cell::painted(
                'x',
                CellStyle {
                    fg: ColorSpec::LightGreen,
                    bg: ColorSpec::Blue,
                    ..CellStyle::default()
                },
            ),
        );
        app.activate_color_palette(ColorTarget::Background);
        app.apply_palette_entry(1);
        assert!(app.sample_canvas_color(Point::new(1, 1)));
        assert_eq!(app.brush.style.bg, ColorSpec::Blue);
        assert!(app.color_commit_pending());
    }

    #[test]
    fn palette_command_completion_handles_both_spellings() {
        let mut app = App::new(Project::new("test", 4, 4), None);
        app.command = "pal".to_owned();
        assert!(app.complete_palette_command(false));
        assert_eq!(app.command, "palette ");
        app.command = "colo".to_owned();
        assert!(app.complete_palette_command(false));
        assert!(matches!(app.command.as_str(), "colors " | "colours "));
    }
}
