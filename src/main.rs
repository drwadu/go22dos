mod config;
mod memory;
mod ui;

use ll::has_colors;
use ncurses::*;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use whoami;

#[derive(Debug, Error)]
pub enum Error {
    PoisenedMutexError,
    MemoryError(#[from] memory::MemoryError),
    UserInterfaceError(#[from] ui::UserInterfaceError),
    IoError(#[from] std::io::Error),
    Unknown,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

type Result<T> = std::result::Result<T, Error>;

fn startup() {
    initscr();
    noecho();
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    if unsafe { has_colors() == 1 } {
        if start_color() == OK {
            config::init_pairs();
        }
    }
}

fn start_screen() {
    let (mut x, mut y) = (0, 0);
    let w = stdscr();

    getmaxyx(w, &mut y, &mut x);
    config::START_SCREEN.iter().enumerate().for_each(|(i, s)| {
        let j = s.len() as i32;
        mvprintw((y / 2) + i as i32, (x / 2) - (j / 2), s);
    })
}

fn display_command(c: i32, i: i32) {
    let w = stdscr();

    let y = getmaxy(w);

    mvprintw(y - 1 as i32, 0 + i, &format!("{}", c as u8 as char));
}

fn display(s: &str) {
    let w = stdscr();

    let y = getmaxy(w);
    let x = getmaxx(w);

    let j = s.len() as i32;
    mvprintw(y - 1 as i32, (x / 2) - (j / 2) as i32, &format!("{s}"));
}

fn display_message(s: &str) {
    let w = stdscr();

    let y = getmaxy(w);

    let j = s.len() as i32;
    mvprintw(y - j as i32, 0 as i32, &format!("{s}"));
}

fn init(file: &str) -> Result<Arc<Mutex<memory::Memory<String, String>>>> {
    memory::deserialize::<String, String>(file)
        .map(|mem| Arc::new(Mutex::new(mem)))
        .ok_or(Error::Unknown)
}

fn save<K, V>(a2do: &impl memory::MemoryManagement<K, V>, file: &str) -> Result<()>
where
    K: memory::Serializable,
    V: memory::Serializable,
{
    a2do.save(file.to_owned()).map_err(Error::MemoryError)
}

fn topics<K, V>(a2do: &impl ui::UserInterface<K, V>, on: usize) -> Result<()>
where
    K: memory::Serializable,
    V: memory::Serializable,
{
    a2do.display_topic_ids(on)
        .map_err(Error::UserInterfaceError)
}

fn items<K, V>(a2do: &impl ui::UserInterface<K, V>, on_topic: usize, on_item: usize) -> Result<()>
where
    K: memory::Serializable,
    V: memory::Serializable,
{
    a2do.display_items(on_topic, on_item)
        .map_err(Error::UserInterfaceError)
}

fn add_topic<K, V>(
    a2do: &mut impl memory::MemoryManagement<K, V>,
    topic: impl Into<K>,
) -> Result<()>
where
    K: memory::Serializable,
    V: memory::Serializable,
{
    a2do.add_topic(topic).map_err(Error::MemoryError)
}

fn add_item<K, V>(a2do: &mut impl memory::MemoryManagement<K, V>, on: usize, item: V) -> Result<()>
where
    K: memory::Serializable,
    V: memory::Serializable,
{
    a2do.add_item(on, item).map_err(Error::MemoryError)
}

fn delete_item<K, V>(
    a2do: &mut impl memory::MemoryManagement<K, V>,
    on: usize,
    on_item: usize,
) -> Result<V>
where
    K: memory::Serializable,
    V: memory::Serializable,
{
    a2do.delete_item(on, on_item).map_err(Error::MemoryError)
}

fn access_item<K, V>(
    a2do: &mut impl memory::MemoryManagement<K, V>,
    on: usize,
    on_item: usize,
) -> Result<V>
where
    K: memory::Serializable,
    V: memory::Serializable,
{
    a2do.access_item(on, on_item).map_err(Error::MemoryError)
}

fn delete_topic<K, V>(a2do: &mut impl memory::MemoryManagement<K, V>, on: usize) -> Result<()>
where
    K: memory::Serializable,
    V: memory::Serializable,
{
    a2do.delete_topic(on).map_err(Error::MemoryError)
}

fn main() -> Result<()> {
    startup();

    let mut file = std::env::args()
        .skip(1)
        .next()
        .ok_or(Error::Unknown)?;
    let mut a2do = init(&file)?;
    let mut on = 0;

    loop {
        start_screen();
        display(&format!(
            "{} @ {}",
            whoami::username(),
            whoami::devicename()
        ));

        let ctx = a2do.lock().unwrap();
        let mut ubt = ctx.idxs.len();
        drop(ctx);

        let c = getch();

        display_command(c, 1);

        match c {
            config::GO_TO_TODOS => {
                clear();
                display("topics");
                display_command(c, 1);
                topics(&a2do, on)?;

                loop {
                    let c = getch();

                    display_command(c, 1);

                    match c {
                        config::DOWN => {
                            if on < ubt - 1 {
                                on += 1;
                                topics(&a2do, on)?;
                            }
                        }
                        config::UP => {
                            if on > 0 {
                                on -= 1;
                                topics(&a2do, on)?;
                            }
                        }
                        config::SG => match getch() {
                            config::SG => {
                                on = 0;
                                topics(&a2do, on)?;
                            }
                            _ => (),
                        },
                        config::BG => {
                            on = ubt - 1;
                            topics(&a2do, on)?;
                        }
                        config::SELECT => {
                            let ctx = a2do.lock().unwrap();
                            let mut ub = unsafe {
                                ctx.data
                                    .get(ctx.idxs.get(on).unwrap())
                                    .map(|v| v.len())
                                    .unwrap_unchecked()
                            };
                            let topic = unsafe { ctx.idxs.get_unchecked(on) }.clone();
                            drop(ctx);
                            let mut on_item = 0;

                            clear();
                            items(&a2do, on, on_item)?;

                            loop {
                                display(&topic);
                                let mut c = getch();
                                display_command(c, 1);

                                match c {
                                    config::APPEND => {
                                        let (mut x, mut y) = (0, 0);
                                        getmaxyx(stdscr(), &mut y, &mut x);
                                        mvprintw(y / 2, (x / 2) as i32, "");

                                        curs_set(CURSOR_VISIBILITY::CURSOR_VISIBLE);

                                        let mut t = "".to_owned();
                                        let mut n = 0;

                                        loop {
                                            c = getch();

                                            match c {
                                                config::ESC => break,
                                                config::REMOVE => {
                                                    t.pop();
                                                    n -= 1;
                                                    mvprintw(y / 2, (x / 2) + n as i32, " ");
                                                    mvprintw(y / 2, (x / 2) as i32, &t);
                                                }
                                                _ => {
                                                    t = format!("{}{}", t, c as u8 as char);
                                                    n += 1;
                                                    mvprintw(y / 2, (x / 2) as i32, &t);
                                                }
                                            }
                                        }

                                        attron(COLOR_PAIR(config::HIGHLIGHT_PAIR));
                                        addstr(&t);
                                        attroff(COLOR_PAIR(config::HIGHLIGHT_PAIR));

                                        add_item(&mut a2do, on, format!("0{}", t))?;

                                        curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

                                        let ctx = a2do.lock().unwrap();
                                        ub = ctx
                                            .data
                                            .get(ctx.idxs.get(on).unwrap())
                                            .map(|v| v.len())
                                            .unwrap();
                                        drop(ctx);

                                        on_item = 0;
                                        clear();

                                        items(&a2do, on, on_item)?;
                                    }
                                    config::TAB => {
                                        access_item(&mut a2do, on, on_item).and_then(|item| {
                                            delete_item(&mut a2do, on, on_item).unwrap();
                                            add_item(&mut a2do, on, format!("1{}", &item[1..]))
                                        })?;

                                        items(&a2do, on, on_item)?;
                                    }
                                    config::DELETE => {
                                        delete_item(&mut a2do, on, on_item)?;

                                        let ctx = a2do.lock().unwrap();
                                        ub = ctx
                                            .data
                                            .get(ctx.idxs.get(on).unwrap())
                                            .map(|v| v.len())
                                            .unwrap();
                                        drop(ctx);
                                        on_item = 0;
                                        clear();

                                        items(&a2do, on, on_item)?;
                                    }
                                    config::DOWN => {
                                        if on_item < ub - 1 {
                                            on_item += 1;
                                            items(&a2do, on, on_item)?;
                                        }
                                    }
                                    config::UP => {
                                        if on_item > 0 {
                                            on_item -= 1;
                                            items(&a2do, on, on_item)?;
                                        }
                                    }
                                    config::SG => match getch() {
                                        config::SG => {
                                            on_item = 0;
                                            items(&a2do, on, on_item)?;
                                        }
                                        _ => (),
                                    },
                                    config::BG => {
                                        on_item = ub - 1;
                                        items(&a2do, on, on_item)?;
                                    }
                                    config::ESC | config::EXIT => {
                                        clear();
                                        topics(&a2do, on)?;

                                        break;
                                    }
                                    _ => (),
                                }
                            }
                        }
                        config::APPEND => {
                            clear();

                            addstr("[new todos-topic]\t");

                            curs_set(CURSOR_VISIBILITY::CURSOR_VISIBLE);

                            echo();

                            let mut c = getch();
                            let mut t = "".to_owned();

                            while c != config::ESC {
                                echo();

                                t = format!("{}{}", t, c as u8 as char);
                                c = getch();

                                noecho();
                            }

                            attron(COLOR_PAIR(config::HIGHLIGHT_PAIR));
                            addstr(&t);
                            attroff(COLOR_PAIR(config::HIGHLIGHT_PAIR));

                            add_topic(&mut a2do, t.clone())?;

                            curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

                            let ctx = a2do.lock().unwrap();
                            ubt = ctx.idxs.len();
                            drop(ctx);

                            on = 0;
                            clear();

                            topics(&a2do, on)?;
                        }
                        config::DELETE => {
                            delete_topic(&mut a2do, on)?;

                            let ctx = a2do.lock().unwrap();
                            ubt = ctx.idxs.len();
                            drop(ctx);

                            on = 0;
                            clear();

                            topics(&a2do, on)?;
                        }
                        config::ESC | config::EXIT => {
                            clear();
                            break;
                        }
                        _ => (),
                    }
                }
            }
            config::QUIT => {
                save(&a2do, &file)?;
                flash();
                endwin();
                return Ok(());
            }
            _ => (),
        }
    }
}
