use chrono::{DateTime, Utc};
use color_eyre::Result;
use crossterm::event::{self, KeyCode};
use ratatui::DefaultTerminal;

use crate::models::{App, InputMode, ModifyingMode, TodoListItem, TodoListItemStatus, TodoListPriority};
use crate::storage::{read_data_from_file, write_data_to_file};

impl App {
    pub(crate) fn new() -> Result<Self> {
        let file_path = std::path::PathBuf::from("TodoList.json");
        let items = read_data_from_file(&file_path)?;
        let mut list_state = ratatui::widgets::ListState::default();

        // If items in list
        if !items.is_empty() {
            list_state.select(Some(0));
        }

        let scroll_state = ratatui::widgets::ScrollbarState::new(items.len());

        Ok(Self {
            path: file_path,
            todo_list_items: items,
            text_input: String::new(),
            cursor_pos: 0,
            input_mode: InputMode::Normal,
            list_state,
            scroll_state,
            priority: TodoListPriority::Medium,
            status: TodoListItemStatus::NotStarted,
            due_date: Utc::now() + chrono::Duration::days(1),
            popup_index: 0,
            popup_text_input: String::new(),
            modifying_mode: ModifyingMode::NewItem,
        })
    }

    fn move_cursor_left(&mut self) {
        self.cursor_pos = self.clamp_cursor(self.cursor_pos.saturating_sub(1));
    }

    fn move_cursor_right(&mut self) {
        self.cursor_pos = self.clamp_cursor(self.cursor_pos.saturating_add(1));
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.text_input.insert(index, new_char);
        self.move_cursor_right();
    }

    fn byte_index(&self) -> usize {
        self.text_input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.cursor_pos)
            .unwrap_or(self.text_input.len())
    }

    fn backspace_char(&mut self) {
        // Can only backspace if not first character
        if self.cursor_pos != 0 {
            let current_index = self.cursor_pos;
            let previous_index = current_index - 1;
            let before = self.text_input.chars().take(previous_index);
            let after = self.text_input.chars().skip(current_index);
            self.text_input = before.chain(after).collect();
            self.move_cursor_left();
        }
    }

    fn delete_char(&mut self) {
        // Only delete if there is text
        if !self.text_input.is_empty() {
            let current_index = self.cursor_pos;
            let before = self.text_input.chars().take(current_index);
            let after = self.text_input.chars().skip(current_index + 1);
            self.text_input = before.chain(after).collect();
        }
    }

    // Ensure cursor stays in text area
    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.text_input.chars().count())
    }

    fn reset_cursor(&mut self) {
        self.cursor_pos = 0;
    }

    fn save_new_item(&mut self) -> Result<()> {
        let parsed_date = DateTime::parse_from_str(
            &format!("{} +0000", self.popup_text_input.trim()),
            "%Y-%m-%d %H:%M %z",
        )
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now() + chrono::Duration::days(1)); // Default to one day in future
        
        let new_item = TodoListItem {
            item_name: self.text_input.clone(),
            priority: self.priority,
            due_date: parsed_date,
            status: self.status,
        };

        if self.modifying_mode == ModifyingMode::NewItem {
            self.todo_list_items.push(new_item);
        } else {
            if let Some(i) = self.list_state.selected() {
                self.todo_list_items[i] = new_item;
            }
        }

        self.text_input.clear();
        self.reset_cursor();

        write_data_to_file(&self.path, &self.todo_list_items)?;

        self.scroll_state = self.scroll_state.content_length(self.todo_list_items.len());

        if self.list_state.selected().is_none() && !self.todo_list_items.is_empty() {
            self.list_state.select(Some(0));
        }

        self.priority = TodoListPriority::Medium;
        self.status = TodoListItemStatus::NotStarted;
        self.due_date = Utc::now() + chrono::Duration::days(1);

        self.popup_text_input.clear();
        self.popup_index = 0;

        Ok(())
    }

    fn delete_selected_item(&mut self) -> Result<()> {
        if let Some(i) = self.list_state.selected() {
            if !self.todo_list_items.is_empty() {
                self.todo_list_items.remove(i);

                write_data_to_file(&self.path, &self.todo_list_items)?;

                let new_selected = i.saturating_sub(1);

                if self.todo_list_items.is_empty() {
                    self.list_state.select(None);
                } else {
                    self.list_state
                        .select(Some(new_selected.min(self.todo_list_items.len() - 1)));
                }

                self.scroll_state = self.scroll_state.content_length(self.todo_list_items.len());
            }
        }

        Ok(())
    }

    pub(crate) fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.render(frame))?;

            if let Some(key) = event::read()?.as_key_press_event() {
                match self.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            break Ok(());
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if !self.todo_list_items.is_empty() {
                                let i = self.list_state.selected().unwrap_or(0);

                                let next = (i + 1).min(self.todo_list_items.len() - 1);

                                self.list_state.select(Some(next));
                            }
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            if !self.todo_list_items.is_empty() {
                                let i = self.list_state.selected().unwrap_or(0);

                                let previous = i.saturating_sub(1);

                                self.list_state.select(Some(previous));
                            }
                        }
                        KeyCode::Char('a') => {
                            self.input_mode = InputMode::Editing;
                        }
                        KeyCode::Enter | KeyCode::Char(' ') => {
                            if let Some(i) = self.list_state.selected() {
                                if let Some(item) = self.todo_list_items.get(i) {
                                    self.priority = item.priority;
                                    self.status = item.status;
                                    self.due_date = item.due_date;
                                }
                            }

                            self.modifying_mode = ModifyingMode::ExistingItem;
                            self.input_mode = InputMode::ModifyingItem;
                        }
                        KeyCode::Char('d') | KeyCode::Delete => {
                            self.delete_selected_item()?;
                        }
                        _ => {}
                    },
                    InputMode::Editing => match key.code {
                        KeyCode::Enter => {
                            if !self.text_input.trim().is_empty() {
                                self.modifying_mode = ModifyingMode::NewItem;
                                self.input_mode = InputMode::ModifyingItem;
                            }
                        }
                        KeyCode::Esc => self.input_mode = InputMode::Normal,
                        KeyCode::Char(to_insert) => self.enter_char(to_insert),
                        KeyCode::Backspace => self.backspace_char(),
                        KeyCode::Delete => self.delete_char(),
                        KeyCode::Left => self.move_cursor_left(),
                        KeyCode::Right => self.move_cursor_right(),
                        _ => {}
                    },
                    InputMode::ModifyingItem => match key.code {
                        KeyCode::Esc => self.input_mode = InputMode::Editing,
                        KeyCode::Up | KeyCode::Char('k') => self.popup_index = self.popup_index.saturating_sub(1),
                        KeyCode::Down | KeyCode::Char('j') => self.popup_index = (self.popup_index + 1).min(2),
                        KeyCode::Left if self.popup_index != 2 => {
                            match self.popup_index {
                                0 => {
                                    self.priority = match self.priority {
                                        TodoListPriority::High => TodoListPriority::Medium,
                                        TodoListPriority::Medium => TodoListPriority::Low,
                                        TodoListPriority::Low => TodoListPriority::High,
                                    };
                                }
                                1 => {
                                    self.status = match self.status {
                                        TodoListItemStatus::Complete => TodoListItemStatus::InProgress,
                                        TodoListItemStatus::InProgress => TodoListItemStatus::NotStarted,
                                        TodoListItemStatus::NotStarted => TodoListItemStatus::Complete,
                                    };
                                }
                                _ => {}
                            }
                        }
                        KeyCode::Right | KeyCode::Char(' ') if self.popup_index != 2 => {
                            match self.popup_index {
                                0 => {
                                    self.priority = match self.priority {
                                        TodoListPriority::Low => TodoListPriority::Medium,
                                        TodoListPriority::Medium => TodoListPriority::High,
                                        TodoListPriority::High => TodoListPriority::Low,
                                    };
                                }
                                1 => {
                                    self.status = match self.status {
                                        TodoListItemStatus::NotStarted => TodoListItemStatus::InProgress,
                                        TodoListItemStatus::InProgress => TodoListItemStatus::Complete,
                                        TodoListItemStatus::Complete => TodoListItemStatus::NotStarted,
                                    };
                                }
                                _ => {}
                            }
                        }
                        KeyCode::Char(to_insert) if self.popup_index == 2 => self.popup_text_input.push(to_insert),
                        KeyCode::Backspace if self.popup_index == 2 => {
                            self.popup_text_input.pop();
                        }
                        KeyCode::Enter => {
                            self.save_new_item()?;
                            self.input_mode = InputMode::Normal;
                        }
                        _ => {}
                    },
                }
            }
        }
    }
}
