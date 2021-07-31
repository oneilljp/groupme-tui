use crate::app::*;

use std::time::Duration;

use crossterm::event::{poll, read, Event, KeyCode};
use crossterm::Result;

use tui::{backend::Backend, Terminal};

pub fn poll_input<B: Backend>(app: &mut App<'static>, terminal: &mut Terminal<B>) -> Result<bool> {
    if poll(Duration::from_millis(500))? {
        match read()? {
            Event::Key(event) => {
                if app.disp == DispMode::Help && event.code == KeyCode::Esc {
                    app.disp = DispMode::Main;
                } else {
                    match app.mode {
                        Modes::GroupNav => {
                            if event.code == KeyCode::Char('j') || event.code == KeyCode::Down {
                                app.groups.next();
                            } else if event.code == KeyCode::Char('k') || event.code == KeyCode::Up
                            {
                                app.groups.previous();
                            } else if event.code == KeyCode::Char('l')
                                || event.code == KeyCode::Enter
                                || event.code == KeyCode::Right
                            {
                                // Clear input for new group
                                if app.group_id
                                    != app.groups.items[app.groups.state.selected().unwrap()].id
                                {
                                    app.input.clear();
                                    app.input.push('▏');
                                    app.input_pos = 0;
                                }
                                app.update_msgs();
                                app.disp = DispMode::Main;
                                app.dm = false;
                                app.mode = Modes::MessageNav;
                            } else if event.code == KeyCode::Char('q') {
                                return Ok(false);
                            } else if event.code == KeyCode::Char('d') {
                                app.mode = Modes::DirectNav;
                            } else if event.code == KeyCode::Char('?') {
                                app.disp = DispMode::Help;
                            }
                        }
                        Modes::DirectNav => {
                            if event.code == KeyCode::Char('j') || event.code == KeyCode::Down {
                                app.dms.next();
                            } else if event.code == KeyCode::Char('k') || event.code == KeyCode::Up
                            {
                                app.dms.previous();
                            } else if event.code == KeyCode::Char('l')
                                || event.code == KeyCode::Enter
                                || event.code == KeyCode::Right
                            {
                                // Clear input for new group
                                if app.dm_id
                                    != app.dms.items[app.dms.state.selected().unwrap()].id
                                {
                                    app.input.clear();
                                    app.input.push('▏');
                                    app.input_pos = 0;
                                }
                                app.update_dmsgs();
                                app.disp = DispMode::Main;
                                app.dm = true;
                                app.mode = Modes::MessageNav;
                            } else if event.code == KeyCode::Char('q') {
                                return Ok(false);
                            } else if event.code == KeyCode::Char('g') {
                                app.mode = Modes::GroupNav;
                            } else if event.code == KeyCode::Char('?') {
                                app.disp = DispMode::Help;
                            }
                        }
                        Modes::MessageNav => {
                            if event.code == KeyCode::Char('q') {
                                return Ok(false);
                            } else if event.code == KeyCode::Char('r') {
                                if app.dm {
                                    app.update_dmsgs();
                                } else {
                                    app.update_msgs();
                                }
                            } else if event.code == KeyCode::Char('i') {
                                app.mode = Modes::Inputting;
                            } else if event.code == KeyCode::Char('h')
                                || event.code == KeyCode::Left
                                || event.code == KeyCode::Esc
                            {
                                app.mode = if app.dm { Modes::DirectNav } else { Modes::GroupNav };
                            } else if event.code == KeyCode::Char('j')
                                || event.code == KeyCode::Down
                            {
                                app.messages.next();
                            } else if event.code == KeyCode::Char('k') || event.code == KeyCode::Up
                            {
                                app.messages.previous();
                            } else if event.code == KeyCode::Enter {
                                if app.dm {
                                    app.dlike();
                                } else {
                                    app.like();
                                }
                            } else if event.code == KeyCode::Char('?') {
                                app.disp = DispMode::Help;
                            }
                        }
                        Modes::Inputting => match event.code {
                            KeyCode::Char(c) => {
                                //app.input.push(c);
                                app.input.insert(app.input_pos, c);
                                app.input_pos += 1;
                            }
                            KeyCode::Backspace => {
                                //app.input.pop();
                                if app.input_pos > 0 {
                                    app.input.remove(app.input_pos - 1);
                                    app.input_pos -= 1;
                                }
                            }
                            KeyCode::Left => {
                                if app.input_pos > 0 {
                                    app.input.remove(app.input_pos);
                                    let tail = app.input.split_off(app.input_pos - 1);
                                    app.input.push('▏');
                                    app.input.push_str(&tail);
                                    app.input_pos -= 1;
                                }
                            }
                            KeyCode::Right => {
                                if app.input_pos < app.input.chars().count() - 1 {
                                    app.input.remove(app.input_pos);
                                    let tail = app.input.split_off(app.input_pos + 1);
                                    app.input.push('▏');
                                    app.input.push_str(&tail);
                                    app.input_pos += 1;
                                }
                            }
                            KeyCode::Enter => {
                                app.input.remove(app.input_pos);
                                if app.dm {
                                    app.send_dmsg();
                                } else {
                                    app.send_msg();
                                }
                                app.input.push('▏');
                                app.input_pos = 0;
                            }
                            KeyCode::Esc => {
                                app.mode = Modes::MessageNav;
                            }
                            _ => {}
                        },
                    }
                }
            }
            Event::Mouse(_event) => {
                return Ok(false);
            }
            Event::Resize(_width, _height) => {
                app.t_width = (terminal.size().unwrap().width as f64 * 0.98 * 0.70) as u16;
            }
        }
    }
    Ok(true)
}
