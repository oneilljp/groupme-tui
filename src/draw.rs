use crate::app::*;
use crate::utils::*;

use std::io;

use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    terminal::Frame,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};

fn draw_startup(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) {
    terminal
        .draw(|f| {
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints(
                    [
                        Constraint::Percentage(5),
                        Constraint::Percentage(15),
                        Constraint::Percentage(75),
                        Constraint::Percentage(5),
                    ]
                    .as_ref(),
                )
                .split(f.size());
            let left_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
                .split(main_chunks[1]);
            let right_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(15), Constraint::Percentage(85)].as_ref())
                .split(main_chunks[2]);
            render_groups(f, app, left_chunks[0]);
            render_direct(f, app, left_chunks[1]);
            let logo = Paragraph::new(logo::LOGO)
                .block(Block::default().borders(Borders::LEFT | Borders::RIGHT | Borders::TOP));
            let changelog = Paragraph::new(include_str!("../CHANGELOG.md").to_string())
                .block(Block::default().borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM));
            f.render_widget(logo, right_chunks[0]);
            f.render_widget(changelog, right_chunks[1]);
        })
        .expect("Bad Term")
}

fn draw_help(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, _app: &mut App) {
    terminal
        .draw(|f| {
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints(
                    [
                        Constraint::Percentage(5),
                        Constraint::Percentage(90),
                        Constraint::Percentage(5),
                    ]
                    .as_ref(),
                )
                .split(f.size());
            let help_menu = Paragraph::new("Help To Be Implemented")
                .block(Block::default().borders(Borders::ALL).title("Help"));
            f.render_widget(help_menu, main_chunks[1]);
        })
        .expect("Bad Term");
}

fn draw_main(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) {
    terminal
        .draw(|f| {
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints(
                    [
                        Constraint::Percentage(5),
                        Constraint::Percentage(15),
                        Constraint::Percentage(75),
                        Constraint::Percentage(5),
                    ]
                    .as_ref(),
                )
                .split(f.size());

            let left_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
                .split(main_chunks[1]);

            let right_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(94), Constraint::Percentage(6)].as_ref())
                .split(main_chunks[2]);
            render_groups(f, app, left_chunks[0]);
            render_direct(f, app, left_chunks[1]);
            render_messages(f, app, right_chunks[0]);

            let input_block = Paragraph::new(app.input.as_ref())
                .style(Style::default().fg(Color::White))
                .block(
                    Block::default()
                        .title("New Message")
                        .borders(Borders::ALL)
                        .border_style(if app.mode == Modes::Inputting {
                            Style::default().fg(Color::Magenta)
                        } else {
                            Style::default()
                        }),
                );
            f.render_widget(input_block, right_chunks[1]);
        })
        .expect("Bad term");
}

/* Render group display to the given Rect chunk
 * f: Frame from terminal.draw
 * app: App, kinda given
 * chunk: Rect to be drawn to
 */
fn render_groups(f: &mut Frame<CrosstermBackend<io::Stdout>>, app: &mut App, chunk: Rect) {
    let group_items: Vec<ListItem> = app
        .groups
        .items
        .iter()
        .map(|i| ListItem::new(i.name.as_ref()))
        .collect();
    let group_list = List::new(group_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("(G)roups")
                .border_style(if app.mode == Modes::GroupNav {
                    Style::default().fg(Color::Magenta)
                } else {
                    Style::default()
                }),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">");
    f.render_stateful_widget(group_list, chunk, &mut app.groups.state);
}

/* Render direct message display to the given Rect chunk
 * f: Frame from terminal.draw
 * app: App, kinda given
 * chunk: Rect to be drawn to
 */
fn render_direct(f: &mut Frame<CrosstermBackend<io::Stdout>>, app: &mut App, chunk: Rect) {
    let dm_items: Vec<ListItem> = app
        .dms
        .items
        .iter()
        .map(|i| ListItem::new(i.name.as_ref()))
        .collect();
    let dm_list = List::new(dm_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("(D)irect Messages")
                .border_style(if app.mode == Modes::DirectNav {
                    Style::default().fg(Color::Magenta)
                } else {
                    Style::default()
                }),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">");
    f.render_stateful_widget(dm_list, chunk, &mut app.dms.state);
}

/* Render direct message display to the given Rect chunk
 * f: Frame from terminal.draw
 * app: App, kinda given
 * chunk: Rect to be drawn to
 */
fn render_messages(f: &mut Frame<CrosstermBackend<io::Stdout>>, app: &mut App, chunk: Rect) {
    let message_items: Vec<ListItem> = app
        .messages
        .items
        .iter()
        .enumerate()
        .map(|(i, m)| {
            if i % 2 == 0 {
                ListItem::new(m.display.clone())
            } else {
                ListItem::new(m.display.clone())
            }
        })
        .collect();
    let msg_list = List::new(message_items)
        .block(
            Block::default()
                .title("Messages")
                .borders(Borders::ALL)
                .border_style(if app.mode == Modes::MessageNav {
                    Style::default().fg(Color::Magenta)
                } else {
                    Style::default()
                }),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Black)
                .add_modifier(Modifier::ITALIC),
        );
    f.render_stateful_widget(msg_list, chunk, &mut app.messages.state);
}

pub fn draw_term(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) {
    match app.disp {
        DispMode::Startup => draw_startup(terminal, app),
        DispMode::Main => draw_main(terminal, app),
        DispMode::Help => draw_help(terminal, app),
    }
}
