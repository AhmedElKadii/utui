use crossterm::event::{self, KeyCode};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListDirection, ListState};

use crate::crud::{self, ProjectData};

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
    render_help_text(frame);
}

fn render_projects_list(frame: &mut Frame, area: Rect, list_state: &mut ListState, project_names: Vec<String>) {
    let list = List::new(project_names)
        .style(Color::White)
        .highlight_style(Modifier::REVERSED)
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, area, list_state);
}

fn render_help_text(frame: &mut Frame) {
    let vertical = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).spacing(1);
    let horizontal = Layout::horizontal([Constraint::Percentage(50); 2]).spacing(1);
    let [bottom, main] = frame.area().layout(&vertical);

    let title = Line::from_iter([
        Span::from("Help:").bold(),
        Span::from(" O - Open"),
    ]);
    frame.render_widget(title.centered(), bottom);
}

pub fn change_list(list_state: &mut ListState, next: bool) {
    match next {
        true => list_state.select_next(),
        false => list_state.select_previous()
    }
}

pub fn expand_project(index: Option<usize>, project_names: &mut Vec<String>, project_datas: &Vec<ProjectData>) {
    match index {
        Some(i) => {
            match project_names[i.clone()].find('\n') {
                Some(e) => {
                    collapse_project(index, project_names);
                    return;
                },
                None => ()
            }

            match project_datas.get(i.clone()) {
                Some(pd) => project_names[i.clone()] = proj_details(pd),
                None => ()
            }
        },
        None => ()
    }
}

fn crop_letters(s: &str, pos: usize) -> &str {
    match s.char_indices().skip(pos).next() {
        Some((pos, _)) => &s[pos..],
        None => "",
    }
}

pub fn collapse_project(index: Option<usize>, project_names: &mut Vec<String>) {
    match index {
        Some(i) => {
            match project_names[i.clone()].find('\n') {
                Some(index) => project_names[i.clone()] = crop_letters(&project_names[i.clone()], project_names[i.clone()].len() - index).to_string(),
                None => ()
            }
        },
        None => ()
    }
}

fn proj_details(project: &ProjectData) -> String {
    let name = &project.name;
    let path = &project.path;

    let str = format!("{name}
{path}");

    return str;
}

