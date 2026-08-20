use color_eyre::Result;
use crossterm::event::{self, KeyCode, KeyEventKind};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Position};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState};
use ratatui::{DefaultTerminal, Frame};
use unicode_width::UnicodeWidthStr;

#[derive(Default, PartialEq)]
pub enum InputStep {
    #[default]
    Name,
    Path,
    Version,
    Template,
    Complete,
    Canceled
}

#[derive(Default)]
pub struct InputHandler {
    pub input: String,
    pub character_index: usize,
    pub step: InputStep,
    list_line_offset: usize,
    last_selected_idx: Option<usize>
}

impl InputHandler {
    pub const fn new() -> Self {
        Self {
            input: String::new(),
            character_index: 0,
            step: InputStep::Name,
            list_line_offset: 0,
            last_selected_idx: None
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
        self.input.clear();
        self.reset_cursor();
    }

    pub fn render_input_box(&mut self, frame: &mut Frame, title_message: &str, list_data: Option<(&mut ListState, Vec<String>)>) {
        let list_height = if let Some((_, list_items)) = &list_data {
            std::cmp::min(4 * list_items.len() as u16, 20)
        } else {
            0
        };

        let window_height = 3 + 1 + list_height;

        let centered_area = frame.area().centered(
            Constraint::Percentage(40), 
            Constraint::Length(window_height)
        );

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Fill(1)
            ])
            .split(centered_area);

        let (msg, style) = (
            vec![
            "Press ".into(),
            "Esc".bold(),
            " to cancel, ".into(),
            "Enter".bold(),
            " to confirm ".into(),
            ],
            Style::default(),
        );

        let input_inner_width = chunks[1].width.saturating_sub(2) as usize;
        
        let input_scroll_offset = if self.character_index >= input_inner_width {
            self.character_index - input_inner_width + 1
        } else {
            0
        };

        let input = Paragraph::new(self.input.as_str())
            .style(Style::default().fg(Color::Yellow))
            .block(Block::bordered().title(title_message))
            .scroll((0, input_scroll_offset as u16));
        frame.render_widget(input, chunks[1]);

        let text = Text::from(Line::from(msg).alignment(Alignment::Right)).patch_style(style);
        let help_message = Paragraph::new(text);
        frame.render_widget(help_message, chunks[0]);

        let _ = crossterm::queue!(
            std::io::stdout(),
            crossterm::cursor::SetCursorStyle::BlinkingBlock
        );

        #[expect(clippy::cast_possible_truncation)]
        frame.set_cursor_position(Position::new(
                chunks[1].x + (self.character_index - input_scroll_offset) as u16 + 1,
                chunks[1].y + 1,
        ));

        if let Some((mut list_state, list_items)) = list_data {
            if list_items.len() == 0 { list_state.select(None); }

            let base_width = chunks[2].width.saturating_sub(4) as usize;
            let indent_prefix = " ▎ ";
            let indent_width = UnicodeWidthStr::width(indent_prefix);

            let mut all_lines: Vec<Line> = Vec::new();
            let mut item_ranges: Vec<(usize, usize)> = Vec::new();

            for item in &list_items {
                let start = all_lines.len();
                let mut lines_iter = item.lines();
                if let Some(name_line) = lines_iter.next() {
                    for line in textwrap::wrap(name_line, base_width) {
                        all_lines.push(Line::from(line.into_owned()));
                    }
                }
                let sub_width = base_width.saturating_sub(indent_width);
                for desc_line in lines_iter {
                    for line in textwrap::wrap(desc_line, sub_width) {
                        all_lines.push(Line::from(format!("{}{}", indent_prefix, line)));
                    }
                }
                item_ranges.push((start, all_lines.len()));
            }

            let available_height = chunks[2].height.saturating_sub(2) as usize;
            let inner_width = chunks[2].width.saturating_sub(2) as usize;

            let selected_idx: Option<usize> = list_state.selected().map(|i| {
                i.min(item_ranges.len().saturating_sub(1))
            });
            if selected_idx != list_state.selected() {
                list_state.select(selected_idx);
            }

            let mut scroll_offset = self.list_line_offset;
            if let Some(idx) = selected_idx {
                if Some(idx) != self.last_selected_idx {
                    if let Some(&(sel_start, sel_end)) = item_ranges.get(idx) {
                        if sel_start < scroll_offset {
                            scroll_offset = sel_start;
                        } else if sel_end > scroll_offset + available_height {
                            scroll_offset = sel_end.saturating_sub(available_height);
                        }
                    }
                }
            }
            let max_offset = all_lines.len().saturating_sub(available_height);
            scroll_offset = scroll_offset.min(max_offset);
            self.list_line_offset = scroll_offset;
            self.last_selected_idx = selected_idx;

            if let Some(idx) = selected_idx {
                if let Some(&(s, e)) = item_ranges.get(idx) {
                    for line in &mut all_lines[s..e] {
                        let text: String = line.spans.iter().map(|sp| sp.content.as_ref()).collect();
                        let text_width = UnicodeWidthStr::width(text.as_str());
                        let padding = inner_width.saturating_sub(text_width);
                        let padded = format!("{}{}", text, " ".repeat(padding));
                        *line = Line::from(padded).style(Style::default().add_modifier(Modifier::REVERSED));
                    }
                }
            }

            let total_lines = all_lines.len();
            let paragraph = Paragraph::new(all_lines)
                .block(Block::bordered())
                .style(Color::White)
                .scroll((scroll_offset as u16, 0));
            frame.render_widget(paragraph, chunks[2]);

            if total_lines > available_height {
                let mut scrollbar_state = ScrollbarState::new(total_lines.saturating_sub(available_height))
                    .position(scroll_offset);
                let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("┐"))
                    .end_symbol(Some("┘"))
                    .track_symbol(Some("│"))
                    .thumb_symbol("█");
                frame.render_stateful_widget(scrollbar, chunks[2], &mut scrollbar_state);
            }
        }
    }
}
