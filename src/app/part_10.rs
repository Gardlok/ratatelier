impl App {
    fn select_all(&mut self) {
        let (width, height) = {
            let canvas = self.project.canvas();
            (canvas.width, canvas.height)
        };
        self.mode = Mode::Select;
        self.selection = Some(Selection {
            anchor: Point::default(),
            head: Point::new(width.saturating_sub(1), height.saturating_sub(1)),
        });
        self.shape_drag = None;
        self.selection_drag = None;
        self.status = format!(
            "Selected entire {width}×{height} canvas; y copies, x cuts, p pastes"
        );
    }
}

#[cfg(test)]
mod select_all_tests {
    use super::*;

    #[test]
    fn ctrl_a_selects_entire_canvas_without_moving_view() {
        let mut app = App::new(Project::new("test", 9, 5), None);
        app.cursor = Point::new(4, 2);
        app.viewport_origin = Point::new(2, 1);

        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Char('a'),
            KeyModifiers::CONTROL,
        )));

        assert_eq!(app.mode, Mode::Select);
        assert_eq!(
            app.selection.unwrap().bounds(),
            RectSpec::new(0, 0, 9, 5)
        );
        assert_eq!(app.cursor, Point::new(4, 2));
        assert_eq!(app.viewport_origin, Point::new(2, 1));
    }
}
