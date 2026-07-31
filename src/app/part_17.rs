use std::cell::{Cell as SafetyCell, RefCell};

const COMMAND_HISTORY_LIMIT: usize = 100;

#[derive(Debug, Clone)]
struct HistoryAction {
    undo_status: String,
    redo_status: String,
}

thread_local! {
    static QUIT_PROMPT_OPEN: SafetyCell<bool> = const { SafetyCell::new(false) };
    static COMMAND_HISTORY: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    static COMMAND_HISTORY_INDEX: SafetyCell<Option<usize>> = const { SafetyCell::new(None) };
    static COMMAND_HISTORY_DRAFT: RefCell<String> = const { RefCell::new(String::new()) };
    static NEXT_HISTORY_ACTION: RefCell<Option<HistoryAction>> = const { RefCell::new(None) };
    static UNDO_ACTIONS: RefCell<Vec<Option<HistoryAction>>> = const { RefCell::new(Vec::new()) };
    static REDO_ACTIONS: RefCell<Vec<Option<HistoryAction>>> = const { RefCell::new(Vec::new()) };
}

fn reset_prepublish_state() {
    QUIT_PROMPT_OPEN.with(|state| state.set(false));
    COMMAND_HISTORY.with(|history| history.borrow_mut().clear());
    COMMAND_HISTORY_INDEX.with(|index| index.set(None));
    COMMAND_HISTORY_DRAFT.with(|draft| draft.borrow_mut().clear());
    NEXT_HISTORY_ACTION.with(|action| *action.borrow_mut() = None);
    UNDO_ACTIONS.with(|actions| actions.borrow_mut().clear());
    REDO_ACTIONS.with(|actions| actions.borrow_mut().clear());
}

impl App {
    pub(crate) fn quit_prompt_open(&self) -> bool {
        QUIT_PROMPT_OPEN.with(SafetyCell::get)
    }

    fn open_quit_prompt(&mut self) {
        QUIT_PROMPT_OPEN.with(|state| state.set(true));
        self.show_help = false;
        self.export_focused = false;
        self.unfocus_inspector();
        self.status = "Unsaved changes: S save · D discard · Esc cancel".to_owned();
    }

    fn close_quit_prompt(&self) {
        QUIT_PROMPT_OPEN.with(|state| state.set(false));
    }

    fn handle_quit_prompt_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('c' | 'C') => {
                self.close_quit_prompt();
                self.status = "Quit cancelled".to_owned();
            }
            KeyCode::Enter | KeyCode::Char('s' | 'S') => {
                self.save(None);
                if !self.dirty {
                    self.close_quit_prompt();
                    self.running = false;
                }
            }
            KeyCode::Char('d' | 'D') => {
                self.close_quit_prompt();
                self.running = false;
            }
            _ => {
                self.status = "Unsaved changes: S save · D discard · Esc cancel".to_owned();
            }
        }
    }

    fn begin_command_history(&self) {
        COMMAND_HISTORY_INDEX.with(|index| index.set(None));
        COMMAND_HISTORY_DRAFT.with(|draft| draft.borrow_mut().clear());
    }

    fn prepare_command_history_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Up => {
                self.navigate_command_history(true);
                true
            }
            KeyCode::Down => {
                self.navigate_command_history(false);
                true
            }
            KeyCode::Enter => {
                let command = self.command.trim().to_owned();
                self.record_command_history(&command);
                false
            }
            KeyCode::Backspace | KeyCode::Char(_) => {
                self.detach_command_history();
                false
            }
            KeyCode::Esc => {
                self.begin_command_history();
                false
            }
            _ => false,
        }
    }

    fn record_command_history(&self, command: &str) {
        if command.is_empty() {
            self.begin_command_history();
            return;
        }
        COMMAND_HISTORY.with(|history| {
            let mut history = history.borrow_mut();
            if history.last().is_none_or(|previous| previous != command) {
                history.push(command.to_owned());
            }
            if history.len() > COMMAND_HISTORY_LIMIT {
                history.remove(0);
            }
        });
        self.begin_command_history();
    }

    fn detach_command_history(&mut self) {
        let was_browsing = COMMAND_HISTORY_INDEX.with(SafetyCell::get).is_some();
        if was_browsing {
            COMMAND_HISTORY_INDEX.with(|index| index.set(None));
            COMMAND_HISTORY_DRAFT.with(|draft| *draft.borrow_mut() = self.command.clone());
            self.command_hint.clear();
        }
    }

    fn navigate_command_history(&mut self, previous: bool) {
        let history = COMMAND_HISTORY.with(|history| history.borrow().clone());
        if history.is_empty() {
            self.command_hint = "No command history yet".to_owned();
            return;
        }

        let current = COMMAND_HISTORY_INDEX.with(SafetyCell::get);
        if current.is_none() && previous {
            COMMAND_HISTORY_DRAFT.with(|draft| *draft.borrow_mut() = self.command.clone());
        }

        let next = if previous {
            Some(current.map_or(history.len() - 1, |index| index.saturating_sub(1)))
        } else {
            match current {
                Some(index) if index + 1 < history.len() => Some(index + 1),
                Some(_) => None,
                None => return,
            }
        };
        COMMAND_HISTORY_INDEX.with(|index| index.set(next));
        self.command = next.map_or_else(
            || COMMAND_HISTORY_DRAFT.with(|draft| draft.borrow().clone()),
            |index| history[index].clone(),
        );
        self.reset_command_completion();
        self.command_hint = next.map_or_else(
            || "History draft restored".to_owned(),
            |index| format!("History {}/{} · Up/Down browse", index + 1, history.len()),
        );
    }

    fn set_next_history_action(
        &self,
        undo_status: impl Into<String>,
        redo_status: impl Into<String>,
    ) {
        NEXT_HISTORY_ACTION.with(|action| {
            *action.borrow_mut() = Some(HistoryAction {
                undo_status: undo_status.into(),
                redo_status: redo_status.into(),
            });
        });
    }
}

fn take_next_history_action() -> Option<HistoryAction> {
    NEXT_HISTORY_ACTION.with(|action| action.borrow_mut().take())
}

fn push_undo_action(action: Option<HistoryAction>) {
    UNDO_ACTIONS.with(|actions| actions.borrow_mut().push(action));
    REDO_ACTIONS.with(|actions| actions.borrow_mut().clear());
}

fn discard_undo_action() {
    UNDO_ACTIONS.with(|actions| {
        actions.borrow_mut().pop();
    });
}

fn trim_undo_actions() {
    UNDO_ACTIONS.with(|actions| {
        let mut actions = actions.borrow_mut();
        if actions.len() > HISTORY_LIMIT {
            actions.remove(0);
        }
    });
}

fn move_undo_action_to_redo() -> Option<HistoryAction> {
    let action = UNDO_ACTIONS.with(|actions| actions.borrow_mut().pop().flatten());
    REDO_ACTIONS.with(|actions| actions.borrow_mut().push(action.clone()));
    action
}

fn move_redo_action_to_undo() -> Option<HistoryAction> {
    let action = REDO_ACTIONS.with(|actions| actions.borrow_mut().pop().flatten());
    UNDO_ACTIONS.with(|actions| actions.borrow_mut().push(action.clone()));
    action
}

#[cfg(test)]
mod prepublish_safety_tests {
    use super::*;

    #[test]
    fn command_history_restores_the_original_draft() {
        let mut app = App::new(Project::new("test", 4, 4), None);
        app.record_command_history("frame next");
        app.record_command_history("layer hide");
        app.command = "frame ".to_owned();
        app.navigate_command_history(true);
        assert_eq!(app.command, "layer hide");
        app.navigate_command_history(true);
        assert_eq!(app.command, "frame next");
        app.navigate_command_history(false);
        app.navigate_command_history(false);
        assert_eq!(app.command, "frame ");
    }

    #[test]
    fn dirty_quit_can_be_cancelled_or_discarded() {
        let mut app = App::new(Project::new("test", 4, 4), None);
        app.dirty = true;
        app.request_quit(false);
        assert!(app.quit_prompt_open());
        app.handle_quit_prompt_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert!(app.running);
        assert!(!app.quit_prompt_open());
        app.request_quit(false);
        app.handle_quit_prompt_key(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE));
        assert!(!app.running);
    }
}
