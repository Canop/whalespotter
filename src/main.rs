/*!

*/
#[macro_use(select)] extern crate crossbeam;
#[macro_use] extern crate lazy_static;

mod computer;
mod file_info;
mod screen;

use std::path::{Path, PathBuf};
use crossterm_screen::AlternateScreen;
use crossterm_input::KeyEvent;
use crossterm_cursor::TerminalCursor;
use termimad::*;
use open;

use crate::{
    computer::{ComputationEvent, Computer},
    screen::Screen,
};

/// find the path to open first: either a passed one
/// or the current dir.
fn starting_path() -> PathBuf {
    if let Some(s) = std::env::args().last() {
        let path = Path::new(&s);
        if path.is_dir() {
            if let Ok(path) = path.canonicalize() {
                return path.to_path_buf();
            }
        }
    }
    std::env::current_dir().unwrap_or_else(|_|Path::new("/").to_path_buf())
}

fn main() {
    let mut screen = Screen::new(starting_path());
    let _alt_screen = AlternateScreen::to_alternate(true);
    let cursor = TerminalCursor::new();
    cursor.hide().unwrap();
    let event_source = EventSource::new();
    let rx_user = event_source.receiver();

    let mut computer = Computer::new();
    computer.do_children(screen.get_root());

    loop {
        screen.display();
        select! {
            recv(computer.rx) -> comp_event => {
                match comp_event {
                    Ok(ComputationEvent::FileInfo(fi)) => {
                        screen.add_to_total_size(fi.size);
                        screen.list_view.add_row(fi);
                    }
                    Ok(ComputationEvent::Finished) => {
                        screen.set_finished();
                    }
                    _ => {
                        // can this really happen ?
                    }
                }
            }
            recv(rx_user) -> user_event => {
                if let Ok(user_event) = user_event {
                    let mut quit = false;
                    match user_event {
                        Event::Key(KeyEvent::Ctrl('q')) => {
                            quit = true;
                        }
                        Event::Key(KeyEvent::Char('\n')) => {
                            let fi = screen.list_view.get_selection();
                            if let Some(fi) = fi {
                                if fi.is_dir {
                                    let path = fi.path.clone();
                                    screen.set_new_root(path);
                                    computer.do_children(screen.get_root());
                                } else {
                                    open::that(&fi.path).unwrap(); // TODO display an error if it fails
                                }
                            }
                        }
                        Event::Key(KeyEvent::F(5)) => {
                            screen.set_new_root(screen.get_root().to_path_buf());
                            computer.do_children(screen.get_root());
                        }
                        Event::Key(KeyEvent::Esc) => {
                            if screen.list_view.has_selection() {
                                screen.list_view.unselect();
                            } else {
                                let path = screen.get_root().parent().map(|p| p.to_path_buf());
                                if let Some(path) = path {
                                    screen.set_new_root(path);
                                    computer.do_children(screen.get_root());
                                } else {
                                    quit = true;
                                }
                            }
                        }
                        Event::Key(KeyEvent::Home) => {
                            screen.list_view.select_first_line();
                        }
                        Event::Key(KeyEvent::End) => {
                            screen.list_view.select_last_line();
                        }
                        Event::Key(KeyEvent::PageUp) => {
                            screen.list_view.try_scroll_pages(-1);
                        }
                        Event::Key(KeyEvent::PageDown) => {
                            screen.list_view.try_scroll_pages(1);
                        }
                        Event::Key(KeyEvent::Up) => {
                            screen.list_view.try_select_next(true);
                        }
                        Event::Key(KeyEvent::Down) => {
                            screen.list_view.try_select_next(false);
                        }
                        Event::Wheel(lines_count) => {
                            screen.list_view.try_scroll_lines(lines_count);
                        }
                        _ => {
                            //input_field.apply_event(&user_event);
                        }
                    };
                    event_source.unblock(quit); // if quit is true, this will lead to channel closing
                } else {
                    // The channel has been closed, which means the event source
                    // has properly released its resources, we may quit.
                    break;
                }
            }
        }
    }

    cursor.show().unwrap();
}

