use crossterm::event::{self, KeyCode};
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Fill, List, ListDirection, ListState, Paragraph, Widget, Wrap};

use crate::Dialogues::DELETE_CONFIRM;
use crate::{AppState, Dialogues, DialogueSelection};
use crate::crud::{self, ProjectData, open_project, delete_project, get_projects};

pub fn render(frame: &mut Frame, app_state: &mut AppState) {
    let constraints = [
        Constraint::Length(1),
        Constraint::Percentage(100),
        Constraint::Length(1),
    ];
    let layout = Layout::vertical(constraints).flex(Flex::SpaceBetween).spacing(1);
    let [top, middle, bottom] = frame.area().layout(&layout);

    let title = Line::from_iter([
        Span::from(" UTUI").blue().bold(),
        Span::from(" v1.0.0").gray(),
    ]);
    frame.render_widget(title.left_aligned(), top);

    if app_state.list_items.len() > 0 {
        render_projects_list(frame, middle, &mut app_state.list_state, app_state.list_items.clone());
    }
    else {
        let empty = Line::from_iter([
            Span::from(" No projects available...").bold(),
            Span::from(" press c to create a project or a to add an existing one."),
        ]);
        frame.render_widget(empty.left_aligned(), middle);
    }

    let area = frame.area();

    match app_state.dialogue_state.current_dialogue {
        Dialogues::DELETE_CONFIRM(with_dir) => {
            let popup_block = Block::bordered().title("Confirm Delete");

            let centered_area = area.centered(Constraint::Percentage(30), Constraint::Percentage(20));
            frame.render_widget(Clear, centered_area);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Percentage(60),
                    Constraint::Percentage(40),
                ])
                .split(centered_area);

            let mut project_name = String::from("undefined");
            if let Some(project) = &app_state.dialogue_state.selected_project {
                project_name = project.name.clone();
            }

            let mut paragraph = Paragraph::new("");
            match with_dir {
                true => {
                    paragraph = Paragraph::new(format!("Are you sure you want to permanently delete {project_name} and its files?"))
                        .block(popup_block)
                        .wrap(Wrap { trim: true })
                        .centered()
                },
                _ => {
                    paragraph = Paragraph::new(format!("Are you sure you want to permanently delete {project_name}?"))
                        .block(popup_block)
                        .wrap(Wrap { trim: true })
                        .centered()
                },
            }
            frame.render_widget(paragraph, centered_area);

            let button_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Fill(1),
                    Constraint::Fill(3),
                    Constraint::Fill(1),
                    Constraint::Fill(3),
                    Constraint::Fill(1),
                ])
                .split(chunks[1]);

            let ok = Block::bordered();
            let mut ok_text = Paragraph::new("DELETE").block(ok.clone()).centered();
            let cancel = Block::bordered();
            let mut cancel_text = Paragraph::new("CANCEL").block(cancel.clone()).centered();
            match app_state.dialogue_state.selection {
                DialogueSelection::OK => {
                    ok.style(Modifier::REVERSED);
                    ok_text = ok_text.style(Modifier::REVERSED);
                },
                DialogueSelection::CANCEL => {
                    cancel.style(Modifier::REVERSED);
                    cancel_text = cancel_text.style(Modifier::REVERSED);
                },
                _ => ()
            }
            frame.render_widget(ok_text, button_chunks[1]);
            frame.render_widget(cancel_text, button_chunks[3]);
        }
        _ => (),
    }

    render_help_text(frame, bottom);
}

pub fn execute_selection(app_state: &mut AppState) {
    match app_state.dialogue_state.current_dialogue {
        Dialogues::DELETE_CONFIRM(with_dir) => {
            match app_state.dialogue_state.selection {
                DialogueSelection::OK => proj_delete(app_state),
                DialogueSelection::CANCEL => ()
            }
        },
        _ => ()
    }

    app_state.dialogue_state.selected_project = None;
    app_state.dialogue_state.current_dialogue = Dialogues::NULL;
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

pub fn change_list(mut app_state: &mut AppState, next: bool) {
    if app_state.list_items.len() == 0 { return; }

    match next {
        true => {
            match app_state.list_state.selected() {
                Some(index) => if index < app_state.list_items.len()-1 { collapse_project(app_state); },
                None => ()
            }
            app_state.list_state.select_next()
        },
        false => {
            match app_state.list_state.selected() {
                Some(index) => if index > 0 { collapse_project(&mut app_state); },
                None => ()
            }
            app_state.list_state.select_previous()
        },
    }
}

fn proj_details(project: &ProjectData) -> String {
    let name = &project.name;
    let path = &project.path;
    let git_status = if project.git_tracked { "tracked" } else { "untracked" };
    let version = &project.editor_version;
    let last_opened = &project.last_opened;

    let str = format!("{name}
Last Opened: {last_opened}
Git Status: {git_status}
Editor Version: {version}
Path: {path}");

    return str;
}

pub fn refresh(app_state: &mut AppState) {
    app_state.list_items.clear();
    app_state.project_data.clear();

    match get_projects() {
        Some(projects) => {
            app_state.list_items = projects.iter().map(|p| p.name.clone() as String).collect();
            app_state.project_data = projects.clone();
        },
        None => ()
    }

    app_state.dialogue_state.selected_project = None;
    app_state.dialogue_state.current_dialogue = Dialogues::NULL;
}

pub fn proj_open(app_state: &mut AppState) {
    if app_state.list_items.len() == 0 { return; }

    match app_state.list_state.selected_mut() {
        Some(i) => {
            match app_state.project_data.get(i.clone()) {
                Some(pd) => open_project(pd),
                None => ()
            }
        },
        None => ()
    }
}

pub fn open_delete_dialogue(app_state: &mut AppState, with_dir: bool) {
    if app_state.list_items.len() == 0 { return; }

    match app_state.list_state.selected_mut() {
        Some(i) => {
            match app_state.project_data.get(i.clone()) {
                Some(pd) => {
                    app_state.dialogue_state.current_dialogue = Dialogues::DELETE_CONFIRM(with_dir);
                    app_state.dialogue_state.selected_project = Some(pd.clone());
                    app_state.dialogue_state.selection = DialogueSelection::CANCEL;
                },
                None => ()
            }
        },
        None => ()
    }
}

pub fn proj_delete(app_state: &mut AppState) {
    match app_state.dialogue_state.selected_project.clone() {
        Some(project) => {
            match app_state.dialogue_state.current_dialogue {
                DELETE_CONFIRM(with_dir) => {
                    delete_project(&project, with_dir);
                    refresh(app_state);
                },
                _ => ()
            }
        },
        None => ()
    }
}

pub fn expand_project(mut app_state: &mut AppState) {
    if app_state.list_items.len() == 0 { return; }

    match app_state.list_state.selected_mut() {
        Some(i) => {
            match app_state.list_items[i.clone()].find('\n') {
                Some(e) => {
                    collapse_project(&mut app_state);
                    return;
                },
                None => ()
            }

            match app_state.project_data.get(i.clone()) {
                Some(pd) => app_state.list_items[i.clone()] = proj_details(pd),
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

pub fn collapse_project(app_state: &mut AppState) {
    if app_state.list_items.len() == 0 { return; }

    match app_state.list_state.selected_mut() {
        Some(i) => {
            match app_state.list_items[i.clone()].find('\n') {
                Some(index) => app_state.list_items[i.clone()] = crop_letters(&app_state.list_items[i.clone()], app_state.list_items[i.clone()].len() - index).to_string(),
                None => ()
            }
        },
        None => ()
    }
}
