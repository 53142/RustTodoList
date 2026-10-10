use chrono::{DateTime, Utc};
use core::fmt;
use ratatui::widgets::{ListState, ScrollbarState};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TodoListPriority {
    Low,
    Medium,
    High,
}

impl fmt::Display for TodoListPriority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::High => write!(f, "High"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TodoListItemStatus {
    NotStarted,
    InProgress,
    Complete,
}

impl fmt::Display for TodoListItemStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotStarted => write!(f, "Not Started"),
            Self::InProgress => write!(f, "In Progress"),
            Self::Complete => write!(f, "Complete"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct TodoListItem {
    pub(crate) item_name: String,
    pub(crate) priority: TodoListPriority,
    pub(crate) due_date: DateTime<Utc>,
    pub(crate) status: TodoListItemStatus,
}

#[derive(PartialEq, Eq)]
pub(crate) enum InputMode {
    Normal,
    Editing,
    ModifyingItem,
}

#[derive(PartialEq, Eq)]
pub(crate) enum ModifyingMode {
    NewItem,
    ExistingItem,
}

pub(crate) struct App {
    pub(crate) path: PathBuf,
    pub(crate) todo_list_items: Vec<TodoListItem>,
    pub(crate) text_input: String,
    pub(crate) cursor_pos: usize,
    pub(crate) input_mode: InputMode,
    pub(crate) modifying_mode: ModifyingMode,
    pub(crate) list_state: ListState,
    pub(crate) scroll_state: ScrollbarState,
    pub(crate) priority: TodoListPriority,
    pub(crate) status: TodoListItemStatus,
    pub(crate) due_date: DateTime<Utc>,
    pub(crate) popup_index: usize,
    pub(crate) popup_text_input: String,
}