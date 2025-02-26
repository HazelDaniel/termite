use std::cmp::max;
use std::error::Error;
use std::collections::{HashMap, HashSet};
use unicode_segmentation::UnicodeSegmentation;
use std::time::Instant;
use std::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::io;
use std::io::{stdin, ErrorKind, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use chrono::format::Item::{Error as ChronoError};
use tokio::task::JoinHandle;
use once_cell::sync::OnceCell;
use rayon::prelude::{IndexedParallelIterator, ParallelIterator, ParallelSlice};
use termion::event::Key;
use crate::automata::EditorFSM;
use crate::editor::Editor;

pub enum PromptCallbackCode {
    Success,
    Failure,
    Continue
}

pub struct FSMActionQueue {
    pub actions:            Vec<FSMAction>
}

pub struct FSMAction {
    pub index:                  usize,
    pub count:                  usize,
    pub payload:                Option<Position>
}

pub trait Promptable {
    fn on_prompt_loop_start(&mut self, prompt: &str) -> Result<(), std::io::Error> {
        Ok(())
    }
    fn prompt<C>(&mut self, mut callback: C, prompt: Option<String>) -> Result<Option<String>, std::io::Error>
    where C: FnMut(&mut Self, Key) {
        use std::io::{Stdin, Write, stdin};
        use termion::input::TermRead;


        let mut result = String::new();
        loop {
            self.on_prompt_loop_start(&result);

            match stdin().keys().next().unwrap_or(Err(io::Error::new(ErrorKind::InvalidInput, "")))? {
                Key::Backspace => {
                    result.pop();
                },
                Key::Char('\n') => {
                    callback(self, Key::Char('\n'));
                    break;
                },
                Key::Char(x) => {
                    if !(x.is_control()) {
                        callback(self, Key::Char(x));
                        result.push(x);
                    }
                },
                Key::Esc => {
                    result.truncate(0);
                    break;
                },
                _ => ()
            }

        }

        if result.is_empty() { return Ok(None); }

        Ok(Some(result))
    }

    fn prompt_exec<C>(&mut self, mut callback: C, prompt: Option<String>) -> Result<(), std::io::Error>
    where C: FnMut(&mut Self, Key) -> PromptCallbackCode {
        use std::io::{Stdin, Write, stdin};
        use termion::input::TermRead;

        loop {
            match stdin().keys().next().unwrap_or(Err(io::Error::new(ErrorKind::InvalidInput, "")))? {
                Key::Esc => {
                    return Ok(())
                },
                k => {
                    match callback(self, k) {
                        PromptCallbackCode::Success | PromptCallbackCode::Failure => {
                            return Ok(())
                        },
                        PromptCallbackCode::Continue => ()
                    }
                },
            }

        }
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, PartialEq)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
    None
}

#[derive(Debug, Default)]
pub struct Selection {
    pub position:       Position,
    pub start:          (u16, u16),
    pub end:            (u16, u16), // range is: [start, end]
}

#[derive(Debug, PartialEq, Hash)]
pub enum VCharacterClass {
    Blank,
    Word,
    Punctuation,
    NonPunctGraph,
    Others
}

impl From<VCharacterClass> for String {
    fn from(c: VCharacterClass) -> Self {
        match c {
            VCharacterClass::Blank => "blank".to_string(),
            VCharacterClass::Word => "word".to_string(),
            VCharacterClass::Punctuation => "punctuation".to_string(),
            VCharacterClass::NonPunctGraph => "nonPunctGraph".to_string(),
            VCharacterClass::Others => "others".to_string(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum NumberMode {
    Decimal,
    Octal,
    Hexadecimal,
    Binary,
    Float,
    None
}

#[derive(Default, Debug)]
pub struct HighlightStreak {
    pub comment:    u16,
    pub quote:      bool,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Size {
    pub width:      u16,
    pub height:     u16,
}

#[derive(Default, Debug)]
pub struct MovementData {
    pub last_nav_position: Position,
}

pub struct HighlightingOptions {
    pub numbers:                        bool,
    pub strings:                        bool,
    pub characters:                     bool,
    pub comments:                       bool,
    pub multiline_comments:             bool,
    pub primary_keywords:               HashSet<String>,
    pub secondary_keywords:             HashSet<String>,
    pub known_items:                    HashSet<String>,
}

impl Default for HighlightingOptions {
    fn default() -> Self {
        Self {
            numbers: true,
            strings: true,
            characters: true,
            comments: true,
            multiline_comments: true,
            primary_keywords: HashSet::from([
                "as".to_string(),
                "break".to_string(),
                "const".to_string(),
                "continue".to_string(),
                "crate".to_string(),
                "else".to_string(),
                "enum".to_string(),
                "extern".to_string(),
                "false".to_string(),
                "fn".to_string(),
                "for".to_string(),
                "if".to_string(),
                "impl".to_string(),
                "in".to_string(),
                "let".to_string(),
                "loop".to_string(),
                "match".to_string(),
                "mod".to_string(),
                "move".to_string(),
                "mut".to_string(),
                "pub".to_string(),
                "ref".to_string(),
                "return".to_string(),
                "self".to_string(),
                "Self".to_string(),
                "static".to_string(),
                "struct".to_string(),
                "super".to_string(),
                "trait".to_string(),
                "true".to_string(),
                "type".to_string(),
                "unsafe".to_string(),
                "use".to_string(),
                "where".to_string(),
                "while".to_string(),
                "dyn".to_string(),
                "abstract".to_string(),
                "become".to_string(),
                "box".to_string(),
                "do".to_string(),
                "final".to_string(),
                "macro".to_string(),
                "override".to_string(),
                "priv".to_string(),
                "typeof".to_string(),
                "unsized".to_string(),
                "virtual".to_string(),
                "yield".to_string(),
                "async".to_string(),
                "await".to_string(),
                "try".to_string(),
                "str".to_string()
            ]),
            secondary_keywords: HashSet::from([
                "bool".to_string(),
                "char".to_string(),
                "i8".to_string(),
                "i16".to_string(),
                "i32".to_string(),
                "i64".to_string(),
                "isize".to_string(),
                "u8".to_string(),
                "u16".to_string(),
                "u32".to_string(),
                "u64".to_string(),
                "usize".to_string(),
                "f32".to_string(),
                "f64".to_string(),
            ]),
            known_items: HashSet::from(["String".to_string(), "Ok".to_string(), "Err".to_string(), "Some".to_string(), "None".to_string(), "Vec".to_string(), "Option".to_string(), "Result".to_string()])
        }
    }
}
pub struct StatusMessage {
    pub text:       String,
    pub time:       Instant,
}

impl StatusMessage {
    pub fn from(message: String) -> Self {
        Self {
            time: Instant::now(),
            text: message,
        }
    }
}

pub struct OrderedLogger {
    pub file:       Arc<Mutex<std::fs::File>>,
}

impl OrderedLogger {
    pub fn new(file: &str) -> Result<OrderedLogger, std::io::Error> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(file).map_err(|e| std::io::Error::new(ErrorKind::Other, "failed to open file for logging!"))?;

        Ok(OrderedLogger{
            file: Arc::new(Mutex::new(file)),
        })
    }

    pub fn log(&self, message: &str) -> Result<(), std::io::Error> {
        let mut file = self.file.lock().expect("couldn't acquire lock on file for logging");
        writeln!(*file, "{}", message).expect("Could not write to file");
        file.flush()?;

        Ok(())
    }
}

pub static LOGGER: OnceCell<OrderedLogger> = OnceCell::new();

#[macro_export]
macro_rules! log {
    ($($arg: tt)*) => {
        {
            let log_message = format!($($arg)*);
            let logger = $crate::LOGGER.get().expect("Logger not initialized");
            &logger.log(&log_message).unwrap();
        }
    }
}



#[derive(PartialEq)]
pub enum TerminalMode {
    Normal,
    Insert,
}

pub fn die(err: impl Error) {
    panic!("{}", err);
}

pub fn find_grapheme_index(haystack: &str, offset: usize, needle: &str) -> Option<usize> {
    let haystack_graphemes: Vec<&str> = haystack.graphemes(true).skip(offset).collect();
    let needle_graphemes: Vec<&str> = needle.graphemes(true).collect();

    haystack_graphemes
        .windows(needle_graphemes.len()) // Create sliding windows of the needle's length
        .position(|window| window == needle_graphemes.as_slice()) // Find first occurrence
}

pub fn find_char_position(text: &str, condition: impl Fn(char) -> bool) -> Option<usize> {
    text.char_indices().find(|&(_, c)| condition(c)).map(|(i, _)| i)
}

pub fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

pub fn is_even(s: usize) -> bool {
    s as i64 & (-1_i64 * s as i64) != 1
}

pub fn is_brace(c: char) -> bool {
    c == '[' || c == ']' || c == '{' || c == '}' || c == '(' || c == ')'
}

pub fn is_opening_brace(c: char) -> bool {
    c == '[' || c == '{' || c == '('
}

pub fn is_closing_brace(c: char) -> bool {
    c == ']' || c == '}' || c == ')'
}

pub fn get_matching_enclosable(c: char) -> Option<char> {
    let brace_hash: HashMap<char, char> = HashMap::from([['[', ']'], ['{', '}'], ['(', ')'], [']', '['], ['}', '{'], [')', '(']]);

    if let Some(c) = brace_hash.get(&c) {
        Some(*c)
    } else {
        None
    }
}

fn parallel_find_first(s: &str, target: char, grapheme_mode: bool) -> Option<usize> {
    let found_index = AtomicUsize::new(usize::MAX); // Store the earliest index found
    let chunk_size = max(s.len() >> 10, 1); // LEN/1024

    if grapheme_mode {
        s.graphemes(true).as_str().as_bytes()
            .par_chunks(chunk_size)
            .enumerate()
            .try_for_each(|(chunk_idx, chunk)| {
                if found_index.load(Ordering::Acquire) != usize::MAX {
                    return Err(());
                }

                if let Some(local_idx) = chunk.iter().position(|&c| c == target as u8) {
                    let global_idx = chunk_idx * chunk_size + local_idx;
                    found_index.fetch_min(global_idx, Ordering::Relaxed);
                    return Err(());
                }

                Ok(())
            })
            .ok();
    } else {
        s.as_bytes()
            .par_chunks(chunk_size)
            .enumerate()
            .try_for_each(|(chunk_idx, chunk)| {
                if found_index.load(Ordering::Acquire) != usize::MAX {
                    return Err(());
                }

                if let Some(local_idx) = chunk.iter().position(|&c| c == target as u8) {
                    let global_idx = chunk_idx * chunk_size + local_idx;
                    found_index.fetch_min(global_idx, Ordering::Relaxed);
                    return Err(());
                }

                Ok(())
            })
            .ok();
    }

    let result = found_index.load(Ordering::Acquire);
    if result == usize::MAX {
        None
    } else {
        Some(result)
    }
}

fn par_find_first(s: &str, target: char, grapheme_mode: bool) -> Option<usize> {
    let found_index = AtomicUsize::new(usize::MAX);
    let chunk_size = max(s.len() >> 10, 1); // LEN/1024

    if grapheme_mode {
        s.graphemes(true).as_str().as_bytes().par_chunks(chunk_size)
            .enumerate()
            .for_each(|(chunk_idx, chunk)| {
                if found_index.load(Ordering::Relaxed) != usize::MAX { return; }
                if let Some(local_idx) = chunk.iter().position(|&c| c == target as u8) {
                    let global_idx = chunk_idx * chunk_size + local_idx;
                    found_index.fetch_min(global_idx, Ordering::Relaxed);
                }
            });
    } else {
        s.as_bytes().par_chunks(chunk_size)
            .enumerate()
            .for_each(|(chunk_idx, chunk)| {
                if found_index.load(Ordering::Relaxed) != usize::MAX { return; }
                if let Some(local_idx) = chunk.iter().position(|&c| c == target as u8) {
                    let global_idx = chunk_idx * chunk_size + local_idx;
                    found_index.fetch_min(global_idx, Ordering::Relaxed);
                }
            });
    }


    let result = found_index.load(Ordering::Relaxed);
    if result == usize::MAX {
        None
    } else {
        Some(result)
    }
}

pub fn get_v_char_class(c: char) -> VCharacterClass {
    if is_word(c) {
        return VCharacterClass::Word;
    } else if c.is_whitespace() {
        return VCharacterClass::Blank;
    } else if c.is_ascii_punctuation() {
        return VCharacterClass::Punctuation;
    } else {
        return VCharacterClass::NonPunctGraph;
    }
}

pub fn get_isolated_v_char_class(c: char) -> VCharacterClass {
    match get_v_char_class(c) {
        VCharacterClass::Word => VCharacterClass::Word,
        VCharacterClass::Blank => VCharacterClass::Blank,
        _ => VCharacterClass::Others
    }
}

pub fn get_isolated_v_str_class(c: &str) -> VCharacterClass {
    match get_v_char_class(c.chars().next().expect("could not get isolated string's character class!")) {
        VCharacterClass::Word => VCharacterClass::Word,
        VCharacterClass::Blank => VCharacterClass::Blank,
        _ => VCharacterClass::Others
    }
}

pub fn find_string_position(texts: &Vec<&str>, condition: impl Fn(&str) -> bool) -> Option<usize> {
    texts.iter().enumerate().find(|(i, & c)| condition(c)).map(|(i, _)| i)
}

pub fn v_jump_to_line(editor: &mut Editor, fsm: &mut EditorFSM, final_key: &char) -> () { // a vertical line jump
    let Position {x, ..} = editor.cursor_position;

    if fsm.command_count > editor.document.rows.len() { // out of bounds
        fsm.command_buffer.push(*final_key);
        return;
    } else {
        if let Some(seek_row) = editor.document.rows.get(fsm.command_count.saturating_sub(1) as usize)
        {
            if seek_row.len < (editor.movement_data.last_nav_position.x) as usize {
                editor.cursor_position.x = seek_row.len.saturating_sub(1) as u16;
            } else {
                editor.cursor_position.x = editor.movement_data.last_nav_position.x;
            }
            editor.cursor_position.y = fsm.command_count.saturating_sub(1) as u16;
        }
    }

    fsm.command_buffer.push(*final_key);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    pub fn run_parallelize_chunk_search() {
        let mut test = "this is great view...............".repeat(10_000_000);
        test.push('b');
        test.push_str(&"this is great view...............".repeat(10_000_000));
        let target = 'b';

        if let Some(index) = par_find_first(&test, target, false) {
            println!("index found at {}", index);
        } else {
            println!("char not found!");
        }
    }
}