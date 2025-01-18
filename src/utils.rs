use std::error::Error;
#[derive(Default, Debug)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

use std::time::Instant;

#[derive(Default, Debug, Clone, Copy)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

#[derive(Default)]
pub struct HighlightingOptions {
    numbers: bool,
    strings: bool,
    characters: bool,
    comments: bool,
    multiline_comments: bool,
    primary_keywords: Vec<String>,
    secondary_keywords: Vec<String>
}

pub struct StatusMessage {
    pub text: String,
    pub time: Instant,
}

impl StatusMessage {
    pub fn from(message: String) -> Self {
        Self {
            time: Instant::now(),
            text: message,
        }
    }
}

#[derive(PartialEq)]
pub enum TerminalMode {
    Normal,
    Insert
}

pub fn die (err: impl Error) {
    panic!("{}", err);
}