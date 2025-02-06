use crate::filetype::FileType;
use crate::row::Row;
use crate::utils::{HighlightStreak, HighlightingOptions, Position};

use std::env;
use std::fs;
use std::thread;
use std::time::Duration;
use unicode_segmentation;

pub struct Document {
    pub rows: Vec<Row>,
    pub file_name: String,
    pub is_loaded: bool,
    pub dirty: bool,
    pub file_type: FileType,
    pub hl_streak: HighlightStreak,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            rows: vec![],
            file_name: "".to_owned(),
            dirty: false,
            is_loaded: false,
            file_type: FileType {
                name: "rust".to_owned(),
                highlighting_ops: HighlightingOptions::default(),
            },
            hl_streak: HighlightStreak::default(),
        }
    }
}

impl Document {
    pub fn load(&mut self) {
        let variables: Vec<String> = env::args().collect();
        if let Some(file_name) = &variables.get(1) {
            let contents =
                fs::read_to_string(file_name).expect("Something went wrong reading the file");
            // todo: later we use a more elegant way to exit with error message
            for (index, row) in contents.lines().enumerate() {
                self.rows.push(Row::from(row.to_owned()));
            }
        }

        self.is_loaded = true;
    }

    pub fn highlight(
        &mut self,
        options: &HighlightingOptions,
        word: &Option<String>,
        until: Option<u16>
    ) {
        // the goal is to append the highlighting option to the 'highlighting' field of all rows in the viewport
        let end: u16 = match until {
            Some(end) => {
                let res = if end > self.rows.len() as u16 {
                    self.rows.len() as u16
                } else {
                    end
                };

                res
            }
            _ => self.rows.len() as u16,
        };

        for row in self.rows.iter_mut().take(end as usize) {
            row.highlight(options, word, &mut self.hl_streak);
        }
    }
}
