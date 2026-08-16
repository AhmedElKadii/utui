use crossterm::event::{self, KeyCode};
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Fill, List, ListDirection, ListState, Padding, Paragraph, Widget, Wrap};

use crate::Dialogues::DELETE_CONFIRM;
use crate::{AppState, Dialogues, DialogueSelection};
use crate::crud::{self, ProjectData, create_project, delete_project, get_projects, open_project};
use crate::input::{self, InputStep};

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

    match app_state.dialogue_state.current_dialogue {
        Dialogues::DELETE_CONFIRM(with_dir) => {
            if let Some(project) = &app_state.dialogue_state.selected_project {
                let mut message = format!("Are you sure you want to delete {}", project.name);
                if with_dir { message.push_str(" and its files?"); }
                else { message.push('?'); }

                popup_dialogue(
                    app_state, 
                    frame, 
                    String::from("Confirm Delete"), 
                    message,
                    Some(String::from("DELETE")), 
                    Some(String::from("CANCEL"))
                );
            }
        },
        Dialogues::INPUT => {
            match app_state.input_handler.step {
                InputStep::Name => {
                    app_state.input_handler.render_input_box(frame, "Project Name");
                },
                InputStep::Path => {
                    app_state.input_handler.render_input_box(frame, "Project Path");
                },
                InputStep::Version => {
                    // TODO: change to list!
                    app_state.input_handler.render_input_box(frame, "Editor Version");
                },
                InputStep::Template => {
                    // TODO: CHANGE TO LIST!
                    app_state.input_handler.render_input_box(frame, "Project Template");
                },
                InputStep::Complete => {
                    match &app_state.dialogue_state.selected_project {
                        Some(project) => match create_project(&project) {
                            // TODO: add error handling popup here!
                            Ok((true, o)) => (),
                            Ok((false, e)) => (),
                            _ => ()
                        },
                        _ => ()
                    }
                    reset_state(app_state);
                    refresh(app_state);
                },
                _ => ()
            }
            // refresh(app_state);
        },
        _ => {
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
        },
    }

    render_help_text(frame, bottom);
}

fn popup_dialogue(
    app_state: &mut AppState,
    frame: &mut Frame,
    title: String,
    message: String,
    ok_label: Option<String>,
    cancel_label: Option<String>,
) {
    let area = frame.area();

    const MIN_WIDTH: u16 = 45;
    const MIN_HEIGHT: u16 = 9;

    let popup_width = ((area.width as f32 * 0.4) as u16)
        .max(MIN_WIDTH)
        .min(area.width);
    let popup_height = ((area.height as f32 * 0.3) as u16)
        .max(MIN_HEIGHT)
        .min(area.height);

    let centered_area = area.centered(Constraint::Length(popup_width), Constraint::Length(popup_height));
    frame.render_widget(Clear, centered_area);

    let popup_block = Block::bordered().title(title);
    let inner_area = popup_block.inner(centered_area);
    frame.render_widget(popup_block, centered_area);

    let chunks = Layout::vertical([
        Constraint::Length(1), // Top margin
        Constraint::Fill(1),   // Message body
        Constraint::Length(1), // Bottom margin
        Constraint::Length(3), // Button row
    ])
    .split(inner_area);

    let message_block = Block::new().padding(Padding::horizontal(1)); 
    let paragraph = Paragraph::new(message)
        .block(message_block)
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Center);
    
    frame.render_widget(paragraph, chunks[1]);

    let button_chunks = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(ok_label.as_ref().map_or(0, |s| s.chars().count() as u16 + 4)),
        Constraint::Length(2), 
        Constraint::Length(cancel_label.as_ref().map_or(0, |s| s.chars().count() as u16 + 4)),
        Constraint::Fill(1),
    ])
    .split(chunks[3]);

    let ok_style = if app_state.dialogue_state.selection == DialogueSelection::OK {
        Modifier::REVERSED
    } else {
        Modifier::empty()
    };
    let cancel_style = if app_state.dialogue_state.selection == DialogueSelection::CANCEL {
        Modifier::REVERSED
    } else {
        Modifier::empty()
    };

    if let Some(ol) = ok_label {
        let ok_btn = Paragraph::new(ol)
            .block(Block::bordered().padding(Padding::horizontal(1)))
            .style(ok_style)
            .alignment(Alignment::Center);
        frame.render_widget(ok_btn, button_chunks[1]);
    }

    if let Some(cl) = cancel_label {
        let cancel_btn = Paragraph::new(cl)
            .block(Block::bordered().padding(Padding::horizontal(1)))
            .style(cancel_style)
            .alignment(Alignment::Center);
        frame.render_widget(cancel_btn, button_chunks[3]);
    }
}

fn reset_state(app_state: &mut AppState) {
    app_state.list_state.select(app_state.selected_index);
    app_state.dialogue_state.selected_project = None;
    app_state.dialogue_state.current_dialogue = Dialogues::NULL;
    app_state.input_handler.step = InputStep::Name;
}

pub fn execute_selection(app_state: &mut AppState) {
    match app_state.dialogue_state.current_dialogue {
        Dialogues::DELETE_CONFIRM(with_dir) => {
            match app_state.dialogue_state.selection {
                DialogueSelection::OK => proj_delete(app_state),
                _ => (),
            }
            reset_state(app_state);
        },
        // TODO: PRESSING ESC COUNTS!!
        Dialogues::INPUT => {
            if app_state.dialogue_state.selection == DialogueSelection::CANCEL { 
                reset_state(app_state); 
                return;
            }

            match &mut app_state.dialogue_state.selected_project {
                Some(project) => {
                    match app_state.input_handler.step {
                        InputStep::Name => {
                            project.name = app_state.input_handler.input.clone();
                            app_state.input_handler.step = InputStep::Path;
                        },
                        InputStep::Path => {
                            project.path = app_state.input_handler.input.clone();
                            app_state.input_handler.step = InputStep::Version;
                        },
                        InputStep::Version => {
                            project.editor_version = app_state.input_handler.input.clone();
                            app_state.input_handler.step = InputStep::Template;
                        },
                        InputStep::Template => {
                            project.template = app_state.input_handler.input.clone();
                            app_state.input_handler.step = InputStep::Complete;
                        },
                        _ => ()
                    }
                },
                _ => ()
            }
            app_state.input_handler.submit_message();
        },
        _ => ()
    }
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
    app_state.list_state.select_first();
}

pub fn open_proj_create_dialogue(app_state: &mut AppState,) {
    app_state.selected_index = app_state.list_state.selected();
    app_state.list_state.select(None);
    app_state.dialogue_state.current_dialogue = Dialogues::INPUT;
    app_state.dialogue_state.selected_project = Some(ProjectData::default());
    app_state.dialogue_state.selection = DialogueSelection::OK;
}

pub fn proj_create(app_state: &mut AppState) {
    let mut name = String::new();
    let mut path = String::new();

    // let mut editor = String::new();
    // let mut template = String::new();
    // let mut is_ready = true;

    // match get_editors() {
    //     Some(editors) => {
    //         let mut i = 0;
    //
    //         for e in &editors {
    //             println!("{}: {:?}", i, e);
    //             i += 1;
    //         }
    //
    //         let mut choice: String = String::new();
    //
    //         io::stdin()
    //             .read_line(&mut choice)
    //             .expect("Failed to read line");
    //
    //         match editors.get(choice.trim().parse::<usize>().unwrap()) {
    //             Some(e) => editor = e.clone(),
    //             None => ()
    //         }
    //     },
    //     None => eprintln!("Fetch failed!")
    // }
    //
    // match get_templates(editor.clone()) {
    //     Some(templates) => {
    //         let mut i = 0;
    //
    //         for t in &templates {
    //             println!("{}: {:?}", i, t.display_name);
    //             i += 1;
    //         }
    //
    //         let mut choice: String = String::new();
    //
    //         io::stdin()
    //             .read_line(&mut choice)
    //             .expect("Failed to read line");
    //
    //         match templates.get(choice.trim().parse::<usize>().unwrap()) {
    //             Some(t) => {
    //                 template = t.name.clone();
    //                 is_ready = t.status == TemplateStatus::READY;
    //             },
    //             None => eprintln!("Failed to get template")
    //         }
    //     },
    //     None => eprintln!("Fetch failed!")
    // }
    
    // match create_project(name, editor, template, path, is_ready) {
    //     Ok((true, o)) => println!("{}", o),
    //     _ => eprintln!("An error occured")
    // }
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

    app_state.selected_index = app_state.list_state.selected();
    app_state.list_state.select(None);
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
