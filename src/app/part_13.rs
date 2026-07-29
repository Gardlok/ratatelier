use std::cell::Cell;

thread_local! {
    static HELP_SCROLL: Cell<u16> = const { Cell::new(0) };
}

impl App {
    pub(crate) fn help_scroll(&self) -> u16 {
        HELP_SCROLL.with(Cell::get)
    }

    fn set_help_scroll(&self, value: u16) {
        HELP_SCROLL.with(|scroll| scroll.set(value));
    }

    fn reset_help_scroll(&self) {
        self.set_help_scroll(0);
    }

    fn scroll_help(&self, delta: i32) {
        let maximum = ui::help_scroll_limit(self, self.regions.help);
        let current = i32::from(self.help_scroll().min(maximum));
        let next = (current + delta).clamp(0, i32::from(maximum)) as u16;
        self.set_help_scroll(next);
    }

    fn handle_help_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('?') | KeyCode::F(1) => {
                self.show_help = false;
                self.reset_help_scroll();
            }
            KeyCode::Up | KeyCode::Char('k') => self.scroll_help(-1),
            KeyCode::Down | KeyCode::Char('j') => self.scroll_help(1),
            KeyCode::PageUp => self.scroll_help(-8),
            KeyCode::PageDown => self.scroll_help(8),
            KeyCode::Home | KeyCode::Char('g') => self.reset_help_scroll(),
            KeyCode::End | KeyCode::Char('G') => {
                self.set_help_scroll(ui::help_scroll_limit(self, self.regions.help));
            }
            _ => {}
        }
    }

    fn handle_help_mouse(&self, mouse: MouseEvent) {
        match mouse.kind {
            MouseEventKind::ScrollUp => self.scroll_help(-3),
            MouseEventKind::ScrollDown => self.scroll_help(3),
            _ => {}
        }
    }
}
