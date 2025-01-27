use unicode_segmentation::UnicodeSegmentation;
use crate::highlighting::Type;
use crate::utils::Position;

pub struct Row {
    pub string: String,
    highlighting: Vec<Type>,
    pub is_highlighted: bool,
    pub len: usize,
}

impl Default for Row {
    fn default() -> Self {
        Self {
            string: String::new(),
            highlighting: vec![],
            is_highlighted: false,
            len: 0,
        }
    }
}

impl From<String> for Row {
    fn from(string: String) -> Self {
        let mut new_row = Row::default();
        new_row.string = string.clone();
        new_row.len = string[..].graphemes(true).count();

        new_row
    }
}

impl Row {
    pub fn render(&self, cursor_pos: &Position, start: u16, end: u16) -> u16 {
        // get highlighting information
        // go through the cells, print based on the highlighting information
        if (self.string.is_empty()) {
            return 0_u16
        }
        for entry in self.string.graphemes(true).skip(start as usize).take(end.saturating_sub(start) as usize) {
            print!("{}", entry);
        }

        end.saturating_sub(start)
    }
}