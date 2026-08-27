use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, List, ListState, Padding, Paragraph, Wrap};

use crate::app::App;
use crate::config;
use crate::dialogue::{Dialogue, DialogueSelection};
use crate::input::InputStep;

pub fn render(frame: &mut Frame, app: &mut App) {
    app.prepare_input_lists();

    let constraints = [
        Constraint::Length(1),
        Constraint::Percentage(100),
        Constraint::Length(1),
    ];
    let layout = Layout::vertical(constraints)
        .flex(Flex::SpaceBetween)
        .spacing(1);
    let [top, middle, bottom] = frame.area().layout(&layout);

    let title = Line::from_iter([
        Span::from(" UTUI").blue().bold(),
        Span::from(format!(" {}", config::VERSION)).gray(),
    ]);
    frame.render_widget(title.left_aligned(), top);

    match &app.dialogue.current {
        Dialogue::DeleteConfirm { with_dir } => {
            if let Some(project) = &app.dialogue.selected_project {
                let message = if *with_dir {
                    format!("Are you sure you want to delete {} and its files?", project.name)
                } else {
                    format!("Are you sure you want to delete {}?", project.name)
                };
                popup_dialogue(
                    app,
                    frame,
                    "Confirm Delete",
                    &message,
                    Some("DELETE"),
                    Some("CANCEL"),
                );
            }
        }
        Dialogue::Error(message) => {
            popup_dialogue(app, frame, "Error", message, Some("OK"), None);
        }
        Dialogue::Info(message) => {
            popup_dialogue(app, frame, "Info", message, None, None);
        }
        Dialogue::Input => match app.input.step {
            InputStep::Name => {
                app.input.render_input_box(frame, "Project Name", None);
            }
            InputStep::Path => {
                let suggestions = app.list_items_buffer.clone();
                app.input.render_input_box(
                    frame,
                    "Project Path",
                    Some((&mut app.list_state, suggestions)),
                );
            }
            InputStep::Version => {
                let items = app.list_items.clone();
                app.input.render_input_box(
                    frame,
                    "Editor Version",
                    Some((&mut app.list_state, items)),
                );
            }
            InputStep::Template => {
                let labels = app.template_labels();
                app.input.render_input_box(
                    frame,
                    "Project Template",
                    Some((&mut app.list_state, labels)),
                );
            }
            _ => {}
        },
        Dialogue::None => {
            if app.project_task.is_some() {
                app.dialogue.current = Dialogue::Info(String::new());
            } else if app.list_items.is_empty() {
                let empty = Line::from_iter([
                    Span::from(" No projects available...").bold(),
                    Span::from(" press c to create a project or a to add an existing one."),
                ]);
                frame.render_widget(empty.left_aligned(), middle);
            } else {
                render_list(frame, middle, &mut app.list_state, app.list_items.clone());
            }
        }
    }

    render_help_text(frame, bottom);
}

fn popup_dialogue(
    app: &App,
    frame: &mut Frame,
    title: &str,
    message: &str,
    ok_label: Option<&str>,
    cancel_label: Option<&str>,
) {
    let area = frame.area();
    const MIN_WIDTH: u16 = 45;
    
    let has_buttons = ok_label.is_some() || cancel_label.is_some();
    let min_height: u16 = if has_buttons { 9 } else { 5 };

    let popup_width = ((area.width as f32 * 0.25) as u16)
        .max(MIN_WIDTH)
        .min(area.width);
    let popup_height = ((area.height as f32 * 0.18) as u16)
        .max(min_height)
        .min(area.height);

    let centered_area = area.centered(
        Constraint::Length(popup_width),
        Constraint::Length(popup_height),
    );
    frame.render_widget(Clear, centered_area);

    let popup_block = Block::bordered().title(title);
    let inner_area = popup_block.inner(centered_area);
    frame.render_widget(popup_block, centered_area);

    let constraints = if has_buttons {
        vec![
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(3),
        ]
    } else {
        vec![
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ]
    };

    let chunks = Layout::vertical(constraints).split(inner_area);
    let text_chunk = if has_buttons { chunks[1] } else { chunks[1] };

    let paragraph = Paragraph::new(message.to_string())
        .block(Block::new().padding(Padding::horizontal(1)))
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Center);
    frame.render_widget(paragraph, text_chunk);

    if has_buttons {
        let button_chunks = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(ok_label.map_or(0, |s| s.chars().count() as u16 + 4)),
            Constraint::Length(2),
            Constraint::Length(cancel_label.map_or(0, |s| s.chars().count() as u16 + 4)),
            Constraint::Fill(1),
        ])
        .split(chunks[3]);

        let ok_style = if app.dialogue.selection == DialogueSelection::Ok {
            Modifier::REVERSED
        } else {
            Modifier::empty()
        };
        let cancel_style = if app.dialogue.selection == DialogueSelection::Cancel {
            Modifier::REVERSED
        } else {
            Modifier::empty()
        };

        if let Some(label) = ok_label {
            let ok_btn = Paragraph::new(label)
                .block(Block::bordered().padding(Padding::horizontal(1)))
                .style(ok_style)
                .alignment(Alignment::Center);
            frame.render_widget(ok_btn, button_chunks[1]);
        }

        if let Some(label) = cancel_label {
            let cancel_btn = Paragraph::new(label)
                .block(Block::bordered().padding(Padding::horizontal(1)))
                .style(cancel_style)
                .alignment(Alignment::Center);
            frame.render_widget(cancel_btn, button_chunks[3]);
        }
    }
}

fn render_list(frame: &mut Frame, area: Rect, list_state: &mut ListState, list_items: Vec<String>) {
    let list = List::new(list_items)
        .style(Color::White)
        .highlight_style(Modifier::REVERSED)
        .highlight_symbol("> ");
    frame.render_stateful_widget(list, area, list_state);
}

fn render_help_text(frame: &mut Frame, area: Rect) {
    let title = Line::from_iter([
        Span::from("RET - Expand | ").bold(),
        Span::from("o - Open | ").bold(),
        Span::from("d - Delete | ").bold(),
        Span::from("D - Delete with Dir | ").bold(),
        Span::from("c - Create | ").bold(),
        Span::from("j/Ctrl-n - Next | ").bold(),
        Span::from("k/Ctrl-p - Prev | ").bold(),
        Span::from("q - Quit").bold(),
    ]);
    frame.render_widget(title.centered(), area);
}
