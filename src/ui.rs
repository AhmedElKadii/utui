use crossterm::event::{self, KeyCode};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect, Flex};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListDirection, ListState};

use crate::crud::{self, ProjectData, open_project, delete_project, get_projects};

pub fn render(frame: &mut Frame, list_state: &mut ListState, project_names: Vec<String>) {
    let constraints = [
        Constraint::Length(1),
        Constraint::Percentage(100),
        Constraint::Length(1),
    ];
    let layout = Layout::vertical(constraints).flex(Flex::SpaceBetween).spacing(1);
    let [top, first, bottom] = frame.area().layout(&layout);

    let title = Line::from_iter([
        Span::from(" UTUI").blue().bold(),
        Span::from(" v1.0.0").gray(),
    ]);
    frame.render_widget(title.left_aligned(), top);

    render_projects_list(frame, first, list_state, project_names);
    render_help_text(frame, bottom);
}

fn render_projects_list(frame: &mut Frame, area: Rect, list_state: &mut ListState, project_names: Vec<String>) {
    let list = List::new(project_names)
        .style(Color::White)
        .highlight_style(Modifier::REVERSED)
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, area, list_state);
}

// TODO: reorder these and make thigns prettier, maybe put some in a "?" menu
fn render_help_text(frame: &mut Frame, area: Rect) {
    let title = Line::from_iter([
        Span::from("RET - Expand | ").bold(),
        Span::from("o - Open | ").bold(),
        Span::from("d - Delete | ").bold(),
        Span::from("D - Delete with Dir | ").bold(),
        // Span::from("c - Create | ").bold(),
        Span::from("j/Ctrl-n - Next | ").bold(),
        Span::from("k/Ctrl-p - Prev | ").bold(),
        Span::from("q - Quit").bold(),
    ]);
    frame.render_widget(title.centered(), area);
}

pub fn change_list(list_state: &mut ListState, next: bool) {
    match next {
        true => list_state.select_next(),
        false => list_state.select_previous()
    }
}

fn proj_details(project: &ProjectData) -> String {
    let name = &project.name;
    let path = &project.path;

    let str = format!("{name}
{path}");

    return str;
}

pub fn refresh(mut project_names: &mut Vec<String>, mut project_datas: &mut Vec<ProjectData>) {
    match get_projects() {
        Some(projects) => {
            *project_names = projects.iter().map(|p| p.name.clone() as String).collect();
            *project_datas = projects.clone();
        },
        None => ()
    }

    // make this not be a list and be a paragraph.
    if project_names.len() == 0 {
        project_names.push(String::from("No projects found... press C to create."));
    }
}

pub fn proj_open(list_state: &mut ListState, project_datas: &Vec<ProjectData>) {
    match list_state.selected_mut() {
        Some(i) => {
            match project_datas.get(i.clone()) {
                Some(pd) => open_project(pd),
                None => ()
            }
        },
        None => ()
    }
}

pub fn proj_delete(list_state: &mut ListState, mut project_names: &mut Vec<String>, project_datas: &Vec<ProjectData>, with_dir: bool) {
    match list_state.selected_mut() {
        Some(i) => {
            match project_datas.get(i.clone()) {
                Some(pd) => {
                    delete_project(pd, with_dir);
                    match get_projects() {
                        Some(projects) => {
                            *project_names = projects.iter().map(|p| p.name.clone() as String).collect::<Vec<String>>();
                        },
                        None => ()
                    }
                },
                None => ()
            }
        },
        None => ()
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
