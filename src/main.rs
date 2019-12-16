#[macro_use(select)]
extern crate crossbeam;
#[macro_use]
extern crate lazy_static;

mod computer;
mod file_info;
mod screen;

use {
    crate::{
        computer::{ComputationEvent, Computer},
        screen::Screen,
    },
    crossterm::{
        cursor,
        event::KeyCode,
        queue,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    },
    open,
    std::{
        io::Write,
        path::{Path, PathBuf},
    },
    termimad::{Event, EventSource},
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
    std::env::current_dir().unwrap_or_else(|_| Path::new("/").to_path_buf())
}

const ENTER: Event = Event::simple_key(KeyCode::Enter);
const F5: Event = Event::simple_key(KeyCode::F(5));
const ESC: Event = Event::simple_key(KeyCode::Esc);
const HOME: Event = Event::simple_key(KeyCode::Home);
const END: Event = Event::simple_key(KeyCode::End);
const PAGE_UP: Event = Event::simple_key(KeyCode::PageUp);
const PAGE_DOWN: Event = Event::simple_key(KeyCode::PageDown);
const UP: Event = Event::simple_key(KeyCode::Up);
const DOWN: Event = Event::simple_key(KeyCode::Down);
const CTRL_Q: Event = Event::crtl_key(KeyCode::Char('q'));

fn main() -> termimad::Result<()> {
    let mut w = std::io::stderr();
    queue!(w, EnterAlternateScreen)?;
    queue!(w, cursor::Hide)?; // hiding the cursor

    let mut screen = Screen::new(starting_path());
    let event_source = EventSource::new()?;
    let rx_user = event_source.receiver();

    let mut computer = Computer::new();
    computer.do_children(screen.get_root());

    loop {
        screen.display(&mut w)?;
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
                        ENTER => {
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
                        F5 => {
                            screen.set_new_root(screen.get_root().to_path_buf());
                            computer.do_children(screen.get_root());
                        }
                        ESC => {
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
                        HOME => {
                            screen.list_view.select_first_line();
                        }
                        END => {
                            screen.list_view.select_last_line();
                        }
                        PAGE_UP => {
                            screen.list_view.try_scroll_pages(-1);
                        }
                        PAGE_DOWN => {
                            screen.list_view.try_scroll_pages(1);
                        }
                        UP => {
                            screen.list_view.try_select_next(true);
                        }
                        DOWN => {
                            screen.list_view.try_select_next(false);
                        }
                        CTRL_Q => {
                            quit = true;
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
    queue!(w, cursor::Show)?;
    queue!(w, LeaveAlternateScreen)?;
    w.flush()?;
    Ok(())
}
