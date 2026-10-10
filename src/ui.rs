use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block,
    BorderType,
    Clear,
    List,
    ListItem,
    Scrollbar,
    ScrollbarOrientation,
};

use crate::models::{App, InputMode, TodoListItemStatus};

fn centered_rect(
    percent_x: u16,
    percent_y: u16,
    r: Rect,
) -> Rect {
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

impl App {
    pub(crate) fn render(&mut self, frame: &mut Frame) {
        let layout = Layout::vertical([
            Constraint::Length(3),
            Constraint::Fill(1),
            Constraint::Length(3),
        ])
        .spacing(1);

        let [top, main, bottom] = frame.area().layout(&layout);

        let title = Line::from(vec![
            Span::from("Todo List TUI").bold().cyan(),
            Span::from(
                " | [a] Add Task | [Enter] Modify Item \
                 | [d] Delete | [q] Quit",
            ),
        ]);

        frame.render_widget(
            Block::bordered().title(title),
            top,
        );

        let items: Vec<ListItem> = self
            .todo_list_items
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
                    item.due_date.format("%Y-%m-%d %H:%M"),
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
            .highlight_style(
                Style::default().add_modifier(Modifier::REVERSED),
            )
            .highlight_symbol("> ");

        self.scroll_state = self
            .scroll_state
            .content_length(self.todo_list_items.len());

        self.scroll_state = self
            .scroll_state
            .position(self.list_state.selected().unwrap_or(0));

        frame.render_stateful_widget(
            list,
            main,
            &mut self.list_state,
        );

        let scrollbar =
            Scrollbar::new(ScrollbarOrientation::VerticalRight);

        frame.render_stateful_widget(
            scrollbar,
            main.inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut self.scroll_state,
        );

        let input_title = match self.input_mode {
            InputMode::Normal => "New Task Input (Press 'a' to type)",
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

        frame.render_widget(
            input_text,
            bottom.inner(Margin {
                vertical: 1,
                horizontal: 1,
            }),
        );

        if matches!(self.input_mode, InputMode::Editing) {
            frame.set_cursor_position((
                bottom.x + 1 + self.cursor_pos as u16,
                bottom.y + 1,
            ));
        }

        if matches!(self.input_mode, InputMode::ModifyingItem) {
            let popup_area = centered_rect(65, 45, frame.area());

            frame.render_widget(Clear, popup_area);

            let popup_block = Block::bordered()
                .border_type(BorderType::Rounded)
                .title(" Configure New Task ")
                .style(
                    Style::default()
                        .fg(Color::Green)
                        .bg(Color::Black),
                );

            let popup_layout = Layout::vertical([
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Length(2),
            ])
            .margin(1);

            let popup_rects = popup_area.inner(Margin {
                vertical: 1,
                horizontal: 2,
            });

            let chunks = popup_layout.split(popup_rects);

            let p_style = if self.popup_index == 0 {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };

            let s_style = if self.popup_index == 1 {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };

            let date_selected = self.popup_index == 2;

            let d_style = if date_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
            } else {
                Style::default()
            };

            let placeholder = self
                .due_date
                .format("%Y-%m-%d %H:%M")
                .to_string();

            let mut date_spans = vec![
                Span::styled("< ", d_style),
            ];

            if self.popup_text_input.is_empty() {
                // Display the example date when the field is empty.
                // Use dark gray text on the yellow selection background.
                let placeholder_style = if date_selected {
                    Style::default()
                        .fg(Color::DarkGray)
                        .bg(Color::Yellow)
                } else {
                    Style::default().fg(Color::DarkGray)
                };

                date_spans.push(Span::styled(
                    placeholder.clone(),
                    placeholder_style,
                ));
            } else {
                let overlap = self
                    .popup_text_input
                    .chars()
                    .count()
                    .min(placeholder.len());

                let typed_text: String = self
                    .popup_text_input
                    .chars()
                    .take(overlap)
                    .collect();

                // Typed text is black on yellow when the date field is selected.
                let typed_style = if date_selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                };

                date_spans.push(Span::styled(
                    typed_text,
                    typed_style,
                ));

                if overlap < placeholder.len() {
                    // Keep the untyped portion visibly darker than the entered text, including when highlighted.
                    let placeholder_style = if date_selected {
                        Style::default()
                            .fg(Color::DarkGray)
                            .bg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    };

                    date_spans.push(Span::styled(
                        &placeholder[overlap..],
                        placeholder_style,
                    ));
                }
            }

            date_spans.push(Span::styled(" >", d_style));

            let p_line = Line::from(vec![
                Span::raw("Priority: "),
                Span::styled(
                    format!("< {} >", self.priority),
                    p_style,
                ),
            ]);

            let s_line = Line::from(vec![
                Span::raw("Status:   "),
                Span::styled(
                    format!("< {} >", self.status),
                    s_style,
                ),
            ]);

            let mut d_line_spans = vec![
                Span::raw("Due:      "),
            ];

            d_line_spans.extend(date_spans);

            let d_line = Line::from(d_line_spans);

            let help_line = Line::from(Span::styled(
                " [↑/↓] Select | [Type] Enter Date | \
                 [Enter] Save | [Esc] Back ",
                Style::default().fg(Color::DarkGray),
            ));

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