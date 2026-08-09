use crossterm::event::{self, KeyCode};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListDirection, ListState};

use crate::crud;

pub fn render(frame: &mut Frame, list_state: &mut ListState, project_names: Vec<String>) {
    let constraints = [
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Fill(1),
    ];
    let layout = Layout::vertical(constraints).spacing(1);
    let [top, first, second] = frame.area().layout(&layout);

    let title = Line::from_iter([
        Span::from("List Widget").bold(),
        Span::from(" (Press 'q' to quit and arrow keys to navigate)"),
    ]);
    frame.render_widget(title.centered(), top);

    render_projects_list(frame, first, list_state, project_names);
}

pub fn render_projects_list(frame: &mut Frame, area: Rect, list_state: &mut ListState, project_names: Vec<String>) {
    let list = List::new(project_names)
        .style(Color::White)
        .highlight_style(Modifier::REVERSED)
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, area, list_state);
}
