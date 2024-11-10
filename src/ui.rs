use std::fmt;

use ncurses::*;
use thiserror::Error;

use crate::config;
use crate::memory;

type Result<T> = std::result::Result<T, UserInterfaceError>;

#[derive(Debug, Error)]
pub enum UserInterfaceError {
    MemoryError(#[from] memory::MemoryError),
    IoError(#[from] std::io::Error),
    Unknown,
}
impl fmt::Display for UserInterfaceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub trait UserInterface<K, V>
where
    K: memory::Serializable,
    V: memory::Serializable,
{
    fn display_topic_ids(&self, on: usize) -> Result<()>;
    fn display_items(&self, on_topic: usize, on_item: usize) -> Result<()>;
}
impl<K, V, T> UserInterface<K, V> for T
where
    T: memory::MemoryManagement<K, V>,
    K: memory::Serializable + std::fmt::Debug,
    V: memory::Serializable + std::fmt::Debug + Into<String>,
{
    fn display_topic_ids(&self, on: usize) -> Result<()> {
        let ctx = self.lock_unwrap();
        match ctx.idxs.is_empty() {
            true => {
                addstr("no topics atm");
            }
            _ => {
                mv(1, 0);
                ctx.idxs.iter().enumerate().for_each(|(i, k)| {
                    let j = i as i32;
                    mv(j, 0);
                    match j == on as i32 {
                        true => {
                            let topic = ctx.idxs.get(on).ok_or(UserInterfaceError::Unknown).unwrap();
                            let items = ctx.data.get(topic).ok_or(UserInterfaceError::Unknown).unwrap();
                            let n = items.len() as f32;
                            let m = items.iter().filter(|item| item.to_string().starts_with('1')).count() as f32;

                            addstr(&format!("[{:.2}]\t ", m / n));

                            attron(COLOR_PAIR(config::HIGHLIGHT_PAIR));
                            addstr(&k.as_ref());
                            attroff(COLOR_PAIR(config::HIGHLIGHT_PAIR));
                        }
                        _ => {
                            //addstr(&format!("[topic]\t {}", k));
                            let topic = ctx.idxs.get(j as usize).ok_or(UserInterfaceError::Unknown).unwrap();
                            let items = ctx.data.get(topic).ok_or(UserInterfaceError::Unknown).unwrap();
                            let n = items.len() as f32;
                            let m = items.iter().filter(|item| item.to_string().starts_with('1')).count() as f32;

                            addstr(&format!("[{:.2}]\t ", m / n));

                            addstr(&k.as_ref());
                        }
                    }
                });
            }
        }
        Ok(())
    }

    fn display_items(&self, on_topic: usize, on_item: usize) -> Result<()> {
        let ctx = self.lock_unwrap();

        let topic = ctx.idxs.get(on_topic).ok_or(UserInterfaceError::Unknown)?;
        let items = ctx.data.get(topic).ok_or(UserInterfaceError::Unknown)?;

        let _y = items.len() as i32;

        let w = stdscr();
        let _x = getmaxx(w);

        match items.is_empty() {
            true => {
                addstr("no items atm");
            }
            _ => items.iter().enumerate().for_each(|(i, item)| {
                mv(i as i32, 0);
                let (cp, s) = match item.as_ref().chars().next() {
                    Some('0') => (COLOR_PAIR(config::CHECKBOX_TODO_PAIR), "[ ]"),
                    Some('1') => (COLOR_PAIR(config::CHECKBOX_DONE_PAIR), "[X]"),
                    _ => (COLOR_PAIR(config::CHECKBOX_DONE_PAIR), "[?]"),
                };
                match i == on_item {
                    true => {
                        attron(cp);
                        addstr(s);
                        attroff(cp);

                        addstr("\t ");

                        attron(COLOR_PAIR(config::HIGHLIGHT_PAIR));
                        addstr(&item.as_ref()[1..]);
                        attroff(COLOR_PAIR(config::HIGHLIGHT_PAIR));
                    }
                    _ => {
                        addstr(&format!("{}\t {}", s, &item.as_ref()[1..]));
                    }
                }
            }),
        }

        Ok(())
    }
}
