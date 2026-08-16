use color_eyre::Result;
use crossterm::event::{self, KeyCode, KeyEventKind};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Position};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, List, ListItem, Paragraph};
use ratatui::{DefaultTerminal, Frame};

#[derive(Default, PartialEq)]
pub enum InputStep {
    #[default]
    Name,
    Path,
    Version,
    Template,
    Complete,
}

#[derive(Default)]
pub struct InputHandler {
    pub input: String,
    pub character_index: usize,
    pub messages: Vec<String>,
    pub step: InputStep
}

impl InputHandler {
    pub const fn new() -> Self {
        Self {
            input: String::new(),
            messages: Vec::new(),
            character_index: 0,
            step: InputStep::Name
        }
    }

    pub fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    pub fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    pub fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    pub fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    pub fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {

            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            let after_char_to_delete = self.input.chars().skip(current_index);

            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    pub fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    pub const fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    pub fn submit_message(&mut self) {
        self.messages.push(self.input.clone());
        self.input.clear();
        self.reset_cursor();
    }

    pub fn render_input_box(&self, frame: &mut Frame, title_message: &str) {
        let centered_area = frame.area().centered(
            Constraint::Percentage(40), 
            Constraint::Length(5)
        );

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(2),
            ])
            .split(centered_area);

        let (msg, style) = (
            vec![
            "Press ".into(),
            "Esc".bold(),
            " to cancel, ".into(),
            "Enter".bold(),
            " to confirm".into(),
            ],
            Style::default(),
        );

        let input = Paragraph::new(self.input.as_str())
            .style(Style::default().fg(Color::Yellow))
            .block(Block::bordered().title(title_message));
        frame.render_widget(input, chunks[0]);

        let text = Text::from(Line::from(msg).alignment(Alignment::Center)).patch_style(style);
        let help_message = Paragraph::new(text);
        frame.render_widget(help_message, chunks[1]);

        #[expect(clippy::cast_possible_truncation)]
        frame.set_cursor_position(Position::new(
                chunks[0].x + self.character_index as u16 + 1,
                chunks[0].y + 1,
        ));
    }
}
