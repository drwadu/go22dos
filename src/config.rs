pub const START_SCREEN: [&'static str; 10] = [
    "            go22dos                   ",
    "                                      ",
    "type t     to go to todos             ",
    "type j/k   to go down/up              ",
    "type s     to select specific todos   ",
    "type TAB   to tick off todo           ",
    "type a     to add todo(s)             ",
    "type d     to delete todo(s)          ",
    "type ESC   to exit todos or text input",
    "type q     to quit                    ",
];

const REGULAR_PAIR: i16 = 0;
const REGULAR_PAIR_FST: i16 = ncurses::COLOR_WHITE;
const REGULAR_PAIR_SND: i16 = ncurses::COLOR_BLACK;
pub const HIGHLIGHT_PAIR: i16 = 1;
const HIGHLIGHT_PAIR_FST: i16 = ncurses::COLOR_BLACK;
const HIGHLIGHT_PAIR_SND: i16 = ncurses::COLOR_WHITE;
pub const CHECKBOX_TODO_PAIR: i16 = 2;
const CHECKBOX_TODO_PAIR_FST: i16 = ncurses::COLOR_BLACK;
const CHECKBOX_TODO_PAIR_SND: i16 = ncurses::COLOR_RED;
pub const CHECKBOX_DONE_PAIR: i16 = 3;
const CHECKBOX_DONE_PAIR_FST: i16 = ncurses::COLOR_BLACK;
const CHECKBOX_DONE_PAIR_SND: i16 = ncurses::COLOR_GREEN;
pub const OTHER_PAIR: i16 = 6;
const OTHER_PAIR_FST: i16 = ncurses::COLOR_BLACK;
const OTHER_PAIR_SND: i16 = ncurses::COLOR_CYAN;

pub fn init_pairs() {
    ncurses::init_pair(REGULAR_PAIR, REGULAR_PAIR_FST, REGULAR_PAIR_SND);
    ncurses::init_pair(HIGHLIGHT_PAIR, HIGHLIGHT_PAIR_FST, HIGHLIGHT_PAIR_SND);
    ncurses::init_pair(
        CHECKBOX_TODO_PAIR,
        CHECKBOX_TODO_PAIR_FST,
        CHECKBOX_TODO_PAIR_SND,
    );
    ncurses::init_pair(
        CHECKBOX_DONE_PAIR,
        CHECKBOX_DONE_PAIR_FST,
        CHECKBOX_DONE_PAIR_SND,
    );
    ncurses::init_pair(OTHER_PAIR, OTHER_PAIR_FST, OTHER_PAIR_SND);
}

//pub const ENTER: i32 = 13;
pub const TAB: i32 = 9;
pub const ESC: i32 = 27;
pub const BG: i32 = 71;
pub const APPEND: i32 = 97;
pub const EDIT: i32 = 98;
pub const DELETE: i32 = 100;
pub const EXIT: i32 = 101;
pub const SG: i32 = 103;
pub const DOWN: i32 = 106;
pub const UP: i32 = 107;
pub const QUIT: i32 = 113;
pub const SELECT: i32 = 115;
pub const GO_TO_TODOS: i32 = 116;
pub const REMOVE: i32 = 127;
