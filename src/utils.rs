use std::error::Error;
use std::collections::HashSet;
use unicode_segmentation::UnicodeSegmentation;
#[derive(Default, Debug)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

#[derive(Default, Debug)]
pub struct HighlightStreak {
    pub comment: u16,
    pub quote: bool,
}

use std::time::Instant;

#[derive(Default, Debug, Clone, Copy)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

#[derive(Default, Debug)]
pub struct MovementData {
    pub last_nav_position: Position,
}

pub struct HighlightingOptions {
    pub numbers: bool,
    pub strings: bool,
    pub characters: bool,
    pub comments: bool,
    pub multiline_comments: bool,
    pub primary_keywords: HashSet<String>,
    pub secondary_keywords: HashSet<String>,
    pub known_items: HashSet<String>,
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
            known_items: HashSet::from(["String".to_string(), "Ok".to_string(), "Err".to_string(), "Some".to_string(), "None".to_string(), "Vec".to_string(), "Option".to_string()])
        }
    }
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

// pub fn fuzzy_find_grapheme_index(haystack: &str, offset: usize, needle: &str) -> Option<usize> {
//     let haystack_graphemes: Vec<&str> = haystack.graphemes(true).skip(offset).collect();
//     let needle_graphemes: Vec<&str> = needle.graphemes(true).collect();
//
//     haystack_graphemes
//         .windows(needle_graphemes.len()) // Create sliding windows of the needle's length
//         .position(|window| window == needle_graphemes.as_slice()) // Find first occurrence
// }
