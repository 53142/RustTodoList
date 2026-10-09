use color_eyre::Result;
use crossterm::event::{self, KeyCode};
use ratatui::{DefaultTerminal, Frame};
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span, ToText};
use ratatui::widgets::{Block, BorderType, Clear, List, ListItem, ListState, Scrollbar, ScrollbarOrientation, ScrollbarState};
use core::fmt;
use std::fs::File;
use std::path::Path;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::io::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
enum TodoListPriority {
    Low,
    Medium,
    High,
}

impl fmt::Display for TodoListPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::High => write!(f, "High"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum TodoListItemStatus {
    NotStarted,
    InProgress,
    Complete,
}

impl fmt::Display for TodoListItemStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotStarted => write!(f, "Not Started"),
            Self::InProgress => write!(f, "In Progress"),
            Self::Complete => write!(f, "Complete"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TodoListItem {
    item_name: String,
    priority: TodoListPriority,
    due_date: DateTime<Utc>,
    status: TodoListItemStatus,
}

#[derive(PartialEq)]
enum InputMode {
    Normal,
    Editing,
    ModifyingItem,
}

struct App<'a> {
    path: &'a Path,
    todo_list_items: Vec<TodoListItem>,
    text_input: String,
    cursor_pos: usize,
    input_mode: InputMode,
    list_state: ListState,
    scroll_state: ScrollbarState,
    priority: TodoListPriority,
    status: TodoListItemStatus,
    due_date: DateTime<Utc>,
    popup_index: usize, // Item selected within popup menu
    popup_text_input: String,
}

fn write_data_to_file(path: &Path, data: &[TodoListItem]) -> Result<()> {
    let mut file = File::create(path)?;
    let serialized = serde_json::to_string_pretty(data)?;
    file.write_all(serialized.as_bytes())?;
    Ok(())
}

fn read_data_from_file(path: &Path) -> Result<Vec<TodoListItem>> {
    // Create file if doesn't exist and return
    if !path.exists() {
        let mut file = File::create(path)?;
        file.write_all(b"[]")?;
        return Ok(vec![]);
    }
    let mut file = File::open(path)?;
    // Return if file empty
    if file.metadata()?.len() == 0 {
        return Ok(vec![]);
    }
    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents)?;
    let items = serde_json::from_str(&file_contents)?;
    Ok(items)
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}

impl App<'_> {
    fn new() -> Result<Self> {
        let file_path = Path::new("TodoList.json");
        let items = read_data_from_file(file_path)?;
        let mut list_state = ListState::default();
        // Select first item if there are items
        if !items.is_empty() {
            list_state.select(Some(0));
        }
        let scroll_state = ScrollbarState::new(items.len());
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
            due_date: Utc::now(),
            popup_index: 0,
            popup_text_input: String::new(),
        })
    }

    fn move_cursor_left(&mut self) {
        self.cursor_pos = self.clamp_cursor(self.cursor_pos.saturating_sub(1));
    }

    fn move_cursor_right(&mut self) {
        self.cursor_pos = self.clamp_cursor(self.cursor_pos.saturating_add(1));
    }

    fn enter_char(&mut self, new_char: char) {
        self.text_input.insert(self.byte_index(), new_char);
        self.move_cursor_right();
    }

    fn enter_char_popup(&mut self, new_char: char) {
        self.popup_text_input.insert(self.byte_index(), new_char);
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
        // Don't backspace if at beginning
        if self.cursor_pos != 0 {
            let current_index = self.cursor_pos;
            let from_left_to_current_index = current_index - 1;
            let before = self.text_input.chars().take(from_left_to_current_index);
            let after = self.text_input.chars().skip(current_index);
            self.text_input = before.chain(after).collect();
            self.move_cursor_left();
        }
    }

    fn delete_char(&mut self) {
        // Don't delete if there are no characters
        if !self.text_input.is_empty() {
            let current_index = self.cursor_pos;
            let before = self.text_input.chars().take(current_index);
            let after = self.text_input.chars().skip(current_index + 1);
            self.text_input = before.chain(after).collect();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.text_input.chars().count())
    }

    fn reset_cursor(&mut self) {
        self.cursor_pos = 0;
    }

    fn save_new_item(&mut self) -> Result<()> {
        self.todo_list_items.push(TodoListItem {
            item_name: self.text_input.clone(),
            priority: self.priority.clone(),
            due_date: self.due_date.clone(),
            status: self.status.clone(),
        });
        self.text_input.clear();
        self.reset_cursor();
        write_data_to_file(self.path, &self.todo_list_items)?;
        self.scroll_state = self.scroll_state.content_length(self.todo_list_items.len());

        if self.list_state.selected().is_none() && !self.todo_list_items.is_empty() {
            self.list_state.select(Some(0));
        }

        // Reset popup form values to defaults
        self.priority = TodoListPriority::Medium;
        self.status = TodoListItemStatus::NotStarted;
        self.due_date = Utc::now();
        self.popup_index = 0;

        Ok(())
    }

    // fn toggle_status(&mut self) -> Result<()> {
    //     if let Some(i) = self.list_state.selected() {
    //         // If item is in list
    //         if let Some(item) = self.todo_list_items.get_mut(i) {
    //             item.status = match item.status {
    //                 TodoListItemStatus::NotStarted => TodoListItemStatus::InProgress,
    //                 TodoListItemStatus::InProgress => TodoListItemStatus::Complete,
    //                 TodoListItemStatus::Complete => TodoListItemStatus::NotStarted,
    //             };
    //             write_data_to_file(self.path, &self.todo_list_items)?;
    //         }
    //     }
    //     Ok(())
    // }

    fn delete_selected_item(&mut self) -> Result<()> {
        if let Some(i) = self.list_state.selected() {
            if !self.todo_list_items.is_empty() {
                self.todo_list_items.remove(i);
                write_data_to_file(self.path, &self.todo_list_items)?;
                let new_selected = i.saturating_sub(1);
                if self.todo_list_items.is_empty() {
                    self.list_state.select(None);
                } else {
                    self.list_state.select(Some(new_selected.min(self.todo_list_items.len() - 1)));
                }
                self.scroll_state = self.scroll_state.content_length(self.todo_list_items.len());
            }
        }
        Ok(())
    }

    fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.render(frame))?;

            if let Some(key) = event::read()?.as_key_press_event() {
                match self.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break Ok(()),
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
                                let prev = i.saturating_sub(1);
                                self.list_state.select(Some(prev));
                            }
                        }
                        KeyCode::Char('i') | KeyCode::Char('a') => {
                            self.input_mode = InputMode::Editing;
                        }
                        KeyCode::Enter | KeyCode::Char(' ') => {
                            // self.toggle_status()?;
                            if let Some(i) = self.list_state.selected() {
                                // If selected item is in list
                                if let Some(item) = self.todo_list_items.get_mut(i) {
                                    self.priority = item.priority.clone();
                                    self.status = item.status.clone();
                                    self.due_date = item.due_date.clone();
                                }
                            }
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
                                self.input_mode = InputMode::ModifyingItem;
                            }
                        }
                        KeyCode::Esc => {
                            self.input_mode = InputMode::Normal;
                        }
                        KeyCode::Char(to_insert) => self.enter_char(to_insert),
                        KeyCode::Backspace => self.backspace_char(),
                        KeyCode::Delete => self.delete_char(),
                        KeyCode::Left => self.move_cursor_left(),
                        KeyCode::Right => self.move_cursor_right(),
                        _ => {}
                    },
                    InputMode::ModifyingItem => match key.code {
                        KeyCode::Esc => {
                            self.input_mode = InputMode::Editing;
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            self.popup_index = self.popup_index.saturating_sub(1);
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            self.popup_index = (self.popup_index + 1).min(2);
                        }
                        KeyCode::Left | KeyCode::Right | KeyCode::Char(' ') => {
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
                                // 2 => {
                                //     if matches!(key.code, KeyCode::Right) {
                                //         self.due_days += 1;
                                //     } else {
                                //         self.due_days = (self.due_days - 1).max(0);
                                //     }
                                // }
                                _ => {}
                            }
                        },
                        KeyCode::Char(to_insert) => self.enter_char_popup(to_insert),
                        KeyCode::Enter => {
                            self.save_new_item()?; // Save using temp values
                            self.input_mode = InputMode::Normal;
                        }
                        _ => {}
                    },
                }
            }
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        let layout = Layout::vertical([
            Constraint::Length(3),
            Constraint::Fill(1),
            Constraint::Length(3),
        ]).spacing(1);
        let [top, main, bottom] = frame.area().layout(&layout);

        let title = Line::from_iter([
            Span::from("Todo List TUI").bold().cyan(),
            Span::from(" | [i/a] Add Task | [Enter/Space] Toggle Status | [d] Delete | [q] Quit"),
        ]);
        frame.render_widget(Block::bordered().title(title), top);

        let items: Vec<ListItem> = self.todo_list_items
            .iter()
            .map(|item| {
                let status_symbol = match item.status {
                    TodoListItemStatus::NotStarted => "[ ]",
                    TodoListItemStatus::InProgress => "[*]",
                    TodoListItemStatus::Complete => "[X]",
                };
                let content = format!(
                    "{} {} | Priority: {} | Due: {}",
                    status_symbol,
                    item.item_name,
                    item.priority,
                    item.due_date.format("%Y-%m-%d %H:%M:%S"),
                );
                ListItem::new(content)
            })
            .collect();

        let list_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title("Todo List")
            .style(Style::new().blue());

        let list = List::new(items)
            .block(list_block)
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("> ");

        self.scroll_state = self.scroll_state.content_length(self.todo_list_items.len());
        self.scroll_state = self.scroll_state.position(self.list_state.selected().unwrap_or(0));

        frame.render_stateful_widget(list, main, &mut self.list_state);
        
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
        frame.render_stateful_widget(
            scrollbar,
            main.inner(Margin { vertical: 1, horizontal: 0 }),
            &mut self.scroll_state,
        );

        let input_title = match self.input_mode {
            InputMode::Normal => "New Task Input (Press 'i' or 'a' to type)",
            InputMode::Editing => "Editing New Task (Press Enter to configure options)",
            InputMode::ModifyingItem => "Configuring Task Details...",
        };
        let input_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(input_title)
            .style(match self.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Yellow),
                InputMode::ModifyingItem => Style::default().fg(Color::DarkGray),
            });

        let input_text = Line::from(self.text_input.clone());
        frame.render_widget(input_block, bottom);
        frame.render_widget(input_text, bottom.inner(Margin { vertical: 1, horizontal: 1 }));

        if matches!(self.input_mode, InputMode::Editing) {
            frame.set_cursor_position((
                bottom.x + 1 + self.cursor_pos as u16,
                bottom.y + 1,
            ));
        } else if matches!(self.input_mode, InputMode::ModifyingItem) {
            frame.set_cursor_position((
                bottom.x + 1 + self.cursor_pos as u16,
                bottom.y + 1,
            ));
        }
        if matches!(self.input_mode, InputMode::ModifyingItem) {
            let popup_area = centered_rect(60, 45, frame.area());
            frame.render_widget(Clear, popup_area); // Clears text underneath

            let popup_block = Block::bordered()
                .border_type(BorderType::Rounded)
                .title(" Configure New Task ")
                .style(Style::default().fg(Color::Green).bg(Color::Black));

           let popup_layout = Layout::vertical([
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Length(2),
            ]).margin(1);

            let popup_rects = popup_area.inner(Margin { vertical: 1, horizontal: 2 });
            let chunks = popup_layout.split(popup_rects);

            let p_style = if self.popup_index == 0 { Style::default().fg(Color::Yellow).add_modifier(Modifier::REVERSED) } else { Style::default() };
            let s_style = if self.popup_index == 1 { Style::default().fg(Color::Yellow).add_modifier(Modifier::REVERSED) } else { Style::default() };
            let d_style = if self.popup_index == 2 { Style::default().fg(Color::Yellow).add_modifier(Modifier::REVERSED) } else { Style::default() };

            let p_line = Line::from(vec![Span::raw("Priority: "), Span::styled(format!("< {} >", self.priority), p_style)]);
            let s_line = Line::from(vec![Span::raw("Status:   "), Span::styled(format!("< {} >", self.status), s_style)]);
            let d_line = Line::from(vec![Span::raw("Due:      "), Span::styled(format!("< {} >", self.due_date.format("%Y-%m-%d %H:%M:%S").to_string()), d_style)]);
            let help_line = Line::from(Span::styled(" [↑/↓] Select | [←/→] Change | [Enter] Save | [Esc] Back ", Style::default().fg(Color::DarkGray)));

            frame.render_widget(popup_block, popup_area);
            if chunks.len() >= 4 {
                frame.render_widget(p_line, chunks[0]);
                frame.render_widget(s_line, chunks[1]);
                frame.render_widget(d_line, chunks[2]);
                frame.render_widget(help_line, chunks[3]);
            }
        }
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut terminal = ratatui::init();
    let app = App::new()?;
    let result = app.run(&mut terminal);
    ratatui::restore();
    result
}